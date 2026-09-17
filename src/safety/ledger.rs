//! SQLite transaction ledger for recording atomic file moves and enabling reliable rollback.

use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

use crate::common::error::Result;
use crate::common::paths::{ensure_parent_dir, get_history_db_path};
use crate::safety::mover::MoveOutcome;

/// Record representing an entire execution batch (`tidy run` or daemon cycle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub id: i64,
    pub uuid: String,
    pub timestamp: String,
    pub command: String,
    pub root_path: String,
    pub status: String,
}

/// Record representing an individual file move within a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationRecord {
    pub id: i64,
    pub run_id: i64,
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub file_size: u64,
    pub was_collision: bool,
    pub status: String,
    pub timestamp: String,
}

/// SQLite-backed transaction ledger.
pub struct Ledger {
    conn: Connection,
}

impl Ledger {
    /// Opens or creates the SQLite ledger database at the platform's default state directory.
    pub fn open_default() -> Result<Self> {
        let db_path = get_history_db_path()?;
        Self::open_at(&db_path)
    }

    /// Opens or creates the SQLite ledger database at a specific path.
    pub fn open_at(db_path: &Path) -> Result<Self> {
        ensure_parent_dir(db_path)?;

        let conn = Connection::open(db_path)?;

        // Configure WAL mode and pragmas for speed and crash-resilience
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let mut ledger = Self { conn };
        ledger.initialize_schema()?;

        Ok(ledger)
    }

    /// Opens an in-memory SQLite ledger database (primarily for testing).
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let mut ledger = Self { conn };
        ledger.initialize_schema()?;
        Ok(ledger)
    }

    /// Creates the database tables and indexes if they do not already exist.
    fn initialize_schema(&mut self) -> Result<()> {
        let sql = "
            CREATE TABLE IF NOT EXISTS runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                timestamp TEXT NOT NULL,
                command TEXT NOT NULL,
                root_path TEXT NOT NULL,
                status TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS operations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id INTEGER NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                source_path TEXT NOT NULL,
                destination_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                was_collision INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_runs_uuid ON runs(uuid);
            CREATE INDEX IF NOT EXISTS idx_runs_timestamp ON runs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_operations_run_id ON operations(run_id);
        ";

        self.conn.execute_batch(sql)?;
        Ok(())
    }

    /// Records an execution batch and all associated file moves in an atomic SQLite transaction.
    pub fn record_run(
        &mut self,
        command: &str,
        root_path: &Path,
        moves: &[MoveOutcome],
    ) -> Result<RunRecord> {
        self.record_run_with_status(command, root_path, moves, "COMPLETED")
    }

    /// Records an execution batch with a specific status (e.g. COMPLETED or PARTIAL_FAILURE).
    pub fn record_run_with_status(
        &mut self,
        command: &str,
        root_path: &Path,
        moves: &[MoveOutcome],
        status: &str,
    ) -> Result<RunRecord> {
        let run_uuid = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT INTO runs (uuid, timestamp, command, root_path, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![run_uuid, timestamp, command, root_path.to_string_lossy(), status],
        )?;

        let run_id = tx.last_insert_rowid();

        for m in moves {
            let op_timestamp = Utc::now().to_rfc3339();
            let was_coll_int = if m.was_collision { 1 } else { 0 };

            tx.execute(
                "INSERT INTO operations (run_id, source_path, destination_path, file_size, was_collision, status, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    run_id,
                    m.source.to_string_lossy(),
                    m.destination.to_string_lossy(),
                    m.file_size as i64,
                    was_coll_int,
                    "MOVED",
                    op_timestamp
                ],
            )?;
        }

        tx.commit()?;

        Ok(RunRecord {
            id: run_id,
            uuid: run_uuid,
            timestamp,
            command: command.to_string(),
            root_path: root_path.to_string_lossy().to_string(),
            status: status.to_string(),
        })
    }

    /// Fetches the most recent completed or partially failed run that has not yet been reverted.
    pub fn get_latest_completed_run(&self) -> Result<Option<RunRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, timestamp, command, root_path, status
             FROM runs
             WHERE status IN ('COMPLETED', 'PARTIAL_FAILURE')
             ORDER BY id DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(RunRecord {
                id: row.get(0)?,
                uuid: row.get(1)?,
                timestamp: row.get(2)?,
                command: row.get(3)?,
                root_path: row.get(4)?,
                status: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Fetches a specific run by its UUID string.
    pub fn get_run_by_uuid(&self, uuid: &str) -> Result<Option<RunRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, timestamp, command, root_path, status
             FROM runs
             WHERE uuid = ?1",
        )?;

        let mut rows = stmt.query(params![uuid])?;
        if let Some(row) = rows.next()? {
            Ok(Some(RunRecord {
                id: row.get(0)?,
                uuid: row.get(1)?,
                timestamp: row.get(2)?,
                command: row.get(3)?,
                root_path: row.get(4)?,
                status: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Fetches all operations for a given run in reverse chronological order (`id DESC`).
    pub fn get_operations_for_run(&self, run_id: i64) -> Result<Vec<OperationRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, run_id, source_path, destination_path, file_size, was_collision, status, timestamp
             FROM operations
             WHERE run_id = ?1
             ORDER BY id DESC",
        )?;

        let rows = stmt.query_map(params![run_id], |row| {
            let was_collision_int: i32 = row.get(5)?;
            let src_str: String = row.get(2)?;
            let dst_str: String = row.get(3)?;
            let size_i64: i64 = row.get(4)?;

            Ok(OperationRecord {
                id: row.get(0)?,
                run_id: row.get(1)?,
                source_path: PathBuf::from(src_str),
                destination_path: PathBuf::from(dst_str),
                file_size: size_i64 as u64,
                was_collision: was_collision_int != 0,
                status: row.get(6)?,
                timestamp: row.get(7)?,
            })
        })?;

        let mut ops = Vec::new();
        for op in rows {
            ops.push(op?);
        }

        Ok(ops)
    }

    /// Marks a run and its operations with a new status (e.g. `REVERTED`).
    pub fn mark_run_status(&mut self, run_id: i64, status: &str) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "UPDATE runs SET status = ?1 WHERE id = ?2",
            params![status, run_id],
        )?;

        tx.execute(
            "UPDATE operations SET status = ?1 WHERE run_id = ?2",
            params![status, run_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Lists recent runs up to `limit`.
    pub fn list_recent_runs(&self, limit: usize) -> Result<Vec<RunRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uuid, timestamp, command, root_path, status
             FROM runs
             ORDER BY id DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(RunRecord {
                id: row.get(0)?,
                uuid: row.get(1)?,
                timestamp: row.get(2)?,
                command: row.get(3)?,
                root_path: row.get(4)?,
                status: row.get(5)?,
            })
        })?;

        let mut runs = Vec::new();
        for r in rows {
            runs.push(r?);
        }

        Ok(runs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_creation_and_run_recording() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("test_history.db");

        let mut ledger = Ledger::open_at(&db_path).unwrap();

        let moves = vec![
            MoveOutcome {
                source: PathBuf::from("/downloads/photo.jpg"),
                destination: PathBuf::from("/downloads/Images/photo.jpg"),
                file_size: 1024,
                was_collision: false,
                is_cross_device: false,
                is_symlink: false,
            },
            MoveOutcome {
                source: PathBuf::from("/downloads/doc.pdf"),
                destination: PathBuf::from("/downloads/Documents/doc.pdf"),
                file_size: 2048,
                was_collision: true,
                is_cross_device: false,
                is_symlink: false,
            },
        ];

        let run = ledger
            .record_run("tidy run", Path::new("/downloads"), &moves)
            .unwrap();

        assert_eq!(run.status, "COMPLETED");

        let latest = ledger.get_latest_completed_run().unwrap().unwrap();
        assert_eq!(latest.id, run.id);
        assert_eq!(latest.uuid, run.uuid);

        let ops = ledger.get_operations_for_run(run.id).unwrap();
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].source_path, PathBuf::from("/downloads/doc.pdf")); // id DESC
        assert!(ops[0].was_collision);

        ledger.mark_run_status(run.id, "REVERTED").unwrap();
        let after = ledger.get_latest_completed_run().unwrap();
        assert!(after.is_none());
    }
}
