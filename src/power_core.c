#include "power_core.h"
#include <glib.h>
#include <glib/gstdio.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void write_script(const char *path, const char *content) {
    FILE *f = fopen(path, "w");
    if (f) {
        fputs(content, f);
        fclose(f);
        g_chmod(path, 0755);
    }
}

void pwraxiom_ensure_scripts(void) {
    const char *config_home = g_get_user_config_dir();
    char dir_path[512];
    snprintf(dir_path, sizeof(dir_path), "%s/pwraxiom/scripts", config_home);
    g_mkdir_with_parents(dir_path, 0755);
    char path[512];

    snprintf(path, sizeof(path), "%s/ultra_save.sh", dir_path);
    write_script(path,
        "#!/bin/bash\n"
        "STATUS_FILE=$1\n"
        "CORES=$2\n"
        "INSTALL_CMD=$3\n"
        "echo 'STATUS:Checking Dependencies...' >> \"$STATUS_FILE\"\n"
        "eval \"$INSTALL_CMD\" >> /dev/null 2>&1\n"
        "sleep 0.4\n"
        "echo 'STATUS:Applying CPU Governor...' >> \"$STATUS_FILE\"\n"
        "cpupower frequency-set -g powersave >> /dev/null 2>&1\n"
        "echo 'STATUS:Setting P-States (Underclocking)...' >> \"$STATUS_FILE\"\n"
        "if [ \"$CORES\" -gt 2 ]; then\n"
        "  cpupower -c 2-$(($CORES-1)) frequency-set -d 800MHz -u 800MHz >> /dev/null 2>&1\n"
        "fi\n"
        "cpupower -c 0,1 frequency-set -d 800MHz -u 1.5GHz >> /dev/null 2>&1\n"
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 0 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null; fi\n"
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do\n"
        "  if [ -w \"$f\" ]; then echo power > \"$f\"; fi\n"
        "done 2>/dev/null\n"
        "sleep 0.4\n"
        "echo 'STATUS:Isolating dGPU (D3Cold)...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo OFF > /sys/kernel/debug/vgaswitcheroo/switch 2>/dev/null; fi\n"
        "echo 'STATUS:Limiting iGPU Frequencies...' >> \"$STATUS_FILE\"\n"
        "for d in /sys/class/drm/card*/gt_max_freq_mhz; do\n"
        "  if [ -w \"$d\" ]; then cat \"${d%%max*}min_freq_mhz\" > \"$d\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "sleep 0.4\n"
        "echo 'STATUS:Enforcing Advanced Link Power...' >> \"$STATUS_FILE\"\n"
        "for h in /sys/class/scsi_host/host*/link_power_management_policy; do\n"
        "  if [ -w \"$h\" ]; then echo min_power > \"$h\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo powersave > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi\n"
        "if [ -w /proc/sys/kernel/nmi_watchdog ]; then echo 0 > /proc/sys/kernel/nmi_watchdog 2>/dev/null; fi\n"
        "if [ -w /sys/module/snd_hda_intel/parameters/power_save ]; then echo 1 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null; fi\n"
        "echo 'STATUS:Finalizing Hardware Locks...' >> \"$STATUS_FILE\"\n"
        "sleep 0.5\n"
        "echo 'STATUS:Done' >> \"$STATUS_FILE\"\n"
        "exit 0\n"
    );

    snprintf(path, sizeof(path), "%s/save.sh", dir_path);
    write_script(path,
        "#!/bin/bash\n"
        "STATUS_FILE=$1\n"
        "CORES=$2\n"
        "INSTALL_CMD=$3\n"
        "echo 'STATUS:Checking Dependencies...' >> \"$STATUS_FILE\"\n"
        "eval \"$INSTALL_CMD\" >> /dev/null 2>&1\n"
        "sleep 0.4\n"
        "echo 'STATUS:Applying Governor...' >> \"$STATUS_FILE\"\n"
        "cpupower frequency-set -g powersave >> /dev/null 2>&1\n"
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do\n"
        "  if [ -w \"$f\" ]; then echo balance_power > \"$f\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 0 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null; fi\n"
        "sleep 0.4\n"
        "echo 'STATUS:Underclocking dGPU (Power Save)...' >> \"$STATUS_FILE\"\n"
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do\n"
        "  if [ -w \"$p\" ]; then echo low > \"$p\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "echo 'STATUS:Managing PCIe ASPM States...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo powersave > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi\n"
        "echo 'STATUS:Finalizing Configuration...' >> \"$STATUS_FILE\"\n"
        "sleep 0.5\n"
        "echo 'STATUS:Done' >> \"$STATUS_FILE\"\n"
        "exit 0\n"
    );

    snprintf(path, sizeof(path), "%s/balanced.sh", dir_path);
    write_script(path,
        "#!/bin/bash\n"
        "STATUS_FILE=$1\n"
        "CORES=$2\n"
        "INSTALL_CMD=$3\n"
        "echo 'STATUS:Checking Dependencies...' >> \"$STATUS_FILE\"\n"
        "eval \"$INSTALL_CMD\" >> /dev/null 2>&1\n"
        "sleep 0.4\n"
        "echo 'STATUS:Restoring Governor & Clocks...' >> \"$STATUS_FILE\"\n"
        "cpupower frequency-set -d 800MHz >> /dev/null 2>&1\n"
        "cpupower frequency-set -g schedutil >> /dev/null 2>&1\n"
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 1 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null; fi\n"
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do\n"
        "  if [ -w \"$f\" ]; then echo balance_performance > \"$f\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "sleep 0.4\n"
        "echo 'STATUS:Waking up GPUs...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo ON > /sys/kernel/debug/vgaswitcheroo/switch 2>/dev/null; fi\n"
        "for d in /sys/class/drm/card*/gt_max_freq_mhz; do\n"
        "  if [ -w \"$d\" ]; then m=$(cat \"${d%%max*}boost_freq_mhz\" 2>/dev/null || cat \"${d%%max*}RP0_freq_mhz\" 2>/dev/null); if [ -n \"$m\" ]; then echo \"$m\" > \"$d\" 2>/dev/null; fi; fi\n"
        "done 2>/dev/null\n"
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do\n"
        "  if [ -w \"$p\" ]; then echo auto > \"$p\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "sleep 0.4\n"
        "echo 'STATUS:Restoring System Defaults...' >> \"$STATUS_FILE\"\n"
        "for h in /sys/class/scsi_host/host*/link_power_management_policy; do\n"
        "  if [ -w \"$h\" ]; then echo max_performance > \"$h\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo default > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi\n"
        "if [ -w /proc/sys/kernel/nmi_watchdog ]; then echo 1 > /proc/sys/kernel/nmi_watchdog 2>/dev/null; fi\n"
        "if [ -w /sys/module/snd_hda_intel/parameters/power_save ]; then echo 0 > /sys/module/snd_hda_intel/parameters/power_save 2>/dev/null; fi\n"
        "echo 'STATUS:Finalizing Hardware Locks...' >> \"$STATUS_FILE\"\n"
        "sleep 0.5\n"
        "echo 'STATUS:Done' >> \"$STATUS_FILE\"\n"
        "exit 0\n"
    );

    snprintf(path, sizeof(path), "%s/perf.sh", dir_path);
    write_script(path,
        "#!/bin/bash\n"
        "STATUS_FILE=$1\n"
        "CORES=$2\n"
        "INSTALL_CMD=$3\n"
        "echo 'STATUS:Checking Dependencies...' >> \"$STATUS_FILE\"\n"
        "eval \"$INSTALL_CMD\" >> /dev/null 2>&1\n"
        "sleep 0.4\n"
        "echo 'STATUS:Applying Performance Governor...' >> \"$STATUS_FILE\"\n"
        "cpupower frequency-set -g performance >> /dev/null 2>&1\n"
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 1 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null; fi\n"
        "sleep 0.4\n"
        "echo 'STATUS:Pushing GPUs to High...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo ON > /sys/kernel/debug/vgaswitcheroo/switch 2>/dev/null; fi\n"
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do\n"
        "  if [ -w \"$p\" ]; then echo high > \"$p\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "sleep 0.4\n"
        "echo 'STATUS:Maximizing PCIe states...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo performance > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi\n"
        "echo 'STATUS:Finalizing Configuration...' >> \"$STATUS_FILE\"\n"
        "sleep 0.5\n"
        "echo 'STATUS:Done' >> \"$STATUS_FILE\"\n"
        "exit 0\n"
    );

    snprintf(path, sizeof(path), "%s/ultra_perf.sh", dir_path);
    write_script(path,
        "#!/bin/bash\n"
        "STATUS_FILE=$1\n"
        "CORES=$2\n"
        "INSTALL_CMD=$3\n"
        "echo 'STATUS:Checking Dependencies...' >> \"$STATUS_FILE\"\n"
        "eval \"$INSTALL_CMD\" >> /dev/null 2>&1\n"
        "sleep 0.4\n"
        "echo 'STATUS:Applying Ultra Performance Governor...' >> \"$STATUS_FILE\"\n"
        "cpupower frequency-set -g performance >> /dev/null 2>&1\n"
        "echo 'STATUS:Overclocking & Voltages...' >> \"$STATUS_FILE\"\n"
        "cpupower frequency-set -d 1.5GHz >> /dev/null 2>&1\n"
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 1 > /sys/devices/system/cpu/cpufreq/boost 2>/dev/null; fi\n"
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do if [ -w \"$f\" ]; then echo performance > \"$f\" 2>/dev/null; fi; done\n"
        "sleep 0.4\n"
        "echo 'STATUS:Maximizing GPU Clocks...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo ON > /sys/kernel/debug/vgaswitcheroo/switch 2>/dev/null; fi\n"
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do\n"
        "  if [ -w \"$p\" ]; then echo high > \"$p\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "for d in /sys/class/drm/card*/gt_max_freq_mhz; do\n"
        "  if [ -w \"$d\" ]; then cat \"${d%%max*}max_freq_mhz\" > \"$d\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "sleep 0.4\n"
        "echo 'STATUS:Enforcing Advanced PCIe ASPM...' >> \"$STATUS_FILE\"\n"
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo performance > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi\n"
        "for h in /sys/class/scsi_host/host*/link_power_management_policy; do\n"
        "  if [ -w \"$h\" ]; then echo max_performance > \"$h\" 2>/dev/null; fi\n"
        "done 2>/dev/null\n"
        "echo 'STATUS:Finalizing Hardware Locks...' >> \"$STATUS_FILE\"\n"
        "sleep 0.5\n"
        "echo 'STATUS:Done' >> \"$STATUS_FILE\"\n"
        "exit 0\n"
    );
}

char* pwraxiom_get_script_path(const char* mode) {
    const char *config_home = g_get_user_config_dir();
    if (strcmp(mode, "Ultra Save") == 0) return g_strdup_printf("%s/pwraxiom/scripts/ultra_save.sh", config_home);
    if (strcmp(mode, "Save") == 0) return g_strdup_printf("%s/pwraxiom/scripts/save.sh", config_home);
    if (strcmp(mode, "Balanced") == 0) return g_strdup_printf("%s/pwraxiom/scripts/balanced.sh", config_home);
    if (strcmp(mode, "Perf") == 0) return g_strdup_printf("%s/pwraxiom/scripts/perf.sh", config_home);
    return g_strdup_printf("%s/pwraxiom/scripts/ultra_perf.sh", config_home);
}