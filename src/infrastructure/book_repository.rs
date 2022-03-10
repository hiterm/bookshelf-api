#[cfg(test)]
mod tests {
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn connect() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&mut conn)
            .await.unwrap();
        assert_eq!(row.0, 150);
    }
}
