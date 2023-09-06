use crate::models::{LoginPayload, User};
use sqlx::postgres::PgPool;
extern crate rand;
use crate::error::PsqlError;

/// Create new user in postgresql database. Connection details to the db in Dev are provided via
/// an environment variable (local .env file), to make it easier for testing.
pub async fn db_psql_create_user<'a>(pool: &PgPool, user: User<'a>) -> Result<i32, PsqlError> {
    let new_user = sqlx::query!(
        r#"
        INSERT INTO users (id, username, pwd)
        VALUES ( $1, $2, $3 )
        returning id
        "#,
        user.id,
        user.username,
        user.pwd
    )
    .fetch_one(pool)
    .await?;

    Ok(new_user.id)
}

pub async fn db_psql_validate_user<'a>(
    pool: &PgPool,
    user: &LoginPayload<'a>,
) -> Result<(), PsqlError> {
    let result = sqlx::query!(
        r#"
            SELECT pwd
            FROM users
            WHERE username = $1
        "#,
        user.username
    )
    .fetch_one(pool)
    .await?;

    if result.pwd.is_empty() {
        return Err(PsqlError::SqlxError(sqlx::Error::RowNotFound));
    }
    if result.pwd == user.pwd {
        Ok(())
    } else {
        Err(PsqlError::PasswordMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::env;

    #[tokio::test]
    async fn psql_create_user() {
        dotenv::dotenv().ok();
        let database_url =
            env::var("DATABASE_URL").expect("Failed to read test 'database_url' env variable.");
        let mut test_id: i32 = env::var("DB_TEST_ID")
            .expect("Failed to read test 'id' env variable")
            .parse()
            .expect("Failed to parse string to u32");
        let test_username = env::var("DB_TEST_USER1_USERNAME")
            .expect("Failed to read test 'user1 username' env variable.");
        let test_pwd =
            env::var("DB_TEST_USER1_PWD").expect("Failed to read test 'user1 pwd' env variable.");
        let test_pool = PgPool::connect(database_url.as_str())
            .await
            .expect("Failed to create psql pool");

        let random_addition: i32 = rand::thread_rng().gen_range(1..=100);
        test_id += random_addition;

        let test_user = User::new(Some(test_id), test_username.as_str(), test_pwd.as_str())
            .expect("Failed to create new user");

        assert!(db_psql_create_user(&test_pool, test_user).await.is_ok());
    }

    #[tokio::test]
    async fn psql_validate_admin_user() {
        dotenv::dotenv().ok();
        let database_url =
            env::var("DATABASE_URL").expect("Failed to read test 'database_url' env variable.");
        let admin_username = env::var("DB_ADMIN_USER_USERNAME")
            .expect("Failed to read test 'admin username' env variable.");
        let admin_pwd =
            env::var("DB_ADMIN_USER_PWD").expect("Failed to read test 'admin pwd' env variable.");

        let test_pool = PgPool::connect(database_url.as_str())
            .await
            .expect("Failed to create psql pool");

        let admin_user = LoginPayload::new(admin_username.as_str(), admin_pwd.as_str())
            .expect("Failed to create new user");

        assert!(db_psql_validate_user(&test_pool, &admin_user).await.is_ok());
    }

    #[tokio::test]
    async fn psql_validate_user_doesnt_exist() {
        dotenv::dotenv().ok();
        let database_url =
            env::var("DATABASE_URL").expect("Failed to read test 'database_url' env variable.");
        let dummy_username = env::var("DB_TEST_USER_DOESNT_EXIST_USERNAME")
            .expect("Failed to read test 'user doenst exist username' env variable.");
        let dummy_pwd = env::var("DB_TEST_USER_DOESNT_EXIST_PWD")
            .expect("Failed to read test 'user doesnt exist pwd' env variable.");

        let test_pool = PgPool::connect(database_url.as_str())
            .await
            .expect("Failed to create psql pool");

        let dummy_user = LoginPayload::new(dummy_username.as_str(), dummy_pwd.as_str())
            .expect("Failed to create new user");

        assert!(db_psql_validate_user(&test_pool, &dummy_user)
            .await
            .is_err());
    }
}
