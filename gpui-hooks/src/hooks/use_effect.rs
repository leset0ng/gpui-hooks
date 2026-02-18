use super::{Dependency, Hook};
use std::any::Any;
use std::cell::RefCell;

/// UseEffect hook - runs side effects when dependencies change
pub struct UseEffect {
    deps: Vec<Box<dyn Dependency>>,
    cleanup: Option<Box<dyn FnOnce()>>,
    has_run: bool,
}

impl UseEffect {
    pub fn new(deps: Vec<Box<dyn Dependency>>) -> Self {
        Self {
            deps,
            cleanup: None,
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

    /// Update dependencies
    pub fn update_deps(&mut self, new_deps: Vec<Box<dyn Dependency>>) {
        self.deps = new_deps;
    }

    /// Mark as run
    pub fn mark_run(&mut self) {
        self.has_run = true;
    }

    /// Set cleanup function
    pub fn set_cleanup(&mut self, cleanup: Box<dyn FnOnce()>) {
        // Run previous cleanup if exists
        if let Some(prev_cleanup) = self.cleanup.take() {
            prev_cleanup();
        }
        self.cleanup = Some(cleanup);
    }

    /// Run cleanup if exists
    pub fn run_cleanup(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

impl Hook for UseEffect {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Trait for using effect hooks
///
/// This trait is automatically implemented for any type that implements `HasHooks`
/// (which includes all types using `#[hook_element]`).
pub trait UseEffectHook {
    /// Get access to the hooks storage (internal use)
    fn _hooks_ref(&self) -> &RefCell<Vec<Box<dyn Hook>>>;

    /// Get and increment the hook index (internal use)
    fn _next_hook_index(&self) -> usize;

    /// Use an effect hook
    /// Note: deps parameter should be passed directly (array or tuple) to avoid temporary value lifetime issues
    /// The effect closure is passed first, followed by the dependencies array.
    fn use_effect<F, C, D>(&self, effect: F, deps: D)
    where
        F: FnOnce() -> Option<C>,
        C: FnOnce() + 'static,
        D: IntoIterator<Item: Dependency + Clone>,
    {
        let idx = self._next_hook_index();
        let hooks_ref = self._hooks_ref();

        // Convert deps to boxed vector
        let boxed_deps: Vec<Box<dyn Dependency>> = deps
            .into_iter()
            .map(|d| Box::new(d.clone()) as Box<dyn Dependency>)
            .collect();

        // Create or update hook
        let hooks_len = hooks_ref.borrow().len();
        if idx >= hooks_len {
            let mut effect_hook = UseEffect::new(boxed_deps.clone());

            // Run effect immediately
            if let Some(cleanup) = effect() {
                effect_hook.set_cleanup(Box::new(cleanup));
            }
            effect_hook.mark_run();

            hooks_ref.borrow_mut().push(Box::new(effect_hook));
        } else {
            let mut hooks = hooks_ref.borrow_mut();
            let hook = hooks.get_mut(idx).expect("Hook index out of bounds");
            let effect_hook = hook
                .as_any_mut()
                .downcast_mut::<UseEffect>()
                .expect("Hook type mismatch at index");

            if effect_hook.deps_changed(&boxed_deps) {
                // Run cleanup from previous effect
                effect_hook.run_cleanup();

                // Update deps
                effect_hook.update_deps(boxed_deps);

                // Run new effect
                if let Some(cleanup) = effect() {
                    effect_hook.set_cleanup(Box::new(cleanup));
                }
                effect_hook.mark_run();
            }
        }
    }

    /// Clean up all effects (call when component is dropped)
    fn cleanup_effects(&self) {
        for hook in self._hooks_ref().borrow_mut().iter_mut() {
            if let Some(effect) = hook.as_any_mut().downcast_mut::<UseEffect>() {
                effect.run_cleanup();
            }
        }
    }
}
