use anyhow::Result;
use debugid::DebugId;
use tokio_postgres::{Client, NoTls};

use crate::debug_entry;
use std::str::FromStr;
pub struct DB {
    client: Client,
}

impl DB {
    pub async fn new(host: &str, dbname: &str, user: &str, password: &str) -> Result<Self> {
        let connect_string = format!(
            "host={} db_name={} user={} password={}",
            host, dbname, user, password
        );
        let (client, connection) = tokio_postgres::connect(&connect_string, NoTls).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let client = client;

        Ok(DB { client })
    }
    // will update the location if insert the debug id again
    pub async fn insert_one_debug_id(&mut self, entry: &debug_entry::DebugEntry) -> Result<()> {
        let debug_id = &entry.debug_id;
        let id = debug_id.to_string();
        let loc = &entry.location;
        let note = &entry.note;
        // self.client.execute(
        //     "INSERT INTO debug_ids (debug_id, location, note) VALUES ($1, $2, $3) ON CONFLICT (debug_id) DO UPDATE SET (location, note) = ($2, $3)",
        //     &[&id, &loc, &note],
        // ).await?;
        self.client.execute(
            "INSERT INTO debug_ids (debug_id, location, note) VALUES ($1, $2, $3) ON CONFLICT (debug_id) DO NOTHING",
            &[&id, &loc, &note],
        ).await?;
        Ok(())
    }
    pub async fn has_debug_id(&self, debug_id: DebugId) -> Result<Option<debug_entry::DebugEntry>> {
        let id = debug_id.to_string();
        let rows = self
            .client
            .query(
                "Select debug_id, location, note from debug_ids WHERE debug_id = $1",
                &[&id],
            )
            .await?;
        if !rows.is_empty(){
            let s: &str = rows[0].get(0);
            let loc: &str = rows[0].get(1);
            let note: &str = rows[0].get(2);
            if let Ok(debug_id) = DebugId::from_str(s) {
                return Ok(Some(debug_entry::DebugEntry {
                    debug_id,
                    location: loc.to_owned(),
                    note: note.to_owned(),
                }));
            }
        }
        Ok(None)
    }
}
