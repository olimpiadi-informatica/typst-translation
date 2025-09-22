use common::error::Error;
use common::user::User;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

pub async fn get_user_by_username<'e, E>(executor: E, username: &str) -> Result<Option<User>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id,
            username,
            password,
            login_epoch,
            automatic_translation_budget
        FROM
            users
        WHERE
            username = ?
        "#,
        username
    )
    .fetch_optional(executor)
    .await?;
    Ok(user)
}

impl DatabaseOps for User {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (
                username,
                password,
                login_epoch,
                automatic_translation_budget
            )
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
            self.username,
            self.password,
            self.login_epoch,
            self.automatic_translation_budget,
        )
        .fetch_one(executor)
        .await?;
        self.id = row.id;
        Ok(())
    }

    async fn update<'e, E>(&self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        sqlx::query!(
            r#"
            UPDATE users
            SET
                username = ?,
                password = ?,
                login_epoch = ?,
                automatic_translation_budget = ?
            WHERE
                id = ?
            "#,
            self.username,
            self.password,
            self.login_epoch,
            self.automatic_translation_budget,
            self.id
        )
        .execute(executor)
        .await?;
        Ok(())
    }

    async fn delete<'e, E>(executor: E, id: i64) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        sqlx::query!("DELETE FROM users WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                username,
                password,
                login_epoch,
                automatic_translation_budget
            FROM
                users
            WHERE
                id = ?
            "#,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(user)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT
                id,
                username,
                password,
                login_epoch,
                automatic_translation_budget
            FROM
                users
            ORDER BY
                id DESC
            "#,
        )
        .fetch_all(executor)
        .await?;
        Ok(users)
    }
}
