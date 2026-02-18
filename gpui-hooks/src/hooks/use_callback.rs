use super::{Dependency, UseMemoHook};

/// UseCallback hook - memoizes a function
/// This is a type alias for documentation purposes
pub struct UseCallback<R: 'static>(std::marker::PhantomData<R>);

/// Trait for using callback hooks
///
/// This trait is automatically implemented for any type that implements `HasHooks`
/// (which includes all types using `#[hook_element]`).
pub trait UseCallbackHook: UseMemoHook {
    /// Use a callback hook
    /// Returns a memoized function
    /// Note: deps parameter should be passed directly (array or tuple) to avoid temporary value lifetime issues
    /// The factory closure is passed first, followed by the dependencies array.
    fn use_callback<F, R, D>(&self, factory: F, deps: D) -> Box<dyn Fn() -> R>
    where
        R: 'static,
        F: FnOnce() -> Box<dyn Fn() -> R>,
        D: IntoIterator<Item: Dependency + Clone>,
    {
        // Use use_memo to memoize the factory function wrapped in Rc
        // use_memo returns Box<dyn Fn() -> Rc<Box<dyn Fn() -> R>>>
        let getter = self.use_memo(|| std::rc::Rc::new(factory()), deps);

        // Return a closure that calls the memoized function
        Box::new(move || {
            let callback_rc = getter();
            let callback = &**callback_rc; // Deref Rc<Box<dyn Fn() -> R>> to &dyn Fn() -> R
            callback()
        })
    }
}
