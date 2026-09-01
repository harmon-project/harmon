<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import type { Client, WebRTCEvent } from "harmon-lib";
	import { push } from "./toast.svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { error, info } from "$lib/log";
	import Fa from "svelte-fa";
	import { faDesktop, faStop } from "@fortawesome/free-solid-svg-icons";

	const { client }: { client: Client } = $props();

	let audioStream: MediaStream | undefined = $state();
	let screenStream: MediaStream | undefined = $state();
	let audioContext = new AudioContext();
	let peers = new SvelteMap<string, RTCPeerConnection>();
	let streams = new SvelteMap<string, MediaStream>();
	let screenStreams = new SvelteMap<string, MediaStream>();
	let sources = new SvelteMap<string, MediaStreamAudioSourceNode>();
	let pendingIceCandidates = new SvelteMap<string, RTCIceCandidateInit[]>();

	// Per-peer state for "perfect negotiation". Both flags are keyed by the
	// remote socket id so the shared `onWebRTCEvent` handler can read them.
	let makingOffer = new Map<string, boolean>();
	let ignoreOffer = new Map<string, boolean>();

	let audioReady = $state(audioContext.state == "running");

	const members = $derived(client.currentChannel?.members ?? []);
	const socketId = $derived(client.id!);

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
					const hasSender = peer
						.getSenders()
						.some((sender) => sender.track?.id === track.id);
					if (!hasSender) {
						peer.addTrack(track, audioStream);
					}
				}
			}
		} catch (err) {
			error("Failed to acquire audio media stream: ", err);
			push("Error acquiring audio media stream.");
		}
	}

	async function getScreenStream() {
		try {
			screenStream = await navigator.mediaDevices.getDisplayMedia({
				video: true,
				audio: false
			});

			// Stop sharing when the user ends it through the browser's own
			// "Stop sharing" control instead of our button.
			for (const track of screenStream.getTracks()) {
				track.onended = () => stopScreenStream();
			}

			for (const [_, peer] of peers) {
				for (const track of screenStream.getTracks()) {
					const hasSender = peer
						.getSenders()
						.some((sender) => sender.track?.id === track.id);
					if (!hasSender) {
						peer.addTrack(track, screenStream);
					}
				}
			}
		} catch (err) {
			error("Failed to acquire screen media stream: ", err);
			push("Error acquiring screen media stream.");
		}
	}

	function stopScreenStream() {
		if (!screenStream) return;

		const tracks = screenStream.getTracks();

		for (const [_, peer] of peers) {
			for (const sender of peer.getSenders()) {
				if (sender.track && tracks.includes(sender.track)) {
					peer.removeTrack(sender);
				}
			}
		}

		for (const track of tracks) {
			track.onended = null;
			track.stop();
		}

		screenStream = undefined;
	}

	async function toggleScreenStream() {
		if (screenStream) {
			stopScreenStream();
		} else {
			await getScreenStream();
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

	async function onWebRTCEvent(remoteId: string, event: WebRTCEvent) {
		const peer = peers.get(remoteId);

		if (!peer) return;

		info(`Received WebRTC event from "${remoteId}": `, event);

		// The peer with the lower socket id is "polite": on a negotiation
		// collision it yields, letting the other side's offer win.
		const polite = socketId < remoteId;

		try {
			switch (event.type) {
				case "offer":
				case "answer": {
					const collision =
						event.type === "offer" &&
						(makingOffer.get(remoteId) || peer.signalingState !== "stable");

					ignoreOffer.set(remoteId, !polite && collision);
					if (ignoreOffer.get(remoteId)) return;

					await peer.setRemoteDescription(event);

					for (const candidate of pendingIceCandidates.get(remoteId) ?? []) {
						await peer.addIceCandidate(candidate);
					}
					pendingIceCandidates.delete(remoteId);

					if (event.type === "offer") {
						// Argument-less setLocalDescription() creates and applies
						// the answer in a single step, so the signaling state can't
						// change between building and setting it.
						await peer.setLocalDescription();
						if (peer.localDescription) {
							await client.sendWebRtcEvent(remoteId, {
								type: peer.localDescription.type,
								sdp: peer.localDescription.sdp
							});
						}
					}
					break;
				}
				case "candidate": {
					if (peer.remoteDescription) {
						try {
							await peer.addIceCandidate(event);
						} catch (err) {
							if (!ignoreOffer.get(remoteId)) throw err;
						}
					} else {
						const candidates = pendingIceCandidates.get(remoteId) ?? [];
						pendingIceCandidates.set(remoteId, [...candidates, event]);
					}
					break;
				}
			}
		} catch (err) {
			error(`Failed to handle WebRTC event from ${remoteId}: `, err);
		}
	}

	async function syncPeers() {
		for (const member of members) {
			if (member.socket_id === socketId || peers.has(member.socket_id)) continue;

			const peer = new RTCPeerConnection({ iceServers: client.iceServers });

			peer.onconnectionstatechange = () => {
				info(`Connection state ${member.socket_id}:`, peer.connectionState);
			};

			peer.oniceconnectionstatechange = () => {
				info(`ICE state ${member.socket_id}:`, peer.iceConnectionState);
			};

			peer.onsignalingstatechange = () => {
				info(`Signaling state ${member.socket_id}:`, peer.signalingState);
			};

			peer.ontrack = (event) => {
				info(`Received track from ${member.socket_id}: `, event);
				const stream = event.streams[0] ?? new MediaStream([event.track]);

				if (event.track.kind === "video") {
					// A video track is a screen share; render it instead of
					// routing it through the audio graph.
					screenStreams.set(member.socket_id, stream);

					event.track.onmute = () => {
						info(`Remote screen track MUTED from ${member.socket_id}`);
						screenStreams.delete(member.socket_id);
					};

					event.track.onunmute = () => {
						info(`Remote screen track UNMUTED from ${member.socket_id}`);
						screenStreams.set(member.socket_id, stream);
					};

					event.track.onended = () => {
						info(`Remote screen track ENDED from ${member.socket_id}`);
						screenStreams.delete(member.socket_id);
					};

					stream.onremovetrack = () => {
						if (stream.getVideoTracks().length === 0) {
							screenStreams.delete(member.socket_id);
						}
					};

					return;
				}

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

				sources.get(member.socket_id)?.disconnect();
				const source = audioContext.createMediaStreamSource(stream);
				source.connect(audioContext.destination);

				sources.set(member.socket_id, source);
			};

			peer.onnegotiationneeded = async () => {
				info(`Negotiation needed for ${member.socket_id}`);
				try {
					makingOffer.set(member.socket_id, true);
					// Argument-less setLocalDescription() atomically creates the
					// offer for the current signaling state and applies it. The
					// browser drives the negotiation order; we never build an SDP
					// by hand across an await, which is what breaks glare handling.
					await peer.setLocalDescription();
					if (peer.localDescription) {
						await client.sendWebRtcEvent(member.socket_id, {
							type: peer.localDescription.type,
							sdp: peer.localDescription.sdp
						});
					}
				} catch (err) {
					error(`Negotiation failed for ${member.socket_id}: `, err);
				} finally {
					makingOffer.set(member.socket_id, false);
				}
			};

			peer.onicecandidate = async (event) => {
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

			peer.onicecandidateerror = (event) => {
				error(`ICE candidate error for ${member.socket_id}: `, event);
			};

			if (audioStream) {
				for (const track of audioStream.getTracks()) {
					info(`Adding audio track to peer for ${member.socket_id}: `, track);
					peer.addTrack(track, audioStream);
				}
			}

			if (screenStream) {
				for (const track of screenStream.getTracks()) {
					info(`Adding screen track to peer for ${member.socket_id}: `, track);
					peer.addTrack(track, screenStream);
				}
			}

			peers.set(member.socket_id, peer);
		}

		for (const [memberId, peer] of peers) {
			if (!members.find((member) => member.socket_id === memberId)) {
				peer.close();
				sources.get(memberId)?.disconnect();
				sources.delete(memberId);
				peers.delete(memberId);
				streams.delete(memberId);
				screenStreams.delete(memberId);
				pendingIceCandidates.delete(memberId);
				makingOffer.delete(memberId);
				ignoreOffer.delete(memberId);
			}
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
			peer.close();
		}
		for (const [_, source] of sources) {
			source.disconnect();
		}
		for (const track of audioStream?.getTracks() ?? []) {
			track.stop();
		}
		for (const track of screenStream?.getTracks() ?? []) {
			track.onended = null;
			track.stop();
		}

		peers.clear();
		streams.clear();
		screenStreams.clear();
		sources.clear();
		audioContext.close();
		pendingIceCandidates.clear();
		makingOffer.clear();
		ignoreOffer.clear();
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

<div class="flex h-full w-full flex-col bg-gray-900">
	<div class="flex flex-1 flex-col gap-3 overflow-y-auto p-4">
		{#if screenStream || screenStreams.size > 0}
			<div class="grid grid-cols-1 gap-3 lg:grid-cols-2">
				{#if screenStream}
					<div
						class="relative overflow-hidden rounded-md border border-gray-700 bg-black"
					>
						<!-- svelte-ignore a11y_media_has_caption -->
						<video
							autoplay
							playsinline
							muted
							use:sink={screenStream}
							class="h-full w-full object-contain"
						></video>
						<span
							class="absolute bottom-2 left-2 rounded bg-black/60 px-2 py-1 text-xs text-white"
						>
							You
						</span>
					</div>
				{/if}
				{#each members as member}
					{@const screen = screenStreams.get(member.socket_id)}
					{#if member.socket_id !== client.id && screen}
						<div
							class="relative overflow-hidden rounded-md border border-gray-700 bg-black"
						>
							<!-- svelte-ignore a11y_media_has_caption -->
							<video
								autoplay
								playsinline
								muted
								use:sink={screen}
								class="h-full w-full object-contain"
							></video>
							<span
								class="absolute bottom-2 left-2 rounded bg-black/60 px-2 py-1 text-xs text-white"
							>
								{member.profile.name}
							</span>
						</div>
					{/if}
				{/each}
			</div>
		{/if}

		{#each members as member}
			{@const stream = streams.get(member.socket_id)}
			<div class="rounded-md border border-gray-700 bg-gray-800 p-3">
				<div class="mb-2 flex items-center justify-between gap-2">
					<span class="font-medium">{member.profile.name}</span>
					<span class="text-xs text-gray-400">{member.socket_id}</span>
				</div>
				{#if member.socket_id !== client.id}
					<audio autoplay playsinline muted use:sink={stream}></audio>
				{/if}
			</div>
		{/each}
	</div>

	<div class="flex items-center justify-center gap-3 border-t border-gray-700 bg-gray-800 p-3">
		<button
			onclick={toggleScreenStream}
			class="flex cursor-pointer items-center gap-2 rounded-md px-4 py-2 text-white {screenStream
				? 'bg-red-600 hover:bg-red-700'
				: 'bg-gray-700 hover:bg-gray-600'}"
		>
			<Fa icon={screenStream ? faStop : faDesktop} />
			{screenStream ? "Stop sharing" : "Share screen"}
		</button>
	</div>
</div>
