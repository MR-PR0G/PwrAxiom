use std::sync::OnceLock;

pub struct HardwareManager;

impl HardwareManager {
    pub fn new() -> Self {
        Self
    }

    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<HardwareManager> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    pub fn has_intel_pstate(&self) -> bool {
        std::path::Path::new("/sys/devices/system/cpu/intel_pstate").exists()
    }

    pub fn has_amd_pstate(&self) -> bool {
        std::path::Path::new("/sys/devices/system/cpu/amd_pstate").exists()
    }

    pub fn has_pcie_aspm(&self) -> bool {
        std::path::Path::new("/sys/module/pcie_aspm").exists()
    }

    pub fn has_sata_storage(&self) -> bool {
        std::path::Path::new("/sys/class/scsi_host").exists()
    }

    pub fn has_intel_hda_audio(&self) -> bool {
        std::path::Path::new("/sys/module/snd_hda_intel").exists()
    }

    pub fn has_intel_cpu(&self) -> bool {
        std::path::Path::new("/sys/devices/system/cpu/intel_pstate").exists()
    }

    pub fn has_amd_cpu(&self) -> bool {
        std::path::Path::new("/sys/devices/system/cpu/cpufreq/boost").exists()
    }

    pub fn has_usb_buses(&self) -> bool {
        std::path::Path::new("/sys/bus/usb/devices").exists()
    }
}