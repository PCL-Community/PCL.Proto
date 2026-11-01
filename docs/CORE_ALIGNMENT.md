# Rust 后端与 PCL.Core 结构对齐

日期：2026-10-30。参考：`../PCL2-CE/PCL.Core`（revision `d6881999`）与 `../PCL2-CE/Plain Craft Launcher 2` 的实例/网络模型。

本文记录本轮“将现有 Rust 功能按 PCL.Core 的结构与关键命名对齐”的落地结果、兼容策略和有意保留的差异。它不宣称补齐了 PCL.Core 的全部子系统；存档 NBT、缓存服务、任务中心、事件总线、账户 OAuth/OpenID 等仍属于后续工作。

## 1. 目标与边界

- 只对齐**已有功能**的模块边界、关键类型和命名，不为了形式一致重写已经更合适的 Rust 实现。
- 保留 Rust 惯用写法：`snake_case` 字段、`Result`/`Option`、`async`、所有权与显式错误；不复制 WPF 的线程模型、全局可变状态或 UI 绑定。
- 不迁移只在 Windows/WPF 上有意义的实现：WMI 设备识别、注册表扫描器、WPF 动画/控件、`System.Windows` 主题与窗口 API 等。
- Tauri 命令名保持不变，避免破坏现有前端调用；前端只同步确实变化的数据结构（Java 运行时、账户档案）。
- 持久化文件沿用原键名；Rust 字段改名时用 `#[serde(rename/alias)]` 保持旧配置可读。

## 2. 新目录结构

```text
src-tauri/src/
  app/
    config.rs          ConfigManager、配置文件读写、设置迁移与 Tauri 设置命令
    metadata.rs        启动器名称、版本、User-Agent 常量（对应 App/Metadata）
    states.rs          AppState、LauncherSetup、WindowSettings（对应 App/States）
    visibility.rs      启动器可见性策略与会话联动
  io/
    files.rs           路径校验、安全拼接、SHA-1、打开目录（对应 IO/Files）
    download/          下载器、Range 分段、全局连接/带宽配额（对应 IO/Download）
    net/http/          Mojang/BMCLAPI/Modrinth 元数据客户端（对应 IO/Net/Http）
    net/proxy.rs       进程级 HTTP 代理策略
  link/
    mcping.rs          服务器列表 ping（对应 Link/McPing）
    scaffolding.rs     Terracotta 联机状态桥接
  minecraft/
    classifier.rs      McVersionCategory / McVersionClassifier
    formatter.rs       McFormatter：版本显示名与 Wiki URL
    maven.rs           MavenArtifact：坐标解析与仓库路径
    version.rs         McVersionManifest / McVersion / McLibrary / McRule / DownloadFile
    mod_loader.rs      ModLoaderType
    installer/         原版安装、库/资源计划、natives 解压
    export/            客户端/服务端导出、加载器文件导出
    instance/
      mod.rs           McInstance（原 GameInstance）
      folder.rs        McFolder（原 GameRepository）与 McFolderType
      info.rs          McInstanceInfo / McInstanceState
      files.rs         实例资源目录扫描与文件操作
      isolation.rs     版本隔离判定
    java/
      installation.rs  JavaInstallation / JavaBrandType / JavaVersion / 探测
      entry.rs         JavaEntry / JavaSource / JavaStorageItem
      manager.rs       JavaManager：排序、合并、选择、可用性
      scanner.rs       跨平台 Java 发现（保留 Proto 的更强实现）
      preference.rs    全局/实例 Java 偏好与选择
      install.rs       Mojang Java 运行时安装
      requirements.rs  版本/加载器 Java 约束
    launch/
      launcher.rs      LaunchOption、classpath、JVM/游戏参数、启动
      session.rs       子进程会话、日志、退出事件
      arguments.rs     全局/实例 JVM、游戏参数
      settings.rs      实例启动附加设置与资源释放
      memory.rs        CE 自动内存公式与实例内存解析
    profile/
      mod.rs           McProfile / ProfileType
      skin.rs          皮肤与档案查询
  utils/
    platform.rs        OS/Architecture/Compatibility
    secret.rs          设备标识与 PCL 哈希
    toys.rs            今日人品
  commands/            按启动、设置、Java、实例、资源、仓库等职责拆分的 IPC 适配层
```

## 3. 关键结构映射

| PCL.Core / CE                                   | Rust                                                              | 说明                                                                                                  |
| ----------------------------------------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `Minecraft.Java.JavaInstallation`               | `minecraft::java::JavaInstallation`                               | 保存 `java_folder`/`java_exe`/`version`/`brand`/`architecture`/`is_64_bit`/`is_jre`/`compatibility`。 |
| `Minecraft.Java.JavaEntry`                      | `minecraft::java::JavaEntry`                                      | `installation` + `is_enabled` + `source`；通过 `Deref` 保留便捷字段访问。                             |
| `Minecraft.Java.JavaManager`                    | `minecraft::java::JavaManager`                                    | 以可执行路径为键；排序、扫描合并、手动导入、启用状态、`SelectSuitableJava` 语义。                     |
| `Minecraft.Java.JavaSource`                     | `minecraft::java::JavaSource`                                     | `AutoScanned` / `AutoInstalled` / `ManualAdded`。                                                     |
| `Minecraft.Java.JavaBrandType`                  | `minecraft::java::JavaBrandType`                                  | 13 个品牌值与偏好排序。                                                                               |
| `Minecraft.Java.JavaStorageItem`                | `minecraft::java::JavaStorageItem`                                | 持久化最小记录（路径、启用、来源），供后续配置拆分复用。                                              |
| `Minecraft.MavenArtifact`                       | `minecraft::maven::MavenArtifact`                                 | 3/4/5 段坐标、classifier、`@ext` 覆盖与仓库相对路径。                                                 |
| `Minecraft.McVersionClassifier`                 | `minecraft::classifier::{McVersionClassifier, McVersionCategory}` | 正式版/快照/远古版/愚人节分类；不含 UI 文案。                                                         |
| `Minecraft.McFormatter`                         | `minecraft::formatter`                                            | `format_version`、Wiki 语言前缀与搜索路径。                                                           |
| `McInstance`（CE）                              | `minecraft::instance::McInstance`                                 | 实例目录、JAR、版本、加载器、全局目录；新增 `info()`/`state()`/`modable()`。                          |
| `McInstanceInfo` / `McInstanceState`（CE）      | `minecraft::instance::{McInstanceInfo, McInstanceState}`          | 加载器版本与派生状态；`McInstance::info()/state()/modable()`。                                        |
| `ModFolder.McFolder`（CE）                      | `minecraft::instance::folder::McFolder`                           | 新增 `McFolderType`（Original/RenamedOriginal/Custom）。                                              |
| `McInstanceFolder`（CE 快捷目录）               | `minecraft::instance::McInstanceFolder`                           | 实例/存档/模组快捷目录。                                                                              |
| `Profile.Models.McProfile` / `ProfileType`      | `minecraft::profile::{McProfile, ProfileType}`                    | 统一账户档案；保留 Microsoft/Authlib/Xbox 启动所需字段。                                              |
| `Profile.Models.SafeProfile`                    | `minecraft::profile::SafeProfile`                                 | `get_account` 的 UI 视图，不返回 access/refresh token、xuid 或密码。                                  |
| `Utils.Result`                                  | Rust 原生 `Result`                                                | 不重复造 `Result` 类型。                                                                              |
| `Utils.Secret.Identify`                         | `utils::secret::Identify`                                         | 设备序列号与启动器标识派生；保留 Proto 的平台实现。                                                   |
| `Link.McPing.Model.McPing*`                     | `link::mcping::model::{McPingResult, McPingPlayerResult, ...}`    | 服务器状态 JSON 的类型化结果；`description` 保留原始 chat component。                                 |
| `App.ConfigEnums.LauncherVisibility`            | `app::visibility::LauncherVisibility`                             | 保留现有隐藏/最小化/保持不变行为，不启用未实现的全屏/立即退出分支。                                   |
| `Utils.SemVer` / `VersionRange`                 | 暂未引入                                                          | 当前版本比较由 `JavaVersion`、Maven/加载器逻辑覆盖；待有调用方时再补。                                |
| `Minecraft.Saves.SaveInfo`                      | 暂未引入                                                          | 存档 NBT 尚未实现，留待后续 `minecraft/saves`。                                                       |
| `IO.Storage.Cache`、`App.Tasks`、`App.EventBus` | 暂未引入                                                          | 属于后续缺失子系统，不在本轮“现有功能对齐”范围内。                                                    |

## 4. 兼容与迁移

- **Tauri 命令名不变**：`get_java_list`、`add_java`、`get_account` 等仍使用原名称；`lib.rs` 的 handler 列表未改。
- **Java JSON**：`JavaEntry` 以 `#[serde(flatten)]` 输出扁平结构；旧字段通过自定义 `Deserialize` 兼容：
  - `directory_path` → `java_folder`
  - `compability` → `compatibility`
  - `is_jdk` → `is_jre` 的反向映射
  - `implementor` → `brand`（按 vendor 字符串识别）
  - `is_user_imported` 不再持久化；旧值会被忽略，扫描后会按 `JavaSource::AutoScanned` 重新标记。
- **账户 JSON**：由枚举 `{ "Offline": { ... } }` 改为 `McProfile` 扁平结构，`profileType` 为 `microsoft`/`offline`/`authlib`/`yggdrasilConnect`。前端 `stores/account.ts`、启动页已同步。
- **配置根**：`AppState.launcher_setup` 使用 `#[serde(rename = "pcl_setup_info")]`，旧 `config.json` 不需要改写。
- **实例文件夹**：`McFolder.kind` 缺省为 `custom`，旧仓库记录可直接反序列化。
- **持久化 Java 列表**：仍以 `Vec<JavaEntry>` 保存在 `AppState`；`JavaStorageItem` 已提供，后续迁移到“只存路径、启动时重探测”的 PCL.Core 模式时可复用。

## 5. 有意保留的差异

1. **Java 扫描与探测**：保留 Proto 的异步、跨平台 BFS、`-XshowSettings:properties` 解析、PE/ELF/Mach-O 架构读取与探测超时控制。它们比 PCL.Core 当前的 Windows 注册表/`where` 扫描更完整，因此只对齐类型与管理器职责，不照搬扫描器实现。
2. **实例模型**：保留 `McInstance` 的 `jar_path`、`json_path`、`natives_path`、`global_dir` 等 Rust 侧显式路径，避免在每次使用时重新推导；新增的 `info()`/`state()` 只做派生，不替换现有字段。
3. **下载器**：保留 Proto 的 Range 分段、续传、全局配额与校验发布流程；PCL.Core 的 `NDlFactory` 抽象面向 C# 对象生命周期，不直接映射为 Rust trait，以免增加无调用方的抽象层。
4. **命令层**：PCL.Core 使用 `RpcService`，Tauri 使用 `#[tauri::command]`。命令集中在 `commands/`，按业务职责分文件，`lib.rs` 直接引用所属模块，不复制 RPC 反射模型。当前职责和前端边界见 [代码组织与维护边界](ARCHITECTURE.md)。
5. **未迁移的 Windows/WPF 能力**：WMI 设备标识、注册表键监控、Toast/窗口特效、动画时钟等不进入 Rust 后端。

## 6. 验证

已执行：

```sh
cd src-tauri && cargo test --lib          # 128 passed, 13 ignored
cd src-tauri && cargo check --all-targets # 无 warning
cd src-tauri && cargo clippy -p pcl-proto --all-targets  # 本 crate 无 warning
pnpm type-check
pnpm check:ui
pnpm ui:build-only
```

未执行真实游戏启动、Windows/Linux 桌面运行及外部服务全量验收。Java 运行时安装测试中标记 `ignored` 的官方网络用例仍需按需显式运行。

## 7. 后续

新增 PCL.Core 子系统时，按同一命名空间放置：

- 存档：`minecraft/saves/{mod,info,nbt,parser}`
- 缓存：`io/storage/cache`
- 任务中心：`app/tasks`
- 事件总线：`app/event_bus`
- 账户 OAuth/OpenID：`minecraft/profile/auth/{microsoft,yggdrasil,authlib}`

新增类型优先复用本轮的 `McProfile`、`McInstance`、`DownloadFile`、`JavaEntry`，避免再引入 `Game*` 前缀的平行模型。
