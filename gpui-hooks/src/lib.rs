//! GPUI Hooks - 为GPUI组件添加hook系统的库
//!
//! 这个库提供了 `#[hook_element]` 属性宏，可以为结构体自动添加hooks字段和相关方法。

use gpui::{Context, IntoElement, Window};
pub use gpui_hooks_macros::hook_element;
pub mod hooks;
use std::fmt;

/// Hook trait，所有hook类型必须实现这个trait
pub trait Hook {
    /// 执行hook操作
    fn execute(&self);
}

/// 为所有实现Fn()的类型实现Hook trait
impl<F> Hook for F
where
    F: Fn() + 'static,
{
    fn execute(&self) {
        (self)();
    }
}

// 为Box<dyn Hook>实现Debug，使其可以用于derive(Debug)
impl fmt::Debug for dyn Hook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hook")
    }
}

// 为Box<dyn Hook>实现PartialEq，使其可以用于derive(PartialEq)
impl PartialEq for dyn Hook {
    fn eq(&self, _other: &dyn Hook) -> bool {
        // 比较trait对象很困难，这里简单返回true
        // 在实际使用中，用户可能需要手动实现PartialEq
        true
    }
}

// 为Box<dyn Hook>实现Eq
impl Eq for dyn Hook {}

pub trait HookedElement {
    fn _use(&mut self, hook: impl crate::Hook + 'static);
    fn _hooks(&self) -> &[::std::boxed::Box<dyn crate::Hook>];
    fn _hooks_mut(&mut self) -> &mut [::std::boxed::Box<dyn crate::Hook>];
    fn _reset(&mut self);
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
