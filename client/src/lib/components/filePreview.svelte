<script lang="ts">
	import { faFile, faMusic, faPlay } from "@fortawesome/free-solid-svg-icons";
	import Fa from "svelte-fa";

	const { file }: { file: File } = $props();

	const isImage = $derived(file.type.startsWith("image/"));
	const isVideo = $derived(file.type.startsWith("video/"));
	const isAudio = $derived(file.type.startsWith("audio/"));

	let url: string | undefined = $state();

	$effect(() => {
		if (!isImage && !isVideo) return;

		const objectUrl = URL.createObjectURL(file);
		url = objectUrl;

		return () => URL.revokeObjectURL(objectUrl);
	});
</script>

<div
	class="relative flex h-24 w-24 items-center justify-center overflow-hidden rounded-xl bg-gray-950/25"
>
	{#if isImage && url}
		<img class="h-full w-full object-cover" alt={file.name} src={url} />
	{:else if isVideo && url}
		<!-- #t=0.1 makes browsers render a frame instead of a blank box -->
		<video class="h-full w-full object-cover" src={`${url}#t=0.1`} preload="metadata" muted
		></video>
		<div class="absolute inset-0 flex items-center justify-center">
			<div class="flex h-8 w-8 items-center justify-center rounded-full bg-black/60">
				<Fa icon={faPlay} />
			</div>
		</div>
	{:else}
		<div class="flex h-full w-full flex-col items-center justify-center gap-1 p-1">
			<Fa class="text-4xl" icon={isAudio ? faMusic : faFile} />
			<span class="w-full truncate text-center text-xs">{file.name}</span>
		</div>
	{/if}
</div>
