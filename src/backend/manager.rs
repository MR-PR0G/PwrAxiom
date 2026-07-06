use std::sync::{Mutex, OnceLock};
use crate::backend::hardware::HardwareManager;
use crate::backend::profiles::{Profile, ProfileRegistry};

pub struct BackendManager {
    current_profile: Profile,
    password: String,
    hardware: HardwareManager,
}

impl BackendManager {
    fn new() -> Self {
        Self {
            current_profile: Profile::Balanced,
            password: String::new(),
            hardware: HardwareManager::new(),
        }
    }

    pub fn global() -> &'static Mutex<Self> {
        static INSTANCE: OnceLock<Mutex<BackendManager>> = OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(Self::new()))
    }

    pub fn current_profile(&self) -> Profile {
        self.current_profile
    }

    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }

    pub fn apply_profile(&mut self, profile: Profile) -> Result<(), String> {
        let pwd = self.password.clone();
        match profile {
            Profile::Balanced => ProfileRegistry::apply_balanced(&self.hardware, &pwd)?,
            Profile::Performance => ProfileRegistry::apply_performance(&self.hardware, &pwd)?,
            Profile::Save => ProfileRegistry::apply_save(&self.hardware, &pwd)?,
            Profile::UltraSave => ProfileRegistry::apply_ultrasave(&self.hardware, &pwd)?,
            Profile::Custom => ProfileRegistry::apply_custom(&self.hardware, &pwd)?,
        }
        self.current_profile = profile;
        Ok(())
    }
}