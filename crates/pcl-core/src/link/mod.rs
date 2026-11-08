#[cfg(feature = "link")]
pub mod discovery;
pub mod mcping;
#[cfg(feature = "link")]
pub mod scaffolding;

/// Initialize the optional Terracotta bridge with a host-owned identity file.
#[cfg(feature = "link")]
pub fn init(identifier_path: std::path::PathBuf) {
    terracotta::init_lib(identifier_path);
}

/// Paths and defaults that a launcher needs when wiring the Lightweight LAN
/// (Terracotta/Scaffolding) multiplayer feature.
#[cfg(feature = "link")]
pub use scaffolding::{LinkOptions, LinkProtocol};
