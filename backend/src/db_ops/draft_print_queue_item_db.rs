use common::draft_print_queue_item::DraftPrintQueueItem;
use common::error::Error;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for DraftPrintQueueItem {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r###" 
            INSERT INTO draft_print_queue (
                statement_version_id,
                language_id,
                added_at
            )
            VALUES (?, ?, ?)
            RETURNING id
            "###,
            self.statement_version_id,
            self.language_id,
            self.added_at,
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
            UPDATE draft_print_queue
            SET
                statement_version_id = ?,
                language_id = ?,
                added_at = ?
            WHERE
                id = ?
            "###,
            self.statement_version_id,
            self.language_id,
            self.added_at,
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
        sqlx::query!("DELETE FROM draft_print_queue WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let draft_print_queue_item = sqlx::query_as!(
            DraftPrintQueueItem,
            r###" 
            SELECT
                id,
                statement_version_id,
                language_id,
                added_at
            FROM
                draft_print_queue
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(draft_print_queue_item)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let draft_print_queue_items = sqlx::query_as!(
            DraftPrintQueueItem,
            r###" 
            SELECT
                id,
                statement_version_id,
                language_id,
                added_at
            FROM
                draft_print_queue
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(draft_print_queue_items)
    }
}
