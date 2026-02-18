//! GPUI Hooks - 为GPUI组件添加hook系统的库
//!
//! 这个库提供了 `#[hook_element]` 属性宏，可以为结构体自动添加hooks字段和相关方法。

use std::cell::RefCell;

use gpui::{Context, IntoElement, Window};
pub use gpui_hooks_macros::hook_element;
pub mod hooks;
use hooks::{HasHooks, Hook, UseEffectHook, UseMemoHook, UseStateHook};

/// HookedElement trait - 管理组件的hooks
/// 使用内部可变性模式，使得hooks可以在&self上调用
///
/// 自动实现了 UseStateHook, UseEffectHook, UseMemoHook
pub trait HookedElement: UseStateHook + UseEffectHook + UseMemoHook {
    /// Get access to the hooks RefCell
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>>;

    /// Get the current hook index
    fn _hook_index(&self) -> usize;

    /// Set the hook index
    fn _set_hook_index(&self, index: usize);

    /// Get the previous hook count
    fn _prev(&self) -> usize;

    /// Set the previous hook count
    fn _set_prev(&self, prev: usize);

    /// Increment hook index and return the old value
    fn _next_hook_index(&self) -> usize {
        let idx = self._hook_index();
        self._set_hook_index(idx + 1);
        idx
    }

    /// Reset hook state at the start of render
    fn _reset(&self) {
        let current = self._hook_index();
        let prev = self._prev();

        // Check if hook count changed from previous render
        if prev != 0 && current != prev {
            panic!(
                "Hook count changed from {} to {}. Hooks must be called in the same order every render.",
                prev, current
            );
        }

        self._set_prev(current);
        self._set_hook_index(0);
    }
}

// Blanket implementation of HasHooks for all HookedElement types
// This enables automatic implementation of UseStateHook, UseEffectHook, UseMemoHook
impl<T: HookedElement> HasHooks for T {
    fn _hooks_storage(&self) -> &RefCell<Vec<Box<dyn Hook>>> {
        HookedElement::_hooks_ref(self)
    }

    fn _next_index(&self) -> usize {
        HookedElement::_next_hook_index(self)
    }
}

/// HookedRender trait - 在GPUI的Render后执行钩子代码
pub trait HookedRender: Sized + HookedElement {
    /// render之前执行的代码（默认执行所有hooks）
    fn pre_render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self._reset();
    }

    /// render的主要逻辑
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement;
}

/// 执行HookedRender的完整流程，在Render实现中调用
pub fn execute_hooked_render<T>(
    this: &mut T,
    window: &mut Window,
    cx: &mut Context<T>,
) -> impl IntoElement
where
    T: HookedRender,
{
    this.pre_render(window, cx);
    this.render(window, cx)
}

// Blanket implementation for Drop to clean up effects
// Note: Users need to manually call cleanup_effects in their Drop impl
// since we can't provide a blanket Drop impl without specialization
