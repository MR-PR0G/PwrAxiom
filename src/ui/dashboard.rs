use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    gdk, Align, Box as GtkBox, Button, CenterBox, FlowBox, Label, Orientation, PolicyType,
    ProgressBar, Revealer, RevealerTransitionType, ScrolledWindow, SelectionMode, ToggleButton,
    Window,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use gatherer::platform::{
    CpuDynamicInfoExt, CpuInfo, CpuInfoExt, CpuStaticInfoExt, FanInfoExt, FansInfo, FansInfoExt,
    GpuDynamicInfoExt, GpuInfo, GpuInfoExt, GpuStaticInfoExt, Processes, ProcessesExt,
};

use crate::backend::{BackendManager, Profile};
use crate::ui::settings;

#[derive(Debug, Default, Clone)]
pub struct SystemSnapshot {
    pub cpu_name: String,
    pub cpu_usage: f64,
    pub cpu_temp: Option<f64>,
    pub cpu_power_w: f64,
    pub cpu_freq_ghz: f64,
    pub cores_usage: Vec<f64>,
    pub cores_freq: Vec<f64>,
    pub fan_rpm: Option<u64>,
    pub mem_used_gb: f64,
    pub mem_total_gb: f64,
    pub gpus: Vec<GpuSnapshot>,
    pub system_total_power_w: f64,
    pub system_avg_temp_c: f64,
}

#[derive(Debug, Default, Clone)]
pub struct GpuSnapshot {
    pub name: String,
    pub is_integrated: bool,
    pub util: f64,
    pub pwr_w: f64,
    pub temp_c: f64,
    pub freq_mhz: f64,
    pub vram_used_gb: f64,
    pub vram_total_gb: f64,
}

struct VisualState {
    pub cpu_usage: f64,
    pub mem_fraction: f64,
    pub gpus_util: Vec<f64>,
}

struct HardwareSupplement {
    cpu_rapl: Option<PathBuf>,
    sys_rapl: Option<PathBuf>,
    hwmon_cpu: Option<PathBuf>,
    bat_path: Option<PathBuf>,
    last_cpu_energy: u64,
    last_sys_energy: u64,
    last_time: Instant,
}

impl HardwareSupplement {
    fn new() -> Self {
        let mut cpu_rapl = None;
        let mut sys_rapl = None;
        let mut hwmon_cpu = None;
        let mut bat_path = None;

        if let Ok(entries) = std::fs::read_dir("/sys/class/powercap") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(name) = std::fs::read_to_string(p.join("name")) {
                    let name_lower = name.trim().to_lowercase();
                    if name_lower.contains("package") || name_lower.contains("rapl:0") {
                        if p.join("energy_uj").exists() {
                            cpu_rapl = Some(p.join("energy_uj"));
                        }
                    } else if name_lower.contains("psys") || name_lower.contains("platform") {
                        if p.join("energy_uj").exists() {
                            sys_rapl = Some(p.join("energy_uj"));
                        }
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(name) = std::fs::read_to_string(p.join("name")) {
                    let nl = name.trim().to_lowercase();
                    if nl.contains("zenpower")
                        || nl.contains("coretemp")
                        || nl.contains("k10temp")
                        || nl.contains("amd_energy")
                        || nl.contains("cpu")
                    {
                        if p.join("power1_average").exists() {
                            hwmon_cpu = Some(p.join("power1_average"));
                        } else if p.join("power1_input").exists() {
                            hwmon_cpu = Some(p.join("power1_input"));
                        }
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(typ) = std::fs::read_to_string(p.join("type")) {
                    if typ.trim().to_lowercase() == "battery" {
                        if p.join("power_now").exists()
                            || (p.join("current_now").exists() && p.join("voltage_now").exists())
                        {
                            bat_path = Some(p);
                            break;
                        }
                    }
                }
            }
        }

        Self {
            cpu_rapl,
            sys_rapl,
            hwmon_cpu,
            bat_path,
            last_cpu_energy: 0,
            last_sys_energy: 0,
            last_time: Instant::now(),
        }
    }

    fn read_energy_delta(path: &Option<PathBuf>, last_energy: &mut u64, elapsed: f64) -> f64 {
        if let Some(p) = path {
            if let Ok(val) = std::fs::read_to_string(p) {
                if let Ok(energy) = val.trim().parse::<u64>() {
                    let mut pwr = 0.0;
                    if elapsed > 0.05 && *last_energy > 0 && energy >= *last_energy {
                        pwr = ((energy - *last_energy) as f64 / 1_000_000.0) / elapsed;
                    }
                    *last_energy = energy;
                    if pwr > 0.1 && pwr < 500.0 {
                        return pwr;
                    }
                }
            }
        }
        0.0
    }

    fn poll_power(&mut self) -> (f64, f64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();

        let mut cpu_pwr =
            Self::read_energy_delta(&self.cpu_rapl, &mut self.last_cpu_energy, elapsed);
        let mut sys_pwr =
            Self::read_energy_delta(&self.sys_rapl, &mut self.last_sys_energy, elapsed);

        self.last_time = now;

        if cpu_pwr == 0.0 {
            if let Some(p) = &self.hwmon_cpu {
                if let Ok(val) = std::fs::read_to_string(p) {
                    if let Ok(uw) = val.trim().parse::<f64>() {
                        let pwr = uw / 1_000_000.0;
                        if pwr > 0.1 && pwr < 500.0 {
                            cpu_pwr = pwr;
                        }
                    }
                }
            }
        }

        if sys_pwr == 0.0 {
            if let Some(p) = &self.bat_path {
                if let Ok(val) = std::fs::read_to_string(p.join("power_now")) {
                    if let Ok(uw) = val.trim().parse::<f64>() {
                        sys_pwr = uw / 1_000_000.0;
                    }
                } else if let (Ok(i), Ok(v)) = (
                    std::fs::read_to_string(p.join("current_now")),
                    std::fs::read_to_string(p.join("voltage_now")),
                ) {
                    if let (Ok(ua), Ok(uv)) = (i.trim().parse::<f64>(), v.trim().parse::<f64>()) {
                        sys_pwr = (ua * uv) / 1_000_000_000_000.0;
                    }
                }
            }
        }

        (cpu_pwr, sys_pwr)
    }

    fn read_core_freqs(&self, cores: usize) -> Vec<f64> {
        (0..cores)
            .map(|i| {
                let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i);
                std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .map(|khz| khz / 1_000_000.0)
                    .unwrap_or(0.0)
            })
            .collect()
    }
}

fn shorten_gpu_name(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("cavelake") || lower.contains("coffeelake") || lower.contains("uhd 630") {
        return "UHD 630".to_string();
    }
    name.replace("Intel(R)", "")
        .replace("UHD Graphics", "UHD")
        .replace("AMD Radeon Graphics", "AMD Radeon")
        .replace("NVIDIA GeForce", "NVIDIA")
        .replace("Laptop GPU", "")
        .trim()
        .to_string()
}

pub fn init_and_start_hardware_polling() -> mpsc::Receiver<SystemSnapshot> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut processes = Processes::new();
        let mut cpu = CpuInfo::new();
        let mut fans = FansInfo::new();
        let mut hw_sup = HardwareSupplement::new();
        let mut gpu = GpuInfo::new();

        cpu.refresh_static_info_cache();
        gpu.refresh_gpu_list();
        gpu.refresh_static_info_cache();

        let mut last_profile_state = Profile::Balanced;

        loop {
            let current_active_profile = if let Ok(manager) = BackendManager::global().lock() {
                manager.current_profile()
            } else {
                Profile::Balanced
            };

            if current_active_profile != last_profile_state {
                last_profile_state = current_active_profile;
                thread::sleep(Duration::from_millis(1500));
                processes = Processes::new();
                cpu = CpuInfo::new();
                cpu.refresh_static_info_cache();
                continue;
            }

            let pool_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                processes.refresh_cache();
                cpu.refresh_dynamic_info_cache(&processes);
                gpu.refresh_dynamic_info_cache(&mut processes);
                fans.refresh_cache();

                let mut snap = SystemSnapshot::default();

                if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                    let mut total = 0.0;
                    let mut free = 0.0;
                    let mut buffers = 0.0;
                    let mut cached = 0.0;
                    for line in content.lines() {
                        let p: Vec<&str> = line.split_whitespace().collect();
                        if p.len() < 2 {
                            continue;
                        }
                        let v = p[1].parse::<f64>().unwrap_or(0.0) / 1_048_576.0;
                        match p[0] {
                            "MemTotal:" => total = v,
                            "MemFree:" => free = v,
                            "Buffers:" => buffers = v,
                            "Cached:" | "SReclaimable:" => cached += v,
                            _ => {}
                        }
                    }
                    snap.mem_total_gb = total;
                    snap.mem_used_gb = (total - free - buffers - cached).max(0.0);
                }

                let c_stat = cpu.static_info();
                let c_dyn = cpu.dynamic_info();

                snap.cpu_name = c_stat.name().to_string();
                snap.cpu_usage = c_dyn.overall_utilization_percent() as f64;
                snap.cpu_temp = c_dyn.temperature().map(|t| t as f64);

                let (cpu_pwr, sys_pwr_sensor) = hw_sup.poll_power();
                snap.cpu_power_w = cpu_pwr;

                let num_cores = std::fs::read_to_string("/sys/devices/system/cpu/possible")
                    .ok()
                    .and_then(|s| {
                        s.trim()
                            .split('-')
                            .last()
                            .unwrap_or("0")
                            .parse::<usize>()
                            .ok()
                    })
                    .map(|n| n + 1)
                    .unwrap_or_else(|| c_stat.logical_cpu_count() as usize);

                snap.cores_usage = c_dyn
                    .per_logical_cpu_utilization_percent()
                    .map(|u| *u as f64)
                    .collect();
                snap.cores_freq = hw_sup.read_core_freqs(num_cores);

                let total_freq: f64 = snap.cores_freq.iter().sum();
                snap.cpu_freq_ghz = if !snap.cores_freq.is_empty() {
                    total_freq / snap.cores_freq.len() as f64
                } else {
                    0.0
                };

                for fan in fans.info() {
                    if snap.fan_rpm.is_none() && fan.rpm() > 0 {
                        snap.fan_rpm = Some(fan.rpm());
                    }
                }

                let mut gpu_pwr_sum = 0.0;
                let mut temp_sum = snap.cpu_temp.unwrap_or(0.0);
                let mut temp_count = if snap.cpu_temp.is_some() { 1.0 } else { 0.0 };

                let gpu_ids: Vec<String> = gpu.enumerate().map(|s| s.to_string()).collect();
                for id in gpu_ids {
                    if let (Some(g_stat), Some(g_dyn)) =
                        (gpu.static_info(&id), gpu.dynamic_info(&id))
                    {
                        let mut g_snap = GpuSnapshot {
                            name: shorten_gpu_name(g_stat.device_name()),
                            is_integrated: g_stat.total_memory() < 2_000_000_000,
                            util: g_dyn.util_percent() as f64,
                            pwr_w: g_dyn.power_draw_watts() as f64,
                            temp_c: g_dyn.temp_celsius() as f64,
                            freq_mhz: g_dyn.clock_speed_mhz() as f64,
                            vram_used_gb: g_dyn.used_memory() as f64 / 1_073_741_824.0,
                            vram_total_gb: g_stat.total_memory() as f64 / 1_073_741_824.0,
                        };

                        if g_snap.is_integrated {
                            g_snap.vram_total_gb = snap.mem_total_gb * 0.5;
                            g_snap.vram_used_gb = (g_snap.util / 100.0) * snap.mem_used_gb * 0.15;
                            if g_snap.temp_c == 0.0 {
                                g_snap.temp_c = snap.cpu_temp.unwrap_or(0.0);
                            }
                        }

                        if g_snap.pwr_w > 0.1 {
                            gpu_pwr_sum += g_snap.pwr_w;
                        }
                        if g_snap.temp_c > 0.1 {
                            temp_sum += g_snap.temp_c;
                            temp_count += 1.0;
                        }

                        snap.gpus.push(g_snap);
                    }
                }

                snap.system_total_power_w = if sys_pwr_sensor > 0.1 {
                    sys_pwr_sensor
                } else {
                    cpu_pwr + gpu_pwr_sum
                };

                snap.system_avg_temp_c = if temp_count > 0.0 {
                    temp_sum / temp_count
                } else {
                    0.0
                };
                snap
            }));

            match pool_result {
                Ok(snap) => {
                    if tx.send(snap).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    processes = Processes::new();
                    cpu = CpuInfo::new();
                    cpu.refresh_static_info_cache();
                }
            }

            thread::sleep(Duration::from_millis(1000));
        }
    });

    rx
}

fn get_theme_config_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    path.push(".pwraxiom_theme");
    path
}

fn save_theme_to_disk(color: &str) {
    let path = get_theme_config_path();
    let _ = std::fs::write(path, color);
}

fn load_theme_from_disk() -> String {
    let path = get_theme_config_path();
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "#00e5ff".to_string())
}

fn apply_theme(base_color: &str) {
    if let Some(display) = gdk::Display::default() {
        let provider = gtk4::CssProvider::new();

        let r_str = if base_color.len() >= 7 {
            &base_color[1..3]
        } else {
            "00"
        };
        let g_str = if base_color.len() >= 7 {
            &base_color[3..5]
        } else {
            "229"
        };
        let b_str = if base_color.len() >= 7 {
            &base_color[5..7]
        } else {
            "255"
        };

        let css = format!(
            ":root {{
                --primary-color: {0};
                --secondary-color: rgba({1}, {2}, {3}, 0.25);
            }}",
            base_color,
            u8::from_str_radix(r_str, 16).unwrap_or(0),
            u8::from_str_radix(g_str, 16).unwrap_or(229),
            u8::from_str_radix(b_str, 16).unwrap_or(255)
        );
        provider.load_from_data(&css);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        save_theme_to_disk(base_color);
    }
}

pub fn load_saved_theme() {
    let saved_color = load_theme_from_disk();
    apply_theme(&saved_color);
}

struct CoreViews {
    usage: Label,
}

struct GpuViews {
    container: GtkBox,
    bar: ProgressBar,
    bar_lbl: Label,
    pwr_lbl: Label,
    temp_lbl: Label,
    vram_lbl: Label,
    freq_lbl: Label,
}

struct DashboardWidgets {
    cpu_brand_lbl: Label,
    total_cpu_lbl: Label,
    max_freq_lbl: Label,
    cpu_pwr_lbl: Label,

    system_pwr_lbl: Label,
    system_temp_lbl: Label,
    mem_bar: ProgressBar,
    mem_lbl: Label,

    core_views: RefCell<Vec<CoreViews>>,
    gpu_views: RefCell<Vec<GpuViews>>,
    gpu_container: GtkBox,
    flow_box: FlowBox,
}

fn open_generic_info_dialog(parent_win: Option<&Window>, title: &str, text: &str) {
    let dialog = Window::builder()
        .title(title)
        .modal(true)
        .default_width(360)
        .default_height(180)
        .resizable(false)
        .build();
    dialog.add_css_class("auth-win");
    if let Some(parent) = parent_win {
        dialog.set_transient_for(Some(parent));
    }

    let vbox = GtkBox::new(Orientation::Vertical, 15);
    vbox.set_margin_top(25);
    vbox.set_margin_bottom(25);
    vbox.set_margin_start(25);
    vbox.set_margin_end(25);
    vbox.set_valign(Align::Center);
    vbox.set_halign(Align::Center);

    let lbl_title = Label::builder()
        .label(title)
        .css_classes(["auth-title"])
        .build();
    let lbl_desc = Label::builder()
        .label(text)
        .css_classes(["auth-desc"])
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .build();

    let close_btn = Button::with_label("Confirm");
    close_btn.add_css_class("boost-btn-style");
    close_btn.connect_clicked(clone!(@weak dialog => move |_| dialog.close()));

    vbox.append(&lbl_title);
    vbox.append(&lbl_desc);
    vbox.append(&close_btn);

    dialog.set_child(Some(&vbox));
    dialog.present();
}

fn build_cpu_panel() -> (GtkBox, Label, Label, Label, Label, FlowBox) {
    let left_section = GtkBox::new(Orientation::Vertical, 5);
    left_section.set_size_request(330, -1);
    left_section.add_css_class("cpu-box");

    let brand_title = Label::builder()
        .css_classes(["cpu-brand"])
        .halign(Align::Start)
        .build();
    left_section.append(&brand_title);

    let flow_box = FlowBox::new();
    flow_box.set_valign(Align::Start);
    flow_box.set_max_children_per_line(2);
    flow_box.set_selection_mode(SelectionMode::None);
    flow_box.set_row_spacing(6);
    flow_box.set_column_spacing(6);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&flow_box)
        .height_request(230)
        .build();

    let cores_wrapper = GtkBox::new(Orientation::Vertical, 0);
    cores_wrapper.add_css_class("cores-wrapper");
    cores_wrapper.append(&scroll);
    left_section.append(&cores_wrapper);

    let bottom_row = GtkBox::new(Orientation::Horizontal, 20);
    bottom_row.set_margin_top(15);
    bottom_row.set_halign(Align::Center);

    let box_use = GtkBox::new(Orientation::Vertical, 2);
    let lbl_use_title = Label::builder()
        .label("USAGE")
        .css_classes(["dash-title"])
        .halign(Align::Center)
        .build();
    let total_cpu_lbl = Label::builder()
        .label("0.0 %")
        .css_classes(["dash-value-small"])
        .halign(Align::Center)
        .build();
    box_use.append(&lbl_use_title);
    box_use.append(&total_cpu_lbl);

    let box_freq = GtkBox::new(Orientation::Vertical, 2);
    let lbl_freq_title = Label::builder()
        .label("FREQ")
        .css_classes(["dash-title"])
        .halign(Align::Center)
        .build();
    let max_freq_lbl = Label::builder()
        .label("N/A")
        .css_classes(["dash-value-small"])
        .halign(Align::Center)
        .build();
    box_freq.append(&lbl_freq_title);
    box_freq.append(&max_freq_lbl);

    let box_pwr = GtkBox::new(Orientation::Vertical, 2);
    let lbl_pwr_title = Label::builder()
        .label("POWER")
        .css_classes(["dash-title"])
        .halign(Align::Center)
        .build();
    let cpu_pwr_lbl = Label::builder()
        .label("N/A")
        .css_classes(["dash-value-small"])
        .halign(Align::Center)
        .build();
    box_pwr.append(&lbl_pwr_title);
    box_pwr.append(&cpu_pwr_lbl);

    bottom_row.append(&box_use);
    bottom_row.append(&box_freq);
    bottom_row.append(&box_pwr);
    left_section.append(&bottom_row);

    (
        left_section,
        brand_title,
        total_cpu_lbl,
        max_freq_lbl,
        cpu_pwr_lbl,
        flow_box,
    )
}

fn build_mid_panel() -> (GtkBox, Label, Label, ProgressBar, Label) {
    let mid_section = GtkBox::new(Orientation::Vertical, 15);
    mid_section.set_hexpand(true);
    mid_section.set_valign(Align::Center);
    mid_section.set_halign(Align::Center);

    let pwr_box = GtkBox::new(Orientation::Vertical, 5);
    pwr_box.set_halign(Align::Center);
    let pwr_title = Label::builder()
        .label("SYS POWER")
        .css_classes(["dash-title"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .build();
    let system_pwr_lbl = Label::builder()
        .label("N/A")
        .css_classes(["dash-value-small"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .build();
    pwr_box.append(&pwr_title);
    pwr_box.append(&system_pwr_lbl);

    let temp_box = GtkBox::new(Orientation::Vertical, 5);
    temp_box.set_halign(Align::Center);
    let temp_title = Label::builder()
        .label("SYS TEMP")
        .css_classes(["dash-title"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .build();
    let system_temp_lbl = Label::builder()
        .label("N/A")
        .css_classes(["dash-value-small"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .build();
    temp_box.append(&temp_title);
    temp_box.append(&system_temp_lbl);

    let mem_box = GtkBox::new(Orientation::Vertical, 5);
    mem_box.set_halign(Align::Center);
    let mem_title = Label::builder()
        .label("MEMORY")
        .css_classes(["dash-title"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .build();
    let mem_bar = ProgressBar::new();
    mem_bar.set_hexpand(true);
    mem_bar.set_height_request(12);
    mem_bar.add_css_class("mem-bar");
    let mem_lbl = Label::builder()
        .label("0.0 / 0.0 GB")
        .css_classes(["dash-value-small"])
        .halign(Align::Center)
        .justify(gtk4::Justification::Center)
        .build();

    let boost_btn = Button::with_label("Boost");
    boost_btn.add_css_class("boost-btn-style");
    boost_btn.set_margin_top(5);

    boost_btn.connect_clicked(|btn| {
        if let Some(root_w) = btn.root().and_then(|r| r.downcast::<Window>().ok()) {
            open_generic_info_dialog(
                Some(&root_w),
                "Memory Optimizer",
                "This feature will be enabled in future versions.",
            );
        }
    });

    mem_box.append(&mem_title);
    mem_box.append(&mem_bar);
    mem_box.append(&mem_lbl);
    mem_box.append(&boost_btn);

    mid_section.append(&pwr_box);
    mid_section.append(&temp_box);
    mid_section.append(&mem_box);

    (
        mid_section,
        system_pwr_lbl,
        system_temp_lbl,
        mem_bar,
        mem_lbl,
    )
}

fn create_gpu_card(gpu: &GpuSnapshot) -> GpuViews {
    let card_box = GtkBox::new(Orientation::Vertical, 0);
    card_box.add_css_class("gpu-card");
    let name_lbl = Label::builder()
        .label(&gpu.name)
        .css_classes(["gpu-name"])
        .halign(Align::Center)
        .build();

    let bar_overlay = gtk4::Overlay::new();
    bar_overlay.add_css_class("gpu-bar-overlay");
    bar_overlay.set_halign(Align::Center);
    let progress_bar = ProgressBar::new();
    progress_bar.set_orientation(Orientation::Vertical);
    progress_bar.set_inverted(true);
    progress_bar.add_css_class("gpu-bar");
    progress_bar.set_vexpand(true);
    let progress_text = Label::builder()
        .label("0%")
        .css_classes(["gpu-bar-text"])
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    bar_overlay.set_child(Some(&progress_bar));
    bar_overlay.add_overlay(&progress_text);

    let stats_box = GtkBox::new(Orientation::Vertical, 4);
    stats_box.add_css_class("gpu-text-box");

    let pwr_hbox = GtkBox::new(Orientation::Horizontal, 5);
    let pwr_lbl_title = Label::builder()
        .label("PWR:")
        .css_classes(["gpu-stat-lbl"])
        .build();
    let pwr_lbl_val = Label::builder()
        .label("N/A")
        .css_classes(["gpu-stat-val"])
        .hexpand(true)
        .halign(Align::End)
        .build();
    pwr_hbox.append(&pwr_lbl_title);
    pwr_hbox.append(&pwr_lbl_val);

    let temp_hbox = GtkBox::new(Orientation::Horizontal, 5);
    let temp_lbl_title = Label::builder()
        .label("TEMP:")
        .css_classes(["gpu-stat-lbl"])
        .build();
    let temp_lbl_val = Label::builder()
        .label("N/A")
        .css_classes(["gpu-stat-val"])
        .hexpand(true)
        .halign(Align::End)
        .build();
    temp_hbox.append(&temp_lbl_title);
    temp_hbox.append(&temp_lbl_val);

    let vbox_vram = GtkBox::new(Orientation::Horizontal, 5);
    let vram_lbl_title = Label::builder()
        .label("VRAM:")
        .css_classes(["gpu-stat-lbl"])
        .build();
    let vram_lbl_val = Label::builder()
        .label("N/A")
        .css_classes(["gpu-stat-val"])
        .hexpand(true)
        .halign(Align::End)
        .build();
    vbox_vram.append(&vram_lbl_title);
    vbox_vram.append(&vram_lbl_val);

    let freq_hbox = GtkBox::new(Orientation::Horizontal, 5);
    let freq_lbl_val = Label::builder()
        .label("N/A")
        .css_classes(["gpu-stat-val"])
        .hexpand(true)
        .halign(Align::End)
        .build();

    let f_lbl_title = Label::builder()
        .label("FREQ:")
        .css_classes(["gpu-stat-lbl"])
        .build();
    freq_hbox.append(&f_lbl_title);
    freq_hbox.append(&freq_lbl_val);

    stats_box.append(&pwr_hbox);
    stats_box.append(&temp_hbox);
    stats_box.append(&vbox_vram);
    stats_box.append(&freq_hbox);

    card_box.append(&name_lbl);
    card_box.append(&bar_overlay);
    card_box.append(&stats_box);

    GpuViews {
        container: card_box,
        bar: progress_bar,
        bar_lbl: progress_text,
        pwr_lbl: pwr_lbl_val,
        temp_lbl: temp_lbl_val,
        vram_lbl: vram_lbl_val,
        freq_lbl: freq_lbl_val,
    }
}

fn update_ui_animated(w: &DashboardWidgets, snap: &SystemSnapshot, vs: &VisualState) {
    w.cpu_brand_lbl.set_label(&snap.cpu_name);
    w.total_cpu_lbl.set_label(&format!("{:.1} %", vs.cpu_usage));

    if snap.cpu_freq_ghz > 0.0 {
        w.max_freq_lbl
            .set_label(&format!("{:.2} GHz", snap.cpu_freq_ghz));
    } else {
        w.max_freq_lbl.set_label("N/A");
    }

    if snap.cpu_power_w > 0.1 {
        w.cpu_pwr_lbl
            .set_label(&format!("{:.1} W", snap.cpu_power_w));
    } else {
        w.cpu_pwr_lbl.set_label("N/A");
    }

    if snap.system_total_power_w > 0.1 {
        w.system_pwr_lbl
            .set_label(&format!("{:.1} W", snap.system_total_power_w));
    } else {
        w.system_pwr_lbl.set_label("N/A");
    }

    if snap.system_avg_temp_c > 0.1 {
        w.system_temp_lbl
            .set_label(&format!("{:.0}°C", snap.system_avg_temp_c));
    } else {
        w.system_temp_lbl.set_label("N/A");
    }

    w.mem_bar.set_fraction(vs.mem_fraction);
    w.mem_lbl.set_label(&format!(
        "{:.1} / {:.1} GB",
        snap.mem_used_gb, snap.mem_total_gb
    ));

    let mut c_views = w.core_views.borrow_mut();
    if c_views.is_empty() && !snap.cores_freq.is_empty() {
        for i in 0..snap.cores_freq.len() {
            let mini_box = GtkBox::new(Orientation::Horizontal, 2);
            mini_box.add_css_class("mini-core");
            let lbl_id = Label::builder()
                .label(&format!("C{}:", i))
                .css_classes(["mini-core-lbl"])
                .build();
            let lbl_val = Label::builder()
                .label("0% | 0°C")
                .css_classes(["mini-core-val"])
                .build();
            mini_box.append(&lbl_id);
            mini_box.append(&lbl_val);
            w.flow_box.insert(&mini_box, -1);
            c_views.push(CoreViews { usage: lbl_val });
        }
    }

    let cpu_temp_str = snap
        .cpu_temp
        .map_or("N/A".to_string(), |t| format!("{:.0}°C", t));
    let mut usage_iter = snap.cores_usage.iter();

    for i in 0..c_views.len() {
        if let Some(view) = c_views.get(i) {
            let freq = snap.cores_freq.get(i).cloned().unwrap_or(0.0);
            if let Some(child) = w.flow_box.child_at_index(i as i32) {
                if freq == 0.0 {
                    child.add_css_class("core-offline");
                    view.usage.set_label("OFFLINE");
                } else {
                    child.remove_css_class("core-offline");
                    let usage = usage_iter.next().cloned().unwrap_or(0.0);
                    view.usage
                        .set_label(&format!("{:.0}% | {}", usage, cpu_temp_str));
                }
            }
        }
    }

    let mut views = w.gpu_views.borrow_mut();
    if views.len() != snap.gpus.len() {
        while let Some(child) = w.gpu_container.first_child() {
            w.gpu_container.remove(&child);
        }
        views.clear();

        if snap.gpus.is_empty() {
            let no_gpu = Label::builder()
                .label("No GPU Detected")
                .css_classes(["gpu-name"])
                .halign(Align::Center)
                .build();
            w.gpu_container.append(&no_gpu);
        } else {
            for g in &snap.gpus {
                let g_view = create_gpu_card(g);
                w.gpu_container.append(&g_view.container);
                views.push(g_view);
            }
        }
    }

    for (i, gpu) in snap.gpus.iter().enumerate() {
        if let Some(view) = views.get(i) {
            if let Some(current_util) = vs.gpus_util.get(i) {
                view.bar
                    .set_fraction((current_util / 100.0).clamp(0.0, 1.0));
                view.bar_lbl.set_label(&format!("{:.0}%", current_util));
            }

            if gpu.is_integrated && gpu.pwr_w == 0.0 {
                view.pwr_lbl.set_label("N/A");
            } else if gpu.pwr_w > 0.0 {
                view.pwr_lbl.set_label(&format!("{:.1} W", gpu.pwr_w));
            } else {
                view.pwr_lbl.set_label("N/A");
            }

            if gpu.temp_c > 0.0 {
                view.temp_lbl.set_label(&format!("{:.0}°C", gpu.temp_c));
            } else {
                view.temp_lbl.set_label("N/A");
            }

            if gpu.freq_mhz > 0.0 {
                view.freq_lbl.set_label(&format!("{:.0} MHz", gpu.freq_mhz));
            } else {
                view.freq_lbl.set_label("N/A");
            }

            if gpu.is_integrated {
                view.vram_lbl.set_label(&format!(
                    "{:.1}/{:.1} GB (Shared)",
                    gpu.vram_used_gb, gpu.vram_total_gb
                ));
            } else if gpu.vram_total_gb > 0.0 {
                view.vram_lbl.set_label(&format!(
                    "{:.1}/{:.1} GB",
                    gpu.vram_used_gb, gpu.vram_total_gb
                ));
            } else {
                view.vram_lbl.set_label("N/A");
            }
        }
    }
}

pub fn create(receiver: mpsc::Receiver<SystemSnapshot>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);

    let main_box = CenterBox::new();
    main_box.add_css_class("glass-monitor");
    main_box.set_size_request(830, 420);

    let (left_sec, cpu_brand_lbl, total_cpu_lbl, max_freq_lbl, cpu_pwr_lbl, flow_box) =
        build_cpu_panel();
    let (mid_sec, system_pwr_lbl, system_temp_lbl, mem_bar, mem_lbl) = build_mid_panel();
    let right_sec = GtkBox::new(Orientation::Horizontal, 15);
    right_sec.add_css_class("gpu-container");

    left_sec.set_margin_end(10);
    right_sec.set_margin_start(10);

    main_box.set_start_widget(Some(&left_sec));
    main_box.set_center_widget(Some(&mid_sec));
    main_box.set_end_widget(Some(&right_sec));

    let widgets = Rc::new(DashboardWidgets {
        cpu_brand_lbl,
        total_cpu_lbl,
        max_freq_lbl,
        cpu_pwr_lbl,
        system_pwr_lbl,
        system_temp_lbl,
        mem_bar,
        mem_lbl,
        core_views: RefCell::new(Vec::new()),
        gpu_views: RefCell::new(Vec::new()),
        gpu_container: right_sec,
        flow_box,
    });

    let target_snapshot = Rc::new(RefCell::new(SystemSnapshot::default()));
    let visual_state = Rc::new(RefCell::new(VisualState {
        cpu_usage: 0.0,
        mem_fraction: 0.0,
        gpus_util: Vec::new(),
    }));

    glib::timeout_add_local(
        Duration::from_millis(100),
        clone!(@strong target_snapshot => @default-return glib::ControlFlow::Continue, move || {
            while let Ok(snap) = receiver.try_recv() {
                *target_snapshot.borrow_mut() = snap;
            }
            glib::ControlFlow::Continue
        }),
    );

    glib::timeout_add_local(
        Duration::from_millis(33),
        clone!(@strong widgets, @strong target_snapshot, @strong visual_state => @default-return glib::ControlFlow::Continue, move || {
            let target = target_snapshot.borrow();
            let mut vs = visual_state.borrow_mut();

            vs.cpu_usage += (target.cpu_usage - vs.cpu_usage) * 0.15;

            let target_mem_frac = if target.mem_total_gb > 0.0 { target.mem_used_gb / target.mem_total_gb } else { 0.0 };
            vs.mem_fraction += (target_mem_frac - vs.mem_fraction) * 0.15;

            if vs.gpus_util.len() != target.gpus.len() {
                vs.gpus_util = target.gpus.iter().map(|g| g.util).collect();
            } else {
                for (i, gpu) in target.gpus.iter().enumerate() {
                    vs.gpus_util[i] += (gpu.util - vs.gpus_util[i]) * 0.15;
                }
            }

            update_ui_animated(&widgets, &target, &vs);
            glib::ControlFlow::Continue
        }),
    );

    container.append(&main_box);

    let toggle_btn = ToggleButton::builder()
        .halign(Align::Center)
        .css_classes(["toggle-arrow"])
        .icon_name("preferences-system-symbolic")
        .build();

    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(400)
        .margin_top(4)
        .margin_bottom(15)
        .build();

    // FIX: فراخوانی اشاره‌گر هوشمند std::boxed::Box به جای GtkBox مخدوش شده
    let apply_theme_hook: std::boxed::Box<dyn Fn(&str)> =
        std::boxed::Box::new(move |color: &str| {
            apply_theme(color);
        });

    let open_dialog_hook: std::boxed::Box<dyn Fn(Option<&Window>, &str, &str)> =
        std::boxed::Box::new(move |parent: Option<&Window>, title: &str, text: &str| {
            open_generic_info_dialog(parent, title, text);
        });

    let settings_view = settings::build_settings_panel(apply_theme_hook, open_dialog_hook);
    revealer.set_child(Some(&settings_view));

    toggle_btn.connect_toggled(clone!(@weak revealer => move |btn| {
        let is_active = btn.is_active();
        revealer.set_reveal_child(is_active);
        btn.set_icon_name("preferences-system-symbolic");
    }));

    let click_gesture = gtk4::GestureClick::new();
    click_gesture.connect_pressed(clone!(@weak toggle_btn => move |_, _, _, _| {
        if toggle_btn.is_active() {
            toggle_btn.set_active(false);
        }
    }));
    main_box.add_controller(click_gesture);

    container.append(&toggle_btn);
    container.append(&revealer);

    let saved_color = load_theme_from_disk();
    glib::idle_add_local(move || {
        apply_theme(&saved_color);
        glib::ControlFlow::Break
    });

    container
}
