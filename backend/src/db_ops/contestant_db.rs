use common::error::Error;
use common::contestant::Contestant;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for Contestant {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r###" 
            INSERT INTO contestants (
                code,
                name,
                online_bit,
                user_id,
                language_id
            )
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "###,
            self.code,
            self.name,
            self.online_bit,
            self.user_id,
            self.language_id,
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
            r###" 
            UPDATE contestants
            SET
                code = ?,
                name = ?,
                online_bit = ?,
                user_id = ?,
                language_id = ?
            WHERE
                id = ?
            "###,
            self.code,
            self.name,
            self.online_bit,
            self.user_id,
            self.language_id,
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
        sqlx::query!("DELETE FROM contestants WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let contestant = sqlx::query_as!(
            Contestant,
            r###" 
            SELECT
                id,
                code,
                name,
                online_bit,
                user_id,
                language_id
            FROM
                contestants
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(contestant)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let contestants = sqlx::query_as!(
            Contestant,
            r###" 
            SELECT
                id,
                code,
                name,
                online_bit,
                user_id,
                language_id
            FROM
                contestants
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(contestants)
    }
}
