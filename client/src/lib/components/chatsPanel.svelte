<script lang="ts">
	import type { Channel, Client } from "harmon-lib";
	import { useAuth } from "$lib/auth";
	import { push } from "./toast.svelte";
	import { faHashtag, faVolume } from "@fortawesome/free-solid-svg-icons";
	import { uint8ArrayToZ32 } from "harmon-lib/utils";
	import Fa from "svelte-fa";
	import { faChromecast } from "@fortawesome/free-brands-svg-icons";

	const {
		client,
		onClickProfile,
		onStartScreenShare,
		onStopScreenShare
	}: {
		client: Client;
		onClickProfile: () => void;
		onStartScreenShare: () => Promise<boolean>;
		onStopScreenShare: () => Promise<boolean>;
	} = $props();

	const auth = useAuth();
	const pubKey = uint8ArrayToZ32(auth?.publicKey!);

	let isScreenSharing = $state(false);

	async function onScreenShareClick() {
		isScreenSharing = isScreenSharing ? await onStopScreenShare() : await onStartScreenShare();
	}

	async function selectChannel(channel: Channel) {
		if (client.currentChannel?.channel.id === channel.id) return;
		await client.joinChannel(channel.id);
	}

	$effect(() => {
		let _ = client.currentChannel;
		isScreenSharing = false;
	});
</script>

<aside class="flex w-60 flex-col border-r">
	<div class="flex flex-col items-center justify-center gap-2 p-4">
		<h1 class="text-center">{client.serverInfo?.title}</h1>
		<h3 class="text-center text-sm">{client.currentChannel?.channel.name}</h3>
	</div>
	<hr />
	<div class="flex grow flex-col gap-2 p-2 text-gray-400">
		{#each client.channelList as channel}
			<button
				onclick={() => selectChannel(channel)}
				class={`flex cursor-pointer items-center rounded-md p-2 hover:bg-gray-800 ${client.currentChannel?.channel.id === channel.id ? "bg-gray-800" : ""}`}
			>
				{#if channel.type === "Text"}
					<span class="flex items-center gap-0.5"
						><Fa icon={faHashtag} /> {channel.name}</span
					>
				{:else}
					<span class="flex items-center gap-0.5"
						><Fa icon={faVolume} /> {channel.name}</span
					>
				{/if}
			</button>
		{/each}
	</div>
	<hr />
	<div class="flex h-20 w-full flex-row items-center justify-center gap-1 p-2">
		<div class="flex w-full flex-col overflow-hidden">
			<button class="cursor-pointer truncate" onclick={onClickProfile}>
				<p class="text-lg">{client.profile?.name}</p>
			</button>
			<button
				class="cursor-pointer truncate"
				onclick={() => {
					navigator.clipboard.writeText(pubKey);
					push("PublicKey copied to clipboard");
				}}
			>
				<p class="truncate text-xs">{pubKey}</p>
			</button>
		</div>
		{#if client.currentChannel?.channel.type == "Voice"}
			<button
				class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md hover:bg-gray-700"
				onclick={onScreenShareClick}
			>
				{#if isScreenSharing}
					<Fa icon={faChromecast} class="text-blue-500"></Fa>
				{:else}
					<Fa icon={faChromecast}></Fa>
				{/if}
			</button>
		{/if}
	</div>
</aside>
