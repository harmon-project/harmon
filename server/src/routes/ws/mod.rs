mod auth;
mod channel;
mod channel_role_permission;
mod life_cycle;
mod message;
mod profile;
mod profile_role;
mod role;
mod webrtc;

use crate::*;

use axum::routing::*;

pub fn router(state: &app::AppState) -> MethodRouter<app::AppState> {
	let (route, app) = wspc::App::build_route();

	app.set_state(state.clone());

	app.on("connect", life_cycle::connect);
	app.on("disconnect", life_cycle::disconnect);

	app.on("auth", auth::auth);
	app.on("requestChallenge", auth::request_challenge);
	app.on("confirmChallenge", auth::confirm_challenge);

	app.on("sendMessage", message::send_message);
	app.on("deleteMessage", message::delete_message);
	app.on("loadMessages", message::load_messages);

	app.on("joinChannel", channel::join_channel);
	app.on("createChannel", channel::create_channel);
	app.on("deleteChannel", channel::delete_channel);
	app.on("listChannels", channel::list_channels);

	app.on("getProfile", profile::get_profile);
	app.on("updateProfile", profile::update_profile);

	app.on("sendWebRTCEvent", webrtc::send_event);
	app.on("getIceServers", webrtc::get_ice_servers);

	app.on("createRole", role::create_role);
	app.on("deleteRole", role::delete_role);
	app.on("listRoles", role::list_roles);

	app.on("createProfileRole", profile_role::create_profile_role);
	app.on("deleteProfileRole", profile_role::delete_profile_role);
	app.on("listProfileRoles", profile_role::list_profile_roles);

	app.on("createChannelRolePermission", channel_role_permission::create_channel_role_permission);
	app.on("deleteChannelRolePermission", channel_role_permission::delete_channel_role_permission);
	app.on("listChannelRolePermissions", channel_role_permission::list_channel_role_permissions);

	route
}
