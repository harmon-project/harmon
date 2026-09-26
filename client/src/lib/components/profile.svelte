<script lang="ts">
	import type { Profile } from "harmon-lib";

	const {
		profile,
		color,
		position,
		onClose
	}: {
		profile: Profile;
		color: string;
		position: { x: number; y: number };
		onClose: () => void;
	} = $props();

	let popup = $state<HTMLDivElement>();

	function onPointerDown(event: PointerEvent) {
		if (!popup?.contains(event.target as Node)) {
			onClose();
		}
	}
</script>

<svelte:window onpointerdown={onPointerDown} />

<div
	bind:this={popup}
	role="dialog"
	style="left: {position.x}px; top: {position.y}px;"
	class="fixed z-50 flex max-h-[calc(100dvh-1rem)] w-72 max-w-[calc(100vw-1rem)] flex-col gap-3 overflow-y-auto rounded-md border border-gray-700 bg-gray-900 p-4 text-white shadow-lg shadow-black/50 outline-none"
>
	<div
		class="flex h-14 w-14 shrink-0 items-center justify-center rounded-full text-2xl"
		style="background-color: {color};"
	>
		{profile.name[0]}
	</div>
	<h2 class="text-lg font-extrabold wrap-break-word">{profile.name}</h2>
	<div class="flex flex-col gap-1 border-t border-gray-700 pt-3">
		<p class="text-xs text-gray-400">Public key</p>
		<p class="text-xs break-all text-gray-300 select-text">{profile.public_key}</p>
	</div>
</div>
