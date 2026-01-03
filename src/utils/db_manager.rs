use anyhow::Context;
use rusqlite::Connection;

use crate::{logger::enums::category::Category, trace};

pub struct Session {
    conn: Option<Connection>,
}

impl Session {
    /// Create a new database session
    pub fn new() -> Result<Self, anyhow::Error> {
        let conn = Connection::open("/config/database.db").context("Failed to open connection to database")?;
        Ok(Self { conn: Some(conn) })
    }

    /// Get a reference to the connection
    pub fn conn(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }

    /// Get a mutable reference to the connection
    pub fn conn_mut(&mut self) -> Option<&mut Connection> {
        self.conn.as_mut()
    }

    /// Consume the session and return the connection
    pub fn into_conn(mut self) -> Option<Connection> {
        self.conn.take()
    }

    /// Explicitly close the session
    pub fn close(&mut self) {
        if let Some(conn) = self.conn.take() {
            let _ = conn.close();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct DbManager;

impl DbManager {
    /**
     * Create tables
     */
    pub fn check_create_tables() -> Result<(), anyhow::Error> {
        let mut session = Session::new()?;
        let conn = session.conn_mut().ok_or_else(|| anyhow::anyhow!("Failed to get connection from session"))?;

        // strikes
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

        // jobs
        conn.execute(
            "CREATE TABLE IF NOT EXISTS jobs (
                    id INTEGER PRIMARY KEY,
                    job_name VARCHAR(255) UNIQUE NOT NULL,
                    last_job_run TEXT NOT NULL
                )",
            (),
        )
        .context("Failed to create jobs table")?;

        trace!(Category::DbManager, "Check-Created db tables");

        Ok(())
    }
}
