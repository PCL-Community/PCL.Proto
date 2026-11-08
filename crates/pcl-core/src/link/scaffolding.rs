//! Bridge to the embedded Terracotta networking stack.
//!
//! The launcher-facing pieces (settings, announcements, accounts) live in the
//! application; this module only translates plain values into Terracotta calls.

use terracotta::{controller, rooms::Room};

pub use terracotta::controller::{
    get_meta as get_terracotta_meta, get_state as get_terracotta_state,
    set_link_options as set_options, set_public_nodes, set_vendor, set_waiting, test_nat,
};
pub use terracotta::{ConnectionType, LinkOptions, ProtocolPreference as LinkProtocol};

pub fn set_host_starting(mc_port: u16, player: String) -> Result<String, String> {
    controller::set_host_starting(mc_port, Some(player))
        .ok_or_else(|| "set host starting failed".to_string())
}

pub fn set_guesting(room_code: String, player: String) -> Result<(), String> {
    let room = Room::from(&room_code).ok_or("invalid_code")?;
    if controller::set_guesting(room, Some(player), vec![]) {
        Ok(())
    } else {
        Err("join_failed".to_string())
    }
}
