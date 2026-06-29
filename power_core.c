#include "power_core.h"
#include <glib.h>

char* get_script_ultra_perf(int cores, const char* install_cmd) {
    return g_strdup_printf(
        "rm -f /tmp/pwraxiom_status; touch /tmp/pwraxiom_status; chmod 666 /tmp/pwraxiom_status; exec > /tmp/pwraxiom_status 2>&1; "
        "echo 'STATUS:Checking Dependencies...'; %s; sleep 0.4; "
        "echo 'STATUS:Applying Ultra Performance Governor...'; "
        "cpupower frequency-set -g performance >/dev/null 2>&1; "
        "echo 'STATUS:Overclocking & Voltages...'; "
        "cpupower frequency-set -d 1.5GHz >/dev/null 2>&1; "
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 1 > /sys/devices/system/cpu/cpufreq/boost; fi; "
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do if [ -w \"$f\" ]; then echo performance > \"$f\"; fi; done; sleep 0.4; "
        "echo 'STATUS:Maximizing GPU Clocks...'; "
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo ON > /sys/kernel/debug/vgaswitcheroo/switch; fi; "
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do if [ -w \"$p\" ]; then echo high > \"$p\"; fi; done; "
        "for d in /sys/class/drm/card*/gt_min_freq_mhz; do if [ -w \"$d\" ]; then cat \"${d%%min*}max_freq_mhz\" > \"$d\" 2>/dev/null; fi; done; sleep 0.4; "
        "echo 'STATUS:Enforcing Advanced PCIe ASPM...'; "
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo performance > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi; "
        "for h in /sys/class/scsi_host/host*/link_power_management_policy; do if [ -w \"$h\" ]; then echo max_performance > \"$h\"; fi; done; "
        "echo 'STATUS:Finalizing Hardware Locks...'; sleep 0.5; echo 'STATUS:Done'; exit 0", 
        install_cmd
    );
}

char* get_script_perf(const char* install_cmd) {
    return g_strdup_printf(
        "rm -f /tmp/pwraxiom_status; touch /tmp/pwraxiom_status; chmod 666 /tmp/pwraxiom_status; exec > /tmp/pwraxiom_status 2>&1; "
        "echo 'STATUS:Checking Dependencies...'; %s; sleep 0.4; "
        "echo 'STATUS:Applying Performance Governor...'; "
        "cpupower frequency-set -g performance >/dev/null 2>&1; "
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 1 > /sys/devices/system/cpu/cpufreq/boost; fi; sleep 0.4; "
        "echo 'STATUS:Pushing GPUs to High...'; "
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo ON > /sys/kernel/debug/vgaswitcheroo/switch; fi; "
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do if [ -w \"$p\" ]; then echo high > \"$p\"; fi; done; sleep 0.4; "
        "echo 'STATUS:Maximizing PCIe states...'; "
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo performance > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi; "
        "echo 'STATUS:Finalizing Configuration...'; sleep 0.5; echo 'STATUS:Done'; exit 0", 
        install_cmd
    );
}

char* get_script_balanced(const char* install_cmd) {
    return g_strdup_printf(
        "rm -f /tmp/pwraxiom_status; touch /tmp/pwraxiom_status; chmod 666 /tmp/pwraxiom_status; exec > /tmp/pwraxiom_status 2>&1; "
        "echo 'STATUS:Checking Dependencies...'; %s; sleep 0.4; "
        "echo 'STATUS:Restoring Governor & Clocks...'; "
        "cpupower frequency-set -d 800MHz >/dev/null 2>&1; "
        "cpupower frequency-set -g schedutil >/dev/null 2>&1; "
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 1 > /sys/devices/system/cpu/cpufreq/boost; fi; "
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do if [ -w \"$f\" ]; then echo balance_performance > \"$f\"; fi; done; sleep 0.4; "
        "echo 'STATUS:Waking up GPUs...'; "
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo ON > /sys/kernel/debug/vgaswitcheroo/switch; fi; "
        "for d in /sys/class/drm/card*/gt_max_freq_mhz; do if [ -w \"$d\" ]; then m=$(cat \"${d%%max*}boost_freq_mhz\" 2>/dev/null || cat \"${d%%max*}RP0_freq_mhz\" 2>/dev/null); if [ -n \"$m\" ]; then echo \"$m\" > \"$d\" 2>/dev/null; fi; fi; done; "
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do if [ -w \"$p\" ]; then echo auto > \"$p\"; fi; done; sleep 0.4; "
        "echo 'STATUS:Restoring System Defaults...'; "
        "for h in /sys/class/scsi_host/host*/link_power_management_policy; do if [ -w \"$h\" ]; then echo max_performance > \"$h\"; fi; done; "
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo default > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi; "
        "if [ -w /proc/sys/kernel/nmi_watchdog ]; then echo 1 > /proc/sys/kernel/nmi_watchdog; fi; "
        "if [ -w /sys/module/snd_hda_intel/parameters/power_save ]; then echo 0 > /sys/module/snd_hda_intel/parameters/power_save; fi; "
        "echo 'STATUS:Finalizing Hardware Locks...'; sleep 0.5; echo 'STATUS:Done'; exit 0", 
        install_cmd
    );
}

char* get_script_save(int cores, const char* install_cmd) {
    return g_strdup_printf(
        "rm -f /tmp/pwraxiom_status; touch /tmp/pwraxiom_status; chmod 666 /tmp/pwraxiom_status; exec > /tmp/pwraxiom_status 2>&1; "
        "echo 'STATUS:Checking Dependencies...'; %s; sleep 0.4; "
        "echo 'STATUS:Applying Governor...'; "
        "cpupower frequency-set -g powersave >/dev/null 2>&1; "
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do if [ -w \"$f\" ]; then echo balance_power > \"$f\"; fi; done; "
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 0 > /sys/devices/system/cpu/cpufreq/boost; fi; sleep 0.4; "
        "echo 'STATUS:Underclocking dGPU (Power Save)...'; "
        "for p in /sys/class/drm/card*/device/power_dpm_force_performance_level; do if [ -w \"$p\" ]; then echo low > \"$p\"; fi; done; "
        "echo 'STATUS:Managing PCIe ASPM States...'; "
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo powersave > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi; "
        "echo 'STATUS:Finalizing Configuration...'; sleep 0.5; echo 'STATUS:Done'; exit 0", 
        install_cmd
    );
}

char* get_script_ultra_save(int cores, const char* install_cmd) {
    return g_strdup_printf(
        "rm -f /tmp/pwraxiom_status; touch /tmp/pwraxiom_status; chmod 666 /tmp/pwraxiom_status; exec > /tmp/pwraxiom_status 2>&1; "
        "echo 'STATUS:Checking Dependencies...'; %s; sleep 0.4; "
        "echo 'STATUS:Applying CPU Governor...'; "
        "cpupower frequency-set -g powersave >/dev/null 2>&1; "
        "echo 'STATUS:Setting P-States (Underclocking)...'; "
        "cpupower -c 2-%d frequency-set -d 800MHz -u 800MHz >/dev/null 2>&1; "
        "cpupower -c 0,1 frequency-set -d 800MHz -u 1.5GHz >/dev/null 2>&1; "
        "if [ -w /sys/devices/system/cpu/cpufreq/boost ]; then echo 0 > /sys/devices/system/cpu/cpufreq/boost; fi; "
        "for f in /sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference; do if [ -w \"$f\" ]; then echo power > \"$f\"; fi; done; sleep 0.4; "
        "echo 'STATUS:Isolating dGPU (D3Cold)...'; "
        "if [ -w /sys/kernel/debug/vgaswitcheroo/switch ]; then echo OFF > /sys/kernel/debug/vgaswitcheroo/switch; fi; "
        "echo 'STATUS:Limiting iGPU Frequencies...'; "
        "for d in /sys/class/drm/card*/gt_max_freq_mhz; do if [ -w \"$d\" ]; then cat \"${d%%max*}min_freq_mhz\" > \"$d\" 2>/dev/null; fi; done; sleep 0.4; "
        "echo 'STATUS:Enforcing Advanced Link Power...'; "
        "for h in /sys/class/scsi_host/host*/link_power_management_policy; do if [ -w \"$h\" ]; then echo min_power > \"$h\"; fi; done; "
        "if [ -w /sys/module/pcie_aspm/parameters/policy ]; then echo powersave > /sys/module/pcie_aspm/parameters/policy 2>/dev/null; fi; "
        "if [ -w /proc/sys/kernel/nmi_watchdog ]; then echo 0 > /proc/sys/kernel/nmi_watchdog; fi; "
        "if [ -w /sys/module/snd_hda_intel/parameters/power_save ]; then echo 1 > /sys/module/snd_hda_intel/parameters/power_save; fi; "
        "echo 'STATUS:Finalizing Hardware Locks...'; sleep 0.5; echo 'STATUS:Done'; exit 0", 
        install_cmd, cores - 1
    );
}