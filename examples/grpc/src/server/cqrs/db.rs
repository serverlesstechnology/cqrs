use sqlx::postgres::{
    PgPool,
    PgPoolOptions,
};

pub async fn db_connection() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            "postgresql://demo_user:demo_pass@localhost:5432/demo",
        )
        .await
        .unwrap();

    Ok(pool)
}
