use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use nixos_needsreboot::NeedsReboot;

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
        if Path::new(NIXOS_NEEDS_REBOOT).exists() {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            let _ = handle.write_all(&fs::read(NIXOS_NEEDS_REBOOT)?);
            let _ = handle.flush();
            std::process::exit(2);
        } else {
            match nixos_needsreboot::needs_reboot()? {
                NeedsReboot::IsLatest => eprintln!("DEBUG: you are using the latest NixOS generation, no need to reboot"),
                NeedsReboot::NoUpdates => eprintln!("DEBUG: no updates available, moar uptime!!!"),
                NeedsReboot::Updates(reason) => {
                    if dry_run {
                        println!("{reason}");
                    } else {
                        fs::write(NIXOS_NEEDS_REBOOT, reason)?;
                    }
                    std::process::exit(2);
                }
            }
        }
    } else {
        eprintln!("This binary is intedned to run only on NixOS.");
        std::process::exit(1);
    }

    Ok(())
}
