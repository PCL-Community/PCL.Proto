# 设置机制审计与补全设计

日期：2026-10-29。审计基线：`8e396ae`。本文是设计与实施计划，**没有宣称下述待办已经实现**。

实施进度（2026-10-30）：配置文件读取失败时不再初始化覆盖，只有配置路径不存在时才创建默认配置；未选实例时不再读写 `none` 页面设置。七类设置页中的未接入字段已禁用并提示。下载六项在桌面端迁入 Rust 配置，启动时一次迁移旧页面值；下载器在后端初始化，页面提交串行化并在失败后保留草稿，业务入口不再携带策略副本。日志显示行数也迁入 Rust 配置，日志页与其他设置共享状态，优先迁移旧日志页值；会话缓冲仍独立限制为 1000 行和 1 MiB。Java 配置修改和扫描使用先保存候选状态、成功后发布的事务。配置恢复界面、revision/多窗口同步、后端能力状态及其余 P0–P6 工作仍未完成；实例页面旧存储仍按版本 ID 区分仓库，不能视为实例隔离完成。

新增进度（2026-10-30）：安装后自动选择实例已接入 Rust 配置和安装完成路径，旧页面值在启动时迁移；仅在实例发布成功且安装目录仍为当前目录时选择，持久化失败单独提示，不改变安装成功状态。尚未完成真实 Tauri 重启和跨平台验收。

新增进度（2026-10-30）：全局游戏窗口的默认、自定义尺寸、同启动器尺寸已接入 Rust 配置与启动参数。启动时读取 Tauri 窗口物理尺寸；自定义尺寸校验为 100–16384 像素。旧页面的可支持模式迁移一次，全屏和最大化暂不可选择，旧值仍保留在页面存储中。旧版游戏按 CE 的 2013-06-17 日期条件补充尺寸参数；更早游戏及不支持相关参数的加载器不保证生效。尚未完成真实游戏、Tauri 和跨平台验收。

新增进度（2026-10-30）：旧实例 Java 偏好改为启动时批量迁移，不再由设置页按当前 ID 直接写入。仅在所有登记仓库可扫描、ID 唯一且路径有效时写入，现有 Rust 偏好优先；重复、缺失或无法确认的记录保留在旧存储并提示。迁移与完成标记同一事务写入，重试不会覆盖新偏好。已经由旧逻辑写入的 Rust 偏好无法自动判断是否曾被错误迁移，需人工核对；其他实例页面的旧存储仍按 ID 区分。

新增进度（2026-10-30）：实例页面新草稿按目录分开保存，切换时同步加载，避免同 ID 实例相互覆盖和切换期间误写。旧 ID 键保留但不再由通用页面读取；除 Java 外，旧数据尚无唯一匹配迁移或认领界面，因此页面暂时不会显示旧值。目录别名规范化、实例移动时迁移以及 Rust 权威配置仍未完成。

新增进度（2026-10-30）：桌面启动时复用 Java 迁移的仓库扫描，为概览、整合包导出和数据包页面的已知表单字段认领唯一匹配的旧记录。新目录键优先，原记录保留；重复、缺失、损坏或无法扫描的记录保持待处理并提示。实例启动设置及具有执行副作用的旧字段未迁移，仍需后续领域配置和认领流程。此过渡迁移使用浏览器本地存储，尚不是统一 Rust 配置迁移，亦未处理目录别名、实例移动和多窗口竞争。

新增进度（2026-10-30）：内存领域模型、全局和实例覆盖的配置接口及启动参数解析已建立，包含 CE 自动分配公式、手动最大/初始堆和 32 位 Java 上限；无新配置时保留原有 `max_memory`。页面控件、旧值迁移、内存可视化与跨平台实测仍未完成，因此内存设置尚不能算接通，控件继续禁用。

新增进度（2026-10-30）：全局和实例内存模式及滑块已连接 Rust 配置与启动解析，实例可跟随全局；旧全局值及唯一匹配的旧实例值在桌面启动时迁移，已有新值和用户编辑优先，无法匹配的记录保留并提示。保存失败的草稿按作用域保留并可重试。内存占用/自动分配可视化仍为占位，切到自动或跟随全局后重新打开页面不会保留此前手动滑块位置；真实 Tauri、游戏启动与跨平台内存读取尚未验收。

新增进度（2026-10-30）：手动最大和初始内存独立持久化，切换自动或跟随全局后仍保留；旧实例记录在唯一匹配且未被新编辑覆盖时可补迁移手动值。内存图展示系统总量、可用量和基于当前设置的估计分配，定期刷新并复用后端启动解析；Java 位数未知时不展示估计值。真实 Tauri、游戏启动以及 Windows/Linux 的系统内存读取仍需验收。

新增进度（2026-10-30）：全局隔离默认策略和实例开关接入 Rust 配置。首次访问实例时按策略及已有模组、存档决定并持久化；修改全局策略前先按旧策略为登记仓库中的现有实例保存决定，仓库不可访问时不提交变更。切换实例开关会警告，且不会搬移文件。启动时冻结游戏目录，`--gameDir`、工作目录、快捷目录和自动内存模组计数使用同一结果。启动前为目标目录准备旧版资源，已有文件相同时复用，冲突或资源子目录为符号链接时报错并保留原文件。旧页面的无效隔离草稿不自动迁移，以保留现有实例默认隔离行为；真实游戏、Tauri 和跨平台验收仍未完成。

新增进度（2026-10-30）：全局及实例 JVM、游戏参数与实例前置 classpath 接入 Rust 配置和启动进程。实例 JVM/游戏参数留空时分别继承全局，支持已实现的元数据替换标记；参数按引号和反斜杠解析成数组，不经过 shell。前置 classpath 使用分号分隔绝对路径。最大堆以内存设置为准，自定义 `-Xms` 优先于初始堆设置。保存失败保留页面草稿；此前未生效的旧参数草稿不自动执行。真实游戏、Tauri 桌面和 Windows/Linux 参数解析尚未验收。

新增进度（2026-10-30）：实例资源页接通真实目录扫描与文件操作，数据包页按存档切换；图片用令牌化自定义协议，避免开启宽泛的 asset 作用域。存档信息页（NBT 解析）、模组元数据与依赖检查、整合包导出页仍未实现。

新增进度（2026-10-30）：实例概览接通重命名（Rust 事务迁移实例目录键并同步 JSON/主 Jar）、描述与收藏、回收站删除、导出启动脚本与补全文件。导出脚本复用启动解析并在补全阶段保留 natives 目录；删除复用既有回收站命令；补全文件按所选 Java 架构校验并重下客户端 Jar、Libraries 与旧版资源。测试游戏、重置、修补核心与实例图标/分类仍未实现。

新增进度（2026-10-30）：实例自定义信息、渲染器、启动前命令与等待、关闭文件校验、使用全局代理、Java Wrapper、LegacyFix、Debug Log4j2 和 LWJGL Unsafe Agent 接入 Rust 实例配置与启动链。渲染器在 Windows 上从腾讯镜像补全 mesa-loader；其余 agent 随包释放到配置目录并按 CE 条件添加。文件校验关闭后启动补全直接复用已有文件。游戏窗口标题改写、认证/服务器组和 Mod 更新仍不可用。全局 HTTP 代理（不使用/系统/自定义）接入下载器、元数据与导出请求，实例开关按 CE 只把自定义代理写入游戏 JVM；代理凭据只支持写在地址中，DoH 仍未实现。以上均未完成真实游戏与跨平台验收。

新增进度（2026-10-30）：启动器可见性的隐藏后退出、隐藏后恢复、最小化和保持不变接入 Rust 配置及游戏会话。仅在游戏输出出现已知就绪标志后改变窗口；所有会话结束后，异常退出或输出不完整会恢复隐藏的窗口。旧的无效选择不自动启用，立即关闭选项仍不可用。就绪检测依赖游戏日志，不能识别的版本将保持窗口不变；真实游戏和各平台窗口行为尚未验收。

## 1. 结论与证据边界

问题主要出在“设置保存”与“功能执行”之间没有连接。页面迁移带来了大量控件和默认值，`usePageSettings()` 也能保存它们，但多数业务模块没有消费这些值。重新打开页面仍能看到选择，不代表设置生效。

目前有明确执行链路的是：

- **下载设置七项**：文件源、版本列表源、总连接数、单文件连接数、限速、HTTP 模式及安装后自动选择。页面 → 语义值转换与串行 IPC → Rust 配置事务；下载器启动时从 Rust 配置初始化，安装成功后按当前设置选择实例。
- **Java 管理与选择**：扫描、导入、启用/禁用、全局选择、实例选择（全局/自动/指定/相对路径）、忽略兼容性警告。Rust 保存，启动前重新探测并选择实际 Java。忽略兼容性只抑制手动选择越界警告，不代表任意 Java 都能运行游戏；实例“自动”当前仍会检查全局手动选择。
- **界面语言**：`pcl-language` / 旧 `ce-language` → `pclLocale.ts` → 响应式翻译与 `document.lang`。
- **日志显示行数**：设置页与日志查看器共用 Rust 配置，裁剪所选游戏进程的会话日志；后端会话缓冲有独立的行数和字节数上限。
- **游戏内存**：全局/实例设置经 Rust 保存，启动时读取当前实例策略并生成最大堆和可选初始堆参数；自动模式按可用内存与实例加载器、模组数量计算。
- **实例隔离**：全局默认值与实例决定经 Rust 保存；游戏目录、工作目录、快捷目录、内存模组计数及旧版资源映射按同一决定使用实例目录或共享目录。
- **自定义启动参数**：全局与实例 JVM/游戏参数及实例前置 classpath 经 Rust 保存，启动时解析、继承并传递给游戏进程。
- **启动器可见性**：游戏会话识别就绪后应用隐藏或最小化策略，隐藏状态在会话结束后按退出结果恢复或关闭。

上述“接通”是当前源码判断，不等同于本轮完成了真实游戏、多平台或全部外部服务验收。Java 下载与原版安装共用后端初始化的下载资源。HTTP 模式也不应被宣传为覆盖全部独立 HTTP 客户端和浏览器 `fetch`。

本次组件回归检查通过。现有设置回归证明了保存、重置、类型过滤和按旧实例 ID 切换，并不能证明保存值被启动器、窗口或网络使用。

## 2. 设置现状清单

### 2.1 统计口径

以下七类页面共列出 **150 个字段绑定**：启动 25、下载与管理 18、个性化 58、其他 14、联机 6、语言 2、实例设置 27。其中 26 个接通，窗口模式及启动器可见性各有一项部分接通；其余字段尚未接入目标行为。窗口模式的全屏、最大化和启动器可见性的立即关闭仍不可用。

这不是全仓功能完成率：Java 管理页的全局选择、运行时列表操作不计入这 150 项；按钮、关于/更新/反馈页、工具页和实例导出选项也不计入。个性化中受 `hasBackground` / `hasMusic` 控制、默认不显示的字段仍计入。下面逐组覆盖这些字段。

“未接入”包括：只保存值、只切换设置页提示/输入框、按钮重置输入内容、复制到无人消费的 Pinia 字段。它不意味着相关底层能力一定完全不存在。

### 2.2 全局启动：尺寸和内存接通，窗口模式部分接通

来源：[`PageSetupLaunch.vue`](../src/views/Setup/PageSetupLaunch.vue)、[`stores/setup.ts`](../src/stores/setup.ts)、[`commands.rs`](../src-tauri/src/commands.rs)、[`launcher.rs`](../src-tauri/src/core/launcher.rs)。

| 项目                     | 旧字段                                                                                                                                                                                                   | 实际情况                                                                             |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| 版本隔离                 | `ComboArgumentIndieV2`                                                                                                                                                                                   | 已接入 Rust 默认策略，首次访问或修改默认值时持久化现有实例决定；旧页面草稿未自动迁移 |
| 窗口标题、自定义信息     | `TextArgumentTitle`, `TextArgumentInfo`                                                                                                                                                                  | 仅同步到 `useSetup().launch`，没有业务消费者；`--versionType` 仍使用启动器常量       |
| 启动器可见性、进程优先级 | `ComboArgumentVisibie`, `ComboArgumentPriority`                                                                                                                                                          | 可见性部分接入，依赖日志识别就绪；立即关闭和进程优先级尚不可用                       |
| 窗口模式与尺寸           | `ComboArgumentWindowType`, `TextArgumentWindowWidth`, `TextArgumentWindowHeight`                                                                                                                         | 默认、自定义及同启动器尺寸已接入；全屏和最大化仍不可用                               |
| 微软认证方式、IP 栈      | `ComboMsAuthType`, `ComboPreferredIpStack`                                                                                                                                                               | 登录/启动未读取                                                                      |
| 内存模式、最大与初始内存 | `LaunchRamType`, `SliderRamCustom`, `SliderRamInitialCustom`                                                                                                                                             | 已接入 Rust 配置及启动参数；未迁移旧值时沿用 `pcl_setup_info.max_memory`             |
| JVM/游戏参数             | `TextAdvanceJvm`, `TextAdvanceGame`                                                                                                                                                                      | 已接入 Rust 配置与启动参数；“恢复 JVM 参数”提交 CE 默认值                            |
| 渲染器、启动前命令与等待 | `ComboAdvanceRenderer`, `TextAdvanceRun`, `CheckAdvanceRunWait`                                                                                                                                          | 尚未接入启动行为                                                                     |
| 高级开关                 | `CheckAdvanceDisableJLW`, `CheckAdvanceDisableLF`, `CheckAdvanceGraphicCard`, `CheckAdvanceNoJavaw`, `CheckAdvanceDisableLwjglUnsafeAgent`, `CheckAdvanceDisableCrashAnalysis`, `CheckAdvanceLockMemory` | 均未消费；部分依赖尚不存在的启动包装器、代理或平台功能                               |

### 2.3 下载与管理：7 项接通，11 项未接入

来源：[`PageSetupGameManage.vue`](../src/views/Setup/PageSetupGameManage.vue)、[`downloadSettings.ts`](../src/api/downloadSettings.ts)、[`task.ts`](../src/stores/task.ts)、[`resources.rs`](../src-tauri/src/core/downloader/resources.rs)。

| 状态   | 项目                                               | 旧字段                                                                                                 |
| ------ | -------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| 接通   | 文件源、版本列表源、HTTP 模式                      | `ComboDownloadSource`, `ComboDownloadVersion`, `ComboDownloadHttpMode`                                 |
| 接通   | 总连接数、单文件连接数、限速                       | `SliderDownloadThread`, `SliderDownloadFileConnection`, `SliderDownloadSpeed`                          |
| 接通   | 安装后自动选中                                     | `CheckDownloadAutoSelectVersion`                                                                       |
| 未接入 | Authlib 修复                                       | `CheckFixAuthlib`                                                                                      |
| 未接入 | 社区资源来源、下载文件名、Mod 显示名、快速下载行为 | `ComboDownloadMod`, `ComboDownloadTranslateV2`, `ComboModLocalNameStyle`, `ComboDownloadQuickBehavior` |
| 未接入 | 隐藏 Quilt、自动安装依赖                           | `CheckDownloadIgnoreQuilt`, `CheckDownloadAutoInstallDependencies`                                     |
| 未接入 | 正式版/快照提醒、自动设置游戏语言、剪贴板读取      | `CheckUpdateRelease`, `CheckUpdateSnapshot`, `CheckHelpLauncherLanguage`, `CheckDownloadClipboard`     |

连接和速度滑块已经做 CE 数值换算及边界校验；不能因为其余下载设置无效而把这六项也禁用。独立的 Forge/社区资源 API、Java 元数据等请求需要另查覆盖范围，不能只凭共享下载器已有设置便宣称全网统一。

### 2.4 个性化：58 项均未接入目标功能

来源：[`PageSetupUI.vue`](../src/views/Setup/PageSetupUI.vue)、[`main.ts`](../src/main.ts)、[`base.css`](../src/assets/base.css)、[`pageNavigation.ts`](../src/layout/pageNavigation.ts)、[`PageLaunchRight.vue`](../src/views/Home/PageLaunchRight.vue)。

| 项目                          | 旧字段                                                                                                                                                                                                                                                                                                | 实际情况                                                                       |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| 透明度、主题模式、浅/深色配色 | `SliderLauncherOpacity`, `ComboDarkMode`, `ComboLightColor`, `ComboDarkColor`                                                                                                                                                                                                                         | 启动固定添加 `theme-blue`；CSS 自行跟随系统深色模式，未读取用户选择            |
| 标志、窗口大小锁定、启动提示  | `CheckLauncherLogo`, `CheckLockWindowSize`, `CheckShowLaunchingHint`                                                                                                                                                                                                                                  | 窗口/启动界面不读取                                                            |
| 材质与模糊                    | `CheckBlur`, `SliderBlurValue`, `SliderBlurSamplingRate`, `ComboBlurType`                                                                                                                                                                                                                             | 无对应应用链路                                                                 |
| 界面/服务器 MOTD 字体         | `ComboUiFont`, `ComboUiMotdFont`                                                                                                                                                                                                                                                                      | 字体由样式固定；存储值不应用                                                   |
| 背景                          | `ComboBackgroundSuit`, `SliderBackgroundOpacity`, `SliderBackgroundBlur`, `CheckAutoPauseVideo`, `CheckBackgroundColorful`                                                                                                                                                                            | 无背景管理消费；打开/刷新/清空按钮为占位                                       |
| 音乐                          | `SliderMusicVolume`, `CheckMusicRandom`, `CheckMusicAuto`, `CheckMusicStart`, `CheckMusicStop`, `CheckMusicSMTC`                                                                                                                                                                                      | 无播放器消费；文件夹等按钮为占位                                               |
| 标题栏 Logo                   | `UiLogoType`, `CheckLogoLeft`, `TextLogoText`                                                                                                                                                                                                                                                         | 仅切换本页表单；图片更换/删除为占位                                            |
| 自定义主页                    | `UiCustomType`, `TextCustomNet`, `ComboCustomPreset`                                                                                                                                                                                                                                                  | 主页仍为固定内容；“刷新主页”只跳转 `/home`；已有主页编辑器不代表这些设置已接通 |
| 隐藏顶级页面（3）             | `CheckHiddenPageDownload`, `CheckHiddenPageSetup`, `CheckHiddenPageTools`                                                                                                                                                                                                                             | 导航及路由未过滤                                                               |
| 隐藏设置页（11）              | `CheckHiddenSetupLaunch`, `CheckHiddenSetupJava`, `CheckHiddenSetupGameManage`, `CheckHiddenSetupGameLink`, `CheckHiddenSetupUI`, `CheckHiddenSetupLauncherLanguage`, `CheckHiddenLauncherMisc`, `CheckHiddenSetupUpdate`, `CheckHiddenSetupAbout`, `CheckHiddenSetupFeedback`, `CheckHiddenSetupLog` | 同上                                                                           |
| 隐藏工具页（2）               | `CheckHiddenToolsGameLink`, `CheckHiddenToolsTest`                                                                                                                                                                                                                                                    | 同上                                                                           |
| 隐藏实例页（9）               | `CheckHiddenVersionEdit`, `CheckHiddenVersionExport`, `CheckHiddenVersionSave`, `CheckHiddenVersionScreenshot`, `CheckHiddenVersionMod`, `CheckHiddenVersionResourcePack`, `CheckHiddenVersionShader`, `CheckHiddenVersionSchematic`, `CheckHiddenVersionServer`                                      | 同上                                                                           |
| 隐藏特定功能（3）             | `CheckHiddenFunctionSelect`, `CheckHiddenFunctionModUpdate`, `CheckHiddenFunctionHidden`                                                                                                                                                                                                              | 操作入口未读取                                                                 |

### 2.5 其他：14 项均未接入目标功能

来源：[`PageSetupLauncherMisc.vue`](../src/views/Setup/PageSetupLauncherMisc.vue)、[`PageLogRight.vue`](../src/views/Tools/PageLogRight.vue)、[`lib.rs`](../src-tauri/src/lib.rs)。

| 项目                           | 旧字段                                                                                                                                      | 实际情况                                                                                                                       |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 公告、最大 FPS                 | `ComboSystemActivity`, `SliderAniFPS`                                                                                                       | 未消费；WebView 没有现成的全局 FPS 设置，不能用保存一个数值模拟实现                                                            |
| 日志行数                       | `SliderMaxLog`                                                                                                                              | 已迁移至 Rust 配置，与日志查看器共用；仅限制显示，游戏会话缓冲另有限制                                                         |
| 硬件加速、遥测                 | `CheckSystemDisableHardwareAcceleration`, `CheckSystemTelemetry`                                                                            | 启动配置/服务未消费；未发现接入的遥测控制链，不能据此推断已在上传遥测                                                          |
| DoH、代理模式/地址/用户名/密码 | `CheckNetDohEnable`, `SystemHttpProxyType`, `TextSystemHttpProxy`, `TextSystemHttpProxyCustomUsername`, `TextSystemHttpProxyCustomPassword` | 网络客户端未读取；“应用代理”为占位；密码被通用存储过滤，仅存当前页内存。系统环境可能影响 HTTP 客户端，但不等于此页代理设置生效 |
| 动画倍率、调试项               | `SliderDebugAnim`, `CheckDebugSkipCopy`, `CheckDebugMode`, `CheckDebugDelay`                                                                | 未消费；Rust 日志级别固定为 Info                                                                                               |

配置导入/导出、停止使用等按钮也为占位。不要把代理“应用”按钮改成成功提示就当作补全。

### 2.6 联机：6 项均未接入

`TextLinkUsername`, `ComboPreferProtocol`, `CheckLatencyFirstMode`, `CheckTryPunchSym`, `CheckEnableIPv6`, `CheckEnableCliOutput` 只保存页面值。`src-terracotta/src/rooms/mod.rs` 的 EasyTier 配置仍直接设定 `latency_first = true`、监听 TCP/UDP 等，没有接收这组用户偏好。设置页“重启生效”的提示不构成重启后真正生效的证据。

### 2.7 语言：一项生效，一项未接入

- `ComboUiLanguage` 实际绑定 `pclLocale`，调用 `setPclLocale()`，有效；`usePageSettings()` 中同名默认字段不是实际来源。
- `ComboUiFormatCulture` 只保存；日期、数字等仍由各调用处直接格式化，没有统一使用这一设置。

### 2.8 实例设置：Java、隔离和内存接通

来源：[`PageInstanceSetup.vue`](../src/views/InstanceSetting/PageInstanceSetup.vue)、[`java.ts`](../src/stores/java.ts)、[`selection.rs`](../src-tauri/src/core/java/selection.rs)。

| 状态   | 项目                                             | 旧字段                                                                                                                                          |
| ------ | ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| 接通   | Java 选择、忽略兼容性警告                        | `ComboArgumentJava`, `CheckAdvanceJava`；改走 Rust `java_preferences`，不以页面通用缓存为准                                                     |
| 接通   | 版本隔离                                         | `ComboArgumentIndieV2`；切换前警告，目录不自动搬迁                                                                                              |
| 未接入 | 标题、明确使用默认标题                           | `TextArgumentTitle`, `CheckArgumentTitleEmpty`；需要 Win32 改写游戏窗口标题，暂不实现                                                           |
| 接通   | 自定义信息                                       | `TextArgumentInfo`；实例值经 Rust 保存，启动时覆盖 `${version_type}`，留空跟随全局                                                              |
| 接通   | 内存模式、最大/初始内存                          | `VersionRamType`, `SliderRamCustom`, `SliderRamInitialCustom`；可跟随全局，启动时解析实例覆盖                                                   |
| 未接入 | 登录限制、认证服务器/注册页/名称、自动进入服务器 | `ComboServerLoginRequire`, `TextServerAuthServer`, `TextServerAuthRegister`, `TextServerAuthName`, `TextServerEnter`                            |
| 接通   | JVM/游戏参数、前置 classpath                     | `TextAdvanceJvm`, `TextAdvanceGame`, `TextAdvanceClasspathHead`；参数留空跟随全局，classpath 为实例专属                                         |
| 接通   | 渲染器                                           | `ComboAdvanceRenderer`；仅 Windows 启用，启动补全阶段从腾讯镜像下载 mesa-loader 并生成 `-javaagent`                                             |
| 接通   | 启动前命令与等待                                 | `TextAdvanceRun`, `CheckAdvanceRunWait`；启动前在全局游戏目录执行，失败只记日志                                                                 |
| 接通   | 文件验证、游戏代理                               | `CheckAdvanceAssetsV2`, `CheckAdvanceUseProxyV2`；关闭校验复用已有文件，代理读取全局自定义代理                                                  |
| 接通   | 包装器、LegacyFix、Debug Log4j2、LWJGL Agent     | `CheckAdvanceDisableJLW`, `CheckAdvanceDisableLF`, `CheckUseDebugLog4j2Config`, `CheckAdvanceDisableLwjglUnsafeAgent`；资源随包，按 CE 条件启用 |
| 未接入 | Mod 更新                                         | `CheckAdvanceDisableModUpdate`；PCL.Proto 没有模组更新检查，控件保持禁用                                                                        |
| 未接入 | 登录限制、认证服务器/注册页/名称、自动进入服务器 | `ComboServerLoginRequire`, `TextServerAuthServer`, `TextServerAuthRegister`, `TextServerAuthName`, `TextServerEnter`；本轮不实现                |

LittleSkin 按钮会填写/确认覆盖表单，但没有把认证配置送入启动链；锁定登录、新建档案按钮是占位。不能标成外置登录已经支持。

### 2.9 相邻页面与工具

- 更新页确实请求 GitHub latest release 并显示结果/链接；这不是自动升级器，也不是 Minecraft 新版本提醒。当前仅比较版本字符串是否相等，尚无可靠的新旧版本排序。
- 反馈页实际请求 GitHub issues，关于页展示真实构建版本；都不属于设置持久化机制。
- 启动器日志页按钮仍为占位；游戏日志查看器已接入运行中及已结束的进程会话，支持选择、导出、清空、自动滚动和终止所选进程，运行栈导出尚不可用。
- 实例概览的 Logo/显示分类、整合包导出的表单选项也使用 `usePageSettings()`；对应业务没有接通。实例概览的重命名、描述、收藏、删除、导出启动脚本、补全文件已接入真实命令；**整合包导出页**与已实现的**原版客户端/服务端导出命令**是不同入口。
- 实例资源页（模组、资源包、光影、投影、存档、截图、数据包）已接入真实目录：`instance_files.rs` 扫描实例游戏目录（遵循实例隔离），支持打开文件夹、从文件安装、文件级启用/禁用（`.disabled` 后缀，与 CE 一致）、回收站删除；数据包页新增存档选择器。图片通过自定义 `pclinstance` 协议按令牌提供，未开放全局资源文件访问。仍未实现：模组元数据/图标解析、模组依赖与更新检查、存档 NBT 详情页、整合包导出页与服务端页。
- 工具测试页 `TextSkinID` 被 `saveSkin()` 使用，确实参与皮肤获取/下载；`TextDownloadUrl` / `TextDownloadFolder` / `TextDownloadName`、四个 `Achievement*TextBox` 与 `CmbHeadSize` 尚未接通对应操作。这些均为工具表单，不应混入启动器偏好。
- 整合包导出页还有 22 个动态包含选项及名称/版本/高级开关，只控制本页选项和提示，最终导出仍为占位。实例概览 `ComboDisplayLogo` / `ComboDisplayType` 只保存而不改变实例元数据。迁移必须覆盖这些 `usePageSettings()` 调用点，分别改成领域设置、局部表单或明确不可用。

## 3. 根因与必须先处理的约束

1. **三套配置并存。** localStorage 按页面存值，`useSetup` 保留另一份启动状态，Rust `PCLSetupInfo/AppState` 决定真实执行。没有一致的读取、提交、应用协议。
2. **持久化键依赖控件与页面。** `TextArgumentTitle` 等 UI ID 被当作业务字段，页面改名可能改变配置位置；同一语义可出现多个存储键。
3. **实例只用 metadata ID。** `pcl-ui:PageInstanceSetup:<id>` 没包含仓库路径，两个仓库的同 ID 实例会共享前端缓存；未选实例时还可能写入 `none`。Java 的路径键已更可靠，其他实例设置必须跟上。
4. **没有统一继承模型。** `2`、空字符串、复选框各自表示“跟随全局/默认/清空”，业务侧没有统一解析；复制当前全局值会冻结继承。
5. **默认值与有效值混淆。** JSON 默认、页面补充默认、Pinia 默认、Rust 默认同时存在。页面声明的 `close-immediately` 与实际控件默认值也不构成执行契约。
6. **保存与应用混淆。** 缺少能力状态、保存错误、应用失败、重启要求和“下次启动使用”的区分。页面卸载时 150 ms 下载设置定时器会被清除，实时应用也可能尚未发生。
7. **写入尚未统一。** Java 命令及扫描现已通过同一锁内事务保存候选状态，保存失败不会发布到内存；其他状态入口和未来设置服务仍需纳入统一提交协议。原子文件写入不能代替版本冲突处理和变更通知。
8. **损坏配置可能被覆盖。** `ConfigManager::new()` 在 `load().is_err()` 时直接初始化并保存；新增字段若缺少兼容默认、未知新版本或单个字段损坏，都不能继续触发这一恢复路径。

## 4. 目标架构与代码组织

### 4.1 状态归属

桌面端由 **Rust 保存唯一权威配置**，Pinia 只保存已提交快照、待提交表单及应用状态。所有业务命令从 Rust 配置服务读取，页面打开与否不影响配置生效。继续复用 `ConfigManager`、现有 JSON 文件和原子写入能力，不再添加第二份桌面配置文件。

将现有内容分成四类：

| 类别           | 内容                                                          | 归属                                                                    |
| -------------- | ------------------------------------------------------------- | ----------------------------------------------------------------------- |
| 用户偏好       | 启动、下载、网络、界面、语言、日志、联机、实例覆盖            | Rust 配置服务；前端只读快照与编辑草稿                                   |
| 运行状态/缓存  | 任务进度、活动连接、Java 扫描结果、窗口当前尺寸、临时启动状态 | 各领域现有服务/store；不伪装成用户设置                                  |
| 选择与目录信息 | 仓库、当前账户/实例、手动导入 Java 与其启用状态               | 继续由现有领域管理；持久化可仍在同一文件，不因此塞进设置表单            |
| 凭据           | 账户 token、代理密码                                          | 领域凭据接口；快照/导出不返回原文，通用设置只保留凭据引用或“已配置”状态 |

网站预览使用同一前端接口的 localStorage 实现，只开放实际能在浏览器生效的界面设置；桌面启动、Java、代理、硬件能力明确不可用。它是另一运行环境的实现，不能与桌面同时读写两套权威配置。浏览器首次启动使用随包的默认快照；由 Rust 默认值导出并做一致性检查，页面不再各写一份默认。

### 4.2 最终目录职责

```text
src-tauri/src/setup.rs                   ConfigManager、AppState、现有配置文件入口
src-tauri/src/settings/
  mod.rs                                配置快照、串行提交、revision 与变更通知
  model.rs                              领域结构、枚举、默认值、校验与能力元数据
  resolve.rs                            全局 + 实例覆盖 → 实际启动/运行策略
  migration.rs                          schema 迁移、旧字段映射、幂等导入
src-tauri/src/commands/settings.rs       薄 IPC：读取、更新、重置、迁移、导入/导出
src-tauri/src/core/launcher.rs           消费解析后的 LaunchPlan，不再读前端字段
src-tauri/src/core/launch_session.rs     子进程、日志、退出事件、窗口可见性恢复
src-tauri/src/core/downloader/           复用 DownloadSettings/DownloadResources
src-tauri/src/core/java/                 保留探测、选择、安装；读取统一偏好
src-tauri/src/core/network.rs            客户端构造/替换、代理与 DNS 策略
src-terracotta/src/                      接收明确的联机选项，不读取 Proto 的存储

src/api/settings.ts                     IPC DTO 与桌面/网站入口选择
src/stores/settings.ts                  权威快照镜像、草稿、提交队列、错误
src/composables/useSettingsGroup.ts      页面分组绑定；不负责持久化/业务执行
src/composables/useSettingsEffects.ts    根组件一次安装前端效果、卸载监听
src/api/pclLocale.ts                     继续负责翻译加载和切换，移出独立持久化
src/layout/{pageNavigation,navMemory}.ts 消费统一的导航可见性策略
src/views/Setup/                        展示与编辑；保留现有 P 系列控件/布局
```

这里只按职责拆分；`network.rs`、`launch_session.rs` 在对应功能落地时才创建。`commands.rs` 可先保留现有文件并声明 `mod settings`，不为目录统一搬动无关命令。不要创建插件系统、任意路径配置引擎、通用事件总线或新 UI 表单框架。

### 4.3 配置模型

持久化文件沿用现有数据并增加独立的 `settings` 块。`schema_version` 表示磁盘格式，`revision` 表示本进程已提交版本，两者不是一回事。配置大致分组如下，具体类型由 Rust 定义，TS 镜像契约，不复制默认值：

```text
settings
  schema_version
  global
    launch          隔离、标题/品牌信息、窗口、内存、参数、启动前命令
    java            全局 Java 偏好（复用现有路径及选择语义）
    download        复用现有 DownloadSettings；另外保留安装后行为
    community       社区资源来源、命名、依赖、快速下载策略
    network         代理模式、代理地址、DNS；不包含密码原文
    appearance      主题/配色、字体、Logo、背景、音乐、主页
    navigation      隐藏入口的稳定 ID 集合
    locale          界面语言、格式区域策略
    logging         显示行数、诊断级别
    launcher        公告、动画、通知、硬件加速等
    multiplayer     用户名、传输偏好、延迟优先、IPv6、诊断
  instances         规范化实例目录 → InstanceOverrides
```

存语义值，不存滑块位置：内存 MiB、速度 bytes/s、连接数真实整数、透明度 0–1；枚举使用稳定语义名称。保留 CE 控件 ID 作为 `data-control` 与迁移映射，不再把它作为业务模型。转换只放在页面输入边界及旧数据迁移处。现有下载枚举数值 ABI 可继续复用，在 IPC 边界集中转换，不为了统一命名破坏已用协议。

实例键复用经过校验的实际目录：仅后端解析路径并确认属于已登记仓库，处理符号链接、平台大小写规则和旧路径别名；不得只用版本 ID，也不得接受任意前端文件路径。未选实例不能保存实例设置。移动/重命名由领域命令同步迁移键，不靠文件夹名猜测；删除实例时同时处理其设置，失败保留可恢复状态。

### 4.4 继承、覆盖、清空与重置

- 只有设计允许的启动/Java等字段存在实例覆盖；界面、下载全局配额、代理本身不复制到每个实例。
- `InstanceOverrides` 使用缺省字段表示继承。未设置 ≠ 空字符串 ≠ `false` ≠ `0`；空标题或禁用命令必须用显式模式表达，例如 `title: game_default | custom(text)`，移除整个字段才是继承。
- 最大/初始内存与模式作为一个内存策略整体覆盖，避免继承后出现初始值大于最大值。自定义 JVM/游戏参数使用明确的“继承或替换”规则；前置 classpath 是独立字段，不把参数字符串盲目拼接。
- Java 保留当前已验证的 `global / auto / exist / relative` 及手动越界警告规则。“自动”是否跳过全局手动选择不是此次架构迁移顺便改变的行为。
- 全局“恢复默认”恢复 Rust 产品默认；实例“跟随全局”删除覆盖，立即展示继承后的有效值。字段、分组、全部重置均经过同一提交协议；重置偏好不清除账户、仓库或游戏文件。
- 统一解析函数返回 `ResolvedLaunchSettings`，必要字段附来源 `default / global / instance`；页面可显示继承来源，诊断可解释最终值。解析结果不得反写成实例覆盖。

### 4.5 读取、提交、并发与错误

IPC 最小集合：`get_settings(scope)`、`update_settings(request)`、`reset_settings(request)`；迁移、配置导入/导出在各自阶段补上。作用域为全局或经过验证的实例目录；返回 `revision`、原始配置、需要的有效值、能力和应用结果。

更新使用按领域区分的强类型变更，例如 `SetLaunch(LaunchSettings)` / `SetAppearance(AppearanceSettings)` / `SetInstanceLaunch(InstanceOverrides)`，并携带 `expected_revision`。首版按分组提交完整值，避免任意 JSON 路径修改器。重置只接受已声明的字段/分组 ID；未知字段、错误作用域、非法枚举或范围直接返回结构化错误。

提交顺序：

1. 单一提交队列串行处理；检查 revision，构造候选配置。字段及跨字段校验失败时，不修改当前状态。
2. 做无副作用预检，例如构造候选 HTTP 客户端、检查路径与平台能力；不要在此执行启动前命令或发起下载。
3. 使用现有临时文件 + flush/sync + 原子替换发布候选文件。文件写入失败则当前内存和效果均保持原值。
4. 发布内存快照、递增 revision，按领域应用变化并发送 `settings-changed` 通知。事件仅携带 revision、作用域及变更分组，不携带凭据。
5. 返回已保存快照和应用结果。保存成功但应用失败时明确显示“已保存、应用失败”，保留上一份可用运行资源并可重试；不谎报生效，不盲目回滚并覆盖更新的配置。

已有 Java/仓库等写入同一文件的入口也必须使用同一配置写入事务/锁；迁移期不能由设置服务保存旧 `AppState` 副本覆盖另一入口的新账户/仓库信息。异步网络、Java 探测不持有全局配置锁，真正提交前重新确认 revision。

前端只维护已提交快照和本地草稿。开关/枚举提交后展示确认结果；滑块在拖动中可做视觉预览，在结束或短防抖后持久化；文本字段在失焦/确认时提交。网络、命令等组合配置使用明确的“应用”。保存失败保留草稿和重试入口；冲突返回最新快照，合并互不冲突的字段，同字段冲突保留用户草稿并提示，不能无条件覆盖或静默丢弃。

提交队列按作用域独立保存待改字段，不在路由卸载时丢掉已确认编辑。防抖期间再次拖动只提交最终值；迟到响应不能覆盖更新的 revision。订阅先于初始读取建立，收到版本缺口时重读快照；多窗口由 Rust 通知同步。首版按单主进程支持，使用独占配置文件锁阻止两个进程同时写入；不设计跨进程合并协议。

效果同样按领域串行应用并记录 revision；异步资源加载返回时核对版本，禁止旧主题/旧代理客户端在较新设置后重新覆盖。首次读取失败时展示重试/恢复入口，业务命令不能悄悄改用默认值继续启动或下载；网站预览单独按浏览器能力降级，不借此掩盖桌面配置错误。

### 4.6 生效时机与能力状态

每个已展示字段必须声明：稳定 ID、作用域、状态、消费模块、校验、适用平台、生效时机、默认值来源。Rust 返回能力和实际状态，前端维护标签、帮助文案和页面位置；不要再维护一张与执行逻辑无关的 `enabled: true` 表。

| 类别          | 例子                                         | 应用规则                                                            |
| ------------- | -------------------------------------------- | ------------------------------------------------------------------- |
| 即时          | 主题、字体、音量、日志显示行数、导航显隐     | 根部集中安装前端效果，不依赖设置页挂载；保存后跨路由持续有效        |
| 进行中可调整  | 下载全局连接配额与限速                       | 复用 `DownloadResources::configure`；已有请求可完成，新请求遵守配额 |
| 下个请求/任务 | HTTP 模式、来源、代理、DNS、联机偏好         | 新请求或新会话取新策略；已有请求保留旧客户端，不中断/重启任务       |
| 下次游戏启动  | Java、隔离、内存、参数、窗口、认证、渲染器   | 启动时一次解析并冻结为 LaunchPlan，不在启动半途中混用新旧配置       |
| 应用重启      | 经平台确认必须启动前设置的 WebView 硬件加速  | 持久化并标记待重启；重启后读取成功才视为实际使用                    |
| 未实现/不支持 | 尚无包装器的禁用选项、平台不支持的窗口材质等 | 禁用并给出原因，后端也拒绝将其报告成已应用                          |

状态必须区分 `implemented / planned / unsupported`；已实现字段另有 `saving / saved / applied / pending_restart / apply_failed` 等提交/应用状态。普通控件不需要铺满状态文案，只在等待、失败或受限时显示有用信息。

## 5. 各领域如何消费配置

### 5.1 启动与进程生命周期

`launch_game` 从权威配置取目标实例的有效设置，执行校验并生成 `LaunchPlan`。计划包含 Java、JVM 参数数组、游戏参数数组、classpath、工作目录、游戏目录、环境与启动前动作。`LaunchOption` 可逐步改为消费此计划；预览与执行共用构造逻辑，日志移除 token/敏感参数。

- 优先补完整的元数据 JVM/游戏参数与规则解析，再叠加用户参数；仅给当前固定参数追加几个字符串不能算现代版本完整支持。
- 内存先按自动/自定义策略计算 MiB，检查可用物理内存、Java 位数、初始/最大关系；32 位上限来自实际运行时，不只改变页面提示。
- 隔离统一决定 `gameDir`、cwd、mods/config/saves 等用户内容目录。共享 libraries/assets/natives 仍按已有安装约定定位。切换策略不自动搬移或删除已有内容，提供位置提示。
- 窗口模式按元数据能力和平台实现；标题/渲染器可能需要包装器/agent 或额外组件，不能用改启动器自身标题假装改了游戏标题。没有可靠实现则保持明确不可用。
- 自定义参数用经过测试的引号/转义解析器转成参数数组，不经 shell 拼接。覆盖冲突参数（如 Xmx）要有唯一顺序规则并在计划中可解释。
- 启动前命令是显式有副作用的功能：明确 shell、cwd、等待、失败是否阻止启动；迁移曾经无效的命令不得在升级后静默开始执行。
- `launch_session` 持有子进程并等待退出，汇报 started/exited/failed，采集有界日志，驱动关闭/隐藏/最小化/退出后恢复。仅成功 spawn 才改变窗口；失败保留启动器，先做到生命周期正确再启用可见性选项。
- GPU 优先级、进程优先级、java/javaw、锁内存等按 OS 能力实现与测试；macOS/Linux 不显示为虚假的 Windows 功能。

### 5.2 下载、网络与社区资源

配置加载完成后立即初始化共享下载资源。删除任务入口从 localStorage 反复携带 `download_settings` 的长期路径，改由 Rust 统一取配置；测试仍可通过内部函数显式注入策略。Java 自动下载也使用同一初始化结果。

`network.rs` 负责为请求类型构造客户端，复用现有 reqwest 与下载器；将元数据、文件下载、账户、皮肤、GitHub 信息、社区资源等调用逐一列入覆盖表。需要遵守应用代理/DoH 的浏览器 `fetch` 迁入受限的后端领域命令，不新增“任意 URL、任意凭据”的代理 IPC。确有必要保留的浏览器请求必须说明不受应用网络设置控制。

代理 `system / direct / custom` 是明确策略；地址和认证在提交前验证，预建客户端通过后再保存/切换；运行中任务保留原客户端。DoH 只有 resolver 已实现并测试后才启用；不能把“改 DNS URL”当作完成 TLS/回退/代理交互。

`download.file_source` 作为下载来源权威；旧 `pcl_setup_info.api_provider` 与 `MinecraftApiClient::api_bases` 的重复来源状态逐步移除，由新策略派生。社区资源不同供应商可有独立来源策略，不能盲目全局替换所有 URL。

自动选择实例在安装成功发布后触发，失败不选择；自动依赖与快速下载要落到资源安装任务，命名设置同时覆盖搜索展示、目标文件名与已有文件冲突规则。没有实际依赖安装器之前保持选项不可用。`CheckFixAuthlib` 不得为了“生效”绕过来源校验或已有下载校验。

### 5.3 界面、导航、主页与媒体

根部效果层将统一快照映射到 CSS 变量/主题 class、字体与窗口 API；只注册一份系统深色模式监听。CSS 媒体查询只在“跟随系统”时决定模式，显式浅/深色覆盖系统偏好。服务器 MOTD 字体只应用到相关内容，保留 PCL2-CE 的页面布局和控件尺寸。

隐藏策略使用稳定入口 ID，统一供标题栏、侧栏和动作入口消费；路由守卫与 `lastSubTab` 记忆同时避开已隐藏目标，提供可恢复的安全入口。隐藏是展示偏好，不是权限控制；不能把所有设置入口隐藏后永久锁死。实例自动跳转和浏览器前进后退也要检查。

主页已有编辑/渲染能力继续复用；增加明确的来源选择和加载状态，分别接通空白、预设、本地文件、网络更新。网络内容做来源与可执行能力约束，不能让导入设置或远程主页直接获得 Tauri 文件/命令权限。背景/音乐复用统一资源路径管理，完成文件列表、刷新、清空、视频暂停与播放器生命周期后才开放相应控件；SMTC 按实际平台能力开放。

### 5.4 语言、日志、联机与系统项

语言继续复用 `pclLocale.ts` 的资源加载、失败重试和过期异步结果保护，只将保存权交给设置服务。启动时先读取配置再初始化语言和前端效果；资源失败回退中文并明确报告失败，不把回退误写成用户的新选择。日期/数字集中通过 locale 格式策略格式化，避免各页直接读浏览器默认区域。

统一一个 `logging.max_lines`，设置页和日志查看器都编辑它；明确它限制显示文本，不是磁盘日志文件保留量。日志查看器接入真实日志/session 通道，日志级别、清理/导出再接现有插件与文件管理。无界日志不得进入全局 Pinia；保留有界缓冲。日志导出脱敏。

联机配置由 Proto 转成 Terracotta 的强类型输入，在新建/加入会话时传入；模块内部不导入 Proto 配置服务。正在运行的房间保持其快照，UI 说明下次连接应用。逐项验证协议、IPv6、打洞、输出；不只将 UI bool 复制到恰好同名的 EasyTier flag。

公告、通知、剪贴板只在对应服务存在后接通，并按系统权限/用户选择执行。遥测默认保持关闭，没有实际遥测服务时禁用该设置；最大 FPS 若 WebView 无可靠实现则撤下承诺，动画速度可单独连接现有 motion/CSS 动画。硬件加速需 Windows/macOS/Linux 分别确认 Tauri/WebView 能力，不许只加启动标记就宣称跨平台支持。

## 6. 配置加载、迁移、凭据与导入导出

### 6.1 安全加载与磁盘格式

先修复加载失败即初始化覆盖：文件不存在才创建默认配置；损坏、权限错误、未知未来 schema 分开处理。保留原文件和迁移前备份，提供可见错误/恢复入口；未知未来 schema 只读拒绝写入。可恢复字段做明确降级记录，不静默丢弃账户/仓库。

有版本迁移先解析 `serde_json::Value`，按旧 schema 转换，再反序列化为当前类型；新增字段配默认，边界值校验与损坏报告独立。保持现有原子写入并补平台验证，尤其 Windows 上替换已存在文件的语义。文件备份数量与清理规则有界。

### 6.2 localStorage 与已有 Rust 配置迁移

1. 明确每个旧键到新语义字段的映射、转换、冲突规则；不把未知页面字段和任意对象直接导入。
2. 已有有效新配置优先。旧 Rust 的账户、仓库、Java 选择/禁用、实例 Java 偏好继续保留；已接通下载偏好和语言迁移后保持行为。
3. 对曾无效的值区分“可恢复偏好”与“会产生新副作用”。主题等可迁移；启动前命令、代理/外置认证、远程主页等进入待确认迁移项，未重新启用前不执行。这是旧无效设置首次获得执行能力时的边界，不要求普通设置每次都确认。
4. 旧实例 ID 唯一匹配仓库实例时才能绑定；多仓库同 ID 不猜测、不广播到所有实例，保留待认领记录。`none` 不迁移到任意真实实例。
5. 日志行数冲突：有效的新配置优先；其次旧日志查看器专用值，再次其他设置值，最后默认。旧默认值没有用户编辑标记，无法可靠推断意图，迁移摘要说明选择，不编造更新时间。
6. 前端读取旧浏览器数据并通过一次性迁移 IPC 提交；Rust 校验、备份、原子提交并记录迁移版本。全量成功后才停止旧写入；失败保持旧数据可恢复，重试幂等。
7. 禁止长期双写。迁移一组即切换其全部读写调用点；旧键在确认新快照可读、重启验证且回退窗口结束后再清理。此前仅保留备份，不继续消费。

Java 旧偏好现已在启动时批量迁移，但仍需纳入统一迁移版本、提供待认领界面及真实重启验证。

### 6.3 凭据与配置导入导出

当前代理密码并未持久化，不能将去掉 `password` 过滤当成修复。代理认证首版可只保存在进程内，UI 明示重启需重新输入；确需“记住密码”时再接系统凭据存储，失败不可静默降级为明文。

设置导出使用版本化白名单格式，默认排除账户凭据、代理密码、设备标识、缓存和运行任务；本机绝对路径明确标注不可移植。导入先预览差异、平台不支持项与校验错误，再通过同一事务一次提交。重置/导入均不删除游戏数据；执行性字段按上一节重新启用。普通偏好导出不复制整个 `AppState`。

## 7. 实施顺序与每批验收

阶段以依赖和验收完成划分，不给未经实现验证的工期承诺。每批完成都更新本审计清单；不等全部 150 项完成才让已有功能受益。

| 阶段                       | 范围和主要落点                                                                                                    | 完成条件                                                                                          |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| P0：诚实状态与数据保护     | 标注/禁用未实现项；`ConfigManager` 区分首次启动与加载失败；实例作用域禁止 `none`；梳理旧键映射                    | 坏配置不被覆盖；有效六项下载和 Java/语言不退化；UI 不再把保存当成生效                             |
| P1：配置主链与已实现项迁移 | `settings/{model,mod,migration}`、薄 IPC、`api/settings`、Pinia；迁移下载/Java/语言；初始化先于业务；统一日志行数 | 不打开设置页也正确生效；重启保持；写盘失败/冲突不丢数据；两个仓库同名实例隔离；老 Java 行为保留   |
| P2：启动策略和生命周期     | `resolve`、LaunchPlan、元数据参数、内存/窗口/隔离、用户参数、进程会话/日志、可见性                                | 捕获实际 argv/cwd/环境验证全局与实例覆盖；原版新旧参数格式通过；启动失败不隐藏；退出后按策略恢复  |
| P3：界面和导航             | 主题、字体、窗口能力、Logo、显隐、格式区域、动画；统一根部效果                                                    | 设置页外也有效；显式主题覆盖系统；返回/直达/记忆路径一致；设置始终可恢复；浏览器和 Tauri 分别验收 |
| P4：网络与资源行为         | 网络客户端策略、代理/DoH、API 来源收口、社区命名/依赖/快速下载、安装后自动选中                                    | Java/原版/导出及声明覆盖的 API 均走新策略；本地代理/下载服务验证；不支持的供应商能力继续禁用      |
| P5：服务依赖项             | Terracotta 偏好、认证方式/外置认证、通知/公告/剪贴板、背景/音乐/主页、配置导入导出                                | 各项有真实消费服务与失败路径；平台能力明确；执行性旧值不静默启用；完成项逐项开放                  |
| P6：清理与最终验收         | 移除旧通用持久化、无消费者 store/默认值、参数透传、废弃字段；更新文档与回归                                       | 每个保留控件都有消费者和测试；无双写；无只保存的可用控件；跨平台证据与剩余不支持项清楚            |

P3 在 P1 稳定后可与 P2 独立推进；P4 的客户端策略可先于 P2 完成，但社区安装行为依赖资源安装器。P5 按服务成熟度拆批，不把“所有占位补齐”作为 P1/P2 的前置条件。

## 8. 回归与验收设计

不为每个 bool 写一份机械测试；测试共享机制的边界，再对每类效果做真实消费验证：

| 层              | 必测内容                                                                                                                                       |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust 模型与解析 | 非法类型/范围/枚举、作用域、初始内存大于最大值、继承/显式空值/重置、Java 原有选择语义、路径别名与同 ID 不同仓库                                |
| 文件与迁移      | 新装、旧 schema、新字段缺省、坏 JSON、未来 schema、无权限/写入失败、原子替换、备份、迁移冲突/重复执行、其他领域同时保存不丢数据                |
| IPC/前端 store  | 读取失败、保存失败保留草稿、revision 冲突、迟到响应、跨页面/窗口同步、防抖/卸载、未选实例、网站能力、字段与默认契约一致性                      |
| 启动消费者      | 使用受控假 executable 捕获真实参数/cwd，不执行个人游戏；验证内存/窗口/参数/继承/命令等待及失败；独立测试子进程退出与窗口恢复；再做真实游戏冒烟 |
| 下载/网络消费者 | 保留现有本地 HTTP 回归；补冷启动直接 Java 下载、配置热更新、代理认证/不泄露日志、旧任务与新客户端边界                                          |
| 视觉/平台       | 真实主题、字体、显隐/恢复、背景/音量效果；Windows/macOS/Linux 的窗口、硬件与进程能力各自记录，浏览器测试不能替代 Tauri                         |
| 导入导出        | 凭据不出现在配置快照/导出/日志；未知字段/版本拒绝；先验证后一次提交，失败原配置和游戏文件保持                                                  |

继续运行现有 `pnpm check:ui` / `pnpm ui:build`、对应 Rust 离线回归和 `git diff --check`。设置回归应验证“已提交快照 → 消费者行为”，保留迁移覆盖；不要把源码包含某个字段名当成生效验收。

## 9. 最终删除/替换清单

- `src/stores/setup.ts`：所有读写转到新设置 store 后删除，不保留第二份启动状态。
- `usePageSettings.ts`：设置迁移完成后删除其持久化职责；临时表单改为普通局部状态，业务偏好改用统一 store。待全部调用点迁移后删除文件。
- `setting-defaults.json`：只在迁移期作为旧默认值/滑块换算依据保留；生产默认改为 Rust，网站默认快照自动导出。保留 UI 对齐所需控件 ID，不保留重复业务默认。
- `downloadSettings.ts`：移除 localStorage 读取；滑块换算成为页面边界/迁移工具，类型与真实值复用新协议；任务不再传全局配置副本。
- `pclLocale.ts`：移除独立语言存储，保留翻译能力；旧 `pcl-language` / `ce-language` 仅由迁移读取。
- `PCLSetupInfo.max_memory/theme/api_provider/selected_java` 与顶层 `java_preferences`：迁到对应语义分组后移除重复字段，保留可测试的旧 schema 解析，不留下两个长期来源。
- `showUnavailable`：非设置页仍可按需使用；设置页只有配套服务验收后才替换占位，永久不支持的选项移除或明确禁用。
- 配置服务不合并任务 store、资源缓存、账号认证和 Terracotta 状态机；它们消费偏好，仍各自管理运行生命周期。

交付标准：一个设置从页面编辑到保存、重启读取、作用域解析、业务消费和错误反馈都有闭环；未完成的项如实不可用。不是“把所有 v-model 都接进一个大对象”。
