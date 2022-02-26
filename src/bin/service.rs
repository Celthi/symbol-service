use anyhow::Result;
use debugid::DebugId;
use poem::{
    handler, listener::TcpListener, middleware::Tracing, web::Query, EndpointExt, Route, Server,
};
use serde::Deserialize;
use std::num::ParseIntError;
use std::process::exit;
use std::str::FromStr;
use symservice::config;
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
use symservice::debug_entry;
#[derive(Deserialize)]
struct DebugIdQuery {
    id: String,
}
fn get_debug_id_from_str(s: &str) -> Result<DebugId> {
    if s.find('-').is_none() {
        let buf = decode_hex(s)?;
        let debug_id = debug_entry::compute_debug_id(&buf, true);
        println!("computed debug id {}", debug_id);
        Ok(debug_id)
    } else {
        let debug_id = DebugId::from_str(s)?;
        Ok(debug_id)
    }
}
#[handler]
async fn query_by_debug_id(Query(params): Query<DebugIdQuery>) -> String {
    let host = config::POSTGRES_HOST;
    let dbname = config::POSTGRES_DB_NAME;
    let user = config::POSTGRES_USER;
    let password = config::POSTGRES_PASSWORD;
    let db_conn = match symservice::db::DB::new(host, dbname, user, password).await {
        Ok(conn) => conn,
        Err(e) => {
            println!("cannot connect to the database: {}.", e);
            exit(3);
        }
    };
    if let Ok(debug_id) = get_debug_id_from_str(&params.id) {
        if let Ok(Some(id)) = db_conn.has_debug_id(debug_id).await {
            format!(
                "debug id = {}, location = {} with note: {}",
                params.id, id.location, id.note
            )
        } else {
            format!(
                "No shared library for debug id ({}), please populate it beforehand.",
                debug_id
            )
        }
    } else {
        format!("{} is not a valid debug id.", params.id)
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    let app = Route::new()
        .at("/by_debug_id", query_by_debug_id)
        .with(Tracing);
    Server::new(TcpListener::bind("0.0.0.0:14308"))
        .run(app)
        .await
}
