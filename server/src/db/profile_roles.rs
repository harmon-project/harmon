use crate::*;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProfileRole {
	pub id: Uuid,
	pub profile_id: Uuid,
	pub role_id: Uuid,
	pub created_at: time::OffsetDateTime,
}

pub async fn insert_profile_role(pool: &sqlx::sqlite::SqlitePool, profile_id: Uuid, role_id: Uuid) -> error::Result<ProfileRole> {
	let id = Uuid::now_v7();
	let created_at = time::OffsetDateTime::now_utc();

	Ok(sqlx::query_as!(
		ProfileRole,
		r#"
			INSERT INTO profile_roles
				(id, profile_id, role_id, created_at)
			VALUES
				($1, $2, $3, $4)
			RETURNING
				id as "id!: Uuid",
				profile_id as "profile_id!: Uuid",
                role_id as "role_id!: Uuid",
				created_at as "created_at!: time::OffsetDateTime"
		"#,
		id,
		profile_id,
		role_id,
		created_at
	)
	.fetch_one(pool)
	.await?)
}

pub async fn delete_profile_role(pool: &sqlx::sqlite::SqlitePool, id: Uuid) -> error::Result<ProfileRole> {
	Ok(sqlx::query_as!(
		ProfileRole,
		r#"
			DELETE FROM
				profile_roles
			WHERE
				id = $1
			RETURNING
				id as "id!: Uuid",
				profile_id as "profile_id!: Uuid",
                role_id as "role_id!: Uuid",
				created_at as "created_at!: time::OffsetDateTime"
		"#,
		id,
	)
	.fetch_one(pool)
	.await?)
}
