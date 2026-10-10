<script lang="ts">
	import { goto } from "$app/navigation";
	import { useAuth } from "$lib/auth";
	import { info } from "$lib/log";
	import { sha256, sign } from "harmon-lib/crypto";
	import { stringToUint8Array, z32toUint8Array } from "harmon-lib/utils";
	import { useStorage } from "$lib/storage";
	import { onMount } from "svelte";
	import { Client } from "harmon-lib";
	import AddServerModal from "$lib/components/addServerModal.svelte";
	import Chat from "$lib/components/chat.svelte";
	import ChatsPanel from "$lib/components/chatsPanel.svelte";
	import SidePanel from "$lib/components/sidePanel.svelte";
	import EditProfile from "$lib/components/editProfile.svelte";
	import VoiceChat from "$lib/components/voiceChat.svelte";

	const auth = useAuth();

	const servers = useStorage<string[]>("servers", []);
	const currentServer = useStorage<string | undefined>("currentServer", undefined);

	let client: Client | undefined = $state();
	let voiceChat: VoiceChat | undefined = $state();

	let showAddServerModal = $state(false);
	let isEditingProfile = $state(false);
	let showNav = $state(false);

	$effect(() => {
		let _ = client?.currentChannel;
		showNav = false;
	});

	async function onClientConnect(client: Client) {
		info("client connect as id: " + client.id);

		const challengeValue = await client.requestChallenge(auth?.publicKey!);

		const hash = sha256(stringToUint8Array(challengeValue.token));
		const signature = sign(hash, auth?.privateKey!);

		const confirmValue = await client.confirmChallenge(challengeValue.token, signature);

		await client.auth(confirmValue.token);

		isEditingProfile = client.profile == undefined;
	}

	async function onClientDisconnect() {
		info("OnSocketDisconnect");
	}

	async function onCurrentServerChange(z32publicKey: string | undefined) {
		client?.close();

		if (!z32publicKey) return;

		const publicKey = z32toUint8Array(z32publicKey);

		client = await Client.init(publicKey);

		client.onConnectionReady = async () => {
			await onClientConnect(client!);
		};

		client.onConnectionClosed = async () => {
			await onClientDisconnect();
		};
	}

	onMount(() => {
		if (!auth) {
			goto("/login");
			return;
		}

		currentServer.subscribe(onCurrentServerChange);
	});
</script>

<div class="h-dvh w-screen overflow-hidden bg-gray-900 text-white">
	{#if isEditingProfile}
		<EditProfile
			onEdit={async (name) => {
				await client?.updateProfile(name);
				isEditingProfile = false;
			}}
			closable={!!client?.profile}
			onClose={() => {
				isEditingProfile = false;
			}}
		/>
	{/if}
	{#if showAddServerModal}
		<AddServerModal
			onServerAdd={(pubKey: string) => {
				servers.update((p) => [...p, pubKey]);
				showAddServerModal = false;
			}}
			onClose={() => {
				showAddServerModal = false;
			}}
		/>
	{/if}
	{#if $servers.length == 0}
		<div class="flex h-full w-full flex-col items-center justify-center gap-4">
			<h1 class="text-2xl">No servers added</h1>
			<button
				class="cursor-pointer rounded-md bg-gray-800 p-2 text-white"
				onclick={() => {
					showAddServerModal = true;
				}}
			>
				Add server
			</button>
		</div>
	{:else if client}
		<div
			class="grid h-full w-full grid-cols-[minmax(0,1fr)] grid-rows-[auto_minmax(0,1fr)] md:grid-cols-[auto_auto_minmax(0,1fr)] md:grid-rows-1"
		>
			<div class="flex items-center gap-3 border-b p-3 md:hidden">
				<button
					class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md hover:bg-gray-700"
					aria-label="Open menu"
					onclick={() => (showNav = true)}
				>
					☰
				</button>
				<span class="truncate"
					>{client.currentChannel?.channel.name ?? client.serverInfo?.title}</span
				>
			</div>
			{#if showNav}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="fixed inset-0 z-20 bg-black/50 md:hidden"
					onclick={() => (showNav = false)}
				></div>
			{/if}
			<div
				class={`fixed inset-y-0 left-0 z-30 flex max-w-[85vw] bg-gray-900 transition-transform md:contents ${showNav ? "translate-x-0" : "-translate-x-full"}`}
			>
				<SidePanel
					{servers}
					{currentServer}
					onAddServer={() => {
						showAddServerModal = true;
					}}
				/>
				<ChatsPanel
					{client}
					onSetMuted={(muted) => voiceChat?.setMuted(muted) ?? false}
					onStartScreenShare={async () => (await voiceChat?.startScreenStream()) ?? false}
					onStopScreenShare={async () => (await voiceChat?.stopScreenStream()) ?? false}
					onClickProfile={() => {
						isEditingProfile = true;
					}}
				/>
			</div>
			{#if client.currentChannel}
				{#key client.currentChannel}
					{#if client.currentChannel.channel.type == "Text"}
						<Chat {client} />
					{:else if client.currentChannel.channel.type == "Voice"}
						<VoiceChat bind:this={voiceChat} {client} />
					{/if}
				{/key}
			{/if}
		</div>
	{:else}
		<div class="flex h-full w-full flex-row">
			<SidePanel
				{servers}
				{currentServer}
				onAddServer={() => {
					showAddServerModal = true;
				}}
			/>
			<div class="flex h-full w-full flex-col items-center justify-center gap-4">
				<h1 class="text-center text-2xl">Select a server</h1>
			</div>
		</div>
	{/if}
</div>
