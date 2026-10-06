use super::*;
use crate::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateRoleParams {
	pub name: String,
	pub permissions: permissions::GlobalPermissions,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeleteRoleParams {
	pub role_id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Role {
	pub id: uuid::Uuid,
	pub name: String,
	pub permissions: permissions::GlobalPermissions,
}

pub async fn create_role(app: wspc::App, socket: wspc::Socket, params: wspc::Params<CreateRoleParams>) -> error::Result<Role> {
	let state = app.get_state::<app::AppState>().unwrap();
	let permissions = auth::get_global_permissions(&state, &socket).await?;

	if !permissions.contains(permissions::GlobalPermissions::MANAGE_ROLES) {
		return Err(error::Error::Unauthorized);
	}

	let permissions = params.permissions & permissions;

	let role = db::create_role(&state.db_pool, &params.name, permissions).await?;

	Ok(role.into())
}

pub async fn delete_role(app: wspc::App, socket: wspc::Socket, params: wspc::Params<DeleteRoleParams>) -> error::Result<Role> {
	let state = app.get_state::<app::AppState>().unwrap();
	let permissions = auth::get_global_permissions(&state, &socket).await?;

	if !permissions.contains(permissions::GlobalPermissions::MANAGE_ROLES) {
		return Err(error::Error::Unauthorized);
	}

	let role = db::delete_role(&state.db_pool, params.role_id).await?;

	Ok(role.into())
}

pub async fn list_roles(app: wspc::App) -> error::Result<Vec<role::Role>> {
	let state = app.get_state::<app::AppState>().unwrap();

	let roles = db::get_roles(&state.db_pool).await?;

	Ok(roles.into_iter().map(Into::into).collect())
}

impl From<db::Role> for Role {
	#[inline(always)]
	fn from(role: db::Role) -> Self {
		Self {
			id: role.id,
			name: role.name,
			permissions: role.permissions,
		}
	}
}
