use super::*;
use crate::*;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IceServer {
	pub urls: Vec<String>,
	pub username: Option<String>,
	pub credential: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebRtcEvent {
	Offer {
		sdp: String,
	},
	Answer {
		sdp: String,
	},
	Pranswer {
		sdp: String,
	},
	Candidate {
		candidate: String,
		#[serde(rename = "sdpMid")]
		sdp_mid: Option<String>,
		#[serde(rename = "sdpMLineIndex")]
		sdp_mline_index: Option<u16>,
		#[serde(rename = "usernameFragment")]
		username_fragment: Option<String>,
	},
	Rollback,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SendEventParams {
	pub socket_id: uuid::Uuid,
	pub event: WebRtcEvent,
}

pub async fn send_event(app: wspc::App, socket: wspc::Socket, params: wspc::Params<SendEventParams>) -> error::Result<()> {
	let state = app.get_state::<app::AppState>().unwrap();

	if !auth::is_auth(&socket) {
		return Err(error::Error::Unauthorized);
	}

	let Some(channel_id) = socket.get_state::<channel::ChannelIdentifier>() else {
		return Err(error::Error::NotInChannel);
	};

	let channel = db::get_channel(&state.db_pool, *channel_id).await?;

	if channel.r#type != db::ChannelType::Voice {
		return Err(error::Error::Unauthorized);
	}

	app.room(params.socket_id).emit("webRtcEvent", (socket.id(), &params.event))?;

	Ok(())
}

pub async fn get_ice_servers(app: wspc::App, socket: wspc::Socket) -> error::Result<Vec<IceServer>> {
	let state = app.get_state::<app::AppState>().unwrap();

	if !auth::is_auth(&socket) {
		return Err(error::Error::Unauthorized);
	}

	Ok(state.config.ice_servers.clone().into_iter().map(Into::into).collect())
}

impl From<config::IceServer> for IceServer {
	#[inline(always)]
	fn from(ice_server: config::IceServer) -> Self {
		Self {
			urls: ice_server.urls,
			username: ice_server.username,
			credential: ice_server.credential,
		}
	}
}
