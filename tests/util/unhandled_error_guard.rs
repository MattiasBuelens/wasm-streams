use std::cell::RefCell;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ErrorEvent, PromiseRejectionEvent, window};

pub struct UnhandledErrorGuard {
    errors: Rc<RefCell<Vec<JsValue>>>,
    listener: JsValue,
}

const ERROR_TYPES: [&str; 2] = ["error", "unhandledrejection"];

impl UnhandledErrorGuard {
    pub fn new() -> Self {
        // Add a listener that collects any errors
        let errors = Rc::new(RefCell::new(vec![]));
        let listener = {
            let errors = AssertUnwindSafe(errors.clone());
            Closure::<dyn FnMut(_)>::new(move |event: JsValue| {
                let err = if let Some(event) = event.dyn_ref::<ErrorEvent>() {
                    Some(event.error())
                } else {
                    event.dyn_ref::<PromiseRejectionEvent>().map(|e| e.reason())
                };
                if let Some(err) = err {
                    errors.borrow_mut().push(err);
                }
            })
        };
        if let Some(window) = window() {
            for event_type in ERROR_TYPES {
                window
                    .add_event_listener_with_callback(event_type, listener.as_ref().unchecked_ref())
                    .unwrap();
            }
        }
        Self {
            errors,
            listener: listener.into_js_value(),
        }
    }
}

impl Default for UnhandledErrorGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UnhandledErrorGuard {
    fn drop(&mut self) {
        // Remove listeners
        if let Some(window) = window() {
            for event_type in ERROR_TYPES {
                window
                    .add_event_listener_with_callback(
                        event_type,
                        self.listener.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        }
        // Panic if there are any errors
        let errors = self.errors.take();
        assert!(
            errors.is_empty(),
            "There were {} unexpected errors",
            errors.len()
        );
    }
}
