<script lang="ts">
	import type { Snippet } from "svelte";
	import { isTauri } from "@tauri-apps/api/core";
	import { openUrl } from "@tauri-apps/plugin-opener";

	const {
		href = "",
		title,
		children
	}: { href?: string; title?: string; children?: Snippet } = $props();

	function onclick(event: MouseEvent) {
		// the desktop webview ignores target="_blank", so hand the link to the system browser
		if (!isTauri()) return;
		event.preventDefault();
		openUrl(href);
	}
</script>

<a {href} {title} target="_blank" rel="noopener noreferrer" {onclick}>{@render children?.()}</a>
