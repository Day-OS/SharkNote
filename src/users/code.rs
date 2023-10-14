use rand::Rng;
use sqlx::Sqlite;

use super::User;

#[derive(sqlx::FromRow)]
pub struct Code {
    pub user_id: String,
    pub code: u32,
}
impl Code {
    pub async fn generate(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
    ) -> Result<u32, sqlx::Error> {
        let code: u32 = rand::thread_rng().gen_range(0..999999);
        let _ = sqlx::query_as::<_, Code>(
            "INSERT INTO user_code (user_id, code) VALUES (?1, ?2) RETURNING *;",
        )
        .bind(user.user_id.clone())
        .bind(code)
        .fetch_one(connection)
        .await?;

        Ok(code)
    }

    pub async fn get(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
    ) -> Result<Self, sqlx::Error> {
        let code = sqlx::query_as::<_, Code>("SELECT * FROM user_code WHERE user_id = ?1;")
            .bind(user.user_id.clone())
            .fetch_one(connection)
            .await?;
        Ok(code)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM user_code WHERE user_id = ?1;")
            .bind(self.user_id)
            .execute(connection)
            .await?;
        Ok(())
    }
}
