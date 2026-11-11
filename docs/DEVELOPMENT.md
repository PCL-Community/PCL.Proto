# 前端维护

页面行为与视觉参考本地 `../PCL2-CE`，保留 PCL.Proto 自身身份。当前职责与设置边界见 [ARCHITECTURE.md](ARCHITECTURE.md)；以下只保留可继续使用的实现约定。

## 实现约定

- 页面使用 Vue 3 SFC、响应式状态、组合式函数与 Vue Router；沿用现有卡片、按钮、左右布局和导航。
- 同构页面沿用源名称，`My*` 控件改为 `P*`；不建立 `Ce*` 或专用 ce 目录。
- `src/assets/layouts` 的 70 份 JSON 是源布局快照，用于追溯和检查，不参与运行时页面渲染。70 是布局文件数量，包含控件及组合布局，不代表 70 条独立路由。
- 源文案从语言资源提取，由 `pclText` 按当前语言解析；表单默认值、导出选项、导航辅助操作、下载分类从源 XAML 和配置提取。
- Lucide 通用图标通过官方 `@lucide/vue` 包具名导入。`pclIcons.ts` 注册 92 个名称（包括源图标和前端辅助图标）；不复制 Lucide SVG 到 public。
- 方块图标只保留一份，放在 `src/assets/icons/instance/`，文件名沿用 CE 的 `images/Blocks` 名称（动画图用同名 `.gif`）。页面和 CE 路径都经 `src/api/pclAssets.ts` 的 `blockImage`/`pclImagePath` 解析，`public/pcl/images` 不再放副本；语言、皮肤、头像等非图标资源仍按 CE 目录留在 public。
- Logo、镐头等自定义 WPF Geometry 从源代码或 XML 提取，保留填充规则、描边与边界，生成 26 份 SVG；图片保留源目录结构。资源来源及许可证见 `public/pcl/LICENCE`。
- 悬停与按下背景（`--color-tint-lighter`、`--half-transparent-blue`）取主题色的半透明叠加，而不是源色阶 `ColorBrush6/7` 的纯色：深色主题下这些纯色阶比卡片表面更暗，会把按钮、列表项和下拉框的悬停画成一块黑。调整 CE 色板时不要改回 `--ce-color-6/7`。
- 交互表面的静止态使用 `--color-tint-rest`，与悬停态同色、仅 alpha 为 0。按钮背景过渡由 CSS `:hover/:active` 承担，`PButton` 只把 scale 交给 Motion。Java 行回弹时图标变灰的原因是禁用状态改色，见下条。
- 图标操作统一复用 `.pcl-icon-button`：对齐 CE `MyIconButton` 的 `ColorBrush4` 常态与 `ColorBrush2` hover，不为 disabled 增加改色规则。Java 行保存时仍临时禁用操作，但图标保持原色；Home 三个账户操作不再单独覆盖为最深的 `--color-tint-dark`。
- 原生主页制作器、加载方式及 CE 兼容边界见 [CUSTOM_HOMEPAGE.md](CUSTOM_HOMEPAGE.md)。
- 原有自定义主页 XAML 输入仍兼容 `My*`，也接受 `P*`；这是用户主页功能，不是整套源页面的运行时解释器。

桌面业务设置通过对应领域 IPC 和 `createSettingsPersistence` 保存；未接入后端的页面偏好不代表功能已实现。设置失败保留草稿，实例作用域使用目录而非名称。布局和路由映射以 `src/router/`、`src/views/` 为准，不另维护逐页副本。

## Proto 自身身份例外

按要求保留 Header 的 Proto 标记、首页 Proto 介绍、关于页开发者 AMagicPear、应用版本、Vue/Rust/Tauri 技术栈及本项目仓库。标题栏 Logo 使用 Proto 原有的 52×16 图形，加载镐头保留原有动画；这两项不跟随 CE 资源。CE Geometry 仍完整保留在 public 资源中。
更新和反馈针对 PCL.Proto 的真实 GitHub Release / Issue。CE 的更新渠道、CDK、赞助入口和 EasyTier 模块下载协议不套用到 Proto。
源中默认隐藏且没有适用功能的网络测试及崩溃按钮保持隐藏。

## 检查

```sh
bun run check:ui
bun run ui:build
```

提取资源、图标映射和多语言文案随仓库直接维护，不覆盖手工 Vue 页面。`src/assets/layouts` 是追溯快照，不参与运行时渲染。组件包的构建与接入见 [组件指南](../packages/components/README.md)。

浏览器可预览界面和公开服务；本机文件、Java、账户和游戏进程能力需要 Tauri。未完成动作必须显示明确提示，不能返回伪造成功。构建通过不代表逐像素对齐或原生 WebView 验收。

## 应用多语言

应用统一使用 `src/api/pclLocale.ts`。页面、组件、导航、路由标题和 store 共用一个响应式语言状态，不再使用 `vue-i18n`。

- `src/assets/languages/*.json`：来自 PCL2-CE 的七种语言资源，随仓库直接维护。
- `src/assets/messages/*.json`：PCL.Proto 自有文案，使用 `Proto.*` 键。新增或修改文案请编辑这里，不要改上游资源。当前提供简体中文和英语。
- `src/assets/language-names.json`：语言选项列表。增加语言时，同时提供对应的上游语言文件。

调用示例：

```ts
import { pclText, pclLocale, setPclLocale } from '@/api/pclLocale'

pclText('Common.Action.Cancel')
pclText('Proto.Instance.Selected', [instance.name])
await setPclLocale('en-US')
```

`pclText` 支持上游 `{DynamicResource Key}` / `{StaticResource Key}` 引用，以及 `{0}`、`{1}` 参数。同一参数出现多次时会全部替换。新增代码直接传翻译键和参数数组，不要拼接句子或自行 `.replace()`。

组件模板直接调用 `pclText` 即可跟随语言变化；脚本中的展示数据使用 `computed` 或 getter，不要在模块初始化时保存已经翻译的字符串。路由等静态配置保存翻译键，在显示时解析。外部资源、用户输入、第三方主页编辑器和接口原始错误保留原文。

启动时在挂载应用前读取并加载语言，优先使用 `pcl-language`，兼容旧的 `ce-language`。不支持的语言或启动加载失败时回退到 `zh-CN`。切换语言先加载资源，再更新界面、`html[lang]` 和存储；失败时保留当前语言，连续切换以最后一次选择为准。存储不可用时仍允许本次会话切换语言。

缺少翻译的键回退到简体中文；未知键保留原文。因此其他五种语言尚未翻译的 Proto 文案会显示中文。自有中文和英文资源必须保持键与参数一致。

运行 `bun run check:ui` 检查资源引用、语言加载、缓存、回退、参数替换、旧配置兼容、失败重试及切换顺序；`bun run ui:build` 同时执行类型检查和生产构建。
