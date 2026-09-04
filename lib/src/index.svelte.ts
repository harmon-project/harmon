import { uint8ArrayToZ32, z32toUint8Array } from "./utils.js";
import { Client as JsonRPCClient } from "./jsonrpc.js";
import { getInfo } from "./http.js";
import { DNSClient } from "./pkdns.js";

type Session = {
	publicKey: Uint8Array;
	isAdmin: boolean;
	authToken: string;
	currentChannel?: CurrentChannel;
	channelList: Channel[];
	profile?: Profile;
	iceServers: RTCIceServer[];
};

type ConnectionData = {
	id: string;
};

export class Client {
	private _url = $state("");
	private _rpc: JsonRPCClient<ClientToServerEvents, ServerToClientEvents>;
	private _session = $state<Session | undefined>();
	private _serverInfo = $state<ServerInfo | undefined>();
	private _connectionData = $state<ConnectionData | undefined>();

	public onConnectionReady?: () => void;
	public onConnectionClosed?: () => void;
	public onMessageReceived?: (message: Message) => void;
	public onChannelMemberJoined?: (member: ChannelMember) => void;
	public onChannelMemberLeft?: (member: ChannelMember) => void;
	public onWebRTCEvent?: (socketId: string, event: WebRTCEvent) => void;

	public close: typeof JsonRPCClient.prototype.close;

	static async init(publicKey: Uint8Array) {
		const dns = new DNSClient();
		const z32PublicKey = uint8ArrayToZ32(publicKey);
		const url = await dns.resolveUrl(z32PublicKey);

		if (!url) {
			throw new Error("Could not resolve URL for the given public key");
		}

		return new Client(url);
	}

	private constructor(url: string) {
		this._rpc = new JsonRPCClient(`${url}/ws`);
		this._url = url;

		this.close = this._rpc.close.bind(this._rpc);

		this._rpc.onOpen = async () => {
			this._serverInfo = await this.getInfo();
		};
		this._rpc.onClose = () => {
			this._session = undefined;
			this._serverInfo = undefined;
			this._connectionData = undefined;

			this.onConnectionClosed?.();
		};

		this._rpc.on("connectionReady", (id) => {
			this._connectionData = { id };
			this.onConnectionReady?.();
		});

		this._rpc.on("messageReceived", (message) => {
			this.onMessageReceived?.(message);
		});

		this._rpc.on("channelMemberJoined", (member) => {
			this._session!.currentChannel?.members.push(member);

			this.onChannelMemberJoined?.(member);
		});

		this._rpc.on("channelMemberLeft", (member) => {
			this._session!.currentChannel!.members = this._session!.currentChannel!.members.filter(
				(m) => m.socket_id !== member.socket_id
			);
			this.onChannelMemberLeft?.(member);
		});

		this._rpc.on("webRTCEvent", (socketId, event) => {
			this.onWebRTCEvent?.(socketId, event);
		});
	}

	async getInfo() {
		return await getInfo(this._url);
	}

	async requestChallenge(publicKey: Uint8Array) {
		return await this._rpc.call("requestChallenge", uint8ArrayToZ32(publicKey));
	}

	async confirmChallenge(token: string, signature: Uint8Array) {
		return await this._rpc.call("confirmChallenge", token, uint8ArrayToZ32(signature));
	}

	async auth(token: string) {
		const payload = await this._rpc.call("auth", token);

		let [profile, channelList, iceServers] = await Promise.all([
			this.getProfile(payload.public_key),
			this.listChannels(),
			this.getIceServers()
		]);

		this._session = {
			publicKey: z32toUint8Array(payload.public_key),
			isAdmin: payload.is_admin,
			authToken: token,
			profile,
			channelList,
			iceServers
		};

		return payload;
	}

	async joinChannel(id: string) {
		if (!this._session) {
			throw new Error("Not authenticated");
		}

		const channel = await this._rpc.call("joinChannel", id);

		this._session.currentChannel = channel;

		return channel;
	}

	async loadMessages(beforeId?: string) {
		return await this._rpc.call("loadMessages", beforeId);
	}

	async sendMessage(message: string, attachments: string[]) {
		return await this._rpc.call("sendMessage", message, attachments);
	}

	async createChannel(name: string) {
		return await this._rpc.call("createChannel", name);
	}

	async deleteChannel(channelId: string) {
		return await this._rpc.call("deleteChannel", channelId);
	}

	async listChannels() {
		return await this._rpc.call("listChannels");
	}

	async getProfile(public_key?: string) {
		return await this._rpc.call("getProfile", public_key);
	}

	async updateProfile(name: string) {
		const profile = await this._rpc.call("updateProfile", name);
		this._session!.profile = profile;

		return profile;
	}

	async sendWebRTCEvent(socket_id: string, event: WebRTCEvent) {
		await this._rpc.call("sendWebRTCEvent", socket_id, event);
	}

	async getIceServers() {
		return await this._rpc.call("getIceServers");
	}

	get url() {
		return this._url;
	}

	get isAuth() {
		return !!this._session;
	}

	get isAdmin() {
		return this._session?.isAdmin ?? false;
	}

	get publicKey() {
		return this._session?.publicKey;
	}

	get channelList() {
		return this._session?.channelList ?? [];
	}

	get currentChannel() {
		return this._session?.currentChannel;
	}

	get profile() {
		return this._session?.profile;
	}

	get serverInfo() {
		return this._serverInfo;
	}

	get isReady() {
		return !!this._connectionData;
	}

	get id() {
		return this._connectionData?.id;
	}

	get iceServers() {
		return this._session?.iceServers ?? [];
	}
}

export type ChannelType = "Text" | "Voice";

export interface Channel {
	id: string;
	name: string;
	type: ChannelType;
}

export interface Message {
	id: string;
	profile: Profile;
	content: string;
	attachments: MessageAttachment[];
	created_at: string;
}

export interface MessageAttachment {
	id: string;
	name: string;
	mime_type: string;
	size: number;
	hash: string;
}

export interface ResponseAuthChallenge {
	token: string;
}

export interface ResponseConfirmAuthChallenge {
	token: string;
	payload: AuthenticatedPayload;
}

export interface AuthenticatedPayload {
	public_key: string;
	is_admin: boolean;
	exp: number;
}

export interface Channel {
	id: string;
	name: string;
}

export interface Profile {
	name: string;
	public_key: string;
}

export interface ServerInfo {
	title: string;
	public_key: string;
}

export interface ChannelMember {
	profile: Profile;
	socket_id: string;
}

export interface CurrentChannel {
	channel: Channel;
	members: ChannelMember[];
}
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

interface ServerToClientEvents {
	connectionReady(id: string): void;

	messageReceived(message: Message): void;

	channelMemberJoined: (member: ChannelMember) => void;
	channelMemberLeft: (member: ChannelMember) => void;
	channelDeleted(channel: Channel): void;

	webRTCEvent(socket_id: string, event: WebRTCEvent): void;
}

interface ClientToServerEvents {
	auth(token: string): AuthenticatedPayload;
	requestChallenge(publicKey: string): ResponseAuthChallenge;
	confirmChallenge(token: string, signature: string): ResponseConfirmAuthChallenge;

	joinChannel(channelId: string): CurrentChannel;
	sendMessage(message: string, attachments: string[]): void;
	loadMessages(beforeId?: string): Message[];

	createChannel(name: string): Channel;
	deleteChannel(channelId: string): Channel;
	listChannels(): Channel[];

	updateProfile(name: string): Profile;
	getProfile(public_key?: string): Profile | undefined;

	sendWebRTCEvent(socketId: string, event: WebRTCEvent): void;
	getIceServers(): RTCIceServer[];
}
