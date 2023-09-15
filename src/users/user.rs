use crate::pages::page::Page;

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha256::digest;
use sqlx::Sqlite;

#[derive(sqlx::FromRow)]
pub struct User {
    pub user_id: String,
    pub password: String,
    pub email: String,
    pub display_name: String,
    pub otp: Option<String>,
    pub configuration_json: String,
    pub is_program_admin: u8,
    pub account_status: String,
}

#[derive(Serialize, Deserialize)]
pub enum UserAccountStatus {
    Normal,
    RegistrationPending { code: u32 },
    PasswordRecovery,
}
impl UserAccountStatus {
    pub fn to_json(&self) -> String {
        json!(self).to_string()
    }
    pub fn from_json(json: String) -> Self {
        serde_json::from_str(&json).unwrap()
    }
}

fn sha256_password(user_id: String, password: String) -> String {
    digest(format!("{}/-/{}", user_id, password))
}

impl User {
    pub async fn new(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user_id: String,
        password: String,
        email: String,
        account_status: UserAccountStatus,
    ) -> Result<User, sqlx::Error> {
        let hash_password = sha256_password(user_id.clone(), password);
        let user = sqlx::query_as::<_, User>("INSERT INTO user (user_id, password, is_program_admin, email, account_status) VALUES (?1, ?2, 0, ?3, ?4) RETURNING *;")
        .bind(user_id)
        .bind(hash_password)
        .bind(email)
        .bind(account_status.to_json())
        .fetch_one(connection)
        .await?;
        Ok(user)
    }

    pub async fn get(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user_id: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user where user_id = ?1;")
            .bind(user_id)
            .fetch_one(connection)
            .await?;
        Ok(user)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM user WHERE user_id = ?1")
            .bind(self.user_id)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn set_status(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        status: UserAccountStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE user SET account_status = ?1 WHERE user_id = ?2;")
            .bind(status.to_json())
            .bind(self.user_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn set_admin(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        b: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE user SET is_program_admin = ?1 WHERE user_id = ?2;")
            .bind(b as u8)
            .bind(self.user_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn change_password(
        self: &mut Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        password: String,
    ) -> Result<(), sqlx::Error> {
        let hash_password = sha256_password(self.user_id.clone(), password);
        sqlx::query("UPDATE user SET password = ?1 WHERE user_id = ?2")
            .bind(hash_password)
            .bind(self.user_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn check_login_credentials(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user_id: String,
        password: String,
    ) -> Result<bool, sqlx::Error> {
        let user = User::get(connection, user_id).await?;
        Ok(sha256_password(user.user_id, password) == user.password)
    }

    pub fn get_owned_pages(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<Vec<Page>, sqlx::Error> {
        todo!()
    }
}
