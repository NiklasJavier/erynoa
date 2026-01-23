//! Database Queries

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::Result;

/// User Model
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub zitadel_id: String,
    pub email: Option<String>,
    pub internal_role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub async fn find_by_zitadel_id(pool: &PgPool, zitadel_id: &str) -> Result<Option<Self>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, zitadel_id, email, internal_role, created_at, updated_at 
             FROM users WHERE zitadel_id = $1"
        )
        .bind(zitadel_id)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT id, zitadel_id, email, internal_role, created_at, updated_at 
             FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn upsert(pool: &PgPool, zitadel_id: &str, email: Option<&str>) -> Result<Self> {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (zitadel_id, email)
             VALUES ($1, $2)
             ON CONFLICT (zitadel_id) DO UPDATE
             SET email = COALESCE(EXCLUDED.email, users.email), updated_at = NOW()
             RETURNING id, zitadel_id, email, internal_role, created_at, updated_at"
        )
        .bind(zitadel_id)
        .bind(email)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Self>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT id, zitadel_id, email, internal_role, created_at, updated_at 
             FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }
}
