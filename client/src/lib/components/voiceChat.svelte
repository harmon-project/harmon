<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import type { Client, WebRTCEvent } from "harmon-lib";
	import { push } from "./toast.svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { error, info } from "$lib/log";

	const { client }: { client: Client } = $props();

	let audioStream: MediaStream | undefined;
	let audioContext = new AudioContext();
	let peers = new SvelteMap<string, RTCPeerConnection>();
	let streams = new SvelteMap<string, MediaStream>();
	let sources = new SvelteMap<string, MediaStreamAudioSourceNode>();
	let pendingIceCandidates = new SvelteMap<string, RTCIceCandidateInit[]>();

	let audioReady = $state(audioContext.state == "running");

	const members = $derived(client.currentChannel?.members ?? []);
	const socketId = $derived(client.id!);

	function audioSink(node: HTMLAudioElement, stream?: MediaStream) {
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
			error("Failed to acquire local media stream: ", err);
			push("Error acquiring local media stream.");
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

	async function onWebRTCEvent(socketId: string, event: WebRTCEvent) {
		const peer = peers.get(socketId);

		if (!peer) return;

		info(`Received WebRTC event from "${socketId}": `, event);

		switch (event.type) {
			case "offer": {
				await peer.setRemoteDescription(event);

				for (const candidate of pendingIceCandidates.get(socketId) ?? []) {
					await peer.addIceCandidate(candidate);
				}
				pendingIceCandidates.delete(socketId);

				const answer = await peer.createAnswer();

				await peer.setLocalDescription(answer);
				await client.sendWebRtcEvent(socketId, answer);
				break;
			}
			case "answer": {
				await peer.setRemoteDescription(event);

				for (const candidate of pendingIceCandidates.get(socketId) ?? []) {
					await peer.addIceCandidate(candidate);
				}
				pendingIceCandidates.delete(socketId);

				break;
			}
			case "candidate": {
				if (peer.remoteDescription) {
					await peer.addIceCandidate(event);
				} else {
					const candidates = pendingIceCandidates.get(socketId) ?? [];
					pendingIceCandidates.set(socketId, [...candidates, event]);
				}
				break;
			}
		}
	}

	async function createOffer(socketId: string) {
		const peer = peers.get(socketId)!;

		info(`Creating offer for ${socketId}`);
		const offer = await peer.createOffer();

		info(`Setting local description for ${socketId}: `, offer);
		await peer.setLocalDescription(offer);

		info(`Sending offer to ${socketId}: `, offer);
		await client.sendWebRtcEvent(socketId, offer);
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

				sources.set(member.socket_id, source);
			};

			peer.onnegotiationneeded = async () => {
				info(`Negotiation needed for ${member.socket_id}`);
				if (socketId > member.socket_id && peer.signalingState == "stable") {
					await createOffer(member.socket_id);
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
					info(`Adding local track to peer for ${member.socket_id}: `, track);
					peer.addTrack(track, audioStream);
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
				pendingIceCandidates.delete(memberId);
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

		peers.clear();
		streams.clear();
		sources.clear();
		audioContext.close();
		pendingIceCandidates.clear();
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
		{#each members as member}
			<div class="rounded-md border border-gray-700 bg-gray-800 p-3">
				<div class="mb-2 flex items-center justify-between gap-2">
					<span class="font-medium">{member.profile.name}</span>
					<span class="text-xs text-gray-400">{member.socket_id}</span>
				</div>
				{#if member.socket_id !== client.id}
					<audio autoplay playsinline muted use:audioSink={streams.get(member.socket_id)}
					></audio>
				{/if}
			</div>
		{/each}
	</div>
</div>
