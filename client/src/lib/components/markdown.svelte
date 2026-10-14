<script lang="ts">
	import SvelteMarkdown from "@humanspeak/svelte-markdown";
	import MarkdownLink from "./markdownLink.svelte";

	const { content }: { content: string } = $props();
</script>

<div class="prose max-w-none wrap-break-word prose-invert prose-a:text-blue-400">
	<SvelteMarkdown source={content} renderers={{ link: MarkdownLink }} />
</div>
