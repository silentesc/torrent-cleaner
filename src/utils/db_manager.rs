use anyhow::Context;
use rusqlite::Connection;

use crate::{logger::enums::category::Category, trace};

pub struct DbManager;

impl DbManager {
    /**
     * Create new db connection
     * Return self or error
     */
    pub fn get_new_conn() -> Result<Connection, anyhow::Error> {
        Connection::open("/config/database.db").context("Failed to open connection to database")
    }

    /**
     * Create tables
     */
    pub fn check_create_tables() -> Result<(), anyhow::Error> {
        let conn = DbManager::get_new_conn()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS strikes (
                    id INTEGER PRIMARY KEY,
                    strike_type VARCHAR(255) NOT NULL,
                    hash VARCHAR(255) NOT NULL,
                    strikes INTEGER NOT NULL,
                    strike_days INTEGER NOT NULL,
                    last_strike_date TEXT NOT NULL,
                    UNIQUE (strike_type, hash)
                )",
            (),
        )
        .context("Failed to create strikes table")?;

        trace!(Category::DbManager, "Check-Created db tables");

        Ok(())
    }
}
