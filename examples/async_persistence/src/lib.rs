//! A small SQLite adapter that keeps persistence separate from GPUI view state.

use rusqlite::{Connection, OptionalExtension, Result, params};
use std::path::Path;

// ANCHOR: sqlite_store
/// One user-selected display preference saved by the separate persistence layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedPreference {
    pub key: String,
    pub value: String,
}

/// A minimal key/value adapter. Call it from a background executor in a GPUI app;
/// SQLite connections perform blocking work and are not GPUI `Global` settings.
pub struct PreferenceStore {
    connection: Connection,
}

impl PreferenceStore {
    /// Open or create a database at the caller-selected path and ensure its schema exists.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS preferences (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            );",
        )?;
        Ok(Self { connection })
    }

    /// Insert or replace several preferences atomically using bound parameters.
    pub fn save_many(&mut self, values: &[SavedPreference]) -> Result<()> {
        let transaction = self.connection.transaction()?;
        {
            let mut statement = transaction.prepare(
                "INSERT INTO preferences (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )?;
            for preference in values {
                statement.execute(params![preference.key, preference.value])?;
            }
        }
        transaction.commit()
    }

    /// Read a single preference without interpreting its value as SQL.
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        self.connection
            .query_row("SELECT value FROM preferences WHERE key = ?1", [key], |row| row.get(0))
            .optional()
    }

    /// Return all rows in a stable order for display or tests.
    pub fn list(&self) -> Result<Vec<SavedPreference>> {
        let mut statement = self
            .connection
            .prepare("SELECT key, value FROM preferences ORDER BY key COLLATE NOCASE, key")?;
        let rows = statement
            .query_map([], |row| Ok(SavedPreference { key: row.get(0)?, value: row.get(1)? }))?;
        rows.collect()
    }
}
// ANCHOR_END: sqlite_store

#[cfg(test)]
mod tests {
    use super::{PreferenceStore, SavedPreference};

    #[test]
    fn transaction_survives_close_and_reopen_with_bound_values() {
        let directory = tempfile::tempdir().unwrap();
        let database_path = directory.path().join("preferences.sqlite3");
        let values = [
            SavedPreference { key: "appearance".into(), value: "dark".into() },
            SavedPreference {
                key: "workspace'; DROP TABLE preferences; --".into(),
                value: "shows parameterized values".into(),
            },
        ];

        {
            let mut store = PreferenceStore::open(&database_path).unwrap();
            store.save_many(&values).unwrap();
            assert_eq!(store.get("appearance").unwrap().as_deref(), Some("dark"));
        }

        let reopened = PreferenceStore::open(&database_path).unwrap();
        assert_eq!(reopened.list().unwrap(), values);
        assert_eq!(
            reopened.get("workspace'; DROP TABLE preferences; --").unwrap().as_deref(),
            Some("shows parameterized values")
        );
        assert_eq!(reopened.get("missing").unwrap(), None);
    }

    #[test]
    fn later_save_replaces_a_value() {
        let directory = tempfile::tempdir().unwrap();
        let mut store =
            PreferenceStore::open(directory.path().join("preferences.sqlite3")).unwrap();
        store
            .save_many(&[SavedPreference { key: "density".into(), value: "comfortable".into() }])
            .unwrap();
        store
            .save_many(&[SavedPreference { key: "density".into(), value: "compact".into() }])
            .unwrap();
        assert_eq!(store.get("density").unwrap().as_deref(), Some("compact"));
    }
}
