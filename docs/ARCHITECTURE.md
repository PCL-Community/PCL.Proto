# 代码组织与维护边界

更新：2026-11-07。第三方接入见 [pcl-core](../crates/pcl-core/README.md)，前端约定见 [DEVELOPMENT.md](DEVELOPMENT.md)，打包见 [PACKAGING.md](PACKAGING.md)，主页兼容见 [CUSTOM_HOMEPAGE.md](CUSTOM_HOMEPAGE.md)。

## 前端

仓库根目录的 `package.json`、Vite 和 TypeScript 配置负责网站与 Tauri 前端；
`packages/components/` 保存组件 npm 包的入口、样式、包描述和独立构建配置，
直接复用 `src/components/widget/` 中的控件。根目录 `Cargo.toml` 管理 Rust 工作区：
`crates/pcl-core/` 是 Minecraft 核心，`src-tauri/` 是桌面应用，`crates/terracotta/` 是独立维护的 Git 子模块；
不要把子模块文件复制到组件包。组件包运行 `bun run components:build` 构建，或执行
`npm pack ./packages/components` 构建并打包。

| 位置                     | 负责的内容                                                   |
| ------------------------ | ------------------------------------------------------------ |
| `src/views/`             | 页面布局、路由参数、该页面的交互与展示状态                   |
| `src/components/widget/` | 复用控件、列表项和领域展示组件                               |
| `src/composables/`       | 组件生命周期、响应式编辑状态、页面切换时的失效保护           |
| `src/stores/`            | 跨页面共享的账户、Java、启动、下载任务等状态                 |
| `src/api/`               | 领域设置、IPC 参数、资源服务请求和来源归一化                 |
| `src/types/`             | 共享数据模型；资源项目、版本、来源与种类统一定义在 `comp.ts` |
| `src/util/`              | 公共机制，如外链、平台、菜单和设置持久化队列                 |

资源数据的依赖方向：

```text
types/comp.ts
  ↑ Modrinth / CurseForge 客户端、筛选规则、收藏、组件
  ↑ api/compProjects.ts（来源分发、双来源搜索与合并）
  ↑ 页面与下载流程
```

两个来源都输出相同的资源模型；CurseForge 不依赖 Modrinth 客户端的类型定义。来源专有的响应类型、请求参数和校验保留在各自客户端。`CompType` 是资源种类的单一定义。

### 设置持久化

启动参数、内存、窗口、下载和实例启动设置各自创建一个 `createSettingsPersistence` 实例。公共实现位于 `src/util/settingsPersistence.ts`，统一处理同一作用域的串行保存、未保存草稿、错误和旧请求的完成顺序。

- 全局作用域是 `null`，实例作用域是实例目录；不同作用域独立排队。
- `read(directory?)` 等待当前保存队列，返回 `{ settings, error, dirty }`；有草稿时优先保留草稿。
- `stage(settings, directory?)` 只暂存；启动参数在离开输入框、切换实例或卸载时使用 `save(directory?)` 提交。
- `write(settings, directory?)` 暂存并提交；失败保留草稿，可再次保存。
- 输入与读取结果都复制成独立快照；旧保存结果不能清除新草稿或覆盖新错误状态。

设置领域负责各自的默认值、IPC 负载和业务校验。组件负责就绪状态、输入转换和本组件的错误展示。实例启动设置的保存队列属于领域模块，页面重新挂载后仍复用同一队列。

## Rust crate 边界

```text
src-tauri（PCL.Proto 应用）
  └── pcl-core（通用 Minecraft 能力，包含安装与启动辅助资源）
        └── terracotta（仅 link feature）
```

`pcl-core` 通过 Git 分发，`publish = false`，不发布到 crates.io。
第三方不需要依赖 Tauri 或使用 PCL.Proto 的配置模型；默认 feature 不编译 Terracotta。
`community`（资源平台 API、资源下载、在线整合包与 Fabric API 随装）、`microsoft`（微软登录）和 `link`（联机）均默认关闭、可独立启用。Tauri 应用显式开启三个 feature。基础安装、本地 mrpack 导入和使用已认证档案启动不要求社区或微软登录 feature。

| 位置 | 职责 |
| --- | --- |
| `pcl-core/src/minecraft/` | 版本、实例、安装、Java 探测/扫描/约束、启动、认证与皮肤协议、存档和服务器文件、导出 |
| `pcl-core/src/io/` | 校验下载、Range、来源回退、可共享的连接/带宽配额、文件与 HTTP 操作 |
| `pcl-core/src/link/` | 服务器 ping、局域网世界发现；`link` feature 提供 Terracotta 桥接 |
| `pcl-core/src/events.rs` | 普通进度回调 `EventSink` |
| `pcl-core/resources/` | 安装桥接、启动兼容 JAR、Log4j 配置与 macOS ARM64 库映射；由所属功能按需使用 |
| `src-tauri/src/config/` | 全部启动器设置、继承、默认值、校验、文件位置、事务持久化、设置到功能输入的折算 |
| `src-tauri/src/login/` | 设备码提示、角色选择、取消、轮询及档案更新 |
| `src-tauri/src/secrets.rs` | 本应用凭据来源与皮肤站注册的 Client ID |
| `src-tauri/src/commands/` | IPC 参数、账号展示模型、通道适配和应用流程 |
| `src-tauri/src/assets.rs` | 本应用包内资源和可写缓存目录的路径；下载与兼容逻辑由 core 执行 |
| `src-tauri/src/` | 窗口/可见性、设备标识、今日人品、外链、自定义协议与资源提供者；`src-tauri/src/link/` 保存大厅公告、Natayark 登录与联机设置 |

游戏启动与管理的协议、校验、下载和文件操作属于 core；PCL.Proto 的配置、交互与展示属于应用。`SafeProfile` 是应用的 WebView 投影，留在账号命令中；core 只提供完整档案与游戏能力。

皮肤会话地址、纹理解码、默认皮肤规则、文件校验和微软上传在 `minecraft::profile::skin`。应用负责选择文件、读取账号、刷新登录、写入配置及皮肤缓存，不向 core 传入 `AppHandle` 或配置单例。

### 设置如何进入 core

- Java 偏好、相对路径、全局回退和扫描节流在 Tauri；core 扫描接收目录列表，安装器接收按优先级排列的 Java 候选并重新探测兼容性。
- 内存自动分配、全局/实例参数继承、隔离决策和“跟随启动器窗口尺寸”都在 Tauri。core 接收具体 MiB、游戏目录、参数与窗口尺寸。
- Mesa loader 的版本、下载与校验由 core 管理，`prepare_natives` 按 Windows 渲染器设置补全；普通启动与导出脚本共用，宿主只指定缓存路径。
- `LaunchExtras` 是一次启动的实际行为输入，不包含“继承全局”选项。`LaunchHelpers` 提供默认资源的替代路径；缺省时 core 按启用行为与版本条件准备自带资源，错误向调用方返回。
- 不再传入 `RuntimePaths` 或配置目录。Java 安装只接收运行时目标目录；authlib-injector 接收 JAR 文件路径；Java wrapper 接收临时目录；启动辅助文件接收专用输出目录。Forge 处理器桥接默认使用本次安装的临时目录，宿主可传路径替换。
- `DownloadSettings` 留在应用；只有连接数、限速、来源和 HTTP 模式转换为 core 的 `DownloadOptions`。来源分三类：文件、版本元数据、社区资源（CurseForge / Modrinth，可选 mcim 镜像）。安装后自动选中实例由应用完成。
- core 不保存代理、User-Agent 或密钥单例。宿主构建 reqwest 客户端，再传入 `Downloader`/认证提供者；OAuth Client ID 逐次传入，CurseForge key 由应用构造成 `RequestSigner` 注册进 `Downloader`。

`Downloader` 的克隆共享客户端、配额和统计；独立构造的下载器相互隔离。
`MinecraftApiClient` 使用传入的下载器，`downloader().await` 返回共享该网络运行状态的文件下载器。
更换代理时应用先预建客户端并保存配置，再替换该实例的客户端；已发出的请求继续使用原客户端。
core 的任务取消、游戏会话、在线整合包暂存及 Terracotta 桥接仍有进程级运行状态；它们不保存启动器设置，任务 ID 需在进程内唯一。
CurseForge Key 与微软 Client ID 由应用分别按需读取。签名器只在目标主机属于 CurseForge 官方域名时附加 `x-api-key`，镜像源不会收到密钥；core 不读取环境变量或要求集中注册凭据。

### 持久化与错误

`ConfigManager::update` 在锁内构造候选状态、原子保存，再发布内存状态。配置存储在 `config.yml`，直接使用 YAML，不导入旧 JSON。缺少配置文件时初始化；已有配置按当前字段读取，缺失或无效字段使用默认值。`config_version` 从 `1` 开始，仅在配置格式变化时递增；版本不一致或存在错误字段（含文件损坏）时，先将原文件备份为同目录的 `config.<UUID>.bak`，再原子写入当前配置。不可读文件或备份失败时停止写入。账户、仓库、实例设置沿用同一事务；配置文件键名不因 Rust 类型移动而改变。

路径校验、符号链接边界、文件校验、失败清理、不覆盖发布和重命名回滚必须保留。前端不接收 token、密码等完整认证数据，账户展示使用 `SafeProfile`。密钥读取和构建环境约定见根目录 README。

### 功能与验收边界

现有代码包含原版、Fabric/LegacyFabric、Forge/NeoForge 和 Modrinth 整合包安装，账户认证、启动补全、存档/服务器文件编辑与服务器 ping。新增 CE 行为先看本地 `../PCL2-CE` 的调用链；复用行为契约，不复制 WPF 对象和线程模型。

历史迁移的阶段统计、旧源码路径、旧测试数量及“尚未实现”清单不再作为当前状态依据。新增能力以实际调用链和对应验收为准：

- 本地 HTTP、假 Java 进程、路径/持久化测试验证行为与失败路径；不等于真实游戏、外部账户或局域网已验收。
- Range 续传限于当前会话，不支持跨应用重启恢复；切换来源可能重新传输整份文件。
- 尚未接入消费者的设置保持禁用或明确提示。跨窗口 revision/通知、配置导入导出、DoH、部分平台窗口能力等旧规划不视为已完成。
- 配置重置、隔离切换、导入等不得隐式删除或搬移游戏数据；远程主页不能获得任意本地文件/命令能力。

## 维护约定与验证

`scripts/` 保存应用共用的图标生成与 CE 资源同步工具；包专属的构建脚本保留在所属包中。前端回归检查统一放在 `tests/ui/*.test.mjs`，由 `bun run check:ui` 使用 Bun 测试运行器逐文件执行；新增测试无需修改命令列表。临时界面对照入口不随应用保留。

新增功能先找所属领域的现有入口。公共规则只保留一个实现，调用方直接 import 所属模块；不要为转调一次函数而增加页面包装、兼容转导出或新的 service 层。页面确有独有状态、校验、错误处理或交互时再保留本地函数。

设置持久化回归位于 `tests/ui/settings-persistence.test.mjs`，覆盖串行与并行作用域、快照隔离、失败重试、过期完成、读取期间编辑、真实领域 IPC 负载和实例编辑器重新挂载。该检查已接入 `bun run check:ui`。

常规验证：`bun run ui:build`、`cargo check --workspace`、`cargo test -p pcl-core --no-default-features --lib`、`cargo check -p pcl-core --no-default-features --features community`、`cargo check -p pcl-core --no-default-features --features microsoft`、`cargo test -p pcl-core --all-features --lib`、`cargo test -p pcl-proto --lib`。离线检查与构建不代表已完成真实游戏启动或 Windows/Linux 桌面验收。
