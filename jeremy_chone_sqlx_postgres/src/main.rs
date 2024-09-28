use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1) Create a connection pool

    // 2) Create a table if not exist

    // 3) Insert a new ticket

    // 4) Select all tickets

    // 5) Select query with map() (build the ticket manually)

    // 6) Select query_as (using derive FromRow)

    Ok(())
}
