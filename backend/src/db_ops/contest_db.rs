use common::contest::{All, Contest, ContestWithAll, ContestWithTasksAndStatus};
use common::error::Error;
use common::language::Language;
use common::task::{Task, TaskDb};
use common::user::User;
use common::user_contest_status::UserContestStatus;
use sqlx::{Executor, Sqlite, SqlitePool, query_as};

use crate::db_ops::translation_db;

pub async fn get_user_contest_statuses_and_tasks(
    pool: &SqlitePool,
    user_id: i64,
) -> Result<Vec<ContestWithTasksAndStatus>, Error> {
    let contests = query_as!(Contest, "SELECT id, name FROM contests ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new();

    for contest in contests {
        let user_contest_status = query_as!(
            UserContestStatus,
            "SELECT * FROM user_contest_status WHERE user_id = ? AND contest_id = ?",
            user_id,
            contest.id
        )
        .fetch_one(pool)
        .await?;

        let tasks_db = query_as!(
            TaskDb,
            "SELECT id, contest_id, name FROM tasks WHERE contest_id = ?",
            contest.id
        )
        .fetch_all(pool)
        .await?;

        let mut tasks: Vec<Task> = Vec::new();
        for task_db in tasks_db {
            let translations =
                translation_db::get_translations_by_task_id(pool, task_db.id).await?;
            tasks.push(Task {
                id: task_db.id,
                contest_id: task_db.contest_id,
                name: task_db.name,
                translations,
            });
        }

        result.push(ContestWithTasksAndStatus {
            contest,
            user_contest_status,
            tasks,
        });
    }

    Ok(result)
}

pub async fn get_all<'e, E>(executor: E) -> Result<Vec<Contest>, Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let contests = sqlx::query_as!(
        Contest,
        r###"
        SELECT *
        FROM contests
        ORDER BY id DESC
        "###
    )
    .fetch_all(executor)
    .await?;
    Ok(contests)
}

pub async fn get_all_contests_with_all(pool: &sqlx::Pool<Sqlite>) -> Result<All, Error> {
    let contests = query_as!(Contest, "SELECT id, name FROM contests ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    let mut result = Vec::new();

    for contest in contests {
        let user_contest_status = query_as!(
            UserContestStatus,
            "SELECT * FROM user_contest_status WHERE contest_id = ?",
            contest.id
        )
        .fetch_all(pool)
        .await?;

        let tasks_db = query_as!(
            TaskDb,
            "SELECT id, contest_id, name FROM tasks WHERE contest_id = ?",
            contest.id
        )
        .fetch_all(pool)
        .await?;

        let mut tasks: Vec<Task> = Vec::new();
        for task_db in tasks_db {
            let translations =
                translation_db::get_translations_by_task_id(pool, task_db.id).await?;
            tasks.push(Task {
                id: task_db.id,
                contest_id: task_db.contest_id,
                name: task_db.name,
                translations,
            });
        }

        result.push(ContestWithAll {
            contest,
            user_contest_status,
            tasks,
        });
    }

    let contestants = sqlx::query_as!(
        common::contestant::Contestant,
        r#"
        SELECT *
        FROM contestants
        ORDER BY code ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let languages = sqlx::query_as!(
        Language,
        r#"
        SELECT *
        FROM languages
        ORDER BY id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let users = sqlx::query_as!(
        User,
        r#"
        SELECT *
        FROM users
        ORDER BY username ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(All {
        contests: result,
        contestants,
        languages,
        users,
    })
}
