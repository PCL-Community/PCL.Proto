# 原生主页与制作器

入口：工具 → 百宝箱 → PCL 主页制作器，或设置 → 个性化 → 主页。

制作器使用项目现有的 PCard、PButton、PTextBox、PComboBox、PHint 等组件。预览与启动页共用 `PCustomHomepage` 和 `xamlParser` 渲染器，不嵌入外部网页，不加载脚本或 WPF 对象。

## 编辑和保存

- 组件模式支持添加、选择、编辑属性、上下移动和删除；新组件添加到所选容器中，卡片添加到根层。
- XAML 模式可以编辑完整文档，导入时保留原文、注释和未知元素。组件修改仅序列化 XML，不删除未知属性或未知元素。
- 草稿单独保存在当前 WebView/浏览器的 localStorage。读取当前主页与导入文件会替换当前编辑内容。
- “应用到主页”先验证，再原子写入启动器配置目录的 `Custom.xaml`，成功后切换为本地主页。路径使用本项目配置目录，不依赖 Windows 可执行文件旁的 `PCL` 目录。
- 网页演示将本地主页保存在当前浏览器中；桌面版将其保存为实际文件。“导出 Custom.xaml”使用下载另存为，可将产物交给 CE 使用。
- 文件使用 UTF-8，最多 2 MiB、5000 个 XML 节点和 64 层嵌套。DTD 与外部实体被拒绝。

## CE 兼容范围

实现以 `../PCL2-CE/Plain Craft Launcher 2/Pages/PageLaunch/PageLaunchRight.xaml.cs`、`Modules/Event/EventTypeMapper.cs` 和 `Modules/Event/CustomEvent.cs` 为行为参照。

支持多根 XAML 片段、显式命名空间和 BOM；`local:MyCard`、`MyHint`、`MyButton`、`MyTextButton`、`MyListItem`、`MyImage`、`MyIconButton`、`MyIconTextButton` 映射为项目组件。旧 Proto 的 `PCard`、`PHint`、`PButton`、`PListItem`、`PImage` 名称也继续兼容。

布局支持 StackPanel、WrapPanel、Grid 的行列定义/跨度、Border；文本支持 TextBlock、Run、Span、Bold、Italic、Underline、LineBreak、Hyperlink 和 Inlines/Content/Children 属性节点；图片支持 HTTP(S) 和内置 `pack://application:,,,/images/…`，支持失败备用源。矢量路径复用 PPath，Lucide 图标复用 PSvgIcon。

基础宽高、边距、对齐、可见性、文字颜色/字号/粗斜体、ARGB 颜色和 ColorBrush1–8 主题资源会转换为原生样式。CE 常见的 `25,38/40,23,15` 卡片内容边距会消除重复标题留白；其他边距保留。WPF 布局和动画不会逐像素复刻。

事件接受中英文 `EventType/EventData`，以及新版 `local:CustomEventService.EventType/EventData` 的属性和属性节点形式：

| 事件 | 行为 |
| --- | --- |
| OpenUrl / 打开网页 | 复用项目外链校验与系统浏览器打开流程 |
| CopyText / 复制文本 | 复制，失败时显示错误 |
| ShowDialog / 弹出窗口 | 标题、内容、确认按钮；支持 `\n` |
| ShowHint / 弹出提示 | Info/Success/Error，兼容 Finish/Critical |
| RefreshHomepage / RefreshPage | 重新加载当前主页，保留可用旧内容 |
| DailyFortune / 今日人品 | 桌面版调用现有人品功能 |
| SwitchPage / 切换页面 | CE 数字和英文 PageType/PageSubType 映射至已有路由；缺少上下文或未实现的页面报错 |
| LaunchGame / 启动游戏 | `实例名或 ID\|服务器`；优先当前目录，无当前实例时搜索已配置目录；唯一匹配才选中，复用现有完整启动流程 |

预览中事件只显示类型和参数，不执行动作。未支持的控件和事件会显示兼容提示；事件按钮禁用，XML 内容仍保留在编辑器中。

占位符支持 `{user}`、`{uuid}`、`{login}`、`{name}`、`{version}`、`{version_path}`、`{verpath}`、`{minecraft}`、`{pcl_version}`、`{date}`、`{time}`、`{hint}`。替换发生在 XML 解析后的文本/属性中，玩家名等数据不会成为 XML 标记。`{hint}` 使用 PCL2-CE 随附的中英文提示资源（Apache-2.0，许可文件见 `src/assets/homepage/CE-LICENSE.txt`），过滤涉及 PCL 本身的条目；当前不读取 CE 的外部 `hints.txt`。

联网主页复用桌面下载请求与代理设置，网页演示受浏览器 CORS 限制；只接受无 URL 凭据的 HTTP(S) 地址。使用最近一次成功的地址/内容缓存，后台更新失败时保留缓存并显示错误，切换地址时避免显示其他主页内容，过期请求不会覆盖新请求。预设使用 CE 的实际编号（0、2–15），0 为随机提示，15 使用 CE 的 Minecraft 官方新闻接口；已移除的 CE 预设 1 不出现在选择器中。第三方预设的在线可用性与全部 XAML 用法未逐一验收。

## 尚未兼容

此实现不是 WPF 运行时。暂不支持任意本地文件图片、Windows 文件/命令执行、下载/整合包/帮助等其他 CE 事件、设置/变量读写、WPF Binding/Style/Template/ObjectDataProvider、CustomLive JSON 补丁协议、CE 网络主页版本 INI 协商和图片磁盘缓存。占位符中的可执行路径、Java 路径、设置值和高级自定义变量也未开放。

以上差异须作为兼容边界保留，不能据此宣称 CE 自定义主页全量对齐。制作器导入未知内容后会保留 XML；导入成功不等于全部功能都可执行。
