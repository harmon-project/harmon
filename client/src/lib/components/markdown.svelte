<script lang="ts">
	import SvelteMarkdown from "@humanspeak/svelte-markdown";

	const { content }: { content: string } = $props();
</script>

<div class="prose prose-invert max-w-none wrap-break-word prose-a:text-blue-400">
	<SvelteMarkdown source={content} />
</div>
