use anyhow::Result;
use symservice::config;
use tokio_postgres::NoTls;
pub async fn create_db() -> Result<()> {
    let conn_string = format!(
        "host={} dbname={} user={} password={}",
        config::POSTGRES_HOST,
        config::POSTGRES_DB_NAME,
        config::POSTGRES_USER,
        config::POSTGRES_PASSWORD
    );
    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;
    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .batch_execute(
            "
        DROP TABLE IF EXISTS debug_ids;
        CREATE TABLE debug_ids (
            debug_id CHAR(36) PRIMARY KEY not NULL,
            location VARCHAR(750),
            note VARCHAR(750)
        );             
    ",
        )
        .await?;

    let name = "c36a2a6e-6a5d-b272-1ca1-3baf2ea5e4b0";
    let data = "/usr/lib64";
    client
        .execute(
            "INSERT INTO debug_ids (debug_id, location) VALUES ($1, $2)",
            &[&name, &data],
        )
        .await?;

    for row in client
        .query("SELECT debug_id, location FROM debug_ids", &[])
        .await?
    {
        let id: &str = row.get(0);
        let loc: &str = row.get(1);

        println!("found library: {} {}", id, loc);
    }
    Ok(())
}
#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<()> {
    create_db().await?;
    Ok(())
}
