//! IndexedDB is the source of truth. Every settings/ledger change uses one transaction.
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
