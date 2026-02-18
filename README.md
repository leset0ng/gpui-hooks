# GPUI Hooks

[![Crates.io](https://img.shields.io/crates/v/gpui-hooks)](https://crates.io/crates/gpui-hooks)
[![Documentation](https://docs.rs/gpui-hooks/badge.svg)](https://docs.rs/gpui-hooks)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> For Chinese version, see [README_zh.md](README_zh.md)

A Rust library that adds React-style Hook system to the [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) framework.

## Features

- **React-style Hooks**: `use_state`, `use_effect`, `use_memo`
- **Attribute Macro**: `#[hook_element]` automatically adds Hook support to structs
- **Type Safety**: Full Rust type system support
- **Zero-cost Abstraction**: Compile-time hook management, minimal runtime overhead
- **GPUI Integration**: Seamless integration with GPUI's `Render` trait

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gpui-hooks = "0.1"
```

**Note**: This library requires the [GPUI](https://crates.io/crates/gpui) framework.

## Quick Start

### 1. Create a Hook Component

```rust
use gpui::{div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Window, WindowBounds, WindowOptions};
use gpui_hooks::{hook_element, HookedRender};
use gpui_hooks::hooks::{UseEffectHook, UseMemoHook, UseStateHook};

#[hook_element]
struct CounterApp {}

impl HookedRender for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // useState - manage counter state
        let (count, set_count) = self.use_state(|| 0i32);

        // useMemo - compute doubled value
        let count_val = count();
        let doubled = self.use_memo([count_val], || count_val * 2);

        // useEffect - side effect when count changes
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

### 2. Run the Application

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

### 3. Run Example

```bash
cargo run --example basic
```

## API Documentation

### Hooks

#### `use_state`

Manages component state.

```rust
let (value, set_value) = self.use_state(|| initial_value);
```

- **Parameters**: Closure returning initial value
- **Returns**: `(getter, setter)` tuple
- **Type Constraint**: `T: Clone + 'static`

#### `use_effect`

Executes side effects.

```rust
self.use_effect(deps, || {
    // Side effect logic
    Some(|| {
        // Cleanup function (optional)
    })
});
```

- **Parameters**:
  - `deps`: Dependency array, re-executes when dependencies change
  - `effect`: Side effect closure, returns optional cleanup function
- **Note**: Components must call `cleanup_effects()` in their `Drop` implementation

#### `use_memo`

Memoizes computed values.

```rust
let memoized = self.use_memo(deps, || compute_expensive_value());
```

- **Parameters**:
  - `deps`: Dependency array, re-computes when dependencies change
  - `compute`: Computation closure
- **Returns**: `getter` function returning memoized value

### Macro

#### `#[hook_element]`

Attribute macro that automatically adds Hook support to structs.

```rust
#[hook_element]
struct MyComponent {
    // Custom fields
}
```

The macro automatically:
1. Adds `_hooks`, `_hook_index`, `_prev` fields
2. Implements `Default` trait
3. Implements `HookedElement` trait
4. Implements `gpui::Render` trait

### Traits

#### `HookedElement`

Basic trait for hook components, providing hook management functionality.

#### `HookedRender`

Extends `gpui::Render` with hook lifecycle management.

## Hook Rules

### 1. Only Call Hooks at the Top Level
❌ Wrong example:
```rust
if condition {
    let (value, set_value) = self.use_state(|| 0); // Wrong!
}
```

✅ Correct example:
```rust
let (value, set_value) = self.use_state(|| 0);
if condition {
    // Use value()
}
```

### 2. Keep Hook Call Order Consistent
Each render must call the same number of hooks in the same order.

### 3. Manually Clean Up Effects
Components using `use_effect` must clean up in their `Drop` implementation:

```rust
impl Drop for MyComponent {
    fn drop(&mut self) {
        self.cleanup_effects();
    }
}
```

## Advanced Usage

### Custom Hooks

Create reusable custom hooks:

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

### Combining Multiple Hooks

```rust
impl HookedRender for MyComponent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (count, set_count) = self.use_state(|| 0);
        let (name, set_name) = self.use_state(|| String::from("World"));
        
        self.use_effect([count()], || {
            println!("Count is now: {}", count());
            None
        });
        
        // ... rendering logic
    }
}
```

## Development Guide

### Build Project

```bash
cargo build
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Code Quality

```bash
cargo clippy
cargo fmt --check
```

### View Documentation

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) (to be created).

1. Fork the project
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) - Excellent Rust GUI framework
- [React](https://reactjs.org/) - Inspiration source
- All contributors

## Contact

For questions or suggestions, please:
- Submit an [Issue](https://github.com/leset0ng/gpui-hooks/issues)
- Join the discussion

---

**Happy Hooking!** 🎣