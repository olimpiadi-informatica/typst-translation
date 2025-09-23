use common::contestant::Contestant;
use common::error::Error;
use common::language::Language;
use sqlx::{Executor, Pool, Sqlite};

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

pub async fn get_contestants_by_language_id<'e, E>(
    executor: E,
    language_id: i64,
) -> Result<Vec<Contestant>, Error>
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
        WHERE
            language_id = ?
        "###,
        language_id
    )
    .fetch_all(executor)
    .await?;
    Ok(contestants)
}

pub async fn get_contestants_by_user_id<'e, E>(
    executor: E,
    user_id: i64,
) -> Result<Vec<Contestant>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let contestants = sqlx::query_as!(
        Contestant,
        "SELECT * FROM contestants WHERE user_id = ?",
        user_id
    )
    .fetch_all(executor)
    .await?;
    Ok(contestants)
}

pub async fn assign_language_to_contestant(
    pool: &Pool<Sqlite>,
    contestant_id: i64,
    language_id: Option<i64>,
    user_id: i64,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    if let Some(lang_id) = language_id {
        let language = Language::get_by_id(&mut *tx, lang_id)
            .await?
            .ok_or(Error::NotFound)?;

        if !language.public && language.user_id != user_id {
            return Err(Error::InvalidInput("Cannot assign language to contestant: language is not public and not owned by the user.".to_string()));
        }
    }

    sqlx::query!(
        r###"
        UPDATE contestants
        SET
            language_id = ?
        WHERE
            id = ?
        "###,
        language_id,
        contestant_id
    )
    .execute(&mut *tx)
    .await?;

    Ok(())
}
