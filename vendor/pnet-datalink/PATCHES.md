# pnet_datalink 0.35.0

Source: libpnet 0.35.0, distributed under the included MIT / Apache-2.0 licenses.

The Windows backend loads `Packet.dll` on demand from `System32\Npcap` or
`System32`, instead of linking the Npcap import library. The loaded library and
function table remain alive for the process lifetime. All required symbols are
resolved together. Without Npcap, opening a capture channel returns
`ErrorKind::NotFound` and the legacy capture interface list is empty. A failed
lookup is not cached, so the launcher can retry after installing Npcap without
restarting. Other platforms retain their upstream implementation.

EasyTier normally enumerates Windows adapters through `network-interface` and uses
WinDivert for fake TCP; its pnet fallback must not make Npcap mandatory at startup.
This patch also removes the build-time requirement for the Npcap SDK.

The workspace enables it through `[patch.crates-io]` in the root `Cargo.toml`.
Do not edit the copy under `Cargo.lock`'s registry cache; this directory is the
source of truth.
