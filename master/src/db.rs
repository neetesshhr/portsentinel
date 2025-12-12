use sqlx::{SqlitePool, Row};
use crate::state::NodeConfig;
use crate::auth::User;

// === INITIALIZATION ===

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Run the schema manually or via sqlx migrate
    // For simplicity without sqlx-cli, we execute the raw SQL string
    let schema = include_str!("../db/schema.sql");
    sqlx::query(schema).execute(pool).await?;
    Ok(())
}

// === USER MANAGEMENT ===

pub async fn get_user_by_username(pool: &SqlitePool, username: &str) -> Option<User> {
    sqlx::query_as::<_, User>(
        "SELECT username, password_hash, role, must_change_password FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn create_user(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (username, password_hash, role, must_change_password) VALUES (?, ?, ?, ?)"
    )
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.role)
    .bind(user.must_change_password)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_user_password(pool: &SqlitePool, username: &str, password_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET password_hash = ?, must_change_password = 0 WHERE username = ?"
    )
    .bind(password_hash)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_users(pool: &SqlitePool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT username, password_hash, role, must_change_password FROM users")
        .fetch_all(pool)
        .await
}

// === NODE MANAGEMENT ===

pub async fn get_all_nodes(pool: &SqlitePool) -> Result<Vec<NodeConfig>, sqlx::Error> {
    sqlx::query_as::<_, NodeConfig>(
        "SELECT id, name, url, token FROM nodes"
    )
    .fetch_all(pool)
    .await
}

pub async fn upsert_node(pool: &SqlitePool, node: &NodeConfig) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO nodes (id, name, url, token) VALUES (?, ?, ?, ?) 
         ON CONFLICT(id) DO UPDATE SET name=excluded.name, url=excluded.url, token=excluded.token"
    )
    .bind(&node.id)
    .bind(&node.name)
    .bind(&node.url)
    .bind(&node.token)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_node(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM nodes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
