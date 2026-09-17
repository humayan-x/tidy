#![allow(dead_code)]
//! Safe execution, non-destructive renaming, atomic moving, and SQLite undo ledger.

pub mod collision;
pub mod ledger;
pub mod mover;
pub mod symlink;
pub mod undo;

#[allow(unused_imports)]
pub use collision::{resolve_collision, resolve_collision_with_claimed, CollisionResolution};
#[allow(unused_imports)]
pub use ledger::{Ledger, OperationRecord, RunRecord};
#[allow(unused_imports)]
pub use mover::{safe_move, safe_move_with_claimed, MoveOutcome};
#[allow(unused_imports)]
pub use symlink::{inspect_symlink, should_traverse_dir, SymlinkType};
#[allow(unused_imports)]
pub use undo::{execute_undo, execute_undo_opt, UndoOutcome, UndoReport};
