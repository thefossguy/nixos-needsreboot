use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

mod compare_nixos_modules;

pub static OLD_SYSTEM_PATH: &str = "/run/booted-system";
pub static NEW_SYSTEM_PATH: &str = "/nix/var/nix/profiles/system";
pub static NIXOS_NEEDS_REBOOT: &str = "/var/run/reboot-required";

fn main() -> Result<(), Box<dyn Error>> {
    let env_args: Vec<String> = env::args().collect();
    let dry_run = env_args.contains(&String::from("--dry-run"));

    if env_args.contains(&String::from("--version")) {
        println!("{}: v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let user = env::var_os("USER")
        .unwrap()
        .into_string()
        .expect("Cannot convert OsString into String");
    if user != "root" && !dry_run {
        println!("ERROR: please run this as root");
        println!("HINT: use the '--dry-run' option");
        std::process::exit(1);
    }

    if Path::new("/nix/var/nix/profiles/system").exists() {
        let old_system_id = fs::read_to_string(OLD_SYSTEM_PATH.to_string() + "/nixos-version")?;
        let new_system_id = fs::read_to_string(NEW_SYSTEM_PATH.to_string() + "/nixos-version")?;

        if Path::new(NIXOS_NEEDS_REBOOT).exists() {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let _ = handle.write_all(&fs::read(NIXOS_NEEDS_REBOOT)?);
            let _ = handle.flush();
        } else if old_system_id == new_system_id {
            eprintln!("DEBUG: you are using the latest NixOS generation, no need to reboot");
        } else {
            let reason = compare_nixos_modules::upgrades_available()?;
            if reason.is_empty() {
                eprintln!("DEBUG: no updates available, moar uptime!!!");
            } else {
                if dry_run {
                    println!("{reason}");
                } else {
                    fs::write(NIXOS_NEEDS_REBOOT, reason)?;
                }
            }
        }
    } else {
        eprintln!("This binary is intedned to run only on NixOS.");
        std::process::exit(1);
    }

    Ok(())
}
