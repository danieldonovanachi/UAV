use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{error, info};

pub const SESSION_USER_KEY: &str = "user_id";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
}

pub struct AuthState {
    pub db: SqlitePool,
}

impl AuthState {
    pub async fn new() -> Result<Self, sqlx::Error> {
        // Create in-memory SQLite database for user storage
        let db = SqlitePool::connect("sqlite::memory:").await?;
        
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL
            )
            "#,
        )
        .execute(&db)
        .await?;

        // Create default admin user (username: admin, password: admin)
        // In production, change this password!
        let default_password_hash = hash("admin", DEFAULT_COST).unwrap();
        sqlx::query(
            "INSERT OR IGNORE INTO users (username, password_hash) VALUES (?, ?)",
        )
        .bind("admin")
        .bind(&default_password_hash)
        .execute(&db)
        .await?;

        info!("Authentication initialized with default admin user (username: admin, password: admin)");
        
        Ok(Self { db })
    }

    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, username, password_hash FROM users WHERE username = ?"
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        let user = match row {
            Some(row) => Some(User {
                id: row.get("id"),
                username: row.get("username"),
                password_hash: row.get("password_hash"),
            }),
            None => None,
        };

        if let Some(user) = user {
            match verify(password, &user.password_hash) {
                Ok(true) => Ok(Some(user)),
                Ok(false) => Ok(None),
                Err(e) => {
                    error!("Password verification error: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub async fn create_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, sqlx::Error> {
        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|e| sqlx::Error::Protocol(format!("Failed to hash password: {}", e)))?;

        let row = sqlx::query(
            "INSERT INTO users (username, password_hash) VALUES (?, ?) RETURNING id",
        )
        .bind(username)
        .bind(&password_hash)
        .fetch_one(&self.db)
        .await?;
        let id: i64 = row.get(0);

        Ok(User {
            id,
            username: username.to_string(),
            password_hash,
        })
    }
}

pub async fn login_handler(
    State(state): State<crate::server::AppState>,
    session: Session,
    Json(login): Json<LoginRequest>,
) -> Response {
    match state.auth_state.authenticate_user(&login.username, &login.password).await {
        Ok(Some(user)) => {
            session
                .insert(SESSION_USER_KEY, user.id)
                .map_err(|e| {
                    error!("Failed to set session: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
                .unwrap();

            info!("User '{}' logged in successfully", login.username);
            Json(LoginResponse {
                success: true,
                message: "Login successful".to_string(),
            })
            .into_response()
        }
        Ok(None) => {
            info!("Failed login attempt for username: {}", login.username);
            (
                StatusCode::UNAUTHORIZED,
                Json(LoginResponse {
                    success: false,
                    message: "Invalid username or password".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LoginResponse {
                    success: false,
                    message: "Internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn logout_handler(session: Session) -> Response {
    session.delete();
    Json(LoginResponse {
        success: true,
        message: "Logged out successfully".to_string(),
    })
    .into_response()
}

pub async fn get_current_user(
    State(state): State<crate::server::AppState>,
    session: Session,
) -> Response {
    let user_id: Option<i64> = session.get(SESSION_USER_KEY).unwrap_or(None);

    match user_id {
        Some(id) => {
            match sqlx::query(
                "SELECT id, username, password_hash FROM users WHERE id = ?"
            )
            .bind(id)
            .fetch_optional(&state.auth_state.db)
            .await
            {
                Ok(Some(row)) => {
                    let user = User {
                        id: row.get("id"),
                        username: row.get("username"),
                        password_hash: row.get("password_hash"),
                    };
                    Json(serde_json::json!({
                        "success": true,
                        "user": {
                            "id": user.id,
                            "username": user.username
                        }
                    }))
                    .into_response()
                },
                Ok(None) | Err(_) => (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "success": false,
                        "message": "User not found"
                    })),
                )
                    .into_response(),
            }
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "message": "Not authenticated"
            })),
        )
            .into_response(),
    }
}

pub async fn require_auth(
    session: Session,
) -> Result<i64, (StatusCode, Json<serde_json::Value>)> {
    let user_id: Option<i64> = session.get(SESSION_USER_KEY).unwrap_or(None);

    match user_id {
        Some(id) => Ok(id),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Authentication required"
            })),
        )),
    }
}
