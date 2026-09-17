//! Filesystem watcher, event listener, and download completion debouncer.

pub mod detector;
pub mod dispatcher;
pub mod engine;
pub mod guard;
pub mod memory;

#[allow(unused_imports)]
pub use detector::PendingFileTracker;
#[allow(unused_imports)]
pub use dispatcher::{WatchConfig, WatchDispatcher};
#[allow(unused_imports)]
pub use engine::WatcherEngine;
#[allow(unused_imports)]
pub use guard::{GuardDecision, WatchGuard};
