use super::{Dependency, Hook};
use std::any::Any;
use std::cell::RefCell;

/// UseMemo hook - memoizes a computed value
pub struct UseMemo<T: 'static> {
    value: Option<T>,
    deps: Vec<Box<dyn Dependency>>,
    has_run: bool,
}

impl<T: 'static> UseMemo<T> {
    pub fn new(deps: Vec<Box<dyn Dependency>>) -> Self {
        Self {
            value: None,
            deps,
            has_run: false,
        }
    }

    /// Check if dependencies have changed
    pub fn deps_changed(&self, new_deps: &[Box<dyn Dependency>]) -> bool {
        if !self.has_run {
            return true;
        }
        if self.deps.len() != new_deps.len() {
            return true;
        }
        self.deps
            .iter()
            .zip(new_deps.iter())
            .any(|(old, new)| !old.equals(new.as_ref()))
    }

    /// Update dependencies and value
    pub fn update(&mut self, value: T, new_deps: Vec<Box<dyn Dependency>>) {
        self.value = Some(value);
        self.deps = new_deps;
        self.has_run = true;
    }

    /// Get the cached value
    pub fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<T: 'static> Hook for UseMemo<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Trait for using memo hooks
///
/// This trait is automatically implemented for any type that implements `HasHooks`
/// (which includes all types using `#[hook_element]`).
pub trait UseMemoHook {
    /// Get access to the hooks storage (internal use)
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>>;

    /// Get and increment the hook index (internal use)
    fn _next_hook_index(&self) -> usize;

    /// Use a memo hook
    /// Returns a getter function that returns the memoized value
    /// Note: deps parameter should be passed directly (array or tuple) to avoid temporary value lifetime issues
    /// The compute closure is passed first, followed by the dependencies array.
    fn use_memo<T, F, D>(&self, compute: F, deps: D) -> Box<dyn Fn() -> T>
    where
        T: Clone + 'static,
        F: FnOnce() -> T,
        D: IntoIterator<Item: Dependency + Clone>,
    {
        let idx = self._next_hook_index();
        let hooks_ref = self._hooks_ref();

        // Convert deps to boxed vector (need to clone for potential double use)
        let boxed_deps_vec: Vec<Box<dyn Dependency>> = deps
            .into_iter()
            .map(|d| Box::new(d.clone()) as Box<dyn Dependency>)
            .collect();

        // Create or update hook
        let hooks_len = hooks_ref.borrow().len();
        if idx >= hooks_len {
            // Need to clone deps for initial creation
            let boxed_deps1: Vec<Box<dyn Dependency>> =
                boxed_deps_vec.iter().map(|d| d.clone_boxed()).collect();
            let boxed_deps2: Vec<Box<dyn Dependency>> =
                boxed_deps_vec.iter().map(|d| d.clone_boxed()).collect();
            let value = compute();
            let mut memo = UseMemo::new(boxed_deps1);
            memo.update(value, boxed_deps2);
            hooks_ref.borrow_mut().push(Box::new(memo));
        } else {
            let mut hooks = hooks_ref.borrow_mut();
            let hook = hooks.get_mut(idx).expect("Hook type mismatch at index");
            let memo = hook
                .as_any_mut()
                .downcast_mut::<UseMemo<T>>()
                .expect("Hook type mismatch at index");

            if memo.deps_changed(&boxed_deps_vec) {
                let value = compute();
                memo.update(value, boxed_deps_vec);
            }
        }

        // Get the raw pointer to the hook - it will be stable after creation
        let hook_ptr: *const dyn Hook = {
            let hooks = hooks_ref.borrow();
            let hook = hooks.get(idx).expect("Hook index out of bounds");
            hook.as_ref() as *const dyn Hook
        };

        // Create getter closure using the stable raw pointer
        let memo_ptr = hook_ptr as *const UseMemo<T>;
        Box::new(move || unsafe { (*memo_ptr).get().unwrap().clone() })
    }
}
