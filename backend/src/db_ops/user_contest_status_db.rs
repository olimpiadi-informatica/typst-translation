use common::error::Error;
use common::user_contest_status::UserContestStatus;
use sqlx::{Executor, Sqlite};

use crate::db_ops::DatabaseOps;

impl DatabaseOps for UserContestStatus {
    async fn insert<'e, E>(&mut self, executor: E) -> Result<(), Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let row = sqlx::query!(
            r###" 
            INSERT INTO user_contest_status (
                user_id,
                contest_id,
                finalized_translations,
                skip_envelope_verification,
                envelope_received_at
            )
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "###,
            self.user_id,
            self.contest_id,
            self.finalized_translations,
            self.skip_envelope_verification,
            self.envelope_received_at,
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
            UPDATE user_contest_status
            SET
                user_id = ?,
                contest_id = ?,
                finalized_translations = ?,
                skip_envelope_verification = ?,
                envelope_received_at = ?
            WHERE
                id = ?
            "###,
            self.user_id,
            self.contest_id,
            self.finalized_translations,
            self.skip_envelope_verification,
            self.envelope_received_at,
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
        sqlx::query!("DELETE FROM user_contest_status WHERE id = ?", id)
            .execute(executor)
            .await?;
        Ok(())
    }

    async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let user_contest_status = sqlx::query_as!(
            UserContestStatus,
            r###" 
            SELECT
                id,
                user_id,
                contest_id,
                finalized_translations,
                skip_envelope_verification,
                envelope_received_at
            FROM
                user_contest_status
            WHERE
                id = ?
            "###,
            id
        )
        .fetch_optional(executor)
        .await?;
        Ok(user_contest_status)
    }

    async fn get_all<'e, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let user_contest_statuses = sqlx::query_as!(
            UserContestStatus,
            r###" 
            SELECT
                id,
                user_id,
                contest_id,
                finalized_translations,
                skip_envelope_verification,
                envelope_received_at
            FROM
                user_contest_status
            ORDER BY
                id DESC
            "###
        )
        .fetch_all(executor)
        .await?;
        Ok(user_contest_statuses)
    }
}
