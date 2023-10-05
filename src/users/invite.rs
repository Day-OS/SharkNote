use sqlx::Sqlite;

#[derive(sqlx::FromRow)]
pub struct Invite {
    email: String,
}
impl Invite {
    pub async fn is_email_invited(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        email: String,
    ) -> bool {
        sqlx::query_as::<_, Invite>("SELECT * FROM user_invited where email = ?1;")
            .bind(email)
            .fetch_one(connection)
            .await
            .is_ok()
    }
    pub async fn invite(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        email: String,
    ) -> Result<(), sqlx::Error> {
        let _ = sqlx::query_as::<_, Invite>(
            "INSERT INTO user_invited (email) VALUES (?1) RETURNING *;",
        )
        .bind(email)
        .fetch_one(connection)
        .await?;

        Ok(())
    }
}
