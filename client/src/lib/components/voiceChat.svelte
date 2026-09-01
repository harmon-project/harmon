<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import type { Client, WebRTCEvent } from "harmon-lib";
	import { push } from "./toast.svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { error, info } from "$lib/log";

	const { client }: { client: Client } = $props();

	interface Peer {
		connection: RTCPeerConnection;
		source: MediaStreamAudioSourceNode | undefined;
		pendingIceCandidates: RTCIceCandidateInit[];
		makingOffer: boolean;
		ignoreOffer: boolean;
	}

	let audioStream: MediaStream | undefined = $state();
	let screenStream: MediaStream | undefined = $state();
	let audioContext = new AudioContext();
	let peers = new SvelteMap<string, Peer>();
	let streams = new SvelteMap<string, MediaStream>();

	let audioReady = $state(audioContext.state == "running");

	const members = $derived(client.currentChannel?.members ?? []);
	const localSocketId = $derived(client.id!);

	function sink(node: HTMLAudioElement | HTMLVideoElement, stream?: MediaStream) {
		const update = (next?: MediaStream) => {
			if (node.srcObject != next) {
				node.srcObject = next ?? null;
			}
			node.play().catch(() => {});
		};

		const destroy = () => {
			node.srcObject = null;
		};

		update(stream);

		return {
			update,
			destroy
		};
	}

	async function getAudioStream() {
		try {
			audioStream = await navigator.mediaDevices.getUserMedia({
				audio: { noiseSuppression: true, echoCancellation: true }
			});

			for (const [_, peer] of peers) {
				for (const track of audioStream.getTracks()) {
					const hasSender = peer.connection
						.getSenders()
						.some((sender) => sender.track?.id === track.id);
					if (!hasSender) {
						peer.connection.addTrack(track, audioStream);
					}
				}
			}
		} catch (err) {
			error("Failed to acquire audio media stream: ", err);
			push("Error acquiring audio media stream.");
		}
	}

	async function resumeAudio() {
		try {
			await audioContext.resume();
		} catch (err) {
			push("Error resuming audio. Check your browser permissions.");
		}
	}

	function onAudioStateChange() {
		audioReady = audioContext.state == "running";
	}

	async function onWebRTCEvent(peerId: string, event: WebRTCEvent) {
		const peer = peers.get(peerId);

		if (!peer) return;

		const polite = localSocketId > peerId;
		info(`Received WebRTC event from "${peerId}": `, event);

		switch (event.type) {
			case "offer": {
				const readyForOffer =
					!peer.makingOffer && peer.connection.signalingState === "stable";
				const offerCollision = !readyForOffer;

				if (offerCollision && !polite) {
					peer.ignoreOffer = true;
					return;
				}

				peer.ignoreOffer = false;

				await peer.connection.setRemoteDescription(event);

				for (const candidate of peer.pendingIceCandidates) {
					await peer.connection.addIceCandidate(candidate);
				}
				peer.pendingIceCandidates = [];

				const answer = await peer.connection.createAnswer();

				await peer.connection.setLocalDescription(answer);
				await client.sendWebRtcEvent(peerId, answer);

				break;
			}
			case "answer": {
				await peer.connection.setRemoteDescription(event);

				for (const candidate of peer.pendingIceCandidates) {
					await peer.connection.addIceCandidate(candidate);
				}

				peer.pendingIceCandidates = [];

				break;
			}
			case "candidate": {
				if (peer.ignoreOffer) {
					break;
				}

				if (peer.connection.remoteDescription) {
					await peer.connection.addIceCandidate(event);
				} else {
					peer.pendingIceCandidates = [...peer.pendingIceCandidates, event];
				}

				break;
			}
		}
	}

	async function createOffer(peerId: string) {
		const peer = peers.get(peerId)!;

		if (peer.makingOffer || peer.connection.signalingState !== "stable") return;

		peer.makingOffer = true;

		try {
			info(`Creating offer for ${peerId}`);
			const offer = await peer.connection.createOffer();

			info(`Setting local description for ${peerId}: `, offer);
			await peer.connection.setLocalDescription(offer);

			info(`Sending offer to ${peerId}: `, offer);
			await client.sendWebRtcEvent(peerId, offer);
		} finally {
			peer.makingOffer = false;
		}
	}

	async function syncPeers() {
		for (const member of members) {
			if (member.socket_id === localSocketId || peers.has(member.socket_id)) continue;

			const connection = new RTCPeerConnection({ iceServers: client.iceServers });
			const peer: Peer = {
				connection,
				source: undefined,
				pendingIceCandidates: [],
				makingOffer: false,
				ignoreOffer: false
			};

			connection.onconnectionstatechange = () => {
				info(`Connection state ${member.socket_id}:`, connection.connectionState);
			};

			connection.oniceconnectionstatechange = () => {
				info(`ICE state ${member.socket_id}:`, connection.iceConnectionState);
			};

			connection.onsignalingstatechange = () => {
				info(`Signaling state ${member.socket_id}:`, connection.signalingState);
			};

			connection.ontrack = (event) => {
				info(`Received track from ${member.socket_id}: `, event);
				const stream = event.streams[0] ?? new MediaStream([event.track]);

				event.track.onmute = () => {
					info(`Remote track MUTED from ${member.socket_id}`);
				};

				event.track.onunmute = () => {
					info(`Remote track UNMUTED from ${member.socket_id}`);
				};

				event.track.onended = () => {
					info(`Remote track ENDED from ${member.socket_id}`);
				};

				streams.set(member.socket_id, stream);

				const source = audioContext.createMediaStreamSource(stream);
				source.connect(audioContext.destination);

				peer.source = source;
			};

			connection.onnegotiationneeded = async () => {
				info(`Negotiation needed for ${member.socket_id}`);
				if (connection.signalingState !== "stable" || peer.makingOffer) {
					return;
				}
				await createOffer(member.socket_id);
			};

			connection.onicecandidate = async (event) => {
				if (!event.candidate) return;

				const candidate: WebRTCEvent = {
					type: "candidate",
					candidate: event.candidate.candidate,
					sdpMid: event.candidate.sdpMid,
					sdpMLineIndex: event.candidate.sdpMLineIndex,
					usernameFragment: event.candidate.usernameFragment
				};
				info(`Sending ICE candidate to ${member.socket_id}: `, candidate);

				await client.sendWebRtcEvent(member.socket_id, candidate);
			};

			connection.onicecandidateerror = (event) => {
				error(`ICE candidate error for ${member.socket_id}: `, event);
			};

			if (audioStream) {
				for (const track of audioStream.getTracks()) {
					info(`Adding audio track to peer for ${member.socket_id}: `, track);
					connection.addTrack(track, audioStream);
				}
			}

			if (screenStream) {
				for (const track of screenStream.getTracks()) {
					info(`Adding screen track to peer for ${member.socket_id}: `, track);
					connection.addTrack(track, screenStream);
				}
			}

			peers.set(member.socket_id, peer);
		}

		for (const [memberId, peer] of peers) {
			if (!members.find((member) => member.socket_id === memberId)) {
				peer.connection.close();
				peer.source?.disconnect();
				peers.delete(memberId);
				streams.delete(memberId);
			}
		}
	}

	export async function startScreenShare() {
		try {
			screenStream = await navigator.mediaDevices.getDisplayMedia({
				video: true,
				audio: false
			});

			for (const [_, peer] of peers) {
				for (const track of screenStream.getTracks()) {
					const hasSender = peer.connection
						.getSenders()
						.some((sender) => sender.track?.id === track.id);
					if (!hasSender) {
						peer.connection.addTrack(track, screenStream);
					}
				}
			}
		} catch (err) {
			error("Failed to acquire screen media stream: ", err);
			push("Error acquiring screen media stream.");
		} finally {
			return !!screenStream;
		}
	}

	export async function stopScreenShare() {
		try {
			for (const [_, peer] of peers) {
				for (const sender of peer.connection.getSenders()) {
					if (sender.track && sender.track.kind === "video") {
						sender.replaceTrack(null);
					}
				}
			}

			if (screenStream) {
				for (const track of screenStream.getTracks()) {
					track.stop();
				}

				screenStream = undefined;
			}
		} catch (err) {
			error("Failed to stop screen media stream: ", err);
			push("Error stopping screen media stream.");
		} finally {
			return !!screenStream;
		}
	}

	onMount(async () => {
		audioContext.onstatechange = onAudioStateChange;

		client.onWebRTCEvent = onWebRTCEvent;
		client.onChannelMemberLeft = syncPeers;
		client.onChannelMemberJoined = syncPeers;

		await syncPeers();
		await getAudioStream();
	});

	onDestroy(() => {
		audioContext.onstatechange = null;

		client.onWebRTCEvent = undefined;
		client.onChannelMemberJoined = undefined;
		client.onChannelMemberLeft = undefined;

		for (const [_, peer] of peers) {
			peer.connection.close();
			peer.source?.disconnect();
		}

		peers.clear();
		streams.clear();
		audioContext.close();
	});
</script>

{#if !audioReady}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		onclick={resumeAudio}
		class="fixed top-0 left-0 z-10 flex h-screen w-screen cursor-pointer items-center justify-center bg-gray-900"
	>
		<p class="text-white">Click to enable audio</p>
	</div>
{/if}

{#snippet profile(name: string)}
	<div class="flex flex-1 flex-col items-center justify-center gap-3 rounded-xl text-center">
		<div
			class="flex h-16 w-16 shrink-0 items-center justify-center rounded-full bg-blue-500 text-lg font-semibold"
		>
			{name[0]}
		</div>
		<span class="text-sm font-semibold text-gray-100">{name}</span>
	</div>
{/snippet}

{#snippet video(stream: MediaStream)}
	<video class="min-h-0 w-full flex-1 bg-black" autoplay playsinline muted use:sink={stream}
	></video>
{/snippet}

<div class="flex h-full w-full flex-col bg-gray-900">
	<div
		class="grid min-h-0 flex-1 auto-rows-[minmax(14rem,1fr)] grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-3 overflow-x-hidden overflow-y-auto p-4"
	>
		{#each members as member}
			{@const stream = streams.get(member.socket_id)}
			<div class="flex min-h-0 w-full flex-col overflow-hidden rounded-2xl bg-gray-800">
				{#if member.socket_id === client.id}
					{#if screenStream}
						{@render video(screenStream)}
					{:else}
						{@render profile(member.profile.name)}
					{/if}
				{:else}
					{#if !!stream && stream.getVideoTracks().length > 0}
						{@render video(stream)}
					{:else}
						{@render profile(member.profile.name)}
					{/if}
					<audio class="hidden" autoplay playsinline muted use:sink={stream}></audio>
				{/if}
			</div>
		{/each}
	</div>
</div>
