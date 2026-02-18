use super::Hook;
use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};

/// UseState hook - manages a mutable state value
pub struct UseState<T: 'static> {
    value: RefCell<T>,
    version: RefCell<usize>,
}

impl<T: 'static> UseState<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: RefCell::new(initial),
            version: RefCell::new(0),
        }
    }

    /// Get an immutable reference to the current value
    pub fn get(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    /// Get a mutable reference to the current value (doesn't trigger re-render)
    pub fn get_mut(&self) -> RefMut<'_, T> {
        self.value.borrow_mut()
    }

    /// Set a new value and increment version (triggers re-render)
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
        *self.version.borrow_mut() += 1;
    }

    /// Get the current version number
    pub fn version(&self) -> usize {
        *self.version.borrow()
    }
}

impl<T: 'static> Hook for UseState<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Trait for using state hooks
///
/// This trait is automatically implemented for any type that implements `HasHooks`
/// (which includes all types using `#[hook_element]`).
pub trait UseStateHook {
    /// Get access to the hooks storage (internal use)
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>>;

    /// Get and increment the hook index (internal use)
    fn _next_hook_index(&self) -> usize;

    /// Use a state hook
    /// Returns (getter function, setter function)
    fn use_state<T, F>(&self, initial: F) -> (Box<dyn Fn() -> T>, Box<dyn Fn(T)>)
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
    {
        let idx = self._next_hook_index();
        let hooks_ref = self._hooks_ref();

        // Create hook if it doesn't exist
        let hooks_len = hooks_ref.borrow().len();
        if idx >= hooks_len {
            let state = UseState::new(initial());
            hooks_ref.borrow_mut().push(Box::new(state));
        }

        // Get the raw pointer to the hook - it will be stable after creation
        let hook_ptr: *const dyn Hook = {
            let hooks = hooks_ref.borrow();
            let hook = hooks.get(idx).expect("Hook index out of bounds");
            hook.as_ref() as *const dyn Hook
        };

        // Create getter closure using the stable raw pointer
        let getter: Box<dyn Fn() -> T> = {
            let state_ptr = hook_ptr as *const UseState<T>;
            Box::new(move || unsafe { (*state_ptr).get().clone() })
        };

        // Create setter closure using the stable raw pointer
        let setter: Box<dyn Fn(T)> = {
            let state_ptr = hook_ptr as *const UseState<T>;
            Box::new(move |new_value: T| unsafe {
                (*state_ptr).set(new_value);
            })
        };

        (getter, setter)
    }
}
