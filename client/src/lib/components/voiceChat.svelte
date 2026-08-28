<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import type { Client, WebRTCEvent } from "harmon-lib";
	import { push } from "./toast.svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { error, info } from "$lib/log";
	import iceServers from "$lib/assets/iceServers.json";

	const { client }: { client: Client } = $props();

	let audioContext = new AudioContext();
	let stream = new MediaStream();
	let streams = new SvelteMap<string, MediaStream>();
	let sources = new SvelteMap<string, MediaStreamAudioSourceNode>();
	let peers = new SvelteMap<string, RTCPeerConnection>();

	let audioReady = $state(audioContext.state == "running");

	const members = $derived(client.currentChannel?.members ?? []);
	const socketId = $derived(client.id!);

	async function getLocalStream() {
		try {
			stream = await navigator.mediaDevices.getUserMedia({
				audio: { noiseSuppression: true, echoCancellation: true }
			});

			for (const [_, peer] of peers) {
				for (const track of stream.getTracks()) {
					const hasSender = peer
						.getSenders()
						.some((sender) => sender.track?.id === track.id);
					if (!hasSender) {
						peer.addTrack(track, stream);
					}
				}
			}
		} catch (err) {
			push("Error accessing the microphone. Check your browser permissions.");
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

				const answer = await peer.createAnswer();

				await peer.setLocalDescription(answer);
				await client.sendWebRTCEvent(socketId, answer);
				break;
			}
			case "answer": {
				await peer.setRemoteDescription(event);
				break;
			}
			case "candidate": {
				await peer.addIceCandidate(event);
				break;
			}
		}
	}

	async function syncPeers() {
		for (const member of members) {
			if (!peers.has(member.socket_id) && member.socket_id != socketId) {
				const peer = new RTCPeerConnection({ iceServers });

				peer.onconnectionstatechange = () => {
					info(`Connection state "${member.socket_id}": `, peer.connectionState);
				};

				peer.oniceconnectionstatechange = () => {
					info(`ICE state "${member.socket_id}": `, peer.iceConnectionState);
				};

				peer.onsignalingstatechange = () => {
					info(`Signaling state "${member.socket_id}": `, peer.signalingState);
				};

				peer.ontrack = (event) => {
					info(`Received track from "${member.socket_id}": `, event);
					const stream = event.streams[0] ?? new MediaStream([event.track]);

					event.track.onmute = () => {
						info(`Remote track MUTED from "${member.socket_id}"`);
					};

					event.track.onunmute = () => {
						info(`Remote track UNMUTED from "${member.socket_id}"`);
					};

					event.track.onended = () => {
						info(`Remote track ENDED from "${member.socket_id}"`);
					};

					const source = audioContext.createMediaStreamSource(stream);
					source.connect(audioContext.destination);

					sources.set(member.socket_id, source);
					streams.set(member.socket_id, stream);
				};
				peer.onicecandidate = async (event) => {
					if (!event.candidate) return;

					const candidate: WebRTCEvent = {
						type: "candidate",
						candidate: event.candidate?.candidate,
						sdpMid: event.candidate?.sdpMid,
						sdpMLineIndex: event.candidate?.sdpMLineIndex,
						usernameFragment: event.candidate?.usernameFragment
					};
					info(`Sending ICE candidate to "${member.socket_id}": `, candidate);
					await client.sendWebRTCEvent(member.socket_id, candidate);
				};
				peer.onicecandidateerror = (event) => {
					error(`ICE candidate error for "${member.socket_id}": `, event);
				};

				for (const track of stream.getTracks()) {
					info(`Adding local track to peer for "${member.socket_id}": `, track);
					peer.addTrack(track, stream);
				}

				peers.set(member.socket_id, peer);

				if (socketId > member.socket_id) {
					info(`Creating offer for "${member.socket_id}"`);
					const offer = await peer.createOffer({ offerToReceiveAudio: true });

					info(`Setting local description for "${member.socket_id}": `, offer);
					await peer.setLocalDescription(offer);
					info(`Sending offer to "${member.socket_id}": `, offer);
					await client.sendWebRTCEvent(member.socket_id, offer);
				}
			}
		}
		for (const [socketId, peer] of peers) {
			if (!members.find((member) => member.socket_id == socketId)) {
				sources.get(socketId)?.disconnect();
				sources.delete(socketId);
				streams.delete(socketId);
				peers.delete(socketId);
				peer.close();
			}
		}
	}

	onMount(async () => {
		audioContext.onstatechange = onAudioStateChange;

		client.onWebRTCEvent = onWebRTCEvent;
		client.onChannelMemberLeft = syncPeers;
		client.onChannelMemberJoined = syncPeers;

		await getLocalStream();
		await syncPeers();
	});

	onDestroy(() => {
		audioContext.onstatechange = null;

		client.onWebRTCEvent = undefined;
		client.onChannelMemberJoined = undefined;
		client.onChannelMemberLeft = undefined;

		for (const [socketId, peer] of peers) {
			sources.get(socketId)?.disconnect();
			peer.close();
		}

		peers.clear();
		streams.clear();
		sources.clear();
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
<div class="flex h-full w-full flex-col bg-gray-900">
	<div class="flex flex-1 flex-col gap-3 overflow-y-auto p-4">
		{#each members as member}
			<div class="rounded-md border border-gray-700 bg-gray-800 p-3">
				<div class="mb-2 flex items-center justify-between gap-2">
					<span class="font-medium">{member.profile.name}</span>
					<span class="text-xs text-gray-400">{member.socket_id}</span>
				</div>
			</div>
		{/each}
	</div>
</div>
