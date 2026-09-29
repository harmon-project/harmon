use crate::*;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct ChannelPermissions(i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct GlobalPermissions(i64);

bitflags::bitflags! {
	impl ChannelPermissions: i32 {
		const READ_MESSAGES	= 1 << 0;
		const SEND_MESSAGES	= 1 << 1;
	}

	impl GlobalPermissions: i64 {
		const READ_MESSAGES	= ChannelPermissions::READ_MESSAGES.bits() as i64;
		const SEND_MESSAGES	= ChannelPermissions::SEND_MESSAGES.bits() as i64;

		const MANAGE_ROLES		= 1 << 32;
		const MANAGE_CHANNELS	= 1 << 33;
	}
}

impl GlobalPermissions {
	#[inline(always)]
	pub const fn to_channel(self) -> ChannelPermissions {
		ChannelPermissions(self.0 as u64 as i32)
	}
}

impl ChannelPermissions {
	#[inline(always)]
	pub const fn to_global(self) -> GlobalPermissions {
		GlobalPermissions(self.0 as u32 as i64)
	}
}

pub async fn get_global_permissions(app: &app::AppState, public_key: crypto::PublicKey) -> error::Result<GlobalPermissions> {
	let is_admin = app.config.admin_public_keys.contains(&public_key);

	if is_admin {
		return Ok(GlobalPermissions::all());
	}

	let Some(profile) = db::get_profile_by_public_key(&app.db_pool, public_key).await? else {
		return Ok(GlobalPermissions::empty());
	};

	let roles = db::get_roles_from_profile(&app.db_pool, profile.id).await?;
	let mut permissions = GlobalPermissions::empty();

	for role in roles {
		permissions |= role.permissions;
	}

	Ok(permissions)
}

pub async fn get_channel_permissions(app: &app::AppState, public_key: crypto::PublicKey, channel_id: Uuid) -> error::Result<ChannelPermissions> {
	let is_admin = app.config.admin_public_keys.contains(&public_key);

	if is_admin {
		return Ok(ChannelPermissions::all());
	}

	let Some(profile) = db::get_profile_by_public_key(&app.db_pool, public_key).await? else {
		return Ok(ChannelPermissions::empty());
	};

	let mut permissions = get_global_permissions(app, public_key).await?.to_channel();

	let role_permissions = db::get_channel_role_permissions_from_profile(&app.db_pool, channel_id, profile.id).await?;

	for role in role_permissions {
		permissions |= role.permissions;
	}

	Ok(permissions)
}
