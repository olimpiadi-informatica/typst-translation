use common::contestant::Contestant;
use common::error::Error;
use sqlx::{Executor, Pool, Sqlite};

use crate::db_ops::language_db;

pub async fn insert<'e, E>(contestant: &mut Contestant, executor: E) -> Result<(), Error>
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
            language_decided,
            language_id
        )
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id as "id!"
        "###,
        contestant.code,
        contestant.name,
        contestant.online_bit,
        contestant.user_id,
        contestant.language_decided,
        contestant.language_id,
    )
    .fetch_one(executor)
    .await?;
    contestant.id = row.id;
    Ok(())
}

pub async fn update<'e, E>(contestant: &Contestant, executor: E) -> Result<(), Error>
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
            language_decided = ?,
            language_id = ?
        WHERE
            id = ?
        "###,
        contestant.code,
        contestant.name,
        contestant.online_bit,
        contestant.user_id,
        contestant.language_decided,
        contestant.language_id,
        contestant.id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn delete<'e, E>(executor: E, id: i64) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM contestants WHERE id = ?", id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn get_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Contestant>, Error>
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
            language_decided,
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

pub async fn get_all<'e, E>(executor: E) -> Result<Vec<Contestant>, Error>
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
            language_decided,
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
            language_decided,
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
        r###"
        SELECT *
        FROM contestants
        WHERE user_id = ?
        ORDER BY code ASC
        "###,
        user_id
    )
    .fetch_all(executor)
    .await?;
    Ok(contestants)
}

pub async fn update_contestant<'e, E>(
    executor: E,
    id: i64,
    code: String,
    name: String,
    online_bit: bool,
) -> Result<(), Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r###"
        UPDATE contestants
        SET
            code = ?,
            name = ?,
            online_bit = ?
        WHERE
            id = ?
        "###,
        code,
        name,
        online_bit,
        id
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn assign_language_to_contestant(
    pool: &Pool<Sqlite>,
    contestant_id: i64,
    language_id: Option<i64>,
    user_id: i64,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let finalized_status = sqlx::query!(
        r###"
        SELECT id
        FROM user_contest_status
        WHERE user_id = ? AND finalized_translations = FALSE
        LIMIT 1
        "###,
        user_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    if finalized_status.is_none() {
        return Err(Error::InvalidInput(
            "Cannot assign language to contestant after a contest has been finalized.".to_string(),
        ));
    }

    if let Some(lang_id) = language_id {
        let language = language_db::get_by_id(&mut *tx, lang_id).await?;

        if !language.public && language.user_id != user_id {
            return Err(Error::InvalidInput("Cannot assign language to contestant: language is not public and not owned by the user.".to_string()));
        }
    }

    let res = sqlx::query!(
        r###"
        UPDATE contestants
        SET
            language_decided = true,
            language_id = ?
        WHERE
            id = ? AND user_id = ?
        "###,
        language_id,
        contestant_id,
        user_id
    )
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    tx.commit().await?;

    Ok(())
}
