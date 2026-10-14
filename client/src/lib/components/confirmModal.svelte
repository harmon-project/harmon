<script lang="ts">
	import type { Snippet } from "svelte";

	const {
		title,
		confirmLabel = "Confirm",
		onConfirm,
		onCancel,
		children
	}: {
		title: string;
		confirmLabel?: string;
		onConfirm: () => void;
		onCancel: () => void;
		children?: Snippet;
	} = $props();

	let confirmButton = $state<HTMLButtonElement>();

	$effect(() => {
		confirmButton?.focus();
	});
</script>

<svelte:window
	onkeydown={(event) => {
		if (event.key === "Escape") onCancel();
	}}
/>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-40 flex items-center justify-center p-4 backdrop-blur-xs"
	oncontextmenu={(event) => event.stopPropagation()}
	onclick={(event) => {
		if (event.target === event.currentTarget) onCancel();
	}}
>
	<div
		role="dialog"
		aria-modal="true"
		aria-label={title}
		class="flex w-full max-w-md flex-col gap-4 rounded-md border border-gray-800 bg-gray-900 p-4 text-white"
	>
		<h1 class="text-lg">{title}</h1>
		{@render children?.()}
		<div class="flex justify-end gap-2">
			<button
				type="button"
				class="cursor-pointer rounded-md px-3 py-2 hover:bg-gray-800"
				onclick={onCancel}
			>
				Cancel
			</button>
			<button
				bind:this={confirmButton}
				type="button"
				class="cursor-pointer rounded-md bg-red-600 px-3 py-2 hover:bg-red-500"
				onclick={onConfirm}
			>
				{confirmLabel}
			</button>
		</div>
	</div>
</div>
