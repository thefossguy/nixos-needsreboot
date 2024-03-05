use std::error::Error;
use std::fs;
use std::path::Path;

mod compare_nixos_modules;

pub static OLD_SYSTEM_PATH: &str = "/run/booted-system";
pub static NEW_SYSTEM_PATH: &str = "/nix/var/nix/profiles/system";
pub static NIXOS_NEEDS_REBOOT: &str = "/var/run/reboot-required";

fn main() -> Result<(), Box<dyn Error>> {
    let user = std::env::var_os("USER")
        .unwrap()
        .into_string()
        .expect("Cannot convert OsString into String");
    if user != "root" {
        println!("ERROR: please run this as root.");
        std::process::exit(1);
    }

    if Path::new("/nix/var/nix/profiles/system").exists() {
        let needs_reboot = Path::new(NIXOS_NEEDS_REBOOT).exists();
        let old_system_id = fs::read_to_string(OLD_SYSTEM_PATH.to_string() + "/nixos-version")?;
        let new_system_id = fs::read_to_string(NEW_SYSTEM_PATH.to_string() + "/nixos-version")?;

        if old_system_id == new_system_id {
            eprintln!("DEBUG: you are using the latest NixOS generation, no need to reboot");
        } else if needs_reboot || compare_nixos_modules::upgrades_available()? {
            fs::File::create(NIXOS_NEEDS_REBOOT)?;
        } else {
            eprintln!("DEBUG: no updates available, moar uptime!!!");
        }
    } else {
        eprintln!("This binary is intedned to run only on NixOS.");
    }

    Ok(())
}
