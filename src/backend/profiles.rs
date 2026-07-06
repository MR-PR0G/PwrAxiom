use crate::backend::executor::execute_batch;
use crate::backend::hardware::HardwareManager;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Profile {
    Custom,
    Balanced,
    Performance,
    Save,
    UltraSave,
}

pub struct ProfileRegistry;

impl ProfileRegistry {
    fn get_cpu_count() -> usize {
        let mut count = 0;
        while std::path::Path::new(&format!("/sys/devices/system/cpu/cpu{}", count)).exists() {
            count += 1;
        }
        if count == 0 { 1 } else { count }
    }

    pub fn apply_balanced(hw: &HardwareManager, password: &str) -> Result<(), String> {
        println!("[INFO] Initializing Balanced Profile optimization matrix...");
        let mut commands = Vec::new();
        let cpu_count = Self::get_cpu_count();

        println!("[DEBUG] System topology detected: {} logical cores.", cpu_count);

        for i in 0..cpu_count {
            commands.push(format!("echo 1 > /sys/devices/system/cpu/cpu{}/online 2>/dev/null || true", i));
        }
        println!("[INFO] Core lifecycle policy: Restoring all {} cores to online status.", cpu_count);

        if hw.has_intel_pstate() {
            println!("[DEBUG] Intel P-State active. Setting governor to balance_performance.");
            commands.push("echo balance_performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1".to_string());
        } else if hw.has_amd_pstate() {
            println!("[DEBUG] AMD P-State active. Setting autonomous governor paths.");
            commands.push("echo active | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1".to_string());
            commands.push("echo balance_performance | tee /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference > /dev/null 2>&1".to_string());
        } else {
            println!("[DEBUG] Generic CPURef engine detected. Mapping schedutil fallback.");
            commands.push("echo schedutil | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1 || echo powersave | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1".to_string());
        }

        if hw.has_pcie_aspm() {
            println!("[DEBUG] PCIe ASPM controller registered. Adjusting policy to default dynamic.");
            commands.push("echo default | tee /sys/module/pcie_aspm/parameters/policy > /dev/null 2>&1".to_string());
        }

        if hw.has_sata_storage() {
            println!("[DEBUG] SATA AHCI links found. Enforcing ALPM medium power mode.");
            commands.push("for i in /sys/class/scsi_host/host*/link_power_management_policy; do echo med_power_with_dipm > $i 2>/dev/null; done".to_string());
        }

        if hw.has_intel_hda_audio() {
            println!("[DEBUG] Sound bus registered. Activating codec standard power-saving timers.");
            commands.push("echo 1 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null".to_string());
            commands.push("echo Y > /sys/module/snd_hda_intel/parameters/power_save_controller 2>/dev/null".to_string());
        }

        commands.push("echo 1500 > /proc/sys/vm/dirty_writeback_centisecs 2>/dev/null".to_string());

        println!("[SUDO_EXEC] Dispatching payload batch to secure hardware execution pipeline.");
        let result = execute_batch(password, &commands);
        if result.is_ok() {
            println!("[INFO] Balanced Profile fully deployed to local hardware subsystems.");
        } else {
            println!("[CRITICAL] Balanced Profile deployment rejected by internal kernel runtime.");
        }
        result
    }

    pub fn apply_performance(hw: &HardwareManager, password: &str) -> Result<(), String> {
        println!("[INFO] Initializing Performance Profile optimization matrix...");
        let mut commands = Vec::new();
        let cpu_count = Self::get_cpu_count();

        for i in 0..cpu_count {
            commands.push(format!("echo 1 > /sys/devices/system/cpu/cpu{}/online 2>/dev/null || true", i));
        }
        println!("[INFO] Core lifecycle policy: Waking all {} cores for maximum throughput parallelism.", cpu_count);

        commands.push("echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1".to_string());
        
        if hw.has_amd_pstate() {
            println!("[DEBUG] Configuring AMD EPP parameters to maximum performance limits.");
            commands.push("echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference > /dev/null 2>&1".to_string());
        }

        if hw.has_intel_cpu() {
            println!("[DEBUG] Intel hardware platform detected. Disabling Turbo restrictive limits.");
            commands.push("echo 0 > /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null".to_string());
        } else if hw.has_amd_cpu() {
            println!("[DEBUG] AMD hardware platform detected. Unlocking Precision Boost frequencies.");
            commands.push("echo 1 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null".to_string());
        }

        if hw.has_pcie_aspm() {
            println!("[DEBUG] PCIe link sleep policies disabled to prevent bus latency spikes.");
            commands.push("echo performance | tee /sys/module/pcie_aspm/parameters/policy > /dev/null 2>&1".to_string());
        }

        if hw.has_sata_storage() {
            println!("[DEBUG] Inhibiting SATA ALPM link transitions to preserve link performance.");
            commands.push("for i in /sys/class/scsi_host/host*/link_power_management_policy; do echo max_performance > $i 2>/dev/null; done".to_string());
        }

        if hw.has_intel_hda_audio() {
            println!("[DEBUG] Deactivating audio controller sleep state to block popping noises.");
            commands.push("echo 0 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null".to_string());
            commands.push("echo N > /sys/module/snd_hda_intel/parameters/power_save_controller 2>/dev/null".to_string());
        }

        commands.push("echo 500 > /proc/sys/vm/dirty_writeback_centisecs 2>/dev/null".to_string());
        commands.push("echo 1 > /proc/sys/net/ipv4/tcp_low_latency 2>/dev/null".to_string());

        println!("[SUDO_EXEC] Dispatching payload batch to secure hardware execution pipeline.");
        let result = execute_batch(password, &commands);
        if result.is_ok() {
            println!("[INFO] Performance Profile successfully applied. Hardware bounds maximized.");
        } else {
            println!("[CRITICAL] Performance Profile application sequence crashed.");
        }
        result
    }

    pub fn apply_save(hw: &HardwareManager, password: &str) -> Result<(), String> {
        println!("[INFO] Initializing Power Save Profile optimization matrix...");
        let mut commands = Vec::new();
        let cpu_count = Self::get_cpu_count();

        for i in 0..cpu_count {
            commands.push(format!("echo 1 > /sys/devices/system/cpu/cpu{}/online 2>/dev/null || true", i));
        }

        commands.push("echo powersave | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1".to_string());
        
        if hw.has_amd_pstate() {
            println!("[DEBUG] Directing AMD energy consumption profile to strict preference lines.");
            commands.push("echo power | tee /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference > /dev/null 2>&1".to_string());
        }

        if hw.has_pcie_aspm() {
            println!("[DEBUG] Enabling PCIe active state power management structures.");
            commands.push("echo powersave | tee /sys/module/pcie_aspm/parameters/policy > /dev/null 2>&1".to_string());
        }

        if hw.has_sata_storage() {
            println!("[DEBUG] Enforcing aggressive SATA ALPM aggressive battery optimization limits.");
            commands.push("for i in /sys/class/scsi_host/host*/link_power_management_policy; do echo min_power > $i 2>/dev/null; done".to_string());
        }

        if hw.has_intel_hda_audio() {
            commands.push("echo 1 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null".to_string());
            commands.push("echo Y > /sys/module/snd_hda_intel/parameters/power_save_controller 2>/dev/null".to_string());
        }

        if hw.has_usb_buses() {
            println!("[DEBUG] Broadcasting auto-suspend rules across all discovered Linux USB endpoints.");
            commands.push("for i in /sys/bus/usb/devices/*/power/control; do echo auto > $i 2>/dev/null; done".to_string());
        }

        commands.push("echo 3000 > /proc/sys/vm/dirty_writeback_centisecs 2>/dev/null".to_string());

        println!("[SUDO_EXEC] Dispatching payload batch to secure hardware execution pipeline.");
        let result = execute_batch(password, &commands);
        if result.is_ok() {
            println!("[INFO] Power Save Profile deployed. Energy consumption footprint reduced.");
        } else {
            println!("[CRITICAL] Power Save Profile layout application encountered a terminal error.");
        }
        result
    }

    pub fn apply_ultrasave(hw: &HardwareManager, password: &str) -> Result<(), String> {
        println!("[INFO] Initializing Ultra Save Profile optimization matrix...");
        let mut commands = Vec::new();
        let cpu_count = Self::get_cpu_count();

        let keep_alive = (cpu_count / 4).max(1);
        println!("[INFO] Core lifecycle policy: Parking cores. Active: {}/{}", keep_alive, cpu_count);

        for i in 0..cpu_count {
            if i < keep_alive {
                commands.push(format!("echo 1 > /sys/devices/system/cpu/cpu{}/online 2>/dev/null || true", i));
            } else {
                commands.push(format!("echo 0 > /sys/devices/system/cpu/cpu{}/online 2>/dev/null || true", i));
            }
        }

        commands.push("echo powersave | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor > /dev/null 2>&1".to_string());
        
        if hw.has_amd_pstate() {
            commands.push("echo power | tee /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference > /dev/null 2>&1".to_string());
        }

        if hw.has_intel_cpu() {
            println!("[DEBUG] Clamping Intel frequencies: Hard locking Turbo limits.");
            commands.push("echo 1 > /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null".to_string());
        } else if hw.has_amd_cpu() {
            println!("[DEBUG] Clamping AMD frequencies: Hard killing CPUBoost pipelines.");
            commands.push("echo 0 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null".to_string());
        }

        if hw.has_pcie_aspm() {
            println!("[DEBUG] Forcing PCIe system controllers into deep L1 link sleep trees.");
            commands.push("echo powersupersave | tee /sys/module/pcie_aspm/parameters/policy > /dev/null 2>&1".to_string());
        }

        if hw.has_sata_storage() {
            commands.push("for i in /sys/class/scsi_host/host*/link_power_management_policy; do echo min_power > $i 2>/dev/null; done".to_string());
        }

        if hw.has_intel_hda_audio() {
            commands.push("echo 1 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null".to_string());
            commands.push("echo Y > /sys/module/snd_hda_intel/parameters/power_save_controller 2>/dev/null".to_string());
        }

        if hw.has_usb_buses() {
            commands.push("for i in /sys/bus/usb/devices/*/power/control; do echo auto > $i 2>/dev/null; done".to_string());
        }

        commands.push("echo 6000 > /proc/sys/vm/dirty_writeback_centisecs 2>/dev/null".to_string());
        commands.push("echo 0 > /proc/sys/kernel/nmi_watchdog 2>/dev/null".to_string());

        println!("[SUDO_EXEC] Dispatching payload batch to secure hardware execution pipeline.");
        let result = execute_batch(password, &commands);
        if result.is_ok() {
            println!("[INFO] Ultra Save Profile deployed. System throttled to maximum preservation boundaries.");
        } else {
            println!("[CRITICAL] Ultra Save Profile runtime optimization vector deployment rejected.");
        }
        result
    }

    pub fn apply_custom(hw: &HardwareManager, password: &str) -> Result<(), String> {
        println!("[INFO] Initializing Custom Setup Profile rule map execution...");
        let commands = Vec::new();
        execute_batch(password, &commands)
    }
}