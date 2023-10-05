use std::vec;

use crate::pages::page::{Page, PageUser};

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha256::digest;

#[derive(sqlx::FromRow)]
pub struct User {
    pub user_id: String,
    pub password: String,
    pub email: String,
    pub display_name: String,
    pub configuration_json: String,
    pub is_program_admin: u8,
    pub account_status: String,
}

#[derive(Serialize, Deserialize)]
pub enum UserAccountStatus {
    Normal,
    RegistrationPending,
    PasswordRecovery,
    Banned,
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
        connection: &mut sqlx::SqliteConnection,
        user_id: String,
        password: String,
        email: String,
        account_status: UserAccountStatus,
    ) -> Result<User, sqlx::Error> {
        let hash_password = sha256_password(user_id.clone(), password);
        let user = sqlx::query_as::<_, User>("INSERT INTO user (user_id, password, is_program_admin, email, account_status) VALUES (?1, ?2, 0, ?3, ?4) RETURNING *; DELETE FROM user_invited WHERE user_id = ?3;")
        .bind(user_id)
        .bind(hash_password)
        .bind(email)
        .bind(account_status.to_json())
        .fetch_one(connection)
        .await?;

        Ok(user)
    }

    pub async fn get(
        connection: &mut sqlx::SqliteConnection,
        user_id: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user where user_id = ?1;")
            .bind(user_id)
            .fetch_one(connection)
            .await?;
        Ok(user)
    }
    pub async fn get_from_email(
        connection: &mut sqlx::SqliteConnection,
        email: String,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user where email = ?1;")
            .bind(email)
            .fetch_one(connection)
            .await?;
        Ok(user)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM user WHERE user_id = ?1")
            .bind(self.user_id)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn set_status(
        self: &Self,
        connection: &mut sqlx::SqliteConnection,
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
        connection: &mut sqlx::SqliteConnection,
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
        connection: &mut sqlx::SqliteConnection,
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
        connection: &mut sqlx::SqliteConnection,
        user_id: String,
        password: String,
    ) -> Result<bool, sqlx::Error> {
        let user = User::get(connection, user_id).await?;
        Ok(sha256_password(user.user_id, password) == user.password)
    }

    pub async fn get_owned_pages(
        self: &Self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<Vec<Page>, sqlx::Error> {
        sqlx::query_as::<_, Page>("SELECT * FROM page WHERE page_id IN (SELECT page_id FROM page_user WHERE user_id = ?1);")
        .bind(self.user_id.clone())
        .fetch_all(connection)
        .await
    }
}
