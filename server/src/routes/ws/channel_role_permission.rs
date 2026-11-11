use super::*;
use crate::*;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateChannelRolePermissionParams {
	pub channel_id: uuid::Uuid,
	pub role_id: uuid::Uuid,
	pub permissions: permissions::ChannelPermissions,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DeleteChannelRolePermissionParams {
	pub id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ListChannelRolePermissionsParams {
	pub channel_id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ChannelRolePermission {
	pub id: uuid::Uuid,
	pub channel_id: uuid::Uuid,
	pub role_id: uuid::Uuid,
	pub permissions: permissions::ChannelPermissions,
}

pub async fn create_channel_role_permission(app: wspc::App, socket: wspc::Socket, params: wspc::Params<CreateChannelRolePermissionParams>) -> error::Result<ChannelRolePermission> {
	let state = app.get_state::<app::AppState>().unwrap();
	let global_permissions = auth::get_global_permissions(&state, &socket).await?;

	if !global_permissions.contains(permissions::GlobalPermissions::MANAGE_CHANNELS) {
		return Err(error::Error::Unauthorized);
	}

	let role = db::get_role(&state.db_pool, params.role_id).await?;
	let channel_permissions = auth::get_channel_permissions(&state, &socket, params.channel_id).await?;

	if !channel_permissions.contains(params.permissions) {
		return Err(error::Error::Unauthorized);
	}

	let channel_role_permission = db::update_or_insert_channel_role_permission(&state.db_pool, params.channel_id, role.id, params.permissions).await?;

	Ok(channel_role_permission.into())
}

pub async fn delete_channel_role_permission(app: wspc::App, socket: wspc::Socket, params: wspc::Params<DeleteChannelRolePermissionParams>) -> error::Result<ChannelRolePermission> {
	let state = app.get_state::<app::AppState>().unwrap();
	let permissions = auth::get_global_permissions(&state, &socket).await?;

	if !permissions.contains(permissions::GlobalPermissions::MANAGE_CHANNELS) {
		return Err(error::Error::Unauthorized);
	}

	let channel_role_permission = db::delete_channel_role_permission(&state.db_pool, params.id).await?;

	Ok(channel_role_permission.into())
}

pub async fn list_channel_role_permissions(app: wspc::App, socket: wspc::Socket, params: wspc::Params<ListChannelRolePermissionsParams>) -> error::Result<Vec<ChannelRolePermission>> {
	let state = app.get_state::<app::AppState>().unwrap();
	let permissions = auth::get_global_permissions(&state, &socket).await?;

	if !permissions.contains(permissions::GlobalPermissions::MANAGE_CHANNELS) {
		return Err(error::Error::Unauthorized);
	}

	db::get_channel(&state.db_pool, params.channel_id).await?;

	let channel_role_permissions = db::get_channel_role_permissions(&state.db_pool, params.channel_id).await?;

	Ok(channel_role_permissions.into_iter().map(Into::into).collect())
}

impl From<db::ChannelRolePermission> for ChannelRolePermission {
	#[inline(always)]
	fn from(value: db::ChannelRolePermission) -> Self {
		Self {
			id: value.id,
			channel_id: value.channel_id,
			role_id: value.role_id,
			permissions: value.permissions,
		}
	}
}
