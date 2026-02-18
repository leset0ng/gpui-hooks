pub mod use_callback;
pub mod use_effect;
pub mod use_memo;
pub mod use_ref;
pub mod use_state;

pub use use_callback::{UseCallback, UseCallbackHook};
pub use use_effect::{UseEffect, UseEffectHook};
pub use use_memo::{UseMemo, UseMemoHook};
pub use use_ref::{UseRef, UseRefHook};
pub use use_state::{UseState, UseStateHook};

use std::any::Any;
use std::cell::RefCell;

/// Trait for hooks that can be stored and managed
pub trait Hook: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Trait for comparing dependencies
pub trait Dependency: Any {
    fn as_any(&self) -> &dyn Any;
    fn equals(&self, other: &dyn Dependency) -> bool;
    fn clone_boxed(&self) -> Box<dyn Dependency>;
}

impl<T: PartialEq + Clone + 'static> Dependency for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, other: &dyn Dependency) -> bool {
        if let Some(other_t) = other.as_any().downcast_ref::<T>() {
            self == other_t
        } else {
            false
        }
    }

    fn clone_boxed(&self) -> Box<dyn Dependency> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Dependency> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

/// Internal trait for types that have hook storage
/// This is implemented by HookedElement and used for blanket implementations
pub trait HasHooks {
    fn _hooks_storage(&self) -> &RefCell<Vec<Box<dyn Hook>>>;
    fn _next_index(&self) -> usize;
}

// Blanket implementations for hook traits
impl<T: HasHooks> UseStateHook for T {
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>> {
        self._hooks_storage()
    }

    fn _next_hook_index(&self) -> usize {
        self._next_index()
    }
}

impl<T: HasHooks> UseEffectHook for T {
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>> {
        self._hooks_storage()
    }

    fn _next_hook_index(&self) -> usize {
        self._next_index()
    }
}

impl<T: HasHooks> UseMemoHook for T {
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>> {
        self._hooks_storage()
    }

    fn _next_hook_index(&self) -> usize {
        self._next_index()
    }
}

impl<T: HasHooks> UseRefHook for T {
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>> {
        self._hooks_storage()
    }

    fn _next_hook_index(&self) -> usize {
        self._next_index()
    }
}

impl<T: HasHooks> UseCallbackHook for T {}
