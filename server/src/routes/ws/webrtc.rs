use super::*;
use crate::*;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebRTCEvent {
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
		sdp_mid: Option<String>,
		sdp_mline_index: Option<u16>,
		username_fragment: Option<String>,
	},
	Rollback,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SendEventParams {
	pub socket_id: uuid::Uuid,
	pub event: WebRTCEvent,
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

	app.room(params.socket_id).emit("webrtcEvent", (socket.id(), &params.event))?;

	Ok(())
}
