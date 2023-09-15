use crate::users::user::User;

//use crate::users::user::User;
use rocket_db_pools::sqlx;
use sqlx::Sqlite;

#[derive(sqlx::FromRow)]
pub struct Page {
    pub page_id: String,
    pub page_display_name: String,
    pub b_is_archived: bool,
}

impl Page {
    pub async fn new(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        page_id: String,
        page_display_name: Option<String>,
    ) -> Result<Page, sqlx::Error> {
        let page = sqlx::query_as::<_, Page>("INSERT INTO page (page_id, page_display_name, b_is_archived) VALUES (?1, ?2, 0) RETURNING *;")
        .bind(page_id)
        .bind(page_display_name)
        .fetch_one(connection)
        .await?;
        Ok(page)
    }

    pub async fn delete(
        self: Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM page WHERE page_id = ?1")
            .bind(self.page_id)
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn get(
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        page_id: String,
    ) -> Result<Page, sqlx::Error> {
        let page = sqlx::query_as::<_, Page>("SELECT * FROM page where page_id = ?1")
            .bind(page_id)
            .fetch_one(connection)
            .await?;
        Ok(page)
    }

    pub async fn set_owner(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: &User,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO page_user (page_id, user_id) VALUES (?1, ?2)")
            .bind(self.page_id.clone())
            .bind(user.user_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn remove_owner(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM page_user WHERE page_id = ?1")
            .bind(self.page_id.clone())
            .execute(connection)
            .await?;
        Ok(())
    }

    pub async fn get_owner(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user where user_id in (SELECT user_id FROM page_user WHERE page_id = ?1)")
            .bind(self.page_id.clone())
            .fetch_one(connection)
            .await?;
        Ok(user)
    }

    pub async fn check_if_user_is_owner(
        self: &Self,
        connection: &mut sqlx::pool::PoolConnection<Sqlite>,
        user: User,
    ) -> Result<bool, sqlx::Error> {
        let owner = self.get_owner(connection).await?;
        Ok(user.user_id == owner.user_id)
    }
}

#[derive(sqlx::FromRow)]
pub struct PageUser {
    pub page_id: String,
    pub user_id: String,
}
