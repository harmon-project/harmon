<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import type { Client } from "harmon-lib";
	import { push } from "./toast.svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { error, info } from "$lib/log";
	import { RTCPeer, type WebRTCEvent } from "harmon-lib/webrtc";

	const { client }: { client: Client } = $props();

	let audioStream: MediaStream | undefined = $state();
	let screenStream: MediaStream | undefined = $state();
	let audioContext = new AudioContext();
	let peers = new SvelteMap<string, RTCPeer>();

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

		info(`Received WebRTC event from "${peerId}": `, event);

		switch (event.type) {
			case "offer": {
				const polite = localSocketId > peerId;
				const offerCollision =
					peer.makingOffer || peer.connection.signalingState !== "stable";

				peer.ignoreOffer = !polite && offerCollision;

				if (peer.ignoreOffer) return;

				await peer.connection.setRemoteDescription(event);

				const answer = await peer.connection.createAnswer();

				await peer.connection.setLocalDescription(answer);
				await client.sendWebRTCEvent(peerId, answer);

				break;
			}
			case "answer": {
				await peer.connection.setRemoteDescription(event);
				break;
			}
			case "candidate": {
				if (peer.ignoreOffer) {
					break;
				}

				await peer.connection.addIceCandidate(event);
				break;
			}
		}
	}

	async function createOffer(peerId: string) {
		const peer = peers.get(peerId)!;

		peer.makingOffer = true;

		try {
			info(`Creating offer for ${peerId}`);
			const offer = await peer.connection.createOffer();

			info(`Setting local description for ${peerId}: `, offer);
			await peer.connection.setLocalDescription(offer);

			info(`Sending offer to ${peerId}: `, offer);
			await client.sendWebRTCEvent(peerId, offer);
		} finally {
			peer.makingOffer = false;
		}
	}

	async function addPeer(socketId: string) {
		const peer = new RTCPeer(client.iceServers, localSocketId > socketId);

		peer.connection.onconnectionstatechange = () => {
			info(`Connection state ${socketId}:`, peer.connection.connectionState);
		};

		peer.connection.oniceconnectionstatechange = () => {
			info(`ICE state ${socketId}:`, peer.connection.iceConnectionState);
		};

		peer.connection.onsignalingstatechange = () => {
			info(`Signaling state ${socketId}:`, peer.connection.signalingState);
		};

		peer.connection.ontrack = (event) => {
			info(`Received track from ${socketId}: `, event);

			event.track.onmute = () => {
				info(`Remote track MUTED from ${socketId}`);
			};

			event.track.onunmute = () => {
				info(`Remote track UNMUTED from ${socketId}`);
			};

			event.track.onended = () => {
				info(`Remote track ENDED from ${socketId}`);
			};

			const stream = event.streams[0] ?? new MediaStream([event.track]);

			if (stream.getVideoTracks().length > 0) {
				peer.videoStream = stream;
			} else {
				peer.audioSource?.disconnect();

				const audioSource = audioContext.createMediaStreamSource(stream);
				audioSource.connect(audioContext.destination);

				peer.audioSource = audioSource;
				peer.audioStream = stream;
			}
		};

		peer.connection.onnegotiationneeded = async () => {
			info(`Negotiation needed for ${socketId}`);
			if (peer.connection.signalingState !== "stable" || peer.makingOffer) {
				return;
			}
			await createOffer(socketId);
		};

		peer.connection.onicecandidate = async (event) => {
			if (!event.candidate) return;

			const candidate: WebRTCEvent = {
				type: "candidate",
				candidate: event.candidate.candidate,
				sdpMid: event.candidate.sdpMid,
				sdpMLineIndex: event.candidate.sdpMLineIndex,
				usernameFragment: event.candidate.usernameFragment
			};
			info(`Sending ICE candidate to ${socketId}: `, candidate);

			await client.sendWebRTCEvent(socketId, candidate);
		};

		peer.connection.onicecandidateerror = (event) => {
			error(`ICE candidate error for ${socketId}: `, event);
		};

		if (audioStream) {
			for (const track of audioStream.getTracks()) {
				info(`Adding audio track to peer for ${socketId}: `, track);
				peer.connection.addTrack(track, audioStream);
			}
		}

		if (screenStream) {
			for (const track of screenStream.getTracks()) {
				info(`Adding screen track to peer for ${socketId}: `, track);
				peer.connection.addTrack(track, screenStream);
			}
		}

		peers.set(socketId, peer);
	}

	async function removePeer(socketId: string) {
		const peer = peers.get(socketId);
		if (!peer) return;

		peer.connection.close();
		peer.audioSource?.disconnect();

		peers.delete(socketId);
	}

	async function syncPeers() {
		for (const member of members) {
			if (!peers.has(member.socket_id) && member.socket_id !== localSocketId) {
				await addPeer(member.socket_id);
			}
		}

		for (const [socketId, _] of peers) {
			if (!members.find((member) => member.socket_id === socketId)) {
				await removePeer(socketId);
			}
		}
	}

	export async function startAudioStream() {
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

	export async function startScreenStream() {
		try {
			screenStream = await navigator.mediaDevices.getDisplayMedia({
				video: true,
				audio: true
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

	export async function stopScreenStream() {
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
		await startAudioStream();
	});

	onDestroy(() => {
		audioContext.onstatechange = null;

		client.onWebRTCEvent = undefined;
		client.onChannelMemberJoined = undefined;
		client.onChannelMemberLeft = undefined;

		for (const member of members) {
			removePeer(member.socket_id);
		}

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
	<video class="min-h-0 w-full flex-1 bg-black" autoplay playsinline use:sink={stream}></video>
{/snippet}

{#snippet audio(stream: MediaStream)}
	<audio class="hidden" autoplay playsinline muted use:sink={stream}></audio>
{/snippet}

<div class="flex h-full w-full flex-col bg-gray-900">
	<div
		class="grid min-h-0 flex-1 auto-rows-[minmax(14rem,1fr)] grid-cols-[repeat(auto-fit,minmax(16rem,1fr))] gap-3 overflow-x-hidden overflow-y-auto p-4"
	>
		{#each members as member}
			{@const peer = peers.get(member.socket_id)}
			<div class="flex min-h-0 w-full flex-col overflow-hidden rounded-2xl bg-gray-800">
				{#if member.socket_id === client.id}
					{#if screenStream}
						{@render video(screenStream)}
					{:else}
						{@render profile(member.profile.name)}
					{/if}
				{:else}
					{#if !!peer?.videoStream && peer?.videoStream.getVideoTracks().length > 0}
						{@render video(peer.videoStream)}
					{:else}
						{@render profile(member.profile.name)}
					{/if}
					{#if peer?.audioStream}
						{@render audio(peer.audioStream)}
					{/if}
				{/if}
			</div>
		{/each}
	</div>
</div>
