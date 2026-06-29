#include "monitor.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <unistd.h>
#include <sys/sysinfo.h>

static GtkWidget *cpu_usage_label = NULL;
static GtkWidget **core_boxes = NULL;
static GtkWidget **core_labels = NULL;
static int num_cores = 0;

typedef struct {
    char name[128];
    GtkWidget *box;
    GtkWidget *label;
} GpuInfo;

static GpuInfo *gpus = NULL;
static int num_gpus = 0;

static GtkWidget *bat_label = NULL;

static unsigned long long prev_total = 0;
static unsigned long long prev_idle = 0;

static void read_system_stats() {
    unsigned long long user, nice, system, idle, iowait, irq, softirq, steal;
    FILE *stat_f = fopen("/proc/stat", "r");
    if (stat_f) {
        char buffer[256];
        if (fgets(buffer, sizeof(buffer), stat_f)) {
            sscanf(buffer, "cpu %llu %llu %llu %llu %llu %llu %llu %llu", 
                   &user, &nice, &system, &idle, &iowait, &irq, &softirq, &steal);
        }
        fclose(stat_f);
    }
    
    unsigned long long total = user + nice + system + idle + iowait + irq + softirq + steal;
    unsigned long long current_idle = idle + iowait;
    
    unsigned long long total_diff = total - prev_total;
    unsigned long long idle_diff = current_idle - prev_idle;
    
    double usage = 0.0;
    if (total_diff > 0) {
        usage = (double)(total_diff - idle_diff) / total_diff * 100.0;
    }
    prev_total = total;
    prev_idle = current_idle;
    
    char usage_text[64];
    snprintf(usage_text, sizeof(usage_text), "CPU USAGE: %.1f%%", usage);
    gtk_label_set_text(GTK_LABEL(cpu_usage_label), usage_text);

    const char *current_mode = get_current_mode();
    int is_ultra_save = (strcmp(current_mode, "Ultra Save") == 0);
    int is_save = (strcmp(current_mode, "Save") == 0);

    for (int i = 0; i < num_cores; i++) {
        char path[128];
        snprintf(path, sizeof(path), "/sys/devices/system/cpu/cpu%d/cpufreq/scaling_cur_freq", i);
        FILE *f = fopen(path, "r");
        
        gtk_widget_remove_css_class(core_boxes[i], "core-red");
        gtk_widget_remove_css_class(core_boxes[i], "core-yellow");
        gtk_widget_remove_css_class(core_boxes[i], "core-green");

        if (f) {
            int freq = 0;
            if(fscanf(f, "%d", &freq) == 1) {
                freq /= 1000; 
                char txt[32];
                snprintf(txt, sizeof(txt), "C%d\n%d", i + 1, freq);
                gtk_label_set_text(GTK_LABEL(core_labels[i]), txt);

                if (is_ultra_save) {
                    if (i < 2) gtk_widget_add_css_class(core_boxes[i], "core-yellow");
                    else gtk_widget_add_css_class(core_boxes[i], "core-red");
                } else if (is_save) {
                    gtk_widget_add_css_class(core_boxes[i], "core-yellow");
                } else {
                    gtk_widget_add_css_class(core_boxes[i], "core-green");
                }
            }
            fclose(f);
        } else {
            char txt[32];
            snprintf(txt, sizeof(txt), "C%d\nOFF", i + 1);
            gtk_label_set_text(GTK_LABEL(core_labels[i]), txt);
            gtk_widget_add_css_class(core_boxes[i], "core-red");
        }
    }

    int gpu_off = 0;
    FILE *vga = fopen("/sys/kernel/debug/vgaswitcheroo/switch", "r");
    if (vga) {
        char state[64];
        if (fgets(state, sizeof(state), vga)) {
            if (strstr(state, "Off")) gpu_off = 1;
        }
        fclose(vga);
    }

    for (int i = 0; i < num_gpus; i++) {
        gtk_widget_remove_css_class(gpus[i].box, "core-red");
        gtk_widget_remove_css_class(gpus[i].box, "core-yellow");
        gtk_widget_remove_css_class(gpus[i].box, "core-green");

        if (i > 0 && (gpu_off || is_ultra_save)) {
            gtk_widget_add_css_class(gpus[i].box, "core-red");
            char txt[256];
            snprintf(txt, sizeof(txt), "%s\n<span size='small'>Disabled</span>", gpus[i].name);
            gtk_label_set_markup(GTK_LABEL(gpus[i].label), txt);
        } else {
            char path[128];
            snprintf(path, sizeof(path), "/sys/class/drm/card%d/gt_cur_freq_mhz", i);
            FILE *f = fopen(path, "r");
            if (f) {
                int freq = 0;
                fscanf(f, "%d", &freq);
                fclose(f);
                char txt[256];
                snprintf(txt, sizeof(txt), "%s\n<span size='small'>%d MHz</span>", gpus[i].name, freq);
                gtk_label_set_markup(GTK_LABEL(gpus[i].label), txt);
            } else {
                char txt[256];
                snprintf(txt, sizeof(txt), "%s\n<span size='small'>Active</span>", gpus[i].name);
                gtk_label_set_markup(GTK_LABEL(gpus[i].label), txt);
            }
            
            if (is_ultra_save || is_save) gtk_widget_add_css_class(gpus[i].box, "core-yellow");
            else gtk_widget_add_css_class(gpus[i].box, "core-green");
        }
    }

    FILE *bat_f = fopen("/sys/class/power_supply/BAT0/power_now", "r");
    FILE *cap_f = fopen("/sys/class/power_supply/BAT0/capacity", "r");
    if (!bat_f) bat_f = fopen("/sys/class/power_supply/BAT1/power_now", "r");
    if (!cap_f) cap_f = fopen("/sys/class/power_supply/BAT1/capacity", "r");

    if (bat_f && cap_f) {
        long power_uw;
        int capacity;
        fscanf(bat_f, "%ld", &power_uw);
        fscanf(cap_f, "%d", &capacity);
        fclose(bat_f);
        fclose(cap_f);
        
        double watts = power_uw / 1000000.0;
        char bat_text[128];
        snprintf(bat_text, sizeof(bat_text), "<span size='large' weight='bold'>%.1f W</span>\n<span size='x-small'>%d%% Cap</span>", watts, capacity);
        gtk_label_set_markup(GTK_LABEL(bat_label), bat_text);
    } else {
        if(bat_f) fclose(bat_f);
        if(cap_f) fclose(cap_f);
        gtk_label_set_markup(GTK_LABEL(bat_label), "<span size='large' weight='bold'>AC</span>\n<span size='x-small'>Power</span>");
    }
}

static gboolean on_timeout(gpointer data) {
    read_system_stats();
    return G_SOURCE_CONTINUE;
}

GtkWidget* create_monitor_widget(void) {
    GtkCssProvider *p = gtk_css_provider_new();
    gtk_css_provider_load_from_string(p,
        ".glass-monitor { background: rgba(18,18,18,0.85); border: 1px solid rgba(255,255,255,0.06); border-radius: 12px; padding: 8px 12px; }"
        ".section-title { font-size: 9px; color: #aaa; font-weight: bold; margin-bottom: 4px; }"
        ".core-box { border-radius: 6px; padding: 3px; min-width: 38px; margin: 1px; transition: all 0.3s ease; }"
        ".core-label { font-size: 9px; font-weight: bold; }"
        ".core-red { background: rgba(255,50,50,0.25); border: 1px solid rgba(255,50,50,0.5); color: #ffb3b3; }"
        ".core-yellow { background: rgba(255,200,50,0.25); border: 1px solid rgba(255,200,50,0.5); color: #ffe6b3; }"
        ".core-green { background: rgba(50,255,50,0.15); border: 1px solid rgba(50,255,50,0.3); color: #b3ffb3; }"
        ".gpu-box { border-radius: 6px; padding: 4px 8px; margin-bottom: 3px; }"
        ".gpu-label { font-size: 10px; font-weight: bold; }"
        ".bat-box { background: rgba(30,30,30,0.5); border-radius: 6px; padding: 8px; color: white; }"
    );
    gtk_style_context_add_provider_for_display(gdk_display_get_default(), GTK_STYLE_PROVIDER(p), 800);

    GtkWidget *main_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 15);
    gtk_widget_add_css_class(main_box, "glass-monitor");

    GtkWidget *cpu_section = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    cpu_usage_label = gtk_label_new("CPU USAGE: 0%");
    gtk_widget_set_halign(cpu_usage_label, GTK_ALIGN_START);
    gtk_widget_add_css_class(cpu_usage_label, "section-title");
    gtk_box_append(GTK_BOX(cpu_section), cpu_usage_label);

    GtkWidget *flowbox = gtk_flow_box_new();
    gtk_flow_box_set_max_children_per_line(GTK_FLOW_BOX(flowbox), 12);
    gtk_flow_box_set_selection_mode(GTK_FLOW_BOX(flowbox), GTK_SELECTION_NONE);
    
    num_cores = get_nprocs();
    core_boxes = g_malloc(num_cores * sizeof(GtkWidget*));
    core_labels = g_malloc(num_cores * sizeof(GtkWidget*));

    for (int i = 0; i < num_cores; i++) {
        core_boxes[i] = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
        gtk_widget_add_css_class(core_boxes[i], "core-box");
        
        core_labels[i] = gtk_label_new("--");
        gtk_widget_set_halign(core_labels[i], GTK_ALIGN_CENTER);
        gtk_label_set_justify(GTK_LABEL(core_labels[i]), GTK_JUSTIFY_CENTER);
        gtk_widget_add_css_class(core_labels[i], "core-label");
        
        gtk_box_append(GTK_BOX(core_boxes[i]), core_labels[i]);
        gtk_flow_box_insert(GTK_FLOW_BOX(flowbox), core_boxes[i], -1);
    }
    gtk_box_append(GTK_BOX(cpu_section), flowbox);
    gtk_box_append(GTK_BOX(main_box), cpu_section);

    GtkWidget *sep1 = gtk_separator_new(GTK_ORIENTATION_VERTICAL);
    gtk_box_append(GTK_BOX(main_box), sep1);

    GtkWidget *gpu_section = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    GtkWidget *gpu_title = gtk_label_new("GRAPHICS");
    gtk_widget_set_halign(gpu_title, GTK_ALIGN_START);
    gtk_widget_add_css_class(gpu_title, "section-title");
    gtk_box_append(GTK_BOX(gpu_section), gpu_title);

    FILE *gpu_f = popen("lspci | grep -i 'vga\\|3d' | sed 's/.*: //'", "r");
    char lines[4][128];
    num_gpus = 0;
    if (gpu_f) {
        while (fgets(lines[num_gpus], sizeof(lines[0]), gpu_f) && num_gpus < 4) {
            lines[num_gpus][strcspn(lines[num_gpus], "\n")] = 0;
            if (strlen(lines[num_gpus]) > 20) { lines[num_gpus][17] = '.'; lines[num_gpus][18] = '.'; lines[num_gpus][19] = '\0'; }
            num_gpus++;
        }
        pclose(gpu_f);
    }
    if (num_gpus == 0) { strcpy(lines[0], "Unknown GPU"); num_gpus = 1; }

    gpus = g_malloc(num_gpus * sizeof(GpuInfo));
    for (int i = 0; i < num_gpus; i++) {
        strcpy(gpus[i].name, lines[i]);
        gpus[i].box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
        gtk_widget_add_css_class(gpus[i].box, "gpu-box");
        
        gpus[i].label = gtk_label_new(lines[i]);
        gtk_widget_set_halign(gpus[i].label, GTK_ALIGN_START);
        gtk_widget_add_css_class(gpus[i].label, "gpu-label");
        
        gtk_box_append(GTK_BOX(gpus[i].box), gpus[i].label);
        gtk_box_append(GTK_BOX(gpu_section), gpus[i].box);
    }
    gtk_box_append(GTK_BOX(main_box), gpu_section);

    GtkWidget *sep2 = gtk_separator_new(GTK_ORIENTATION_VERTICAL);
    gtk_box_append(GTK_BOX(main_box), sep2);

    GtkWidget *bat_section = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    GtkWidget *bat_title = gtk_label_new("POWER");
    gtk_widget_set_halign(bat_title, GTK_ALIGN_START);
    gtk_widget_add_css_class(bat_title, "section-title");
    gtk_box_append(GTK_BOX(bat_section), bat_title);

    GtkWidget *b_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_add_css_class(b_box, "bat-box");
    bat_label = gtk_label_new("--");
    gtk_widget_set_halign(bat_label, GTK_ALIGN_CENTER);
    gtk_label_set_justify(GTK_LABEL(bat_label), GTK_JUSTIFY_CENTER);
    gtk_box_append(GTK_BOX(b_box), bat_label);
    gtk_box_append(GTK_BOX(bat_section), b_box);
    gtk_box_append(GTK_BOX(main_box), bat_section);

    read_system_stats();
    g_timeout_add(1500, on_timeout, NULL);

    return main_box;
}