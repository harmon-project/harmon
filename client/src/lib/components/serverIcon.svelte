<script lang="ts">
	import { faTrash, faTriangleExclamation } from "@fortawesome/free-solid-svg-icons";
	import { getInfo } from "harmon-lib/http";
	import { DNSClient } from "harmon-lib/pkdns";
	import { contextMenu, type MenuItem } from "$lib/components/contextMenu.svelte";
	import Loading from "./loading.svelte";
	import Fa from "svelte-fa";

	type ServerIconProps = {
		onClick: () => void;
		onRemove: () => void;
		publicKey: string;
	};

	const { onClick, onRemove, publicKey }: ServerIconProps = $props();

	const menuItems: MenuItem[] = $derived([
		{ label: "Remove Server", icon: faTrash, danger: true, onSelect: onRemove }
	]);

	async function get_data() {
		const dnsClient = new DNSClient();

		const url = await dnsClient.resolveUrl(publicKey);

		if (!url) {
			throw new Error("Failed to resolve URL for public key: " + publicKey);
		}

		const data = await getInfo(url);

		return { data, url };
	}

	const data = get_data();
</script>

{#await data}
	<button
		class="group relative flex h-12 w-12 cursor-pointer flex-col items-center justify-center rounded-xl bg-zinc-600"
		onclick={onClick}
		use:contextMenu={menuItems}
	>
		<Loading />
		<div
			class="pointer-events-none absolute left-full ml-2 flex translate-x-2 items-center justify-center rounded-md bg-slate-700 p-1 opacity-0 transition-all duration-200 group-hover:translate-x-0 group-hover:opacity-100"
		>
			<p>Loading...</p>
		</div>
	</button>
{:then { data, url }}
	<button
		class="group relative flex cursor-pointer flex-col items-center justify-center rounded-xl"
		onclick={onClick}
		use:contextMenu={menuItems}
	>
		<img src={`${url}/icon`} alt="icon" class="h-12 w-12 rounded-xl" />
		<div
			class="pointer-events-none absolute left-full ml-2 flex translate-x-2 items-center justify-center rounded-md bg-slate-700 p-1 opacity-0 transition-all duration-200 group-hover:translate-x-0 group-hover:opacity-100"
		>
			<p>{data.title}</p>
		</div>
	</button>
{:catch error}
	<button
		class="group relative flex h-12 w-12 cursor-pointer flex-col items-center justify-center rounded-xl bg-zinc-600"
		onclick={onClick}
		use:contextMenu={menuItems}
	>
		<Fa icon={faTriangleExclamation} />
		<div
			class="pointer-events-none absolute left-full ml-2 flex translate-x-2 items-center justify-center rounded-md bg-slate-700 p-1 opacity-0 transition-all duration-200 group-hover:translate-x-0 group-hover:opacity-100"
		>
			<p>{error.message}</p>
		</div>
	</button>
{/await}
