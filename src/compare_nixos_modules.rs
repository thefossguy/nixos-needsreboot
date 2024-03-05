use std::{error::Error, fmt, fs};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{NEW_SYSTEM_PATH, OLD_SYSTEM_PATH};

#[derive(EnumIter)]
enum ModuleType {
    LinuxKernel,
    Systemd,
}

// for printing messages
impl fmt::Display for ModuleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LinuxKernel => write!(f, "Linux Kernel"),
            Self::Systemd => write!(f, "Systemd"),
        }
    }
}

impl ModuleType {
    fn get_nix_store_path(&self, use_old_path: bool) -> Result<String, Box<dyn Error>> {
        let suffix = match self {
            Self::LinuxKernel => "/kernel",
            Self::Systemd => "/systemd",
        };
        let strip_suffix = match self {
            Self::Systemd => false,
            Self::LinuxKernel => true,
        };

        let system_path = if use_old_path {
            OLD_SYSTEM_PATH.to_string()
        } else {
            NEW_SYSTEM_PATH.to_string()
        };

        let tmp_module_path = fs::read_link(system_path + suffix)?
            .into_os_string()
            .into_string()
            .expect("Cannot convert PathBuf to String");

        let nix_module_path = if strip_suffix {
            let split_module_path = tmp_module_path.split('/').collect::<Vec<&str>>();
            let mut module_dir = split_module_path
                .get(1..4)
                .ok_or("Cannot find the module's directory in /nix/store")?
                .join("/");
            module_dir.insert(0, '/');
            module_dir
        } else {
            tmp_module_path
        };

        Ok(nix_module_path)
    }

    fn get_linux_version(linux_path: &str) -> Result<String, Box<dyn Error>> {
        let lib_modules_path = fs::read_dir(linux_path)?
            .nth(0)
            .ok_or("Expected one directory in ".to_string() + linux_path)??
            .path()
            .into_os_string()
            .into_string()
            .expect("Cannot convert PathBuf to String");
        let linux_version = lib_modules_path.split('/').nth(6).ok_or(
            "Could not determine Linux kernel version from path: ".to_string() + &lib_modules_path,
        )?;

        Ok(linux_version.to_string())
    }

    fn get_systemd_version(systemd_path: &str) -> Result<String, Box<dyn Error>> {
        let split_systemd_path = systemd_path.split('-').collect::<Vec<&str>>();
        let systemd_version = split_systemd_path
            .get(2..)
            .ok_or("Could not determine Systemd version from path: ".to_string() + systemd_path)?
            .join("-");
        Ok(systemd_version)
    }

    fn get_version(&self) -> Result<(String, String), Box<dyn Error>> {
        let old_module_root_path = self.get_nix_store_path(true)?;
        let new_module_root_path = self.get_nix_store_path(false)?;

        let old_module_version: String;
        let new_module_version: String;

        match self {
            Self::LinuxKernel => {
                let linux_path = old_module_root_path + "/lib/modules";
                old_module_version = Self::get_linux_version(&linux_path)?;
                new_module_version = Self::get_linux_version(&linux_path)?;
            }
            Self::Systemd => {
                old_module_version = Self::get_systemd_version(&old_module_root_path)?;
                new_module_version = Self::get_systemd_version(&new_module_root_path)?;
            }
        }

        Ok((old_module_version, new_module_version))
    }
}

pub fn upgrades_available() -> Result<bool, Box<dyn Error>> {
    let mut needs_reboot = false;
    'x: for module in ModuleType::iter() {
        let (mut old_module_version, mut new_module_version) = module.get_version()?;

        if old_module_version != new_module_version {
            if old_module_version.len() != new_module_version.len() {
                if old_module_version.contains("-rc") && !new_module_version.contains("-rc") {
                    old_module_version = old_module_version.replace("-rc", ".");
                    new_module_version.push_str(".0");
                } else if new_module_version.contains("-rc") && !old_module_version.contains("-rc")
                {
                    new_module_version = new_module_version.replace("-rc", ".");
                    old_module_version.push_str(".0");
                } else if new_module_version.contains("-rc") && old_module_version.contains("-rc") {
                    new_module_version = new_module_version.replace("-rc", ".");
                    old_module_version = old_module_version.replace("-rc", ".");
                }
            }

            let old_version = &old_module_version.split('.').collect::<Vec<&str>>();
            let new_version = &new_module_version.split('.').collect::<Vec<&str>>();

            for (old, new) in old_version.iter().zip(new_version.iter()) {
                if new > old {
                    eprintln!("DEBUG: needs upgrading for module '{module}'");
                    needs_reboot = true;
                    break 'x;
                }
            }
        }
    }

    Ok(needs_reboot)
}
