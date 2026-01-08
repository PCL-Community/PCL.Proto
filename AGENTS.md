# AGENTS.md - PCL.Proto Development Guide

This document provides essential information for AI agents working on the PCL.Proto project.

## Project Overview

PCL.Proto is a Minecraft launcher prototype built with:

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia + Vue Router
- **Backend**: Rust + Tauri 2
- **UI Framework**: Custom components with Pug templates
- **Package Manager**: pnpm

## Build & Development Commands

### Development

```bash
# Start development server (frontend + Tauri)
pnpm run dev

# Frontend-only development
pnpm run ui:dev
```

### Building

```bash
# Full build (frontend + Tauri)
pnpm run build

# Frontend-only build
pnpm run ui:build

# Type checking
pnpm run type-check
```

### Formatting & Linting

```bash
# Format code with Prettier
pnpm run format

# Type checking (also runs as part of ui:build)
pnpm run type-check
```

### Tauri Commands

```bash
# Run any Tauri command
pnpm run tauri [command]

# Generate icons
pnpm run tauri icon
```

### Deployment

```bash
# Deploy to GitHub Pages
pnpm run deploy
```

## Code Style Guidelines

### TypeScript/JavaScript

**Imports:**

- Use ES modules (`import/export`)
- Group imports: external libraries first, then internal modules
- Use `@/` alias for internal imports (configured in Vite)
- Example from `src/main.ts:12-20`:
  ```typescript
  import { createApp } from 'vue'
  import { createPinia } from 'pinia'
  import router from '@/router/index'
  ```

**Naming Conventions:**

- **Variables/Functions**: camelCase (`getAccount`, `username`)
- **Types/Interfaces**: PascalCase (`AccountInner`, `IVersionShow`)
- **Constants**: UPPER_SNAKE_CASE for true constants
- **Store names**: kebab-case (`account-info`)
- **Component files**: PascalCase (`Modal.vue`, `PButton.vue`)

**Formatting:**

- **Semicolons**: No semicolons (Prettier config)
- **Quotes**: Single quotes
- **Line length**: 100 characters (Prettier config)
- **Trailing commas**: Not used in examples

**Error Handling:**

- Use try/catch for async operations
- Log errors with `console.error`
- Example from `src/stores/account.ts:31`:
  ```typescript
  } catch (error) {
    console.error('Failed to initialize account:', error)
  }
  ```

**TypeScript:**

- Use strict typing
- Define interfaces for complex data structures
- Use `ref` for reactive state in composables
- Prefer composition API with `<script setup>`

### Vue Components

**Component Structure:**

```vue
<script setup lang="ts">
// Imports first
import { useModal } from '@/composables/useModal'
import PButton from './widget/PButton.vue'

// Type definitions
import type { ModalButtonOption } from '@/types/modal'

// Logic
const { isOpen, options, close, inputValue } = useModal()
</script>

<template lang="pug">
// Pug template syntax
</template>

<style scoped>
// Scoped styles
</style>
```

**Template Language:**

- Use **Pug** for templates (`lang="pug"`)
- Use kebab-case for template attributes and events
- Use Vue directives (`v-if`, `v-for`, `v-model`)

**State Management:**

- Use **Pinia** stores for global state
- Use `ref` and `computed` for component state
- Store naming: `useStoreName()` pattern

### Rust (Backend)

**File Structure:**

- Backend code in `src-tauri/src/`
- Core logic in `src-tauri/src/core/`
- Utilities in `src-tauri/src/util/`

**Naming:**

- **Modules**: snake_case (`api_client`, `game_launcher`)
- **Structs/Enums**: PascalCase (`GameInstance`, `GameJava`)
- **Functions**: snake_case (`from_version_folder`)
- **Variables**: snake_case (`version_folder`, `json_files`)

**Error Handling:**

- Use `thiserror` crate for custom error types
- Use `Result<T, E>` for fallible operations
- Use `?` operator for error propagation

**Imports:**

- Group standard library imports first
- Then external crate imports
- Then internal module imports
- Example from `src-tauri/src/core/game.rs:1-6`:

  ```rust
  use std::{fs, path::PathBuf, sync::Arc};

  use crate::core::{
      api_client::game::VersionDetails, java::JavaRuntime, launcher::GameLaunchError,
      mcmod::PluginType, repository::GameRepository,
  };
  ```

## Project Structure

```
src/
├── api/              # API clients and data fetching
├── assets/           # Static assets (fonts, icons, images)
├── components/       # Vue components
│   ├── icons/       # Icon components
│   └── widget/      # Reusable UI widgets
├── composables/      # Vue composables
├── layout/          # Layout components
├── locales/         # Internationalization
├── router/          # Vue Router configuration
├── stores/          # Pinia stores
├── types/           # TypeScript type definitions
├── util/            # Utility functions
└── views/           # Page components

src-tauri/
├── src/
│   ├── core/        # Core Rust logic
│   └── util/        # Rust utilities
└── Cargo.toml       # Rust dependencies
```

## Testing Notes

- No test framework detected in package.json
- Type checking serves as primary validation: `pnpm run type-check`
- Manual testing through development server: `pnpm run dev`

## Git & Workflow

**Submodules:**

- Project uses git submodules (locales directory)
- Initialize with: `git submodule update --init --recursive`

**Commit Messages:**

- Follow conventional commits style
- Use present tense ("Add feature" not "Added feature")
- Reference issues when applicable

## Important Notes for Agents

1. **Always run type checking** after making changes: `pnpm run type-check`
2. **Format code** before committing: `pnpm run format`
3. **Check for submodule updates** when cloning
4. **Tauri requires platform-specific toolchains** (MSVC for Windows, Xcode for macOS)
5. **Use pnpm** not npm or yarn for package management
6. **Component naming**: Use PascalCase for Vue component files
7. **Store pattern**: Use `useStoreName()` pattern for Pinia stores
8. **Error handling**: Always handle async errors with try/catch
9. **Type safety**: Define interfaces for all complex data structures
10. **Internationalization**: Support both English (en-US) and Chinese (zh-CN) locales

## Performance Considerations

- Use `ref` for reactive primitives, `reactive` for objects
- Implement proper cleanup in composables
- Use `v-memo` for expensive template computations when needed
- Consider lazy loading for large components

## Security Notes

- Never commit secrets or API keys
- Validate all user input in Rust backend
- Use Tauri's secure IPC for frontend-backend communication
- Follow Rust's memory safety guarantees for backend code
