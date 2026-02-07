# PCL.Proto 开发指南

## 项目技术栈
- **前端**：Vue 3 + TypeScript + Vite + Pinia + Vue Router  
- **后端**：Rust + Tauri 2  
- **UI**：自定义组件  
- **包管理**：pnpm  

## 核心开发命令
```bash
# 开发
pnpm run dev          # 全栈开发
pnpm run ui:dev       # 仅前端

# 构建
pnpm run build        # 全栈构建
pnpm run ui:build     # 仅前端构建
pnpm run type-check   # 类型检查

# 测试
pnpm run type-check   # 主要验证方式
cargo test            # Rust测试（在src-tauri目录下）
```

## 代码风格要点

### TypeScript/Vue
- **导入**：使用 `@/` 别名，外部库在前
- **命名**：
  - 变量/函数：camelCase
  - 类型/接口：PascalCase  
  - 组件文件：PascalCase（如 `Modal.vue`）
  - Store命名：kebab-case（如 `account-info`）
- **格式化**：无分号、单引号、100字符行宽
- **错误处理**：async操作必须用 try/catch
- **组件**：使用 `<script setup>` + Pug模板 + 组合式API

### Rust
- **命名**：模块/函数/变量用 snake_case，结构体/枚举用 PascalCase
- **错误**：使用 `thiserror` + `Result<T, E>` + `?` 传播
- **导入**：标准库 → 外部crate → 内部模块

## 项目结构
```
src/
├── components/   # Vue组件
├── composables/  # 组合式函数
├── stores/       # Pinia仓库
├── types/        # TS类型定义
└── views/        # 页面

src-tauri/
└── src/
    ├── core/     # Rust核心逻辑
    └── util/     # 工具函数
```

## 关键提醒
1. 始终在修改后运行 `pnpm run type-check`
2. 提交前格式化代码：`pnpm run format`
3. 使用 `pnpm`，勿用 npm/yarn
4. Vue组件文件使用 **PascalCase**
5. Pinia仓库使用 `useStoreName()` 模式
6. 支持中英文国际化（en-US / zh-CN）
7. 子模块初始化：`git submodule update --init --recursive`
8. 后端验证用户输入，利用Tauri安全IPC