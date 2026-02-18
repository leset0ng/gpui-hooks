# GPUI Hooks

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> 英文版本 (English Version): [README.md](README.md)

一个为 [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) 框架添加 React 风格 Hook 系统的 Rust 库。

## 特性

- **React 风格 Hooks**：`use_state`, `use_effect`, `use_memo`
- **属性宏**：`#[hook_element]` 自动为结构体添加 Hook 支持
- **类型安全**：完整的 Rust 类型系统支持
- **零开销抽象**：编译时 Hook 管理，运行时开销最小
- **GPUI 集成**：与 GPUI 的 `Render` trait 无缝集成

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
gpui-hooks = "0.1"
```

**注意**：本库需要与 [GPUI](https://crates.io/crates/gpui) 框架一起使用。

## 快速开始

### 1. 创建 Hook 组件

```rust
use gpui::{div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Window, WindowBounds, WindowOptions};
use gpui_hooks::{hook_element, HookedRender};
use gpui_hooks::hooks::{UseEffectHook, UseMemoHook, UseStateHook};

#[hook_element]
struct CounterApp {}

impl HookedRender for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // useState - 管理计数器状态
        let (count, set_count) = self.use_state(|| 0i32);

        // useMemo - 计算双倍值
        let count_val = count();
        let doubled = self.use_memo([count_val], || count_val * 2);

        // useEffect - 副作用，当count变化时执行
        self.use_effect([count_val], || {
            println!("Effect: count changed to {}", count_val);
            Some(|| println!("Effect cleanup"))
        });

        div()
            .child(format!("Count: {}", count()))
            .child(format!("Doubled (useMemo): {}", doubled()))
            .child(div().child("click me").on_click(cx.listener(
                move |_this, _, _window, cx| {
                    set_count(count() + 1);
                    cx.notify();
                },
            )))
    }
}
```

### 2. 运行应用

```rust
fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| CounterApp::default())
            },
        ).unwrap();
    });
}
```

### 3. 运行示例

```bash
cargo run --example basic
```

## API 文档

### Hooks

#### `use_state`

管理组件状态。

```rust
let (value, set_value) = self.use_state(|| initial_value);
```

- **参数**：闭包，返回初始值
- **返回值**：`(getter, setter)` 元组
- **类型约束**：`T: Clone + 'static`

#### `use_effect`

执行副作用。

```rust
self.use_effect(deps, || {
    // 副作用逻辑
    Some(|| {
        // 清理函数（可选）
    })
});
```

- **参数**：
  - `deps`：依赖数组，当依赖变化时重新执行
  - `effect`：副作用闭包，返回可选的清理函数
- **注意**：组件必须在 `Drop` 实现中调用 `cleanup_effects()`

#### `use_memo`

记忆化计算值。

```rust
let memoized = self.use_memo(deps, || compute_expensive_value());
```

- **参数**：
  - `deps`：依赖数组，当依赖变化时重新计算
  - `compute`：计算闭包
- **返回值**：`getter` 函数，返回记忆化的值

### 宏

#### `#[hook_element]`

属性宏，自动为结构体添加 Hook 支持。

```rust
#[hook_element]
struct MyComponent {
    // 自定义字段
}
```

宏会自动：

1. 添加 `_hooks`, `_hook_index`, `_prev` 字段
2. 实现 `Default` trait
3. 实现 `HookedElement` trait
4. 实现 `gpui::Render` trait

### Trait

#### `HookedElement`

Hook 组件的基本 trait，提供 Hook 管理功能。

#### `HookedRender`

扩展 `gpui::Render`，添加 Hook 生命周期管理。

## Hook 规则

### 1. 只在顶层调用 Hook

❌ 错误示例：

```rust
if condition {
    let (value, set_value) = self.use_state(|| 0); // 错误！
}
```

✅ 正确示例：

```rust
let (value, set_value) = self.use_state(|| 0);
if condition {
    // 使用 value()
}
```

### 2. 保持 Hook 调用顺序一致

每次渲染必须以相同的顺序调用相同数量的 Hook。

### 3. 手动清理 Effect

使用 `use_effect` 的组件必须在 `Drop` 实现中清理：

```rust
impl Drop for MyComponent {
    fn drop(&mut self) {
        self.cleanup_effects();
    }
}
```

## 高级用法

### 自定义 Hook

创建可复用的自定义 Hook：

```rust
trait UseCounter {
    fn use_counter(&self, initial: i32) -> (Box<dyn Fn() -> i32>, Box<dyn Fn(i32)>, Box<dyn Fn()>);
}

impl<T: UseStateHook> UseCounter for T {
    fn use_counter(&self, initial: i32) -> (Box<dyn Fn() -> i32>, Box<dyn Fn(i32)>, Box<dyn Fn()>) {
        let (count, set_count) = self.use_state(|| initial);
        let increment = {
            let count = count.clone();
            let set_count = set_count.clone();
            Box::new(move || set_count(count() + 1))
        };
        (count, set_count, increment)
    }
}
```

### 组合多个 Hook

```rust
impl HookedRender for MyComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (count, set_count) = self.use_state(|| 0);
        let (name, set_name) = self.use_state(|| String::from("World"));

        self.use_effect([count()], || {
            println!("Count is now: {}", count());
            None
        });

        // ... 渲染逻辑
    }
}
```

## 开发指南

### 构建项目

```bash
cargo build
cargo build --release
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo clippy
cargo fmt --check
```

### 查看文档

```bash
cargo doc --open
```

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)（待创建）。

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 致谢

- [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) - 优秀的 Rust GUI 框架
- [React](https://reactjs.org/) - 灵感来源
- 所有贡献者

## 联系方式

如有问题或建议，请：

- 提交 [Issue](https://github.com/your-username/gpui-hooks/issues)
- 参与讨论

---

**快乐 Hooking！** 🎣
