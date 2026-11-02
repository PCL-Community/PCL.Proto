# PCL2-CE 后端迁移记录

> 2026-10-30：Rust 模块与关键类型已按 PCL.Core/CE 结构对齐，详见 [Rust 后端与 PCL.Core 结构对齐](CORE_ALIGNMENT.md)。本文保留原有功能迁移与验收记录；文中 `core/`、`util/`、`setup.rs` 等旧路径为迁移时记录，当前路径以对齐文档为准。

2026-11-02：新增本地和在线 Modrinth 整合包安装，支持原版与 Fabric、可选文件选择、校验下载和覆盖文件；实例隔离与发布受安装事务保护。资源详情保留单独保存整合包文件的入口，CurseForge 继续使用原有下载流程。任务页按行显示阶段并支持取消，Fabric 阶段反映实际 profile 获取和库分析。Forge、NeoForge、其他整合包格式及真实 Tauri 和游戏启动仍待完成或验证。

在线验收：使用 Modrinth 的 `1.20.1 OFF Backport 1.0.0`（Fabric 0.16.10，索引含 41 个文件）在临时仓库完成整合包下载、游戏及附加文件安装，验证实例 JSON、模组文件和隔离配置；安装目录在测试结束后清理。该测试不覆盖原生界面操作或实际启动游戏。

2026-11-02：Fabric 与 LegacyFabric 下载页改为展示独立安装器版本并接入保存对话框和任务中心。Rust 校验元数据的官方 Maven 地址，下载使用临时文件、不覆盖发布与 64 KiB 最小大小检查；这不是散列验证。元数据和短文件清理测试通过，真实 Tauri 对话框与安装器下载仍待验收。

2026-11-02：实例服务器页面读取隔离后的游戏目录中的 `servers.dat`，支持添加、编辑、删除、状态查询和从现有启动流程直连。新版本使用 Quick Play，旧版本使用服务器与端口参数。NBT 编辑保留未知标签与压缩方式，对损坏文件和过期列表拒绝写入。服务器探测支持域名握手、SRV 优先级与加权、响应长度限制和 pong 校验；本地 DNS/TCP 测试覆盖显式端口、SRV 目标以及慢速 DNS 后回退到另一个 SRV 目标。旧版探测协议、真实 DNS 与服务器、Tauri 和 Minecraft 进服仍待验证或实现。外部进程在版本检查与文件替换之间写入时仍可能竞争。

服务器查询按 SRV 顺序逐个解析并尝试目标，各目标与地址共享整体截止时间；单个 DNS 或 TCP 候选无响应时可继续尝试后续目标。本地 TCP 测试覆盖无响应、畸形数据、成功回退和截止时间；当前测试未覆盖重构后从 SRV 回退至原域名默认端口的完整网络路径。

从存档数据包页进入资源下载时保留当前实例和存档，详情下载默认定位到该存档的 `datapacks` 目录，快捷下载的当前实例模式使用同一目标；切换实例后拒绝使用旧目标。Rust 检查存档范围并拒绝链接的数据包目录，下载仍使用临时文件及不覆盖发布。离线测试覆盖目标传递、实例切换、缺失存档和符号链接；尚未完成真实 Tauri 文件对话框和在线下载验收，外部进程在校验后替换目录仍可能造成竞争。

联机局域网扫描修复多个世界端口重复、乱序标签引起的溢出和无可用组播套接字时的空转；端口刷新与过期使用单调时钟。扫描器仍需真实局域网及原生联机流程验收。

Terracotta 子模块已合并上游 `ef8156d`（合并提交 `282efbc`），保留桌面库、PCL.Proto 接口和 EasyTier 依赖；上游新增内容主要为文档和构建工作流。工作流适配本分支的可执行文件，跳过已移除的 macOS 安装包，并限制上游文档自动发布仅在上游仓库运行。扫描器修复提交为 `cc7cc72`；跨平台 CI 和实际联机未验证。

更新：2026-10-29。功能参考 `../PCL2-CE`，revision `d68819990da092ed7e582b94b950874fa30baffe`。

本记录区分“代码已接通”“离线回归通过”“官方服务验证通过”和“真实游戏启动通过”。原版安装通过不代表游戏启动、加载器或账户认证已经完成。

设置的实际生效范围、统一配置架构与后续补全阶段见 [设置机制审计与补全设计](SETTINGS_DESIGN.md)。该文档是审计和计划，未实施部分不计入本记录的已完成迁移。

## 迁移原则

先遵循 Rust 与 Vue 3 的惯用方式，再对齐 CE 的用户功能、阶段名称与文件约定。直接迁移行为契约，不复刻 WPF、VB/C# 的线程模型或全局状态。Rust 服务负责下载、校验、文件发布与错误；Tauri 命令只适配参数和状态；Pinia 管理界面任务，命令返回值决定任务最终成功或失败。

## 第一批：原版安装与共享下载

对应 CE 源文件：

- `Pages/PageDownload/ModDownloadLib.cs` 的 `McDownloadClientLoader`：元数据、支持库、资源安装阶段。
- `Modules/Minecraft/ModLibrary.cs`：支持库规则、本地库与平台选择。
- `Modules/Minecraft/ModAssets.cs`：资源索引、散列对象、旧资源布局。

Proto 实现：

| 文件                                              | 职责                                                                                               |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `src-tauri/src/core/minecraft.rs`                 | Mojang 元数据类型、有序规则、平台上下文、本地库 classifier、共享支持库路径                         |
| `src-tauri/src/core/api_client.rs`                | manifest v2、版本 JSON 的 SHA-1/ID 校验、保留完整 JSON、官方地址与 BMCLAPI 地址转换                |
| `src-tauri/src/core/downloader.rs`、`downloader/` | 共享 HTTP 客户端、全局连接与带宽配额、Range 分段与会话内续传、下载源回退、校验、临时文件发布与统计 |
| `src-tauri/src/core/installer.rs`                 | 安装计划、并发下载、natives 解压、旧资源映射、实例发布与失败清理                                   |
| `src-tauri/src/core/version_export.rs`            | 客户端/服务端导出，复用相同下载器与经校验的版本元数据                                              |
| `src-tauri/src/core/repository.rs`、`game.rs`     | 按需扫描、精确识别实例 JSON、自定义名称与原版版本号分离                                            |
| `src-tauri/src/commands.rs`                       | 选择安装文件夹、扫描与选择实例的 IPC，拒绝越界索引和不存在的实例                                   |
| `src/stores/task.ts`                              | Vue 响应式任务、IPC 任务隔离、终态与速度管理、错误保留                                             |

已改变的行为：

1. 不再依赖“先点击版本、再从系统临时目录复制 JSON”才能安装。每次安装从已验证的 manifest 元数据开始，完整保留 `arguments`、`javaVersion` 等字段。
2. 下载最多尝试三次，网络、流中断及校验失败会传播到调用者；普通永久性 HTTP 4xx 不重试，408/429 可以重试。连接和读取有超时。
3. 只有大小及 SHA-1 均正确的本地文件才被复用。下载写入同目录唯一 `.part` 文件，校验后再 rename；失败不覆盖原文件，取消下载 future 时清理其临时文件。
4. 库规则按顺序执行，最后匹配的 `allow`/`disallow` 生效；支持无 OS 条件、OS 名称、架构/版本正则及功能条件。非法正则返回错误。
5. 下载当前平台的 artifact 和旧版 native classifier；classifier-only 库不再导致解析失败。解压遵循 `extract.exclude`，不释放 `META-INF`，拒绝目录越界及符号链接。
6. 资源按散列去重，验证散列及逻辑路径。支持 `virtual` 与 `map_to_resources`，日志配置放入 `assets/log_configs`。第三方文件来源不被错误替换为 BMCLAPI。
7. 新实例目录必须由本次操作独占创建；已有目录不覆盖。客户端 JAR 和 JSON 使用实例名，JSON 的 `clientVersion` 保留来源版本。全部阶段成功后才发布正式 JSON；失败清理本次新建的实例目录，已校验的共享缓存保留。
8. 安装与启动共用支持库规则和路径；classpath 使用系统分隔符，缺失库直接报错。启动侧改用实际下载的日志配置。
9. 重新进入实例列表会重新扫描。选择 Minecraft 文件夹会同步后端安装目录，即使该文件夹还没有实例。前端仓库字段统一为 `repositories`。
10. 文件全部完成后，任务仍等待安装命令完成；解压或发布失败不会显示安装成功。错误保留在任务页，终态任务不再贡献速度或剩余文件数，迟到的 IPC 不会重启任务。阶段进度按时间节流，缓存命中不计入网络速度。

任务阶段保持现有数值协议，文案引用 CE 语言资源：

| ID  | 安装阶段           |
| --- | ------------------ |
| 0   | 下载原版 Json 文件 |
| 1   | 下载核心 Jar 文件  |
| 2   | 下载原版支持库文件 |
| 3   | 下载原版资源文件   |

## 验证记录

- `cargo check -p pcl-proto --lib` 通过。
- 与本轮相关的 Rust 核心回归 19 项通过；另有路径边界测试。旧有依赖个人 Java、配置、游戏目录或外部服务的测试不作为离线回归运行。
- 本地 HTTP 模拟服务覆盖完整安装：503 重试、缺失 server 元数据、classifier-only 库、native 排除与 Zip Slip 拒绝、资源去重/旧布局、共享缓存复用、自定义实例名与重新扫描、已有目录保护、版本 JSON 篡改、文件大小/SHA-1 错误、失败不发布实例、关闭 UI 通道后继续完成安装。
- 组件回归检查扩展了真实 Pinia store 的边界回归：下载阶段全部完成但发布失败、终态后的迟到 IPC、并行任务、速度清零、IPC ID 隔离、导出防重复。
- `pnpm ui:build` 通过，包含 UI/语言/组件/页面切换契约、Vue 类型检查和生产构建。
- 官方服务验证通过：manifest v2 的真实元数据与资源索引能够为 `rd-132211`、`b1.7.3`、`1.2.5`、`1.16.5`、`1.21.1` 和本次官方最新正式版 `26.3` 生成安装计划。这里没有下载这六个版本的全部文件。
- 官方 `1.2.5` 的自定义命名完整安装通过：实际下载并校验客户端、支持库、native 与资源，完成旧资源映射，再通过实例扫描确认实例名、原版版本号与 JAR。
- 官方 `1.2.5` 客户端及服务端导出通过，包含服务端启动脚本；未执行这些脚本。
- `cargo clippy -p pcl-proto --lib` 执行完成，仓库及 Terracotta 仍有旧代码告警，未将此次迁移扩展成全仓 lint 清理。

复现离线检查：

```sh
cargo test -p pcl-proto --lib core:: -- \
  --skip core::api_client::tests::forge_test \
  --skip core::api_client::tests::mc_manifest \
  --skip core::launcher::game_launch_test \
  --skip core::platform::read_header_test
cargo test -p pcl-proto --lib util::file::path_tests
pnpm ui:build
git diff --check
```

实网检查显式运行，测试使用独立临时目录，结束时删除其测试数据：

```sh
cargo test -p pcl-proto --lib official_metadata_across_generations -- --ignored --nocapture
cargo test -p pcl-proto --lib official_vanilla_install_with_custom_name -- --ignored --nocapture
cargo test -p pcl-proto --lib official_client_and_server_export -- --ignored --nocapture
```

## 第二批：下载策略与活动连接

旧页面的活动连接写死为 `—`，Rust 没有提供活动请求统计。旧下载器每个安装任务同时处理 8 个文件，每个文件只有一个请求，出错后重新下载整个文件；多个安装任务也没有共享连接上限。设置页的下载选项此前只保存在浏览器中。

本轮参考 CE 的 `AdaptiveRangeDownloader.cs`、`DownloadResourceManager.cs`、`FileDownloader.cs`、`ModDownload.cs` 和 `ModSetup.cs` 迁移原版安装与客户端/服务端导出的共享下载器：

| 行为       | 当前实现                                                                                                                                        |
| ---------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| 连接配额   | 所有安装/导出共享，默认 64，范围 1–256；单文件默认 8，范围 1–8。降低上限时允许已有请求完成，后续请求遵守新值                                    |
| 小文件     | 小于 4 MiB 直接顺序下载，避免 Range 探测往返；复用连接池                                                                                        |
| 大文件     | 先用一字节 Range 探测；按约 8 MiB 划分，初始最多 1024 段，失败拆分后最多 2048 段；每段使用独立文件句柄与偏移                                    |
| 中断与慢段 | 保留当前会话内已写入部分，仅续传剩余区间；剩余至少 2 MiB 时可拆分。运行至少 8 秒、还有至少 1 MiB 且明显慢于同伴的分段才恢复；限速时关闭慢段判断 |
| Range 降级 | 忽略 Range、416、Content-Range/长度/编码不可靠时取消其他分段，丢弃临时文件并回退顺序下载；最终仍验证完整文件 SHA-1                              |
| 下载源     | 官方与 BMCLAPI 按偏好排序；库同时包含 `/maven` 和 `/libraries`。永久 4xx 跳过该来源，网络、流中断和校验错误可重试/换源；第三方来源不随意重写    |
| 自动源选择 | 官方 manifest 成功用时低于 4 秒时，自动文件源优先官方，否则优先镜像。版本列表单独读取来源设置；Auto 官方请求超时为 5 秒，随后回退镜像           |
| HTTP       | Auto 下小文件优先协商 HTTP/2，大文件/Range 使用 HTTP/1.1；显式 HTTP/2 仍允许 HTTP/1.1 回退，协商失败也可重试 HTTP/1.1                           |
| 限速       | 共享令牌桶，128 KiB 突发容量；设置页滑块按 CE 的分段公式转换为字节/秒，可实时调整                                                               |
| 页面统计   | 每 300 ms 查询全局活动请求数/上限、实际接收速度；包括重试流量，不把缓存命中计为网络流量。离开页面停止轮询，迟到响应不更新已卸载组件             |
| 磁盘       | 64 KiB 缓冲写入，完整校验后发布；可重新下载的缓存不逐个执行 `fsync`                                                                             |

设置页的下载源、版本列表源、连接数、单文件连接数、限速和 HTTP 协议均已接入 Rust。安装、导出和获取版本列表时也携带已保存的设置，首次下载不依赖先打开设置页。活动连接表示正在执行的 HTTP 请求/HTTP2 流，并不等于系统 TCP socket 数。

后续迁移：桌面端这六项现由 Rust 配置保存并在后端启动时应用；前端首次运行仅在 Rust 尚无该组设置时迁移旧 localStorage 值，后续安装、导出、Java 下载与启动补全均不再传入策略副本。写入串行化并先保存再应用；失败保留前端草稿和重试入口。测试覆盖一次性迁移、新值优先、写盘失败及前端提交顺序。跨窗口 revision 和通知、真正的 Tauri 重启验收尚未完成。

新增验证：

- Rust 核心离线回归 29 项通过，另有 1 项路径边界测试。覆盖全局/单文件上限、实时调整和取消释放、共享限速、分段拼接及断流续传、Range 探测及实际分段响应异常的顺序降级、校验失败保留原文件、临时文件清理、小文件不探测与缓存零流量。
- 前端检查覆盖 CE 滑块边界、损坏配置回退、持久化设置传入安装/导出 IPC；`pnpm ui:build` 通过。
- Chrome 使用真实 Vue 面板与模拟 Tauri 统计，验证 `23 / 64`、速度显示、空闲归零、失败清除旧数据、恢复以及卸载后停止请求。这不替代真实 Tauri WebView 验收。
- 实网下载官方 `1.21.1` 客户端通过：文件 26,836,906 字节，实际接收 26,836,907 字节（含一字节 Range 探测），下载/校验用时 17.163 秒。
- 本地按请求限速服务的单次对照：同一份 32 MiB 文件、同一配置下，单连接顺序下载 2.777 秒，最多 8 连接的 Range 下载 1.750 秒；两份内容及 SHA-1 一致，Range 额外接收一字节。文件实际分成 4 段；使用 Rust debug 构建，计时包含校验和磁盘操作。此结果仅说明单请求受限场景的收益，**不是 CE 实际运行对比，也不是实网加速承诺**。
- `cargo clippy -p pcl-proto --lib` 执行通过；已有全仓告警仍单独保留。

额外显式检查：

```sh
cargo test -p pcl-proto --lib controlled_range_benchmark -- --ignored --nocapture
cargo test -p pcl-proto --lib official_large_client_range_smoke -- --ignored --nocapture
```

与 CE 的剩余差异：版本列表目前按偏好顺序超时/失败回退，尚未复刻 CE 的延时并行竞速；文件切换来源或整个传输重试会重新下载该文件，不持久化跨来源分段。没有新增 UI 取消入口、跨应用重启断点恢复或任意未知大小 URL 的通用下载入口。

## 第三批：Java 管理、选择与多平台运行时

参考 CE 的 `ModJava.cs`、`ModLaunch.cs`、`PageSetupJava.xaml.cs`、`PageInstanceSetup.xaml.cs`，以及 `PCL.Core/Minecraft/Java` 下的管理器、扫描器、解析器和用户偏好。保留 CE 的选择契约，将 Windows 路径与架构规则扩展至 macOS、Linux。

本次 26.3 故障的直接原因是实际执行了 Java 21（最高 class 65），游戏要求 Java 25（class 69）。本机旧配置曾将 `/usr/bin/java` 缓存为 25，但实际探测为 Zulu 21.0.7；新逻辑只迁移旧选择路径，版本与架构重新从可执行文件读取。

| 链路     | 当前行为                                                                                                                                                                                                                                        |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 探测     | 执行 `-XshowSettings:properties -version`，旧 Java 回退 `-version`；校验真实版本、Java Home、架构和供应商，解析 Java 6/8 更新号及现代 EA 版本。处理系统 stub、wrapper、符号链接、Java 8 嵌套 JRE；超时 5 秒，输出有界且同步排空两个管道         |
| 扫描     | 环境变量、PATH、系统 JVM 根、SDKMAN/asdf/jabba/mise、Minecraft 与启动器 runtime；Windows 追加注册表和 Store runtime，macOS 追加 JDK bundles、Applications、Homebrew。关键词遍历有深度限制并防链接循环；自动扫描间隔至少 13 秒，手动刷新强制重扫 |
| 状态     | 统一 Rust 配置保存全局路径、运行时启用/禁用和实例偏好；扫描保留禁用状态与并发导入，失效项移除。配置写入临时文件后替换，序列化失败保留原文件；旧实例 localStorage 偏好只在没有新设置时迁移                                                       |
| 需求     | 原版/快照日期、OptiFine、Forge（含 Java 7 和 8u321 上限）、Fabric、LiteLoader、Cleanroom、LabyMod；读取 JSON `javaVersion`、非连续 `compatibleJavaMajors` 集合与 JAR `version.json`。仅为 Java 需求解析父版本元数据，并检查继承循环             |
| 选择     | 实例指定/启动器目录内相对路径 → 全局指定 → 自动候选；每次选择重新探测实际 executable。自动候选按最低可接受 major、原生架构、JDK、供应商、最新补丁排序，禁用、失效或不能运行的候选不会启动                                                       |
| 手动覆盖 | 与当前 CE 一致：指定版本越界时警告后仍使用，忽略兼容性只跳过警告。CE 当前 `AutoSelect` 在启动选择时仍继续检查全局指定；实例 Auto 不会绕过全局手动覆盖                                                                                           |
| 补全     | 缺失时重扫；用户确认后按平台/架构及 Mojang component 下载，再重新选择。复用共享下载源、连接/带宽配额、大小/SHA-1 校验；独立 staging、失败/取消清理、Unix 执行权限与安全链接处理。旧 Java 和受限 Java 8 更新范围要求手动安装                     |
| 界面     | 全局自动/指定、导入、刷新、详情、打开目录、启用/禁用；实例全局/自动/相对/指定、忽略兼容性；32 位内存警告读取后端 Java 信息。启动前提供下载进度和取消，拒绝异步期间切换到其他实例后继续启动                                                      |

平台处理：Windows x86/x64/ARM64、macOS Intel/Apple Silicon、Linux x86/x64/ARM64 的本地 Java 均可识别，架构兼容性按实际主机 CPU 判断（包含 Rosetta/Windows 模拟下的启动器）。可运行性最终由实际执行验证。Mojang 当前没有 Linux ARM64 运行时，也未必提供每个平台的所有 Java major；这种情况明确提示手动安装，不使用其他架构冒充原生运行时。Linux ARM64 可导入本地 ARM64 Java。选择 ARM Java 并不代表实例已有的 native 库也已经匹配 ARM，二者联动仍属于后续启动链路工作。

Java 验证：

- 本轮 Rust 核心离线回归 47 项通过（其中 Java 18 项），另有配置回归 2 项、路径边界回归 1 项通过。Java 覆盖需求规则、major 集合、排序、缓存版本失真、手动越界警告、禁用/失效回退、别名去重、Java 6 回退、Java 8 JDK、相对路径与 manifest 链接边界、继承循环、扫描目录布局、探测大输出/超时，以及模拟 HTTP 完整安装、错误散列和取消清理。`pnpm ui:build` 通过。
- 配置回归覆盖旧字段迁移、不覆盖新偏好、保存失败保留旧文件；前端真实 Pinia store 的模拟 IPC 回归覆盖迁移、持久化、保存失败、迟到请求隔离、相对路径和禁用候选。
- macOS ARM64 本机只读扫描通过，识别本地 Java 8、17、21，以及用户 Applications 中 Rider 自带的 Java 25.0.2。官方 ARM64 Java 25.0.1 已在独立临时目录完整下载、校验、执行并清理，报告原生架构兼容；没有改写用户启动器配置。
- Chrome 使用真实 Vue 组件与模拟 Tauri IPC，检查全局选择、启用/禁用、实例选项和相对路径保存；这不等于原生 Tauri 验收。
- Windows/Linux/macOS 构建矩阵已加入 Java 和配置测试；尚未执行远端 CI。本机 Windows 交叉检查在下载目标依赖阶段受阻，没有取得 Windows 编译结果。

显式只读/实网检查：

```sh
cargo test -p pcl-proto --lib setup::java_config_tests
cargo test -p pcl-proto --lib native_java_discovery_smoke -- --ignored --nocapture
cargo test -p pcl-proto --lib official_native_java25_install_smoke -- --ignored --nocapture
```

## 资源来源：接入 CurseForge（2026-10-31）

对应 CE 源文件：`Modules/Minecraft/ModComp.cs` 的 `CompProjectRequest.GetCurseForgeAddress`、`_BuildFromCurseForge`、`CompFile`、`CompFilesGet`、`CompProjectsGet`（双请求 + 去重 + 默认排序权重），以及 `PCL.Core/App/Secrets.cs` 的 `CurseForgeAPIKey`。

| 文件                                      | 职责                                                                                                                |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/src/app/secrets.rs`            | 编译期 `CURSEFORGE_API_KEY` 优先、运行环境变量兜底；构建脚本可读取根目录 `.env`，空白值按未配置处理                 |
| `src-tauri/src/io/net/http/api_client.rs` | `curseforge_base`、带 `x-api-key` 的 `fetch_with_curseforge` / `curseforge_available` 命令，复用同一 TTL 缓存       |
| `src/api/curseforgeApi.ts`                | CF 响应归一化成 Modrinth 的 `ISearchHit` / `IProject` / `IVersion` 形状；参数构造、分类 ID、文件分页与 CDN 地址改写 |
| `src/api/compProjects.ts`                 | 来源派发（工程 ID 纯数字即 CF，对照 CE 的 `IsFromCurseForge`）、双来源合并去重、默认顺序的来源下载量权重            |
| `src/api/compResources.ts`                | `tag` 的 `CFid/MRslug` 双前缀映射到同一份标签文案；可选标签不再要求 Modrinth 侧有值                                 |

已改变的行为：

1. 来源筛选改成「全部 / CurseForge / Modrinth」，默认全部：并发请求两个来源，Modrinth 条目在前；跨来源去重比较类型、加载器、游戏版本、更新时间及名称、简介或 slug。数据来自搜索结果，可能与 CE 的完整工程元数据不同。
2. 「全部」的默认排序按 CE 的来源权重折算下载量（模组/整合包 MR×7、数据包 CF×10、资源包与光影 MR×5），避免单一来源的下载量口径压倒另一个来源。
3. 世界资源页不再被阻止：CF `classId=17` 提供，且该页收起 Modrinth 选项（对照 CE 的 `SupportModrinth=False`）。
4. 只存在 CF 侧的分类（如 `407/` 生物群系）现在可选；某一侧缺少映射时该侧不请求，而不是发出一个无过滤的请求。
5. CF 分页用 `index`（结果偏移量），并按上游规则把 `index + pageSize` 钳在 10000 内；`totalCount` 也钳在 10000，不当作目录总量展示。
6. 只有一个来源失败而另一个来源仍有结果时，列表继续展示并给出 CE 的 `Download.Comp.List.Source.*Unavailable` 警告，不再整页报错。
7. 桌面端文件由启动器下载并显示任务进度；详情页选择保存路径，快捷下载可选择实例或文件夹。CF 缺 `downloadUrl` 时按 `files/{id/1000}/{id%1000}/{文件名}` 补全并把 `media*` 域名改写成 `edge.forgecdn.net`。
8. 未配置密钥（`curseforge_available` 为 false）时禁用 CurseForge 选项并给出提示；网页预览没有可用的密钥通道，因此该来源在预览版始终不可用。

已知边界：没有接 CF 的镜像源（CE 的 `mod.mcimirror.top/curseforge`）、Mod 依赖解析、本地文件指纹更新和 CurseForge 整合包直接安装；数据包需手动选择存档文件夹，实例选择目前为单选；已有同名文件不会覆盖。CF 的光影加载器筛选按 CE 同样被忽略；详情页最多取 8 页共 400 个文件；CF 的 `-snapshot` 版本标记不进入版本徒章（CE 会拼上「预览版」后缀）。

验证记录：

- `node tests/ui/curseforge.test.mjs`（已加入 `pnpm check:ui`）用假 invoke 覆盖归一化、参数构造、分页停止条件、双来源合并与只一侧失败时的警告。
- 字段名与枚举值对照 2026-10-31 的真实响应核实（`pagination.totalCount/index/pageSize/resultCount`、`latestFilesIndexes[].modLoader` 无 `loaderName`、`File.fileLength`、`gameVersions` 混有 `Client`/`Server`/`Forge`、classId 6/4471/12/6552/17/6945、`index + pageSize > 10000` 直接报错）。
- 带 `.env` 密钥的只读搜索请求返回 HTTP 403（响应服务器标识为 Kestrel），尚不能判定是网络代理还是 API 拒绝；真实搜索及 CF 文件下载尚未验收。

## 当前边界与下一批

2026-10-30 启动参数进展：原版启动读取版本 JSON 的现代 JVM/游戏参数及旧版 `minecraftArguments`，按规则筛选并替换占位符，参数作为 argv 直接交给进程。受控假 Java 测试验证路径与用户名中的空格、条件参数、旧版参数和工作目录；Rust 核心离线回归 48 项通过、7 项跳过。六个不同时代官方版本的元数据可继续生成安装计划，但该检查不运行游戏。这还不是完整启动链路：全部继承格式、所选 Java 架构与 native 联动、游戏进程生命周期以及真实游戏启动均未验证或实现。

后续进展：实例扫描、Java 需求和启动共享版本继承解析，处理父版本 JAR、循环/越界父级、子版本优先的支持库及混合新旧参数；已安装的无下载块 Maven 库按坐标定位，缺失库直接报错。新版 Forge/NeoForge 按 CE 规则使用实例自身 JAR，显式 `jar` 字段优先；Java 选择和启动重新扫描实例以避免使用旧缓存路径。受控假 Java 进程测试覆盖继承实例从扫描到 argv 的链路，另核对了 Fabric 官方 profile 的元数据形态。Java 下载取消时涉及的目录创建和发布改为同步完成，以消除异步目录操作可能在清理后继续运行的竞争；取消回归在本机重复 12 次通过。此处仅支持已存在的文件和上述继承格式，不代表加载器安装、全部加载器合并规则或真实游戏启动已完成；未知版本的 Forge 基于元数据发布时间判断，缺少版本和时间信息时仍需进一步处理。

后续进展：启动补全和原版安装计划现支持带 Maven 仓库地址的无 `downloads` 支持库。优先使用 profile 的 SHA-1 和大小，缺失时从仓库获取 `.sha1` 和响应长度；文件通过共享下载器校验后才发布，校验失败保留原文件。本地 HTTP 测试覆盖缺失元数据、损坏内容及非法仓库地址；官方 Fabric 1.20.1/0.16.14 profile 的全部库已在临时目录下载验证。仅支持 HTTPS 仓库及有响应长度、可获取 SHA-1 的文件；未验证真实游戏启动。

后续进展：新建实例时可选择 Fabric 或 LegacyFabric Loader。安装器验证所选游戏版本及 Loader 与 profile 一致，将主类、参数和支持库合并到原版元数据，并在所有文件下载与校验完成后发布自定义名称的实例 JSON。LegacyFabric 使用独立官方元数据及 intermediary，Maven native classifier 同样经校验下载并解压。本地服务测试覆盖成功、损坏库不发布、旧参数合并，并用受控 Java 进程检查 Fabric 主类和 classpath；前端任务测试覆盖两种 Loader 版本传递。官方 Fabric 1.20.1/0.16.14 和 LegacyFabric 1.12.2/0.20.1 的 profile 库已在临时目录下载验证。Fabric API、已有实例修改及其他加载器安装仍未接入；尚未实测完整官方实例安装和真实游戏启动。

后续验证：在独立临时目录分别完成官方 Fabric 1.20.1/0.16.14 与 LegacyFabric 1.12.2/0.20.1 的完整文件安装、实例扫描、x64 native 准备和受控进程 argv 检查，目录已清理；这不代表真实 Java 游戏已运行。仓库添加、仓库选择与实例选择现通过配置事务持久化，写入失败不发布内存变更；实例扫描后再次检查仓库身份，新增路径做规范化和重复检查。尚未验证真实 Tauri 重启与多进程并发写入。

后续进展（2026-10-30）：Fabric 版本列表增加官方源与镜像回退，修复镜像 profile 地址被误识别为 Mojang 元数据的问题。安装页选择 Fabric 后按 CE 生成默认实例名并切换图标、收起版本列表，用户编辑的名称保留；列表加载失败可重试。重新运行官方 Fabric 1.20.1/0.16.14 全量安装测试，临时目录中的实例发布、扫描、native 准备及受控进程参数检查通过；测试读取路径已适配默认实例隔离。仍未实测真实 Tauri 点击流程或游戏启动，CE 自动选择和安装 Fabric API 尚未接通，独立 Fabric 下载页的点击安装也仍是占位。

后续进展：实例扫描从继承链中的已知 Maven 坐标和补丁字段提取加载器版本，向前端返回 `pluginsVersion`；Legacy Fabric 与 Fabric 区分，NeoForge 与 Forge 共存标识时优先 NeoForge，OptiFine 可与加载器同时展示。实例详情、搜索和导出选项消费这些信息；读取当前实例时重新扫描以更新旧缓存。离线测试覆盖继承版本优先、组合加载器、旧配置反序列化以及 Fabric/Legacy Fabric 安装后扫描。识别尚未覆盖 CE 所有历史 JSON、缺失版本号的加载器和 Mod 目录中的 Fabric API；扫描失败仍回退到缓存，未完成真实 Tauri 验收。

后续进展：实例列表使用目录而非元数据 ID 作为行标识，选择 IPC 校验目录名并在指定仓库中定位对应实例，避免不同目录的相同 JSON ID 导致选错；事务成功后返回实际选择。前端防止过期读取覆盖新选择，并抑制重复点击。离线测试覆盖同 ID 实例和异步响应倒序；旧页面设置仍按 ID 存储，实例设置隔离并未因此完成。

后续进展：全局窗口默认、自定义和同启动器尺寸已连接配置事务及启动参数；同启动器尺寸通过 Tauri 获取物理像素，旧版元数据按 CE 的发布日期条件补充尺寸。受控进程与配置测试覆盖实际参数、迁移、范围及写盘失败。全屏和最大化仍不可用，早期版本或不支持尺寸参数的加载器不能保证效果，未验证真实游戏或各平台窗口行为。

后续进展：旧实例 Java 偏好迁移改为启动时批量执行，扫描登记仓库并只将唯一 ID 绑定到对应目录，已有 Rust 偏好优先；重复 ID、目录不可访问、数据无效或缺失时保留旧值并提示。迁移结果和完成标记通过同一配置事务提交。本地测试覆盖同 ID、已有配置、重试、写入失败和目录不可访问；无法追溯旧逻辑已错误写入的偏好，也不能保证扫描期间外部程序不修改实例目录，尚未验收真实 Tauri 重启。

后续进展：实例概览的实例、存档和 Mod 目录快捷方式已接入后端，并与游戏启动共用当前游戏目录解析；存档和 Mod 目录按需创建，不覆盖已有内容。命令只接受当前选择的实例与预定义目录类型；测试覆盖目录不存在、同名文件、原有数据保护和实例切换。文件管理器启动后的系统错误及真实 Tauri、Windows/Linux 桌面交互尚未验收。

全局默认隔离策略和实例开关已接入 Rust 配置。首次访问按策略及已有存档、模组决定并按实例目录保存；更改默认策略前，先为登记仓库中的现有实例保存旧策略决定，仓库不可访问时不提交变更。实例开关切换前提示，旧内容不搬移。启动工作目录、快捷目录和自动内存模组计数与冻结的游戏目录一致；启动前为选定目录准备旧资源。相同目标文件按大小和 SHA-1 复用，冲突时停止并保留已有内容；拒绝资源目录中的符号链接。旧页面隔离草稿先前未生效，不自动迁移；真实 Tauri、游戏和跨平台验收仍待完成。

全局及实例 JVM/游戏参数和实例前置 classpath 现由 Rust 保存并进入启动参数数组。实例的 JVM 与游戏字段分别以空值继承全局，支持元数据标记；用户 JVM 参数中的最大堆由内存设置覆盖，自定义初始堆优先。原硬编码的 G1 优化参数改为可编辑的 CE 默认 JVM 参数；旧页面的无效参数草稿未自动执行。受控进程测试覆盖继承、引号、标记、classpath 顺序和堆参数，真实游戏及跨平台行为仍待验收。

启动准备进一步按重新探测的 Java 架构计算库规则、旧版 native classifier 和 JVM 参数规则，复用下载器校验及补全有完整下载元数据的库；每个启动使用独立临时 native 目录，进程退出后清理。未提供下载元数据的 Maven 库仍需预先安装。受控测试覆盖 x86/x64 classifier、并发目录隔离、下载与缓存复用，以及 ARM Java 的参数规则；查看了官方 1.21.1 的 macOS ARM64 和 x64 LWJGL JAR，二者使用不同资源路径，仍同时保留在 classpath。旧版 LWJGL 核心 classifier 启动前检查 Mach-O、ELF 或 PE 二进制架构；继承的重复 native 采用子版本优先。实网校验官方 1.16.5 的 macOS LWJGL 3 包和 1.2.5 的 macOS、Linux、Windows LWJGL 2 包：前者支持 x64、后者支持 x86 与 x64，均不含 ARM64。其他 native 库、现代 classpath native 和加载器专用依赖尚未检查二进制兼容性；启动器异常退出也可能遗留临时目录。没有完成真实游戏和跨平台运行验收。

游戏进程现由会话服务持有，分别读取 stdout/stderr，记录运行状态及退出码，游戏退出后清理其 native 目录。日志页可选择会话、清空显示记录及终止选定的运行中进程；每个会话最多保留 1000 行、1 MiB，单行超过 8192 字节会丢弃该行，已结束会话最多保留 8 个。当前账户 access token 的完整原文会从输出中替换，但这不保证识别其他凭据或变形后的令牌。受控子进程测试覆盖双输出流、令牌替换、退出码、清空和并行进程的定向终止；网页预览检查了空状态，尚未进行真实 Tauri 日志交互或游戏验收。窗口可见性、完整崩溃分析和会话跨启动器重启保存尚未实现。

本批没有验证真实游戏进程启动，也没有完成全部版本继承格式、native 二进制兼容性检查和实例隔离的整体迁移。Java 管理与选择见第三批。旧版 x64 native 在 ARM 主机上能下载和解压，并不代表能用 ARM Java 启动。BMCLAPI 的地址转换有离线测试，但没有进行 BMCLAPI 真实服务安装验收；分段下载、会话内续传和下载源降级见第二批；版本列表竞速、Minecraft 安装的 UI 用户取消及应用崩溃后的安装恢复尚未实现。新安装拒绝覆盖已有目录，原地修复另做明确入口。

后续按依赖推进：

1. **启动链路**：统一解析继承后的版本、Mojang 参数与占位符、所选 Java 的版本/架构、日志与进程状态；在真实桌面环境验收新旧原版启动，再做缺失依赖修复。
2. **加载器安装**：新建 Fabric 和 LegacyFabric Loader 实例已接入；继续完成 Forge / NeoForge 的 installer/processor，OptiFine、其他加载器和 API 附加项分别按 CE 的兼容与互斥规则接入。当前 UI 仍明确阻止尚未实现的加载器安装。
3. **配置和账户**：将前端偏好映射到 Rust 配置、处理持久化和重置，分离账户展示数据与凭据，再接微软 OAuth/令牌刷新及外置登录。
4. **资源与整合包**：Modrinth 与 CurseForge 的检索已接通（含双来源合并）；继续接真实文件安装、依赖解析、更新及收藏批量操作，之后接整合包导入/导出和存档操作。

Windows/Linux 真实桌面行为与 Java 扫描、macOS WebView 安装交互、账户认证和联机仍需各自验收，不能用本批 Rust 服务测试代替。
