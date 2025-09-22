use common::error::Error;
use common::translation::Translation;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for Translation {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r###" 
            INSERT INTO translations (
                task_id,
                language_id,
                content_hash,
                last_updated_at
            )
            VALUES (?, ?, ?, ?)
            RETURNING id
            "###,
            self.task_id,
            self.language_id,
            self.content_hash,
            self.last_updated_at,
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
            UPDATE translations
            SET
                task_id = ?,
                language_id = ?,
                content_hash = ?,
                last_updated_at = ?
            WHERE
                id = ?
            "###,
            self.task_id,
            self.language_id,
            self.content_hash,
            self.last_updated_at,
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
        sqlx::query!("DELETE FROM translations WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let translation = sqlx::query_as!(
            Translation,
            r###" 
            SELECT
                id,
                task_id,
                language_id,
                content_hash,
                last_updated_at
            FROM
                translations
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(translation)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let translations = sqlx::query_as!(
            Translation,
            r###" 
            SELECT
                id,
                task_id,
                language_id,
                content_hash,
                last_updated_at
            FROM
                translations
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(translations)
    }
}
