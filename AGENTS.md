# GPUI Hooks - 开发者指南

本文档为代码助手（如Claude、Cursor、Copilot等）提供本项目的构建、测试、代码风格和开发规范的详细说明。

## 项目概述

GPUI Hooks 是一个为 [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 框架添加 React 风格 Hook 系统的 Rust 库。项目采用 Cargo workspace 结构，包含两个 crate：

- `gpui-hooks`：主库，提供 Hook 实现和核心 trait
- `gpui-hooks-macros`：过程宏 crate，提供 `#[hook_element]` 属性宏

## 工作空间结构

```
gpui-hooks/
├── Cargo.toml          # Workspace 配置
├── gpui-hooks/
│   ├── Cargo.toml      # 主库配置
│   ├── src/
│   │   ├── lib.rs      # 主库入口，定义核心 trait
│   │   ├── hooks.rs    # Hook trait 定义和通用实现
│   │   └── hooks/      # 具体 Hook 实现
│   │       ├── use_state.rs
│   │       ├── use_effect.rs
│   │       └── use_memo.rs
│   └── examples/
│       └── basic.rs    # 使用示例
└── gpui-hooks-macros/
    ├── Cargo.toml      # 宏库配置
    └── src/
        └── lib.rs      # `#[hook_element]` 宏实现
```

## 构建命令

### 基础构建

```bash
# 构建所有 workspace 成员（debug 模式）
cargo build

# 构建所有 workspace 成员（release 模式）
cargo build --release

# 构建特定 crate
cargo build -p gpui-hooks
cargo build -p gpui-hooks-macros

# 检查编译（不生成二进制）
cargo check
cargo check --release
```

### 单个 crate 构建

```bash
cd gpui-hooks && cargo build
cd gpui-hooks-macros && cargo build
```

## 测试命令

### 运行所有测试

```bash
# 运行 workspace 中所有测试
cargo test

# 运行特定 crate 的测试
cargo test -p gpui-hooks
cargo test -p gpui-hooks-macros

# 显示详细输出
cargo test --verbose
```

### 运行单个测试

```bash
# 运行特定测试文件中的所有测试
cargo test --test <test_file_name>

# 运行单个测试函数
cargo test <test_function_name>

# 注意：当前项目尚未添加测试，以上命令为通用模式
```

### 测试覆盖率（如已安装 tarpaulin）

```bash
cargo tarpaulin --workspace --ignore-tests
```

## Lint 和格式化

### Clippy（代码质量检查）

```bash
# 运行 clippy 检查
cargo clippy

# 以更严格的规则运行
cargo clippy -- -D warnings

# 修复可自动修复的问题
cargo clippy --fix

# 针对特定 crate
cargo clippy -p gpui-hooks
```

### Rustfmt（代码格式化）

```bash
# 检查格式化
cargo fmt --check

# 格式化所有代码
cargo fmt

# 格式化特定 crate
cd gpui-hooks && cargo fmt
```

### 预提交检查

建议在提交前运行：

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## 代码风格指南

### 导入顺序和分组

遵循以下导入顺序（每个组之间空一行）：

1. 标准库 (`std`, `core`, `alloc`)
2. 外部 crate（按字母顺序）
3. 当前 crate 的父模块
4. 当前模块的子模块
5. `self` 导入

示例：

```rust
// 标准库
use std::cell::{Ref, RefCell, RefMut};
use std::any::Any;

// 外部 crate
use gpui::{Context, IntoElement, Window};
use gpui_hooks_macros::hook_element;

// 当前 crate
use super::Hook;
use crate::hooks::{HasHooks, UseStateHook};
```

### 命名约定

- **结构体/枚举**：`PascalCase`（如 `UseState`, `UseEffect`）
- **Trait**：`PascalCase` 并以 `Hook` 结尾（如 `UseStateHook`, `HookedElement`）
- **函数/方法**：`snake_case`（如 `use_state`, `get_mut`）
- **变量/参数**：`snake_case`（如 `hook_index`, `boxed_deps`）
- **常量**：`SCREAMING_SNAKE_CASE`（如 `MAX_HOOK_COUNT`）
- **模块**：`snake_case`（如 `hooks`, `use_state`）

### 类型和泛型

- 使用显式类型标注，避免不必要的类型推断
- 泛型参数使用单个大写字母或描述性名称：`T`, `F`, `D`
- 生命周期参数使用 `'a`, `'b` 等

### 错误处理

- **Hook 顺序错误**：使用 `panic!`（因为这是编程错误）
- **运行时错误**：当前库主要使用 panic，未来可能添加 `Result` 返回
- **依赖检查失败**：静默处理，返回 false

错误消息应提供足够的信息：

```rust
panic!(
    "Hook count changed from {} to {}. Hooks must be called in the same order every render.",
    prev, current
);
```

### Unsafe 代码指南

本库在以下情况下使用 `unsafe`：

1. **Raw pointers 用于闭包捕获**：Hook 实现中使用原始指针创建稳定的闭包引用
2. **类型转换**：使用 `downcast_ref` 和 `downcast_mut` 进行安全的 trait object 转换

必须遵循的安全准则：

- 确保指针在闭包生命周期内有效
- 使用 `#[inline(never)]` 防止闭包内联导致悬垂指针
- 添加安全注释说明不变量

示例：

```rust
// 安全：hook_ptr 在闭包生命周期内有效，因为 hooks 向量在组件生命周期内稳定
let getter: Box<dyn Fn() -> T> = {
    let state_ptr = hook_ptr as *const UseState<T>;
    Box::new(move || unsafe { (*state_ptr).get().clone() })
};
```

### 注释规范

- **文档注释**：使用 `///` 为公共 API 提供文档
- **模块注释**：使用 `//!` 为模块提供概述
- **实现注释**：使用 `//` 解释复杂逻辑
- **TODO/FIXME**：使用 `// TODO:` 或 `// FIXME:` 标记待办事项
- **安全注释**：`// SAFETY:` 解释 unsafe 代码的安全保证

文档注释应包含：

- 简要描述
- 示例代码（如适用）
- Panics 情况
- 安全性说明

### 代码组织

- 每个文件不超过 500 行
- 每个函数不超过 50 行
- 复杂的逻辑拆分为辅助函数
- 相关的类型和 trait 放在同一模块中

## Hook 系统特殊规范

### Hook 规则

1. **顺序不变性**：Hook 必须在每次渲染中以相同顺序调用
2. **条件调用**：不能在条件语句、循环或嵌套函数中调用 Hook
3. **函数组件**：Hook 只能在实现了 `HookedElement` 的结构体中使用

### 生命周期管理

- **Effect 清理**：组件必须手动调用 `cleanup_effects()` 在 Drop 实现中
- **内存泄漏**：确保 effect 清理函数被正确调用
- **依赖数组**：Hook 依赖项必须正确实现 `PartialEq + Clone + 'static`

### 性能考虑

- **Memoization**：使用 `use_memo` 避免重复计算
- **Effect 依赖**：正确指定依赖数组避免不必要的 effect 执行
- **闭包创建**：避免在渲染中创建不必要的闭包

## 宏开发指南

### `#[hook_element]` 宏

- 只能用于具有命名字段的结构体
- 自动添加 `_hooks`, `_hook_index`, `_prev` 字段
- 自动实现 `Default`, `HookedElement`, `gpui::Render`
- 保持原始字段不变

### 过程宏规范

- 使用 `syn` 和 `quote` crate 进行语法树操作
- 提供清晰的错误消息
- 保持宏输出格式良好

## 示例和演示

### 运行示例

```bash
# 运行基础示例
cargo run --example basic

# 构建示例
cargo build --examples
```

### 创建新示例

1. 在 `gpui-hooks/examples/` 目录下创建 `<name>.rs`
2. 添加 `[[example]]` 到 `gpui-hooks/Cargo.toml`
3. 确保示例展示 Hook 的主要功能

## 提交和版本控制

### 提交消息格式

```
类型(范围): 简要描述

详细说明（可选）

- 列表项（可选）
```

类型包括：`feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### 分支策略

- `main`：稳定版本
- `develop`：开发分支
- `feature/*`：新功能
- `fix/*`：错误修复

## 故障排除

### 常见问题

1. **Hook 顺序错误**：检查条件渲染中的 Hook 调用
2. **编译错误**：确保 GPUI 版本兼容
3. **运行时 panic**：检查 effect 清理函数的调用

### 调试建议

- 启用 `RUST_BACKTRACE=1` 环境变量
- 使用 `dbg!()` 宏调试 Hook 状态
- 检查依赖数组是否按预期变化

---

_本文档最后更新：2026-02-18_  
_适用于所有 AI 编码助手和开发者_
