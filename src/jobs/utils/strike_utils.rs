use anyhow::Context;
use chrono::{Duration, Local, NaiveDate};
use rusqlite::{Connection, params};

use crate::{jobs::enums::strike_type::StrikeType, logger::enums::category::Category, trace, warn};

#[derive(Clone)]
pub struct StrikeRecord {
    id: i32,
    strike_type: String,
    hash: String,
    strikes: i32,
    strike_days: i32,
    last_strike_date: NaiveDate,
}

impl StrikeRecord {
    pub fn is_limit_reached(&self, required_strikes: i32, min_strike_days: i32) -> bool {
        let today_local = Local::now().date_naive();
        let yesterday_local = today_local - Duration::days(1);
        if self.last_strike_date == today_local || self.last_strike_date == yesterday_local {
            return self.strikes >= required_strikes && self.strike_days >= min_strike_days;
        } else {
            return false;
        }
    }

    /* Getter */
    pub fn id(&self) -> &i32 {
        &self.id
    }
    pub fn strike_type(&self) -> &str {
        &self.strike_type
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn strikes(&self) -> &i32 {
        &self.strikes
    }
    pub fn strike_days(&self) -> &i32 {
        &self.strike_days
    }
    pub fn last_strike_date(&self) -> &NaiveDate {
        &self.last_strike_date
    }
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
    pub fn check_create_tables(&mut self) -> Result<(), anyhow::Error> {
        self.conn
            .execute(
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
        Ok(())
    }

    /**
     * Get strikes
     */
    pub fn get_strikes(&mut self, strike_type: &StrikeType, hashes: Option<Vec<String>>) -> Result<Vec<StrikeRecord>, anyhow::Error> {
        // Build statement from sql query
        let mut stmt = match &hashes {
            Some(hashes) => {
                let placeholders = std::iter::repeat("?").take(hashes.len()).collect::<Vec<&str>>().join(",");
                let sql = format!("SELECT id, strike_type, hash, strikes, strike_days, last_strike_date FROM strikes WHERE strike_type = ?1 AND hash IN ({})", placeholders);
                self.conn.prepare(sql.as_str()).context("Failed to prepare get_strikes select")?
            }
            None => self
                .conn
                .prepare("SELECT id, strike_type, hash, strikes, strike_days, last_strike_date FROM strikes WHERE strike_type = ?1")
                .context("Failed to prepare get_strikes select")?,
        };

        let rows = match hashes {
            // Execute query based on if hashes are passed or not
            Some(hashes) => {
                let mut params: Vec<String> = vec![strike_type.to_string()];
                params.extend(hashes);
                let params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|hash| hash as &dyn rusqlite::ToSql).collect();
                stmt.query(params.as_slice()).context("Failed to execute query to get strikes")?
            }
            None => stmt.query(params![strike_type.to_string()]).context("Failed to execute query to get strikes")?,
        }
        // Map results
        .mapped(|row| {
            let last_strike_date_str: String = row.get(5)?;
            let last_strike_date = NaiveDate::parse_from_str(&last_strike_date_str, "%Y-%m-%d").map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(StrikeRecord {
                id: row.get(0)?,
                strike_type: row.get(1)?,
                hash: row.get(2)?,
                strikes: row.get(3)?,
                strike_days: row.get(4)?,
                last_strike_date,
            })
        });

        let mut strike_records: Vec<StrikeRecord> = Vec::new();
        for row in rows {
            match row {
                Ok(strike_record) => strike_records.push(strike_record),
                Err(e) => {
                    anyhow::bail!(e);
                }
            }
        }

        Ok(strike_records)
    }

    /**
     * Strike multiple
     */
    pub fn strike(&mut self, strike_type: &StrikeType, hashes: Vec<String>) -> Result<(), anyhow::Error> {
        // Get current strike records
        let strike_records = self.get_strikes(strike_type, Some(hashes.clone())).context("Failed to get strike types")?;

        // Open transaction
        let tx = self.conn.transaction().context("Failed to get transaction")?;

        // Handle hashes
        for hash in hashes {
            // Try to get the strike record of the hash
            let strike_records_for_hash: Vec<StrikeRecord> = strike_records
                .clone()
                .into_iter()
                .filter(|strike_record| strike_record.strike_type == strike_type.to_string() && strike_record.hash == hash)
                .collect();
            // This should never be the case due to the unique contraint but you never know
            if strike_records_for_hash.len() > 1 {
                warn!(
                    Category::Striker,
                    "Hash {} for strike type {} is {} times in the db. This should be impossible",
                    hash,
                    strike_type.to_string(),
                    strike_records_for_hash.len(),
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
                            "UPDATE strikes SET strikes = strikes + 1, strike_days = strike_days + 1, last_strike_date = ?1 WHERE strike_type = ?2 AND hash = ?3",
                            params![today_local.format("%Y-%m-%d").to_string(), strike_type.to_string(), hash],
                        )
                        .context("Failed to insert new strike")?;
                        trace!(Category::Striker, "Hash {} ({}) was last striked yesterday, strikes and strike days have been increased", hash, strike_type.to_string(),);
                    }
                    // If the strike record was last striked today, just increase strikes
                    else if strike_record.last_strike_date == today_local {
                        tx.execute("UPDATE strikes SET strikes = strikes + 1 WHERE strike_type = ?1 AND hash = ?2", params![strike_type.to_string(), hash])
                            .context("Failed to insert new strike")?;
                        trace!(Category::Striker, "Hash {} ({}) was last striked today, strikes have been increased", hash, strike_type.to_string());
                    }
                    // If the strike record was not striked today or yesterday, reset it
                    else {
                        tx.execute(
                            "UPDATE strikes SET strikes = 1, strike_days = 1, last_strike_date = ?1 WHERE strike_type = ?2 AND hash = ?3",
                            params![today_local.format("%Y-%m-%d").to_string(), strike_type.to_string(), hash],
                        )
                        .context("Failed to insert new strike")?;
                        trace!(Category::Striker, "Hash {} ({}) was not striked today or yesterday, everything has been reset", hash, strike_type.to_string(),);
                    }
                }
                // If the strike record of the hash doesn't exist, strike for the first time
                None => {
                    tx.execute(
                        "INSERT INTO strikes (strike_type, hash, strikes, strike_days, last_strike_date) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![strike_type.to_string(), hash, 1, 1, Local::now().date_naive().format("%Y-%m-%d").to_string()],
                    )
                    .context("Failed to insert new strike")?;
                    trace!(Category::Striker, "Hash {} ({}) has been striked for the first time", hash, strike_type.to_string());
                }
            }
        }

        tx.commit().context("Failed to commit strikes")?;

        Ok(())
    }

    /**
     * Delete strikes
     */
    pub fn delete(&mut self, strike_type: StrikeType, hashes: Vec<String>) -> Result<(), anyhow::Error> {
        let placeholders = std::iter::repeat("?").take(hashes.len()).collect::<Vec<&str>>().join(",");
        let sql = format!("DELETE FROM strikes WHERE strike_type = ?1 AND hash IN ({})", placeholders);

        let mut params: Vec<String> = vec![strike_type.to_string()];
        params.extend(hashes);
        let params: Vec<&dyn rusqlite::ToSql> = params.iter().map(|hash| hash as &dyn rusqlite::ToSql).collect();

        self.conn.execute(&sql, params.as_slice()).context("Failed to delete strikes")?;

        Ok(())
    }
}
