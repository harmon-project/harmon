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

	Ok(sqlx::query_as!(
		Role,
		r#"
            INSERT INTO roles
                (id, name, permissions, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $4)
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
		created_at
	)
	.fetch_one(pool)
	.await?)
}

pub async fn get_roles_from_profile(pool: impl sqlx::SqliteExecutor<'_>, profile_id: Uuid) -> error::Result<Vec<Role>> {
	Ok(sqlx::query_as!(
		Role,
		r#"
			SELECT
				roles.id as "id!: Uuid",
				roles.name,
				roles.permissions,
				roles.created_at as "created_at!: time::OffsetDateTime",
				roles.updated_at as "updated_at!: time::OffsetDateTime"
			FROM
				roles
			INNER JOIN
				profile_roles ON profile_roles.role_id = roles.id
			WHERE
				profile_roles.profile_id = $1
		"#,
		profile_id,
	)
	.fetch_all(pool)
	.await?)
}
