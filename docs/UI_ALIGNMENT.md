# PCL2-CE 页面对齐记录

参考仓库：`../PCL2-CE`（PCL-Community/PCL2-CE）。
参考 revision：`d68819990da092ed7e582b94b950874fa30baffe`。

本文保留前端迁移阶段的范围与验收记录。2026-10-29 起的后端迁移及原版安装验证见 [后端迁移记录](BACKEND_ALIGNMENT.md)。

## 实现约定

- 页面使用 Vue 3 SFC、响应式状态、组合式函数与 Vue Router；沿用现有卡片、按钮、左右布局和导航。
- 同构页面沿用源名称，`My*` 控件改为 `P*`；不建立 `Ce*` 或专用 ce 目录。
- `src/assets/layouts` 的 70 份 JSON 是源布局快照，用于追溯和检查，不参与运行时页面渲染。70 是布局文件数量，包含控件及组合布局，不代表 70 条独立路由。
- 源文案从语言资源提取，由 `pclText` 按当前语言解析；表单默认值、导出选项、导航辅助操作、下载分类从源 XAML 和配置提取。
- Lucide 通用图标通过官方 `@lucide/vue` 包具名导入。`pclIcons.ts` 注册 92 个名称（包括源图标和前端辅助图标）；不复制 Lucide SVG 到 public。
- Logo、镐头等自定义 WPF Geometry 从源代码或 XML 提取，保留填充规则、描边与边界，生成 26 份 SVG；图片保留源目录结构。资源来源及许可证见 `public/pcl/LICENCE`。
- 悬停与按下背景（`--color-tint-lighter`、`--half-transparent-blue`）取主题色的半透明叠加，而不是源色阶 `ColorBrush6/7` 的纯色：深色主题下这些纯色阶比卡片表面更暗，会把按钮、列表项和下拉框的悬停画成一块黑。同步 CE 色板时不要改回 `--ce-color-6/7`。
- 交互表面的静止态使用 `--color-tint-rest`，与悬停态同色、仅 alpha 为 0。按钮背景过渡由 CSS `:hover/:active` 承担，`PButton` 只把 scale 交给 Motion。相关样式约定由 `tests/ui/components.test.mjs` 检查；Java 行回弹时图标变灰的原因是禁用状态改色，见下条。
- 图标操作统一复用 `.pcl-icon-button`：对齐 CE `MyIconButton` 的 `ColorBrush4` 常态与 `ColorBrush2` hover，不为 disabled 增加改色规则。Java 行保存时仍临时禁用操作，但图标保持原色；Home 三个账户操作不再单独覆盖为最深的 `--color-tint-dark`。
- 原有自定义主页 XAML 输入仍兼容 `My*`，也接受 `P*`；这是用户主页功能，不是整套源页面的运行时解释器。

设置的默认值、持久化和重置集中在 `usePageSettings`，页面动作由页面函数处理。通用设置行通过 `SetupItem` 的标签和控件插槽组合，按钮与列表使用标准 `@click` 事件；文本框、下拉框将原生属性及监听器传给内部控件。组件内部样式由组件维护，菜单卡片通过 `overflow-visible` 显式放开裁剪。

## Proto 自身身份例外

按要求保留 Header 的 Proto 标记、首页 Proto 介绍、关于页开发者 AMagicPear、应用版本、Vue/Rust/Tauri 技术栈及本项目仓库。标题栏 Logo 使用 Proto 原有的 52×16 图形，加载镐头保留原有动画；同步脚本不覆盖这两项。CE Geometry 仍完整保留在 public 资源中。
更新和反馈针对 PCL.Proto 的真实 GitHub Release / Issue。CE 的更新渠道、CDK、赞助入口和 EasyTier 模块下载协议不套用到 Proto。
源中默认隐藏且没有适用功能的网络测试及崩溃按钮保持隐藏。

## 同步和检查

```sh
python3 scripts/sync-pcl-ui.py ../PCL2-CE
node tests/ui/pcl-ui.test.mjs ../PCL2-CE
pnpm ui:build
git diff --check
```

同步脚本只更新提取资源和图标映射，不生成或覆盖手工 Vue 页面。已验证连续两次同步的 248 个相关文件哈希一致。
检查脚本覆盖源布局哈希、文案引用、图片路径、图标映射、Geometry 边界与填充规则、同构命名、import 文件名大小写、关键默认值、导出选项、导航辅助操作、版本分类、加载器互斥/显隐/警告和公开 API 的畸形响应与 HTTP 错误。

## 接口范围

- Minecraft 版本列表与 Modrinth 搜索/详情/版本在桌面沿用 Tauri；普通浏览器使用相同公开服务。GitHub 更新和反馈使用 Proto 仓库。
- 原有原版下载、实例、Java、离线账号和联机接口继续复用；依赖本机环境的动作仅桌面可用。
- 本轮主要迁移页面与文案。源端文件操作、加载器安装、OAuth/微软认证、账号更改、收藏夹文件夹管理和批量下载等尚未实现的动作显示明确提示，不伪造成功。
- 一部分源设置仅保存前端偏好，尚未映射到 Rust 启动器配置。实例警告、内嵌模组、存档详情、启动进度等通过 props/slots 接收状态；后端没有供数时显示空状态或隐藏条件面板。
- 侧栏刷新重新挂载当前页面；设置初始化入口目前提示未接入。

## 验证范围与限制

已执行源资源一致性检查、Vue 类型检查和 Vite 生产构建。组件整理后新增原生属性转发、标签 ID、设置持久化/重置与实例隔离回归检查，并纳入 `pnpm check:ui`。浏览器复核了设置表单、标签聚焦、卡片折叠、按钮单次触发、输入禁用、皮肤菜单、资源下载跳转与大厅空值校验。浏览器在 850×600 检查主要设置、实例、存档、资源、日志、账号及联机页面的空状态、资源加载和横向溢出。
真实服务验证包含 Modrinth 列表与 Sodium 详情、游戏版本/加载器筛选、收藏添加与取消、Proto Release 与 Issue 获取。另已验证 OAuth/密码模式切换、皮肤与新建档案菜单、LittleSkin 预设覆盖的确认/取消、侧栏刷新，以及联机完成页在窄窗口下无横向溢出。收藏与认证字段测试数据已恢复。

没有运行 Windows WPF 参考程序，未进行逐像素截图比较。未验证真实游戏启动/安装、Java 扫描、联机建房与账号认证，未运行 Rust 构建。生产构建仍有现有 SkinViewer 与入口 chunk 大小告警。

## 源布局与前端入口

表中路径按当前迁移结果列出；左侧页面复用导航模型，FormMain 复用应用根布局。

| 源布局                      | 前端入口                                                  |
| --------------------------- | --------------------------------------------------------- |
| `PCompItem`                 | `src/components/widget/PCompItem.vue`                     |
| `PageComp`                  | `src/views/Download/PageComp.vue`                         |
| `PageDownloadCompDetail`    | `src/views/Download/PageDownloadCompDetail.vue`           |
| `PageDownloadDataPack`      | `src/views/Download/PageDownloadDataPack.vue`             |
| `PageDownloadMod`           | `src/views/Download/PageDownloadMod.vue`                  |
| `PageDownloadPack`          | `src/views/Download/PageDownloadPack.vue`                 |
| `PageDownloadResourcePack`  | `src/views/Download/PageDownloadResourcePack.vue`         |
| `PageDownloadShader`        | `src/views/Download/PageDownloadShader.vue`               |
| `PageDownloadWorld`         | `src/views/Download/PageDownloadWorld.vue`                |
| `PageDownloadCleanroom`     | `src/views/Download/PageDownloadCleanroom.vue`            |
| `PageDownloadClient`        | `src/views/Download/PageDownloadClient.vue`               |
| `PageDownloadCompFavorites` | `src/views/Download/PageDownloadCompFavorites.vue`        |
| `PageDownloadFabric`        | `src/views/Download/PageDownloadFabric.vue`               |
| `PageDownloadForge`         | `src/views/Download/PageDownloadForge.vue`                |
| `PageDownloadInstall`       | `src/views/Download/PageDownloadInstall.vue`              |
| `PageDownloadLabyMod`       | `src/views/Download/PageDownloadLabyMod.vue`              |
| `PageDownloadLeft`          | `src/views/Download/PageDownloadLeft.ts`                  |
| `PageDownloadLegacyFabric`  | `src/views/Download/PageDownloadLegacyFabric.vue`         |
| `PageDownloadLiteLoader`    | `src/views/Download/PageDownloadLiteLoader.vue`           |
| `PageDownloadNeoForge`      | `src/views/Download/PageDownloadNeoForge.vue`             |
| `PageDownloadOptiFine`      | `src/views/Download/PageDownloadOptiFine.vue`             |
| `PageHelpDetail`            | `src/views/Tools/PageHelpDetail.vue`                      |
| `PageHomepageNewsView`      | `src/views/Home/PageHomepageNewsView.vue`                 |
| `PLocalModItem`             | `src/components/widget/PLocalModItem.vue`                 |
| `PageInstanceCompJarInJar`  | `src/views/InstanceSetting/PageInstanceCompJarInJar.vue`  |
| `PageInstanceCompResource`  | `src/views/InstanceSetting/PageInstanceCompResource.vue`  |
| `PageInstanceExport`        | `src/views/InstanceSetting/PageInstanceExport.vue`        |
| `PageInstanceInstall`       | `src/views/InstanceSetting/PageInstanceInstall.vue`       |
| `PageInstanceLeft`          | `src/views/InstanceSetting/PageInstanceLeft.ts`           |
| `PageInstanceModDisabled`   | `src/views/InstanceSetting/PageInstanceModDisabled.vue`   |
| `PageInstanceOverall`       | `src/views/InstanceSetting/PageInstanceOverall.vue`       |
| `PageInstanceSavesDatapack` | `src/views/InstanceSetting/PageInstanceSavesDatapack.vue` |
| `PageInstanceSavesInfo`     | `src/views/InstanceSetting/PageInstanceSavesInfo.vue`     |
| `PageInstanceSavesLeft`     | `src/views/InstanceSetting/PageInstanceSavesLeft.vue`     |
| `PageInstanceSaves`         | `src/views/InstanceSetting/PageInstanceSaves.vue`         |
| `PageInstanceScreenshot`    | `src/views/InstanceSetting/PageInstanceScreenshot.vue`    |
| `PageInstanceServer`        | `src/views/InstanceSetting/PageInstanceServer.vue`        |
| `PageInstanceSetup`         | `src/views/InstanceSetting/PageInstanceSetup.vue`         |
| `ServerCard`                | `src/components/widget/ServerCard.vue`                    |
| `PMsgLogin`                 | `src/components/widget/PMsgLogin.ts`                      |
| `PSkin`                     | `src/components/widget/PSkin.vue`                         |
| `PageLaunchLeft`            | `src/views/Home/PageLaunchLeft.vue`                       |
| `PageLaunchRight`           | `src/views/Home/PageLaunchRight.vue`                      |
| `PageLoginAuth`             | `src/views/Home/PageLoginAuth.vue`                        |
| `PageLoginMs`               | `src/views/Home/PageLoginMs.vue`                          |
| `PageLoginOffline`          | `src/views/Home/PageLoginOffline.vue`                     |
| `PageLoginProfile`          | `src/views/Home/PageLoginProfile.vue`                     |
| `PageLoginProfileSkin`      | `src/views/Home/PageLoginProfileSkin.vue`                 |
| `PageLogLeft`               | `src/views/Tools/PageLogLeft.vue`                         |
| `PageLogRight`              | `src/views/Tools/PageLogRight.vue`                        |
| `PageSelectLeft`            | `src/views/InstanceSelect/PageSelectLeft.vue`             |
| `PageSelectRight`           | `src/views/InstanceSelect/PageSelectRight.vue`            |
| `PageSetupAbout`            | `src/views/Setup/PageSetupAbout.vue`                      |
| `PageSetupFeedback`         | `src/views/Setup/PageSetupFeedback.vue`                   |
| `PageSetupGameLink`         | `src/views/Setup/PageSetupGameLink.vue`                   |
| `PageSetupGameManage`       | `src/views/Setup/PageSetupGameManage.vue`                 |
| `PageSetupJava`             | `src/views/Setup/PageSetupJava.vue`                       |
| `PageSetupLaunch`           | `src/views/Setup/PageSetupLaunch.vue`                     |
| `PageSetupLauncherLanguage` | `src/views/Setup/PageSetupLauncherLanguage.vue`           |
| `PageSetupLauncherMisc`     | `src/views/Setup/PageSetupLauncherMisc.vue`               |
| `PageSetupLeft`             | `src/views/Setup/PageSetupLeft.ts`                        |
| `PageSetupLog`              | `src/views/Setup/PageSetupLog.vue`                        |
| `PageSetupUI`               | `src/views/Setup/PageSetupUI.vue`                         |
| `PageSetupUpdate`           | `src/views/Setup/PageSetupUpdate.vue`                     |
| `PageSpeedLeft`             | `src/views/Download/PageSpeedLeft.vue`                    |
| `PageSpeedRight`            | `src/views/Download/PageSpeedRight.vue`                   |
| `PageToolsGameLink`         | `src/views/Tools/PageToolsGameLink.vue`                   |
| `PageToolsLeft`             | `src/views/Tools/PageToolsLeft.ts`                        |
| `PageToolsTest`             | `src/views/Tools/PageToolsTest.vue`                       |
| `FormMain`                  | `src/App.vue` + `src/components/WindowHeader.vue`         |
