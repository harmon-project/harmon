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

pub async fn get_channel_role_permissions_from_profile(pool: impl sqlx::SqliteExecutor<'_>, channel_id: Uuid, profile_id: Uuid) -> error::Result<Vec<ChannelRolePermission>> {
	Ok(sqlx::query_as!(
		ChannelRolePermission,
		r#"
			SELECT
				channel_role_permissions.id as "id!: Uuid",
				channel_role_permissions.channel_id as "channel_id!: Uuid",
				channel_role_permissions.role_id as "role_id!: Uuid",
				channel_role_permissions.permissions as "permissions!: i32",
				channel_role_permissions.created_at as "created_at!: time::OffsetDateTime",
				channel_role_permissions.updated_at as "updated_at!: time::OffsetDateTime"
			FROM
				channel_role_permissions
			INNER JOIN
				profile_roles ON profile_roles.role_id = channel_role_permissions.role_id
			WHERE
				channel_role_permissions.channel_id = $1
				AND
				profile_roles.profile_id = $2
		"#,
		channel_id,
		profile_id,
	)
	.fetch_all(pool)
	.await?)
}

pub async fn update_or_insert_channel_role_permission(pool: impl sqlx::SqliteExecutor<'_>, channel_id: Uuid, role_id: Uuid, permissions: i32) -> error::Result<ChannelRolePermission> {
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

pub async fn delete_channel_role_permission(pool: impl sqlx::SqliteExecutor<'_>, id: Uuid) -> error::Result<ChannelRolePermission> {
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
