# Application packaging

Use `bun run build` (or `bun run tauri build`) to build distributable applications.
Tauri automatically merges `src-tauri/tauri.macos.conf.json` on macOS, including
Intel, Apple Silicon, and universal builds. No separate resource-copy step is needed.

## macOS

```text
PCL.Proto.app/
└── Contents/
    ├── Info.plist
    ├── MacOS/
    │   └── pcl-proto
    └── Resources/
        ├── icon.icns
        ├── frontend/      # Vite output: HTML, JS, CSS, images, fonts
        └── launcher/      # JAR helpers, Log4j XML, ARM64 library mapping
```

The executable contains native code and Tauri's runtime metadata. The frontend
uses Tauri's local asset protocol with a filesystem asset provider, preserving
the existing IPC and SPA routing. Launcher helpers are passed directly to Java
from the bundle. Terracotta's standalone web server is not linked into this app,
so its separate web archive is not part of the PCL.Proto executable or bundle.

Resource paths are resolved relative to the running executable, so moving or
renaming the complete `.app` is supported. Distribute the complete `.app` via the
DMG or Tauri's app archive; the executable in `Contents/MacOS` alone is insufficient.
Bundle resources are read-only at runtime and must be included before code signing.

`bun run dev` continues to use Vite and reads helper files from the checkout.
The helper files live in `crates/pcl-core/resources/{launch,installer}`. macOS
packaging copies them into the bundle, and the app supplies those paths to core.
The macOS app disables core's `embedded-resources` feature. Its executable contains
no embedded helper JARs, Log4j XML, or ARM64 library mapping. Core's default feature
still provides embedded resources for other consumers; a host can disable it with
`default-features = false` and supply its own helper paths and ARM64 mapping.
Missing external files fail explicitly instead of falling back to an extracted
cache. Use package-specific builds when distributing the app; `cargo build
--workspace --all-features` intentionally enables core's embedded resources too.
Configuration, accounts, Minecraft instances, downloaded runtime files, and caches
stay in their existing writable locations. Downloaded Mesa loader files always
use the writable configuration resource directory, including in a macOS bundle.
Existing extracted launcher resources are left in place for older installed
versions, but the macOS app no longer uses them.
Build-time API configuration retains its existing handling; the root `.env` is
never included as a bundle resource.

## Windows and Linux

Windows is distributed as a portable ZIP, not an MSI or NSIS setup installer.
Build with `bun run build`, then run `bun run package:portable`. The ZIP in
`target/portable` contains `PCL.Proto.exe`, `WinDivert64.sys`, its license, and a
README. Extract the entire directory and run `PCL.Proto.exe` directly; there is no
prerequisite script or wrapper. No executable packer or self-extracting app
wrapper is used.

`WinDivert64.sys` stays beside the executable and is loaded on demand by EasyTier.
Npcap is not a build dependency: the vendored `pnet_datalink` backend loads
`Packet.dll` from `System32\Npcap` or `System32` only when the WinDivert path is
unavailable, instead of linking it into the import table. On first use of the
multiplayer lobby, the launcher downloads the pinned Npcap installer, verifies its
SHA-256 and Authenticode signature, and starts the official installer with
administrator permission. The free Npcap license does not grant redistribution
rights or silent installation, so its DLLs and installer are not release assets.
See [PATCHES.md](../vendor/pnet-datalink/PATCHES.md), the
[Npcap license](https://github.com/nmap/npcap/blob/master/LICENSE), and the
[installation guide](https://npcap.com/guide/npcap-users-guide.html).

WebView2 is a Tauri runtime prerequisite and is not managed by the launcher.

For Windows development, run `bun run setup:windows` once. This prepares the
Protobuf compiler and WinDivert driver under `target/build-tools`. `bun run dev`
and `bun run build` reject missing build tools; the Npcap SDK is no longer needed.
Build scripts stage `WinDivert64.sys` beside the executable and test binaries.
If invoking Cargo directly, set `PROTOC` to the prepared `bin/protoc.exe` (or use
your existing system toolchain).

CI downloads checksum-pinned native dependencies; only the WinDivert driver is
needed on the Windows runner.

On Linux, `bun run package:portable` creates a conventional `tar.gz` beside the
deb/rpm packages. It uses system GTK 3 and WebKitGTK 4.1 libraries; users who want
package-managed prerequisites should use deb/rpm. The archive does not bundle
an entire Linux runtime or introduce an executable wrapper.

The frontend is embedded. Core prepares its default launcher helpers in the
application-selected writable resource directory only when a launch needs them.
Forge processor bridges are prepared in the installation temporary directory.
The macOS resource map does not apply to these targets. If future dependencies
need additional files, ship them beside the executable or in its resource
directory instead of forcing them into a single EXE.

New installations use `.minecraft` beside the launcher executable as the default
game directory on Windows and Linux. AppImage builds use the directory containing
the original `.AppImage`, outside its temporary mount. If this location cannot be
written, the launcher falls back to `.minecraft` inside its config directory.
macOS keeps its game data outside the `.app`, in the existing config directory.

On startup, existing launcher-local directories and these official Java Edition
default game directories are added to the folder list without changing an existing
selection or duplicating a path:

| Platform | Official default game directory                                        |
| -------- | ---------------------------------------------------------------------- |
| Windows  | `%APPDATA%\.minecraft` (the user's roaming application-data directory) |
| Linux    | `~/.minecraft`                                                         |
| macOS    | `~/Library/Application Support/minecraft`                              |

Official directories are added only when their `versions` subdirectory exists;
the launcher does not create an empty official directory. These are default game
data paths, not the installation directory of the official Launcher application.
Custom game directories configured in the official Launcher can be added manually.
