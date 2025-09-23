use common::error::Error;
use common::language::Language;
use common::user_contest_status::UserContestStatus;
use sqlx::{SqlitePool, query, query_as};

pub async fn finalize_translation(
    pool: &SqlitePool,
    user_id: i64,
    contest_id: i64,
) -> Result<(), Error> {
    let result = query!(
        r#"
        UPDATE user_contest_status
        SET finalized_translations = TRUE
        WHERE contest_id = ? AND user_id = ?
        "#,
        contest_id,
        user_id
    )
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    Ok(())
}

pub async fn set_translation_session_token(
    pool: &SqlitePool,
    user_id: i64,
    task_id: i64,
    language_id: i64,
    session_token: String,
) -> Result<(), Error> {
    let mut tx = pool.begin().await?;

    let user_contest_status = query_as!(
        UserContestStatus,
        r#"
            SELECT user_contest_status.*
            FROM user_contest_status
            JOIN tasks ON user_contest_status.contest_id = tasks.contest_id
            WHERE tasks.id = ? AND user_contest_status.user_id = ?
        "#,
        task_id,
        user_id
    )
    .fetch_one(&mut *tx)
    .await?;

    if user_contest_status.finalized_translations {
        return Err(Error::InvalidInput(
            "Cannot set session token for a finalized translation.".to_string(),
        ));
    }

    let language = query_as!(
        Language,
        "SELECT * FROM languages WHERE languages.id = ?",
        language_id
    )
    .fetch_one(&mut *tx)
    .await?;

    if language.user_id != user_id {
        return Err(Error::InvalidInput(
            "Cannot set session token for a language you are not translating.".to_string(),
        ));
    }

    let result = query!(
        "UPDATE translations SET session_token = ? WHERE task_id = ? AND language_id = ?",
        session_token,
        task_id,
        language_id
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    tx.commit().await?;
    Ok(())
}
