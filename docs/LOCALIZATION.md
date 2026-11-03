# 应用多语言

应用统一使用 `src/api/pclLocale.ts`。页面、组件、导航、路由标题和 store 共用一个响应式语言状态，不再使用 `vue-i18n`。

- `src/assets/languages/*.json`：从 PCL2-CE 同步的七种语言资源，由 `scripts/sync-pcl-ui.py` 管理。
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
