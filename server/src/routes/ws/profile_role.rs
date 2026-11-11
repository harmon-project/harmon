use super::*;
use crate::*;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateProfileRoleParams {
	pub public_key: crypto::PublicKey,
	pub role_id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DeleteProfileRoleParams {
	pub profile_role_id: uuid::Uuid,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ListProfileRolesParams {
	pub public_key: crypto::PublicKey,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileRole {
	pub id: uuid::Uuid,
	pub profile_id: uuid::Uuid,
	pub role_id: uuid::Uuid,
}

pub async fn create_profile_role(app: wspc::App, socket: wspc::Socket, params: wspc::Params<CreateProfileRoleParams>) -> error::Result<ProfileRole> {
	let state = app.get_state::<app::AppState>().unwrap();
	let permissions = auth::get_global_permissions(&state, &socket).await?;

	if !permissions.contains(permissions::GlobalPermissions::MANAGE_ROLES) {
		return Err(error::Error::Unauthorized);
	}

	let Some(profile) = db::get_profile_by_public_key(&state.db_pool, params.public_key).await? else {
		return Err(error::Error::ProfileDoesNotExist);
	};

	let role = db::get_role(&state.db_pool, params.role_id).await?;

	if !permissions.contains(role.permissions) {
		return Err(error::Error::Unauthorized);
	}

	let assignment = db::insert_profile_role(&state.db_pool, profile.id, role.id).await?;

	Ok(assignment.into())
}

pub async fn delete_profile_role(app: wspc::App, socket: wspc::Socket, params: wspc::Params<DeleteProfileRoleParams>) -> error::Result<ProfileRole> {
	let state = app.get_state::<app::AppState>().unwrap();
	let permissions = auth::get_global_permissions(&state, &socket).await?;

	if !permissions.contains(permissions::GlobalPermissions::MANAGE_ROLES) {
		return Err(error::Error::Unauthorized);
	}

	let assignment = db::delete_profile_role(&state.db_pool, params.profile_role_id).await?;

	Ok(assignment.into())
}

pub async fn list_profile_roles(app: wspc::App, socket: wspc::Socket, params: wspc::Params<ListProfileRolesParams>) -> error::Result<Vec<ProfileRole>> {
	let state = app.get_state::<app::AppState>().unwrap();

	if auth::is_auth(&socket) {
		return Err(error::Error::Unauthorized);
	}

	let Some(profile) = db::get_profile_by_public_key(&state.db_pool, params.public_key).await? else {
		return Err(error::Error::ProfileDoesNotExist);
	};

	let profile_roles = db::get_profile_roles(&state.db_pool, profile.id).await?;

	Ok(profile_roles.into_iter().map(Into::into).collect())
}

impl From<db::ProfileRole> for ProfileRole {
	#[inline(always)]
	fn from(value: db::ProfileRole) -> Self {
		Self {
			id: value.id,
			profile_id: value.profile_id,
			role_id: value.role_id,
		}
	}
}
