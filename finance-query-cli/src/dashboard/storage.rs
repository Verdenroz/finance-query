use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub id: i64,
    pub name: String,
    pub symbols: Vec<String>,
}

pub struct DashboardStorage {
    conn: Connection,
}

impl DashboardStorage {
    pub fn new() -> Result<Self> {
        let path = Self::get_db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path).context("Failed to open dashboard database")?;

        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?;
        Ok(data_dir.join("fq").join("watchlists.db"))
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS watchlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                symbols TEXT NOT NULL
            )",
            [],
        )?;

        // Create default watchlist if none exist
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM watchlists", [], |row| row.get(0))?;

        if count == 0 {
            self.create_watchlist("Default", &[])?;
        }

        Ok(())
    }

    // Watchlist operations
    pub fn get_watchlists(&self) -> Result<Vec<Watchlist>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, symbols FROM watchlists ORDER BY id")?;

        let watchlists = stmt
            .query_map([], |row| {
                let symbols_json: String = row.get(2)?;
                let symbols: Vec<String> = serde_json::from_str(&symbols_json).unwrap_or_default();

                Ok(Watchlist {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    symbols,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(watchlists)
    }

    pub fn get_watchlist(&self, id: i64) -> Result<Option<Watchlist>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, symbols FROM watchlists WHERE id = ?")?;

        let result = stmt.query_row(params![id], |row| {
            let symbols_json: String = row.get(2)?;
            let symbols: Vec<String> = serde_json::from_str(&symbols_json).unwrap_or_default();

            Ok(Watchlist {
                id: row.get(0)?,
                name: row.get(1)?,
                symbols,
            })
        });

        match result {
            Ok(watchlist) => Ok(Some(watchlist)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_watchlist(&self, name: &str, symbols: &[String]) -> Result<i64> {
        let symbols_json = serde_json::to_string(symbols)?;

        self.conn.execute(
            "INSERT INTO watchlists (name, symbols) VALUES (?, ?)",
            params![name, symbols_json],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_watchlist(&self, id: i64, symbols: &[String]) -> Result<()> {
        let symbols_json = serde_json::to_string(symbols)?;

        self.conn.execute(
            "UPDATE watchlists SET symbols = ? WHERE id = ?",
            params![symbols_json, id],
        )?;

        Ok(())
    }

    pub fn add_symbol_to_watchlist(&self, watchlist_id: i64, symbol: &str) -> Result<()> {
        let watchlist = self
            .get_watchlist(watchlist_id)?
            .ok_or_else(|| anyhow::anyhow!("Watchlist not found"))?;

        let mut symbols = watchlist.symbols;
        let symbol_upper = symbol.to_uppercase();

        if !symbols.contains(&symbol_upper) {
            symbols.push(symbol_upper);
            self.update_watchlist(watchlist_id, &symbols)?;
        }

        Ok(())
    }

    pub fn remove_symbol_from_watchlist(&self, watchlist_id: i64, symbol: &str) -> Result<()> {
        let watchlist = self
            .get_watchlist(watchlist_id)?
            .ok_or_else(|| anyhow::anyhow!("Watchlist not found"))?;

        let symbols: Vec<String> = watchlist
            .symbols
            .into_iter()
            .filter(|s| !s.eq_ignore_ascii_case(symbol))
            .collect();

        self.update_watchlist(watchlist_id, &symbols)?;
        Ok(())
    }
}
