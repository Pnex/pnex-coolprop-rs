//! Shared application state.
//!
//! CoolProp's C API keeps its own process-global registry of AbstractState
//! handles; the server additionally tracks live handles so that requests
//! against stale/freed handles return a clean 404 instead of relying on
//! CoolProp's internal error text.
//!
//! The Rust-side registry is deliberately **process-global** too (a static
//! behind the handle), matching the C side: every `Router` built from
//! [`crate::router`] — including the fresh routers built per request in
//! tests — sees the same live handles.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Stateless handle-registry facade; the actual set is the process-global
/// static below (see module docs).
#[derive(Clone, Default)]
pub struct AppState;

impl AppState {
    pub fn new() -> Self {
        Self
    }

    fn global() -> &'static Mutex<HashSet<i64>> {
        static LIVE: OnceLock<Mutex<HashSet<i64>>> = OnceLock::new();
        LIVE.get_or_init(|| Mutex::new(HashSet::new()))
    }

    /// Register a freshly created handle.
    pub fn insert_handle(&self, handle: i64) {
        Self::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(handle);
    }

    /// Forget a handle (after `AbstractState_free`).
    pub fn remove_handle(&self, handle: i64) {
        Self::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&handle);
    }

    /// Returns `Err(UnknownHandle)` unless the handle is live in this process.
    pub fn require_handle(&self, handle: i64) -> Result<(), crate::error::ApiError> {
        let known = Self::global()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&handle);
        if known {
            Ok(())
        } else {
            Err(crate::error::ApiError::UnknownHandle(handle))
        }
    }
}

pub type SharedState = axum::extract::State<AppState>;
