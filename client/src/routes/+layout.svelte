<script lang="ts">
	import "./layout.css";
	import icon from "$lib/assets/icon.png";
	import Toast from "$lib/components/toast.svelte";
	import ContextMenu from "$lib/components/contextMenu.svelte";
	import { type Snippet } from "svelte";

	let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
	<title>Harmon</title>
	<link rel="icon" href={icon} />
</svelte:head>

<Toast />
<ContextMenu />

{@render children?.()}
