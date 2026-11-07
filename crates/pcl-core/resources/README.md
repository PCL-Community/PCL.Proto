# Minecraft 安装与启动辅助资源

这些资源是 `pcl-core` 对应功能的默认实现，不包含启动器配置、身份或界面素材。

| 位置 | 用途 |
| --- | --- |
| `installer/forge-installer.jar` | Forge / NeoForge 处理器的无头安装桥接 |
| `launch/java-wrapper.jar` | Windows UTF-8 代码页下的 Java 启动包装器 |
| `launch/legacyfix.jar` | 早期 Minecraft 的启动修复 |
| `launch/lwjgl-unsafe-agent.jar` | LWJGL 3.4.1 的 unsafe 内存代理 |
| `launch/log4j2-debug.xml` / `launch/log4j2-legacy-debug.xml` | 调试日志配置 |
| `launch/macos-arm64-libraries.json` | macOS ARM64 的原生库替换映射 |

JAR 和 XML 来自 [PCL2-CE](https://github.com/PCL-Community/PCL2-CE) 的
`Plain Craft Launcher 2/Resources/`（源仓库 Apache-2.0）。原生库映射来自
[HMCL](https://github.com/HMCL-dev/HMCL) 的
`HMCL/src/main/resources/assets/natives.json`；上游许可与来源随资源沿用。

安装器仅在执行处理器时把默认桥接写入本次安装的临时目录，结束后清理。
启动资源仅在调用方启用对应行为且满足版本条件时，写入 `LaunchRequest.helper_directory`；
原生库映射直接在内存中使用，不改写已安装的版本文件。
调用方可通过 `forge_installer` 或 `LaunchHelpers` 提供替代资源；core 不改写这些路径。

PCL.Proto 在 macOS 上将资源复制到 `.app/Contents/Resources/launcher/` 并传入包内路径，
其它平台使用 core 的默认资源。应用的资源目录、打包方式和下载缓存位置由 Tauri 决定。
Mesa loader 不随包分发，由应用下载到可写目录。
