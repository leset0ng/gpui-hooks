use super::Hook;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

/// UseRef hook - maintains a mutable reference to a value
pub struct UseRef<T: 'static> {
    current: Rc<RefCell<T>>,
}

impl<T: 'static> UseRef<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: Rc::new(RefCell::new(initial)),
        }
    }

    /// Get a clone of the Rc<RefCell<T>>
    pub fn get_ref(&self) -> Rc<RefCell<T>> {
        self.current.clone()
    }
}

impl<T: 'static> Hook for UseRef<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Trait for using ref hooks
///
/// This trait is automatically implemented for any type that implements `HasHooks`
/// (which includes all types using `#[hook_element]`).
pub trait UseRefHook {
    /// Get access to the hooks storage (internal use)
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>>;

    /// Get and increment the hook index (internal use)
    fn _next_hook_index(&self) -> usize;

    /// Use a ref hook
    /// Returns a shared reference to the mutable value
    fn use_ref<T, F>(&self, initial: F) -> Rc<RefCell<T>>
    where
        T: 'static,
        F: FnOnce() -> T,
    {
        let idx = self._next_hook_index();
        let hooks_ref = self._hooks_ref();

        // Create hook if it doesn't exist
        let hooks_len = hooks_ref.borrow().len();
        if idx >= hooks_len {
            let hook = UseRef::new(initial());
            hooks_ref.borrow_mut().push(Box::new(hook));
        }

        // Get the raw pointer to the hook - it will be stable after creation
        let hook_ptr: *const dyn Hook = {
            let hooks = hooks_ref.borrow();
            let hook = hooks.get(idx).expect("Hook index out of bounds");
            hook.as_ref() as *const dyn Hook
        };

        // Get a clone of the Rc<RefCell<T>> from the hook
        unsafe {
            let ref_hook = &*(hook_ptr as *const UseRef<T>);
            ref_hook.get_ref()
        }
    }
}
