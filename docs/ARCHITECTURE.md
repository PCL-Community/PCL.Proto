# 代码组织与维护边界

更新：2026-11-01。本文描述当前实现的职责划分；功能对齐记录见 [CORE_ALIGNMENT.md](CORE_ALIGNMENT.md)，设置规划见 [SETTINGS_DESIGN.md](SETTINGS_DESIGN.md)。规划中的能力不等于已经实现。

## 前端

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

## Rust 后端

`src-tauri/src/commands/mod.rs` 只声明命令模块。`lib.rs` 直接注册各模块的命令；模块调整不改变前端 IPC 名称或参数。

| 命令模块                | 职责                                                     |
| ----------------------- | -------------------------------------------------------- |
| `launch.rs`             | 启动控制、取消、进度转发和启动脚本导出；共用启动计划构建 |
| `launch_settings.rs`    | 全局/实例启动参数与实例启动附加设置                      |
| `memory.rs`             | 内存设置和预览                                           |
| `java.rs`               | Java 列表、偏好、探测、安装与安装取消                    |
| `instances.rs`          | 实例选择、说明、重命名、隔离与实例目录操作               |
| `instance_resources.rs` | 实例资源目录、文件列表、导入、启禁、删除与修复           |
| `repositories.rs`       | Minecraft 文件夹管理和文件夹内的实例查询                 |
| `versions.rs`           | 游戏版本与加载器元数据查询                               |
| `network.rs`            | 代理设置                                                 |
| `account.rs`            | 返回不含凭据的账户投影                                   |
| `skin.rs`               | 皮肤解析、缓存、另存与正版皮肤上传                       |

命令层负责参数适配、任务编排与 UI 通道；Minecraft、Java、下载和文件操作的领域实现保留在 `minecraft/`、`io/` 等现有模块。`ConfigManager` 负责事务更新与落盘。目录边界、符号链接检查、配置保存失败处理和重命名回滚不能在整理时省略。

## 维护约定与验证

`scripts/` 只保留图标生成与 CE 资源同步工具。前端回归检查统一放在 `tests/ui/*.test.mjs`，由 `bun run check:ui` 使用 Bun 测试运行器逐文件执行；新增测试无需修改命令列表。临时界面对照入口不随应用保留。

新增功能先找所属领域的现有入口。公共规则只保留一个实现，调用方直接 import 所属模块；不要为转调一次函数而增加页面包装、兼容转导出或新的 service 层。页面确有独有状态、校验、错误处理或交互时再保留本地函数。

本次整理的设置回归位于 `tests/ui/settings-persistence.test.mjs`，覆盖串行与并行作用域、快照隔离、失败重试、过期完成、读取期间编辑、真实领域 IPC 负载和实例编辑器重新挂载。该检查已接入 `bun run check:ui`。

常规验证：`bun run ui:build`、`cargo check --workspace`、`cargo test -p pcl-proto --lib`。离线检查与构建不代表已完成真实游戏启动或 Windows/Linux 桌面验收。
