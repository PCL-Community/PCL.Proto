//! Referenced from JavaData.cs of PCL-Community/PCL.Neo/PCL.Neo.Core
//! [PLC.Neo.Core](https://github.com/PCL-Community/PCL.Neo) | MIT license

use std::{collections::HashSet, env, fs, path::Path, process::Command};

#[derive(Debug, PartialEq, serde::Serialize, Clone)]
enum Architecture {
    X86,
    X64,
    Arm64,
    FatFile,
    Unknown,
}

#[derive(Debug, serde::Serialize, Clone)]
enum Compability {
    Perfect,
    Translation,
    No,
    Unknown,
}

/// struct of java runtime
#[derive(Debug, serde::Serialize, Clone)]
pub struct JavaRuntime {
    directory_path: String,
    pub(crate) is_user_imported: bool,
    version: String,
    slug_version: i32,
    is_64_bit: bool,
    architecture: Architecture,
    compability: Compability,
    is_jdk: bool,
    java_exe: String,
    implementor: Option<String>,
}

impl PartialEq for JavaRuntime {
    fn eq(&self, other: &Self) -> bool {
        self.directory_path == other.directory_path
    }
}

#[derive(Debug)]
pub enum JavaRuntimeConstructorError {
    MissingFile,
    InvalidRuntime,
}

impl JavaRuntime {
    /// read release file of java runtime
    fn read_release_file(release_file: &Path) -> (Option<String>, Option<String>, Architecture) {
        let release_content = std::fs::read_to_string(release_file).unwrap();

        let mut implementor = None;
        let mut version = None;
        let mut architecture = Architecture::Unknown;

        for line in release_content.lines() {
            if line.starts_with("IMPLEMENTOR=") {
                implementor = Some(
                    line.split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("JAVA_VERSION=") {
                version = Some(
                    line.split('=')
                        .nth(1)
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("OS_ARCH=") {
                let arch = line.split('=').nth(1).unwrap_or("").trim_matches('"');
                architecture = match arch {
                    "x86_64" => Architecture::X64,
                    "aarch64" => Architecture::Arm64,
                    "i386" | "i686" => Architecture::X86,
                    // 可以根据需要添加更多架构映射
                    _ => Architecture::Unknown,
                };
            }

            // 如果已经获取了所有需要的信息，可以提前结束循环
            if !implementor.is_none() && !version.is_none() && architecture != Architecture::Unknown
            {
                break;
            }
        }

        (implementor, version, architecture)
    }

    /// parse java version string to slug version number
    fn parse_to_slug_version(version: &str) -> Option<i32> {
        let mut version_split: std::str::Split<'_, char> = version.split('.');
        // for purpose of distinguish java 8 or 11 and later
        // example:
        // java 8: 1.8.0_362
        // java 11: 11.0.19
        let first = version_split.next();
        if let Some(first) = first {
            if first == "1" {
                if let Some(second) = version_split.next() {
                    return second.parse().ok();
                }
            } else {
                return first.parse().ok();
            }
        }
        None
    }

    /// read architecture of java runtime from the elf/pe head
    fn read_architecture(java_path: &str) -> Architecture {
        Architecture::Unknown
    }
    /// search java runtime in system
    pub async fn search() -> Vec<Self> {
        let mut collect_paths: HashSet<String> = HashSet::new();
        let home_dir = env::home_dir().unwrap();
        // TODO: check java home
        // search macOS specific directory
        #[cfg(target_os = "macos")]
        {
            pub fn search_macos(base_dir: &Path) -> HashSet<String> {
                let mut result = HashSet::new();
                if !base_dir.exists() || !base_dir.is_dir() {
                    println!("macOS search path not exists: {:?}", base_dir);
                    return result;
                }
                if let Ok(entries) = fs::read_dir(base_dir) {
                    for entry in entries.flatten() {
                        let entry_path = entry.path();
                        if entry_path.is_dir() {
                            let java_path = entry_path.join("Contents/Home/bin/java");
                            if java_path.exists() {
                                result.insert(java_path.to_string_lossy().into_owned());
                            }
                        }
                    }
                }
                result
            }
            collect_paths.extend(search_macos(Path::new("/Library/Java/JavaVirtualMachines")));
            collect_paths.extend(search_macos(Path::new(
                &home_dir.join("Library/Java/JavaVirtualMachines"),
            )));
        }
        // search PATH
        {
            if let Ok(path_var) = env::var("PATH") {
                for path in env::split_paths(&path_var) {
                    let exe_path = path.join(if cfg!(target_os = "windows") {
                        "java.exe"
                    } else {
                        "java"
                    });
                    if exe_path.exists() {
                        collect_paths.insert(exe_path.to_string_lossy().into_owned());
                    }
                }
            }
        }
        // walk windows directory
        // #[cfg(target_os = "windows")]
        // {
        //     for dirve in (b'A'..=b'Z')
        //         .map(|c| format!("{}:", c as char))
        //         .map(PathBuf::from)
        //         .collect::<Vec<PathBuf>>()
        //     {
        //         if let Ok(metadata) = fs::metadata(&dirve){
        //             if metadata.is_dir(){
        //             }
        //         }
        //     }
        // }
        // 使用try_from映射valid_paths到结果
        collect_paths
            .iter()
            .filter_map(|path| Self::try_from(path.as_str()).ok())
            .collect()
    }
}

impl TryFrom<&str> for JavaRuntime {
    type Error = JavaRuntimeConstructorError;
    fn try_from(java_path: &str) -> Result<Self, Self::Error> {
        // println!("[java] 创建JavaRuntime: {java_path}");
        let java_path = Path::new(java_path);
        if !java_path.exists() {
            return Err(JavaRuntimeConstructorError::MissingFile);
        }
        let directory = java_path.parent().unwrap();
        // 检查是否有javac来判断是否是JDK
        let is_jdk = directory
            .join(if cfg!(target_os = "windows") {
                "javac.exe"
            } else {
                "javac"
            })
            .exists();

        // 尝试读取RELEASE文件的信息
        let mut version: Option<String> = None;
        let mut implementor: Option<String> = None;
        let mut architecture: Architecture = Architecture::Unknown;
        if let Some(parent_dir) = directory.parent() {
            let release_file = parent_dir.join("release");
            if release_file.exists() {
                let (imp, ver, arch) = Self::read_release_file(&release_file);
                implementor = imp;
                version = ver;
                architecture = arch;
            }
        }

        // 若版本未被设置，运行 java -version 获取版本
        if version.is_none() {
            // 如果Error返回InvalidRuntime
            let output = Command::new(java_path)
                .arg("-version")
                .output()
                .map_err(|_| JavaRuntimeConstructorError::InvalidRuntime)?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version_regex = regex::Regex::new(r#"version\s+"([\d._]+)"#).unwrap();
            if let Some(captures) = version_regex.captures(&stderr) {
                version = Some(captures[1].to_string());
            }
        }

        // 设置slug Version
        let slug_version = Self::parse_to_slug_version(version.as_deref().unwrap_or_default());

        Ok(JavaRuntime {
            directory_path: directory.to_string_lossy().to_string(),
            is_user_imported: false,
            version: version.unwrap_or_default(),
            slug_version: slug_version.unwrap_or(0),
            is_64_bit: architecture != Architecture::X86 && architecture != Architecture::Unknown,
            architecture,
            compability: Compability::Unknown,
            is_jdk,
            java_exe: java_path.to_string_lossy().to_string(),
            implementor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_test1() {
        println!(
            "{:?}",
            JavaRuntime::try_from(
                "/Library/Java/JavaVirtualMachines/zulu-21.jdk/Contents/Home/bin/java",
            )
            .unwrap()
        );
    }

    #[test]
    fn java_test_2() {
        let java_runtime = JavaRuntime::try_from("/usr/bin/java").unwrap();
        println!("{:?}", java_runtime);
        assert_eq!(java_runtime.version, "24.0.2");
    }

    #[tokio::test]
    async fn java_search() {
        let java_runtimes = JavaRuntime::search().await;
        println!("{:?}", java_runtimes);
    }
}
