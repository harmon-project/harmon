<script lang="ts">
	import SvelteMarkdown from "@humanspeak/svelte-markdown";

	const { content }: { content: string } = $props();
</script>

<div class="prose prose-invert prose-a:text-blue-400">
	<SvelteMarkdown source={content} />
</div>
