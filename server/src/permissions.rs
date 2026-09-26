use crate::*;

use uuid::Uuid;

bitflags::bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct ChannelPermissions: i32 {
		const READ_MESSAGES	= 1 << 0;
		const SEND_MESSAGES	= 1 << 1;
	}

	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub struct GlobalPermissions: i64 {
		const READ_MESSAGES	= 1 << 0;
		const SEND_MESSAGES	= 1 << 1;

		const MANAGE_ROLES		= 1 << 32;
		const MANAGE_CHANNELS	= 1 << 33;
	}
}

pub async fn get_global_permissions(pool: impl sqlx::SqliteExecutor<'_>, profile_id: Uuid) -> error::Result<GlobalPermissions> {
	let roles = db::get_roles_from_profile(pool, profile_id).await?;
	let mut permissions = GlobalPermissions::empty();

	for role in roles {
		permissions |= GlobalPermissions::from_bits_truncate(role.permissions);
	}

	Ok(permissions)
}

pub async fn get_channel_permissions(pool: impl sqlx::SqliteExecutor<'_>, channel_id: Uuid, profile_id: Uuid) -> error::Result<ChannelPermissions> {
	let role_permissions = db::get_channel_role_permissions_from_profile(pool, channel_id, profile_id).await?;
	let mut permissions = ChannelPermissions::empty();

	for role in role_permissions {
		permissions |= ChannelPermissions::from_bits_truncate(role.permissions);
	}

	Ok(permissions)
}
