use crate::app::tea_model::Model;
use std::cell::RefCell;

thread_local! {
    static VIEW_MODEL_CONTEXT: RefCell<Option<*const Model>> = RefCell::new(None);
}

pub struct ViewModelContext;

impl ViewModelContext {
    /// Establishes a view model context for the duration of the closure.
    /// This should only be called from the main view entry point.
    pub fn with_model<F, R>(model: &Model, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        VIEW_MODEL_CONTEXT.with(|ctx| {
            let old_model = ctx.replace(Some(model as *const Model));
            let result = f();
            ctx.replace(old_model);
            result
        })
    }

    /// Gets the current model from the context.
    /// Panics if called outside of `with_model()`.
    pub fn current() -> ModelRef {
        VIEW_MODEL_CONTEXT.with(|ctx| {
            let model_ptr = ctx
                .borrow()
                .expect("ViewModelContext::current() called outside of with_model() scope");
            ModelRef { ptr: model_ptr }
        })
    }

    /// Checks if a model context is currently active.
    pub fn is_active() -> bool {
        VIEW_MODEL_CONTEXT.with(|ctx| ctx.borrow().is_some())
    }
}

/// A safe reference to the current model in the view context.
pub struct ModelRef {
    ptr: *const Model,
}

impl ModelRef {
    /// Gets a reference to the current model.
    /// This is safe because the model is guaranteed to be valid
    /// for the duration of the view context.
    pub fn get(&self) -> &Model {
        unsafe { &*self.ptr }
    }
}

// Convenience methods for common model access patterns
impl ModelRef {
    pub fn state(&self) -> &crate::app::tea_model::AppState {
        &self.get().state
    }

    pub fn connection_status(&self) -> &crate::app::tea_model::ConnectionStatus {
        &self.get().connection_status
    }

    pub fn ui_is_rounded(&self) -> bool {
        self.get().config.ui_block_is_rounded
    }

    pub fn init(&self) -> &crate::app::tea_model::ModelInit {
        &self.get().init
    }

    pub fn session(&self) -> Option<&opencode_sdk::models::Session> {
        self.get().session()
    }

    pub fn client_base_url(&self) -> &str {
        self.get().client_base_url()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tea_model::Model;

    #[test]
    fn test_context_lifecycle() {
        let model = Model::new();

        // Context should not be active initially
        assert!(!ViewModelContext::is_active());

        // Context should be active within with_model
        ViewModelContext::with_model(&model, || {
            assert!(ViewModelContext::is_active());
            let model_ref = ViewModelContext::current();
            assert_eq!(model_ref.ui_is_rounded(), model.config.ui_block_is_rounded);
        });

        // Context should not be active after with_model
        assert!(!ViewModelContext::is_active());
    }

    #[test]
    #[should_panic(expected = "ViewModelContext::current() called outside of with_model() scope")]
    fn test_context_panic_outside_scope() {
        ViewModelContext::current();
    }
}

