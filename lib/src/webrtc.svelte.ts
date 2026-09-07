export type WebRTCEvent =
	| {
			type: "answer" | "offer" | "pranswer" | "rollback";
			sdp?: string;
	  }
	| {
			type: "candidate";
			candidate?: string;
			sdpMLineIndex?: number | null;
			sdpMid?: string | null;
			usernameFragment?: string | null;
	  };

export class RTCPeer {
	public connection: RTCPeerConnection;
	public makingOffer: boolean = $state(false);
	public ignoreOffer: boolean = $state(false);
	public isPolite: boolean = $state(false);

	public audioSource?: MediaStreamAudioSourceNode = $state();
	public audioStream?: MediaStream = $state();
	public videoStream?: MediaStream = $state();

	constructor(iceServers: RTCIceServer[], isPolite: boolean) {
		this.connection = new RTCPeerConnection({ iceServers });
		this.isPolite = isPolite;
	}
}
