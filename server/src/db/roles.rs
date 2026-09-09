use sqlx::query_as;

use crate::*;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Role {
	pub id: Uuid,
	pub name: String,
	pub permissions: i64,
	pub created_at: time::OffsetDateTime,
	pub updated_at: time::OffsetDateTime,
}

pub async fn create_role(pool: impl sqlx::SqliteExecutor<'_>, name: &str, permissions: i64) -> error::Result<Role> {
	let id = Uuid::now_v7();
	let created_at = time::OffsetDateTime::now_utc();
	let updated_at = time::OffsetDateTime::now_utc();

	Ok(query_as!(
		Role,
		r#"
            INSERT INTO roles
                (id, name, permissions, created_at, updated_at)
            VALUES
                (?, ?, ?, ?, ?)
            RETURNING
                id as "id!: Uuid",
                name,
                permissions,
                created_at as "created_at!: time::OffsetDateTime",
                updated_at as "updated_at!: time::OffsetDateTime"
        "#,
		id,
		name,
		permissions,
		created_at,
		updated_at
	)
	.fetch_one(pool)
	.await?)
}
