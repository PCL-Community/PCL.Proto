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

// macOS 的写法是有意保留的，同时别让 clippy 因为变体名以 OS 结尾而报错。
#[allow(non_camel_case_types, clippy::enum_variant_names)]
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
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::macOS,
            _ => Self::Linux,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy, Default, PartialEq, Eq)]
pub enum Compatibility {
    Perfect,
    Translation,
    No,
    #[default]
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum ArchitectureError {
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
pub(crate) fn assert_compatibility(arch: &Architecture) -> Compatibility {
    compatibility_for(OS::current(), host_architecture(), arch)
}

/// The host can differ from the launcher process when Rosetta or Windows emulation is in use.
pub fn host_architecture() -> &'static str {
    static ARCH: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        #[cfg(windows)]
        for key in ["PROCESSOR_ARCHITEW6432", "PROCESSOR_ARCHITECTURE"] {
            if let Ok(value) = std::env::var(key) {
                match value.to_ascii_lowercase().as_str() {
                    "arm64" => return "aarch64".into(),
                    "amd64" => return "x86_64".into(),
                    "x86" => return "x86".into(),
                    _ => (),
                }
            }
        }
        #[cfg(target_os = "macos")]
        if std::process::Command::new("/usr/sbin/sysctl")
            .args(["-n", "hw.optional.arm64"])
            .output()
            .is_ok_and(|output| output.status.success() && output.stdout.starts_with(b"1"))
        {
            return "aarch64".into();
        }
        #[cfg(target_os = "linux")]
        if let Ok(output) = std::process::Command::new("uname").arg("-m").output() {
            match String::from_utf8_lossy(&output.stdout).trim() {
                "aarch64" | "arm64" => return "aarch64".into(),
                "x86_64" | "amd64" => return "x86_64".into(),
                "i386" | "i486" | "i586" | "i686" => return "x86".into(),
                _ => (),
            }
        }
        std::env::consts::ARCH.into()
    });
    &ARCH
}

pub(crate) fn compatibility_for(
    system: OS,
    current_arch: &str,
    arch: &Architecture,
) -> Compatibility {
    if matches!(system, OS::macOS) && matches!(arch, Architecture::FatFile) {
        return Compatibility::Perfect;
    }
    match system {
        OS::macOS | OS::Windows => match current_arch {
            "x86_64" => match arch {
                Architecture::X64 => Compatibility::Perfect,
                Architecture::X86 if matches!(system, OS::Windows) => Compatibility::Translation,
                _ => Compatibility::No,
            },
            "aarch64" => match arch {
                Architecture::Arm64 => Compatibility::Perfect,
                Architecture::X64 => Compatibility::Translation,
                Architecture::X86 if matches!(system, OS::Windows) => Compatibility::Translation,
                _ => Compatibility::No,
            },
            "x86" | "i686" => match arch {
                Architecture::X86 => Compatibility::Perfect,
                _ => Compatibility::No,
            },
            _ => Compatibility::Unknown,
        },
        OS::Linux => match current_arch {
            "x86_64" => match arch {
                Architecture::X64 => Compatibility::Perfect,
                Architecture::X86 => Compatibility::Translation,
                _ => Compatibility::No,
            },
            "aarch64" => match arch {
                Architecture::Arm64 => Compatibility::Perfect,
                _ => Compatibility::No,
            },
            "x86" | "i686" => match arch {
                Architecture::X86 => Compatibility::Perfect,
                _ => Compatibility::No,
            },
            _ => Compatibility::Unknown,
        },
    }
}
