//! macOS 应用包内的只读前端资源，通过 Tauri 的资源协议提供。
//!
//! 启动器自带的辅助资源由 [`crate::assets`] 解析，这里只处理前端资源。

#[cfg(not(dev))]
use std::path::{Path, PathBuf};

#[cfg(not(dev))]
mod frontend {
    use super::*;
    use crate::assets::bundle_root;
    use std::{borrow::Cow, collections::BTreeMap};
    use tauri::utils::assets::{AssetKey, AssetsIter, CspHash};

    pub struct FrontendAssets(BTreeMap<String, PathBuf>);

    impl FrontendAssets {
        pub fn new() -> std::io::Result<Self> {
            let root = bundle_root(&tauri::utils::platform::current_exe()?)
                .ok_or_else(|| std::io::Error::other("Run PCL.Proto from its .app bundle"))?
                .join("frontend");
            Self::from_directory(&root)
        }

        fn from_directory(root: &Path) -> std::io::Result<Self> {
            let mut files = BTreeMap::new();
            let mut directories = vec![root.to_owned()];
            while let Some(directory) = directories.pop() {
                for entry in std::fs::read_dir(directory)? {
                    let entry = entry?;
                    let path = entry.path();
                    let kind = entry.file_type()?;
                    if kind.is_dir() {
                        directories.push(path);
                    } else if kind.is_file() {
                        let key =
                            format!("/{}", path.strip_prefix(root).unwrap().to_string_lossy());
                        files.insert(key, path);
                    }
                }
            }
            if !files.contains_key("/index.html") {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Missing frontend/index.html in the app bundle",
                ));
            }
            Ok(Self(files))
        }
    }

    impl<R: tauri::Runtime> tauri::Assets<R> for FrontendAssets {
        fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
            match std::fs::read(self.0.get(key.as_ref())?) {
                Ok(bytes) => Some(Cow::Owned(bytes)),
                Err(error) => {
                    log::error!("Could not read bundle asset {}: {error}", key.as_ref());
                    None
                }
            }
        }

        fn iter(&self) -> Box<AssetsIter<'_>> {
            Box::new(self.0.iter().filter_map(|(key, path)| {
                std::fs::read(path)
                    .ok()
                    .map(|bytes| (Cow::Borrowed(key.as_str()), Cow::Owned(bytes)))
            }))
        }

        fn csp_hashes(&self, _html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
            Box::new(std::iter::empty())
        }
    }
}

#[cfg(not(dev))]
pub use frontend::FrontendAssets;
