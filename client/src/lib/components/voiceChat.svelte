<script lang="ts">
	import { onDestroy, onMount } from "svelte";
	import type { Client, WebRTCEvent } from "harmon-lib";
	import { push } from "./toast.svelte";
	import { SvelteMap } from "svelte/reactivity";
	import { error, info } from "$lib/log";
	import iceServers from "$lib/assets/iceServers.json";

	const { client }: { client: Client } = $props();

	let stream = $state(new MediaStream());
	let streams = new SvelteMap<string, MediaStream>();
	let peers = new SvelteMap<string, RTCPeerConnection>();

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
			push("Erro ao acessar o microfone. Verifique as permissões do navegador.");
		}
	}

	async function onWebRTCEvent(socketId: string, event: WebRTCEvent) {
		const peer = peers.get(socketId);

		if (!peer) return;

		info(`Received WebRTC event from ${socketId}: `, event);

		switch (event.type) {
			case "offer": {
				await peer.setRemoteDescription(event);

				const answer = await peer.createAnswer();

				await peer.setLocalDescription(answer);

				await client.sendWebRTCEvent(socketId, { ...answer, type: "answer" });
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
					info(`Sending ICE candidate to ${member.socket_id}: `, candidate);
					await client.sendWebRTCEvent(member.socket_id, candidate);
				};
				peer.onicecandidateerror = (event) => {
					error(`ICE candidate error for ${member.socket_id}: `, event);
				};

				for (const track of stream.getTracks()) {
					info(`Adding local track to peer for ${member.socket_id}: `, track);
					peer.addTrack(track, stream);
				}

				peers.set(member.socket_id, peer);

				if (socketId > member.socket_id) {
					info(`Creating offer for ${member.socket_id}`);
					const offer = await peer.createOffer({ offerToReceiveAudio: true });

					info(`Setting local description for ${member.socket_id}: `, offer);
					await peer.setLocalDescription(offer);
					info(`Sending offer to ${member.socket_id}: `, offer);
					await client.sendWebRTCEvent(member.socket_id, offer);
				}
			}
		}
		for (const [socketId, peer] of peers) {
			if (!members.find((member) => member.socket_id == socketId)) {
				peer.close();
				peers.delete(socketId);
				streams.delete(socketId);
			}
		}
	}

	onMount(async () => {
		client.onWebRTCEvent = onWebRTCEvent;
		client.onChannelMemberLeft = syncPeers;
		client.onChannelMemberJoined = syncPeers;

		await getLocalStream();
		await syncPeers();
	});

	onDestroy(() => {
		client.onWebRTCEvent = undefined;
		client.onChannelMemberJoined = undefined;
		client.onChannelMemberLeft = undefined;

		for (const [_, peer] of peers) {
			peer.close();
		}
		
		peers.clear();
		streams.clear();
	});
</script>

<div class="flex h-full w-full flex-col bg-gray-900">
	<div class="flex flex-1 flex-col gap-3 overflow-y-auto p-4">
		{#each members as member}
			<div class="rounded-md border border-gray-700 bg-gray-800 p-3">
				<div class="mb-2 flex items-center justify-between gap-2">
					<span class="font-medium">{member.profile.name}</span>
					<span class="text-xs text-gray-400">{member.socket_id}</span>
				</div>
				{#if member.socket_id !== client.id}
					{#if streams.get(member.socket_id)}
						<audio autoplay playsinline srcObject={streams.get(member.socket_id)}
						></audio>
					{/if}
				{/if}
			</div>
		{/each}
	</div>
</div>
