use anyhow::Context;
use chrono::{Duration, Local, NaiveDate};
use rusqlite::{Connection, params};

use crate::{jobs::enums::strike_type::StrikeType, logger::logger::Logger};

#[derive(Clone)]
pub struct StrikeRecord {
    id: i32,
    strike_type: String,
    hash: String,
    strikes: i32,
    strike_days: i32,
    last_strike_date: NaiveDate,
}

pub struct StrikeUtils {
    conn: Connection,
}

impl StrikeUtils {
    pub fn new() -> Result<Self, anyhow::Error> {
        let conn = Connection::open("/config/database.db").context("Failed to open connection to database")?;
        Ok(Self { conn })
    }

    /**
     * Create tables
     */
    pub fn create_tables(&mut self) -> Result<(), anyhow::Error> {
        self.conn
            .execute(
                "CREATE TABLE strikes (
                    id INTEGER PRIMARY KEY,
                    strike_type VARCHAR(255) NOT NULL,
                    hash VARCHAR(255) NOT NULL,
                    strikes INTEGER NOT NULL,
                    strike_days INTEGER NOT NULL,
                    last_strike_date TEXT DEFAULT CURRENT_DATE NOT NULL,
                    UNIQUE (strike_type, hash)
                )",
                (),
            )
            .context("Failed to create strikes table")?;
        Ok(())
    }

    /**
     * Get strikes
     */
    pub fn get_strikes(&mut self, strike_type: StrikeType) -> Result<Vec<StrikeRecord>, anyhow::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, strike_type, hash, strikes, strike_days, last_strike_date FROM strikes WHERE strike_type = ?1")
            .context("Failed to prepare get_strikes select")?;

        let rows = stmt
            .query_map(params![strike_type.to_string()], |row| {
                Ok(StrikeRecord {
                    id: row.get(0)?,
                    strike_type: row.get(1)?,
                    hash: row.get(2)?,
                    strikes: row.get(3)?,
                    strike_days: row.get(4)?,
                    last_strike_date: row.get(5)?,
                })
            })
            .context("Failed to execute get_strikes statement")?;

        let mut strike_records: Vec<StrikeRecord> = Vec::new();
        for row in rows {
            match row {
                Ok(strike_record) => strike_records.push(strike_record),
                Err(e) => {
                    Logger::error(format!("Failed to get strike record: {}", e).as_str());
                    continue;
                }
            }
        }

        Ok(strike_records)
    }

    /**
     * Strike multiple
     */
    pub fn strike(&mut self, strike_type: StrikeType, hashes: Vec<&str>) -> Result<(), anyhow::Error> {
        // Get current strike records
        let strike_records = self.get_strikes(strike_type.clone()).context("Failed to get strike types")?;

        // Open transaction
        let tx = self.conn.transaction().context("Failed to get transaction")?;

        // Handle hashes
        for hash in hashes {
            // Try to get the strike record of the hash
            let strike_records_for_hash: Vec<StrikeRecord> = strike_records
                .clone()
                .into_iter()
                .filter(|strike_record| strike_record.strike_type == strike_type.clone().to_string() && strike_record.hash == hash)
                .collect();
            // This should never be the case due to the unique contraint but you never know
            if strike_records_for_hash.len() > 1 {
                Logger::warn(
                    format!(
                        "Hash {} for strike type {} is {} times in the db. This should be impossible",
                        hash,
                        strike_type.to_string(),
                        strike_records_for_hash.len(),
                    )
                    .as_str(),
                );
            }
            // Check for strike record of the hash
            match strike_records_for_hash.get(0) {
                // If the strike record of the hash exists, handle multiple scenarios
                Some(strike_record) => {
                    let today_local = Local::now().date_naive();
                    let yesterday_local = today_local - Duration::days(1);
                    // If the strike record was last striked yesterday, increase everything
                    if strike_record.last_strike_date == yesterday_local {
                        tx.execute(
                            "UPDATE SET strikes = strikes + 1, strike_days = strike_days + 1, last_strike_date = ?1 WHERE strike_type = ?2 AND hash = ?3",
                            params![today_local, strike_type.to_string(), hash],
                        )
                        .context("Failed to insert new strike")?;
                        Logger::trace(format!("Hash {} ({}) was last striked yesterday, strikes and strike days have been increased", hash, strike_type.to_string(),).as_str());
                    }
                    // If the strike record was last striked today, just increase strikes
                    else if strike_record.last_strike_date == today_local {
                        tx.execute("UPDATE SET strikes = strikes + 1 WHERE strike_type = ?1 AND hash = ?2", params![strike_type.to_string(), hash])
                            .context("Failed to insert new strike")?;
                        Logger::trace(format!("Hash {} ({}) was last striked today, strikes have been increased", hash, strike_type.to_string(),).as_str());
                    }
                    // If the strike record was not striked today or yesterday, reset it
                    else {
                        tx.execute(
                            "UPDATE SET strikes = 1, strike_days = 1, last_strike_date = ?1 WHERE strike_type = ?1 AND hash = ?2",
                            params![today_local, strike_type.to_string(), hash],
                        )
                        .context("Failed to insert new strike")?;
                        Logger::trace(format!("Hash {} ({}) was not striked today or yesterday, everything has been reset", hash, strike_type.to_string(),).as_str());
                    }
                }
                // If the strike record of the hash doesn't exist, strike for the first time
                None => {
                    tx.execute(
                        "INSERT INTO strikes (strike_type, hash, strikes, strike_days) VALUES (?1, ?2, ?3, ?4)",
                        params![strike_type.to_string(), hash, 1, 1],
                    )
                    .context("Failed to insert new strike")?;
                }
            }
        }

        tx.commit().context("Failed to commit strikes")?;

        Ok(())
    }
}
