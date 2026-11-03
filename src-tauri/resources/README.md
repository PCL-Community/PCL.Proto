# Launcher resources

Copied from [PCL2-CE](https://github.com/PCL-Community/PCL2-CE) `Plain Craft Launcher 2/Resources/`
(Apache License 2.0) so the instance settings page can launch games the same way:

- `java-wrapper.jar` — Java Launch Wrapper (`CheckAdvanceDisableJLW`)
- `legacyfix.jar` — LegacyFix for versions before 1.6 (`CheckAdvanceDisableLF`)
- `lwjgl-unsafe-agent.jar` — LWJGL 3.4.1 unsafe-memory agent (`CheckAdvanceDisableLwjglUnsafeAgent`)
- `log4j2-debug.xml`, `log4j2-legacy-debug.xml` — debug logging configs (`CheckUseDebugLog4j2Config`)

On macOS they are copied to `.app/Contents/Resources/launcher/` and read directly
from the bundle. Other platforms embed them with `include_bytes!` and extract them
to `<config dir>/resources` on first use. The renderer agent (mesa-loader) is
downloaded to the writable config directory at launch time instead of being bundled.

`macos-arm64-libraries.json` contains the macOS ARM64 library coordinates and
checksums from HMCL's `HMCL/src/main/resources/assets/natives.json`. They are
applied to the launch metadata in memory, without changing installed version files.
Like HMCL, legacy launch defaults leave `-XstartOnFirstThread` to the version's
JVM metadata: forcing it breaks LWJGL 2's AWT window initialization. On macOS,
launches and exported scripts also clear `DYLD_LIBRARY_PATH`; inherited Homebrew
libraries were observed crashing AWT/AppKit during image decoding.
