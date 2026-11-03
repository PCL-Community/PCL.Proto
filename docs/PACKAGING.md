# Application packaging

Use `pnpm build` (or `pnpm tauri build`) to build distributable applications.
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

`pnpm dev` continues to use Vite and source-tree launcher resources.
Configuration, accounts, Minecraft instances, downloaded runtime files, and caches
stay in their existing writable locations. Existing extracted launcher resources
are left in place for older installed versions, but the macOS app no longer uses them.
Build-time API configuration retains its existing handling; the root `.env` is
never included as a bundle resource.

## Windows and Linux

The existing embedded frontend and launcher-resource extraction remain unchanged.
The macOS resource map does not apply to these targets.

New installations use `.minecraft` beside the launcher executable as the default
game directory on Windows and Linux. AppImage builds use the directory containing
the original `.AppImage`, outside its temporary mount. If this location cannot be
written, the launcher falls back to `.minecraft` inside its config directory.
macOS keeps its game data outside the `.app`, in the existing config directory.

On startup, existing launcher-local directories and these official Java Edition
default game directories are added to the folder list without changing an existing
selection or duplicating a path:

| Platform | Official default game directory |
| --- | --- |
| Windows | `%APPDATA%\.minecraft` (the user's roaming application-data directory) |
| Linux | `~/.minecraft` |
| macOS | `~/Library/Application Support/minecraft` |

Official directories are added only when their `versions` subdirectory exists;
the launcher does not create an empty official directory. These are default game
data paths, not the installation directory of the official Launcher application.
Custom game directories configured in the official Launcher can be added manually.
