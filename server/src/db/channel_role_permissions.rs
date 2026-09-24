use crate::*;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChannelRolePermission {
	pub id: Uuid,
	pub channel_id: Uuid,
	pub role_id: Uuid,
	pub permissions: i32,
	pub created_at: time::OffsetDateTime,
	pub updated_at: time::OffsetDateTime,
}

pub async fn update_or_insert_channel_role_permission(pool: &sqlx::sqlite::SqlitePool, channel_id: Uuid, role_id: Uuid, permissions: i32) -> error::Result<ChannelRolePermission> {
	let id = Uuid::now_v7();
	let created_at = time::OffsetDateTime::now_utc();

	Ok(sqlx::query_as!(
		ChannelRolePermission,
		r#"
			INSERT INTO channel_role_permissions
				(id, channel_id, role_id, permissions, created_at, updated_at)
			VALUES
				($1, $2, $3, $4, $5, $5)
			ON CONFLICT (channel_id, role_id) DO UPDATE SET
				permissions = excluded.permissions,
				updated_at = excluded.updated_at
			RETURNING
				id as "id!: Uuid",
				channel_id as "channel_id!: Uuid",
                role_id as "role_id!: Uuid",
				permissions as "permissions!: i32",
				created_at as "created_at!: time::OffsetDateTime",
                updated_at as "updated_at!: time::OffsetDateTime"
		"#,
		id,
		channel_id,
		role_id,
		permissions,
		created_at
	)
	.fetch_one(pool)
	.await?)
}

pub async fn delete_channel_role_permission(pool: &sqlx::sqlite::SqlitePool, id: Uuid) -> error::Result<ChannelRolePermission> {
	Ok(sqlx::query_as!(
		ChannelRolePermission,
		r#"
			DELETE FROM
				channel_role_permissions
			WHERE
				id = $1
			RETURNING
				id as "id!: Uuid",
				channel_id as "channel_id!: Uuid",
                role_id as "role_id!: Uuid",
				permissions as "permissions!: i32",
				created_at as "created_at!: time::OffsetDateTime",
                updated_at as "updated_at!: time::OffsetDateTime"
		"#,
		id,
	)
	.fetch_one(pool)
	.await?)
}
