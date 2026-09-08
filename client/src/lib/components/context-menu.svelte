<script lang="ts">
	import type { Snippet } from "svelte";

	type Position = { x: number; y: number };

	type ContextMenuParams = {
		position?: Position | null;
		children: Snippet;
	};

	let { position = $bindable(null), children }: ContextMenuParams = $props();

	let menu = $state<HTMLDivElement>();
	let width = $state(0);
	let height = $state(0);

	const left = $derived(position ? Math.min(position.x, window.innerWidth - width - 8) : 0);
	const top = $derived(position ? Math.min(position.y, window.innerHeight - height - 8) : 0);

	$effect(() => {
		if (position) menu?.focus();
	});

	function closeOnOutside(event: PointerEvent) {
		if (position && !menu?.contains(event.target as Node)) position = null;
	}
</script>

<svelte:window onpointerdown={closeOnOutside} onwheel={() => (position = null)} />

{#if position}
	<div
		bind:this={menu}
		bind:clientWidth={width}
		bind:clientHeight={height}
		role="menu"
		tabindex="-1"
		style="left: {left}px; top: {top}px;"
		onclick={(event) => {
			if (event.target !== event.currentTarget) position = null;
		}}
		onkeydown={(event) => {
			if (event.key === "Escape") position = null;
		}}
		class="fixed z-50 flex w-max max-w-[50vw] flex-col gap-0.5 rounded-md border border-gray-700 bg-gray-900 p-1 text-sm shadow-lg shadow-black/50 outline-none *:flex *:w-full *:cursor-pointer *:items-center *:gap-2 *:rounded-sm *:px-2 *:py-1.5 *:text-left *:transition *:hover:bg-white/10"
	>
		{@render children()}
	</div>
{/if}
