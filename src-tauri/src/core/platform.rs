use std::fs;
use std::path::Path;

use goblin::Object;

/// The architecture of java runtime
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub enum Architecture {
    X86,
    X64,
    Arm64,
    FatFile,
    Unknown,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Windows,
    Linux,
    #[serde(rename = "osx")]
    macOS,
}

impl OS {
    pub fn current() -> Self {
        match tauri_plugin_os::platform() {
            "windows" => Self::Windows,
            "macos" => Self::macOS,
            _ => Self::Linux,
        }
    }

    pub fn is_current(&self) -> bool {
        return *self == Self::current();
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum Compability {
    Perfect,
    Translation,
    No,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub(crate) enum ArchitectureError {
    /// The architecture of java runtime is unknown
    UnknownArchitecture,
    /// The file is invalid
    InvalidFile,
}

impl Architecture {
    /// read architecture of java runtime from the pe head
    #[cfg(target_os = "windows")]
    pub fn read_header(java_path: &Path) -> Result<Self, ArchitectureError> {
        use goblin::pe;
        let buffer = fs::read(java_path).map_err(|_| ArchitectureError::InvalidFile)?;
        let buffer_arch = Object::parse(&buffer).map_err(|_| ArchitectureError::InvalidFile)?;
        match buffer_arch {
            Object::PE(pe) => match pe.header.coff_header.machine {
                pe::header::COFF_MACHINE_X86 => Ok(Architecture::X86),
                pe::header::COFF_MACHINE_X86_64 => Ok(Architecture::X64),
                pe::header::COFF_MACHINE_ARM64 => Ok(Architecture::Arm64),
                _ => Err(ArchitectureError::UnknownArchitecture),
            },
            _ => Err(ArchitectureError::InvalidFile),
        }
    }

    /// read architecture of java runtime from the elf head
    #[cfg(target_os = "linux")]
    pub fn read_header(java_path: &Path) -> Result<Self, ArchitectureError> {
        use goblin::elf;
        let buffer = fs::read(java_path).map_err(|_| ArchitectureError::InvalidFile)?;
        let buffer_arch = Object::parse(&buffer).map_err(|_| ArchitectureError::InvalidFile)?;
        match buffer_arch {
            Object::Elf(elf) => match elf.header.e_machine {
                elf::header::EM_386 => Ok(Architecture::X86),
                elf::header::EM_X86_64 => Ok(Architecture::X64),
                elf::header::EM_AARCH64 => Ok(Architecture::Arm64),
                _ => Err(ArchitectureError::UnknownArchitecture),
            },
            _ => Err(ArchitectureError::InvalidFile),
        }
    }

    /// read architecture of java runtime from the mach-o head
    #[cfg(target_os = "macos")]
    pub fn read_header(java_path: &Path) -> Result<Self, ArchitectureError> {
        use goblin::mach;
        let buffer = fs::read(java_path).map_err(|_| ArchitectureError::InvalidFile)?;
        let buffer_arch = Object::parse(&buffer).map_err(|_| ArchitectureError::InvalidFile)?;
        match buffer_arch {
            Object::Mach(mach::Mach::Fat(_multi_arch)) => Ok(Architecture::FatFile),
            Object::Mach(mach::Mach::Binary(macho)) => match macho.header.cputype {
                mach::cputype::CPU_TYPE_X86 => Ok(Architecture::X86),
                mach::cputype::CPU_TYPE_X86_64 => Ok(Architecture::X64),
                mach::cputype::CPU_TYPE_ARM64 => Ok(Architecture::Arm64),
                _ => Err(ArchitectureError::UnknownArchitecture),
            },
            _ => Err(ArchitectureError::InvalidFile),
        }
    }
}

/// assert compability of java runtime according to the current OS
pub(super) fn assert_compability(arch: &Architecture) -> Compability {
    let system = OS::current();
    let current_arch = tauri_plugin_os::arch();
    match system {
        OS::macOS | OS::Windows => match current_arch {
            "x86_64" => match arch {
                Architecture::X64 => Compability::Perfect,
                Architecture::X86 => Compability::Translation,
                _ => Compability::No,
            },
            "aarch64" => match arch {
                Architecture::Arm64 => Compability::Perfect,
                Architecture::X64 => Compability::Translation,
                _ => Compability::No,
            },
            _ => Compability::Unknown,
        },
        OS::Linux => match current_arch {
            "x86_64" => match arch {
                Architecture::X64 => Compability::Perfect,
                _ => Compability::No,
            },
            "aarch64" => match arch {
                Architecture::Arm64 => Compability::Perfect,
                _ => Compability::No,
            },
            _ => Compability::Unknown,
        },
    }
}

#[cfg(target_os = "macos")]
#[test]
fn read_header_test() {
    let arch = Architecture::read_header(Path::new(
        "/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home/bin/java",
    ));
    println!("{:?}", arch);
}

#[cfg(target_os = "linux")]
#[test]
fn read_linux_test() {
    let arch = Architecture::read_header(Path::new("/Users/amagicpear/Downloads/ncmdump"));
    assert_eq!(arch, Ok(Architecture::X64));
    let arch2 =
        Architecture::read_header(Path::new("/Users/amagicpear/Downloads/mihomo-linux-arm64"));
    assert_eq!(arch2, Ok(Architecture::Arm64));
}

#[cfg(target_os = "windows")]
#[test]
pub fn read_pe_test() {
    let arch = Architecture::read_header(Path::new(
        "/Users/amagicpear/Downloads/mihomo-windows-amd64-v1.exe",
    ));
    assert_eq!(arch, Ok(Architecture::X64));
    let arch2 = Architecture::read_header(Path::new(
        "/Users/amagicpear/Downloads/mihomo-windows-386.exe",
    ));
    assert_eq!(arch2, Ok(Architecture::X86));
}

#[test]
fn system_arch_test() {
    let current_arch = tauri_plugin_os::arch();
    println!("current arch: {}", current_arch);
    let system = tauri_plugin_os::platform();
    println!("system: {}", system);
}
