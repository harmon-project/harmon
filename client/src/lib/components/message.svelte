<script lang="ts">
	import {
		faDownload,
		faFile,
		faTrash,
		faPencil,
		faEllipsisH,
		faCopy,
		faHashtag
	} from "@fortawesome/free-solid-svg-icons";
	import type { Message } from "harmon-lib";
	import Fa from "svelte-fa";
	import ContextMenu from "./context-menu.svelte";
	import Markdown from "./markdown.svelte";

	type MessageParams = {
		url: string;
		message: Message;
		onEdit?: () => void;
		onDelete?: () => void;
	};

	const { url, message, onEdit, onDelete }: MessageParams = $props();

	let menuPosition = $state<{ x: number; y: number } | null>(null);

	function openMenu(event: MouseEvent) {
		event.preventDefault();
		menuPosition = { x: event.clientX, y: event.clientY };
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		const now = new Date();

		const isToday =
			date.getDate() === now.getDate() &&
			date.getMonth() === now.getMonth() &&
			date.getFullYear() === now.getFullYear();

		if (isToday) {
			return date.toLocaleTimeString("pt-BR", {
				hour: "2-digit",
				minute: "2-digit"
			});
		}

		return date.toLocaleString("pt-BR", {
			day: "2-digit",
			month: "2-digit",
			year: "numeric",
			hour: "2-digit",
			minute: "2-digit"
		});
	}
</script>

<div
	role="listitem"
	class="group relative flex flex-row gap-1 p-2 hover:bg-gray-800"
	oncontextmenu={openMenu}
>
	<div class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-blue-500">
		{message.profile.name[0]}
	</div>
	<!-- Menu com os ícones de acesso direto -->
	<div
		class="absolute top-1 right-8 flex items-center justify-center gap-1 rounded-md bg-gray-900 px-1 py-1 opacity-0 transition-opacity *:cursor-pointer *:rounded-sm *:p-1 *:transition group-hover:opacity-100 *:hover:bg-white/10 [&_svg]:transition-transform [&>*:hover>svg]:scale-110"
	>
		{#if onEdit}
			<button> <Fa icon={faPencil} /> </button>
		{/if}
		{#if onDelete}
			<button onclick={onDelete} class="text-red-400">
				<Fa icon={faTrash} />
			</button>
		{/if}
		<button onclick={openMenu}>
			<Fa icon={faEllipsisH} />
		</button>
	</div>
	<!-- Menu de contexto (clicando na ellipsis) -->
	<ContextMenu bind:position={menuPosition}>
		{#if onDelete}
			<button onclick={onDelete} class="text-red-400">
				<Fa icon={faTrash} />
				Deletar
			</button>
		{/if}
		{#if onEdit}
			<button onclick={onEdit}>
				<Fa icon={faPencil} />
				Editar
			</button>
		{/if}
		<button onclick={() => navigator.clipboard.writeText(message.content)}>
			<Fa icon={faCopy} />
			Copiar texto
		</button>
		<button onclick={() => navigator.clipboard.writeText(message.id)}>
			<Fa icon={faHashtag} />
			Copiar id
		</button>
	</ContextMenu>
	<div class="min-w-0 shrink">
		<div class="flex gap-2">
			<p class="text-1xl text-gray-1 00 font-extrabold">{message.profile.name}</p>
			<p class="text-sm text-gray-400">{formatDate(message.created_at)}</p>
		</div>
		<Markdown content={message.content} />
		<div class="flex flex-col items-start gap-4">
			{#each message.attachments as attachment}
				<div class="group relative mt-1 flex max-h-96">
					<a
						download={attachment.name}
						href={`${url}/files/${attachment.id}`}
						target="_blank"
						rel="noopener noreferrer"
						class="absolute top-1 right-1 z-10 hidden cursor-pointer rounded-sm bg-gray-900 p-1 group-hover:flex"
					>
						<Fa class="text-2xl" icon={faDownload} />
					</a>
					{#if attachment.mime_type.startsWith("audio")}
						<audio
							class="h-20 rounded-lg"
							controls
							src={`${url}/files/${attachment.id}`}
						></audio>
					{:else if attachment.mime_type.startsWith("image")}
						<img
							class="h-64 rounded-lg"
							alt={attachment.hash}
							src={`${url}/files/${attachment.id}`}
						/>
					{:else if attachment.mime_type.startsWith("video")}
						<video
							controls
							preload="metadata"
							class="h-64 rounded-lg"
							src={`${url}/files/${attachment.id}`}
						>
							<track kind="captions" />
						</video>
					{:else}
						<div class="flex gap-2 rounded-lg p-2 hover:bg-gray-900">
							<Fa class="text-5xl" icon={faFile} />
							<p>{attachment.name}</p>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	</div>
</div>
