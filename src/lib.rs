use std::error::Error;
use std::fs;

mod compare_nixos_modules;

pub static OLD_SYSTEM_PATH: &str = "/run/booted-system";
pub static NEW_SYSTEM_PATH: &str = "/nix/var/nix/profiles/system";

#[derive(Debug)]
pub enum NeedsReboot {
    /// No reboot needed, running the latest NixOS generation
    IsLatest,
    /// No reboot needed, no updates available
    NoUpdates,
    /// Reboot needed, updates available
    Updates(String),
}

pub fn needs_reboot() -> Result<NeedsReboot, Box<dyn Error>> {
    let old_system_id = fs::read_to_string(OLD_SYSTEM_PATH.to_string() + "/nixos-version")?;
    let new_system_id = fs::read_to_string(NEW_SYSTEM_PATH.to_string() + "/nixos-version")?;

    Ok(if old_system_id == new_system_id {
        NeedsReboot::IsLatest
    } else {
        let reason = compare_nixos_modules::upgrades_available()?;
        if reason.is_empty() {
            NeedsReboot::NoUpdates
        } else {
            NeedsReboot::Updates(reason)
        }
    })
}
