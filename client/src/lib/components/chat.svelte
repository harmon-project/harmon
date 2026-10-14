<script lang="ts">
	import { onMount, tick } from "svelte";
	import { Client, type Message } from "harmon-lib";
	import Fa from "svelte-fa";
	import { faTrash, faUpload } from "@fortawesome/free-solid-svg-icons";
	import { postFiles } from "harmon-lib/http";
	import FilePreview from "./filePreview.svelte";
	import Msg from "./message.svelte";
	import Loading from "./loading.svelte";

	let { client }: { client: Client } = $props();

	let messages = $state<Message[]>([]);
	let messageContent = $state("");
	let messageFiles: File[] = $state([]);

	let scrollContainer: HTMLDivElement | null = null;
	let isLoadingMessages = false;
	let isSendingMessage = $state(false);
	let isDraggingFiles = $state(false);
	// dragenter/dragleave fire for every child element, so count them to know when the drag really left
	let dragDepth = 0;
	let hasMore = true;

	const beforeId = $derived<string | undefined>(messages?.[0]?.id);

	async function scrollToBottom() {
		await tick();
		if (scrollContainer) {
			scrollContainer.scrollTop = scrollContainer.scrollHeight;
		}
	}

	async function sendMessage() {
		if ((!messageContent.trim() && messageFiles.length == 0) || isSendingMessage) return;

		try {
			isSendingMessage = true;

			if (messageFiles.length > 0) {
				const files = await postFiles(client.url, messageFiles);
				const fileIds = files.map((f) => f.id);

				await client.sendMessage(messageContent, fileIds);
			} else {
				await client.sendMessage(messageContent, []);
			}
		} finally {
			messageContent = "";
			messageFiles = [];
			isSendingMessage = false;
		}
	}

	function addFiles(files: Iterable<File>) {
		messageFiles = [...messageFiles, ...files];
	}

	function onFileInputChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		addFiles(input.files ?? []);
		input.value = "";
	}

	let textarea: HTMLTextAreaElement | undefined = $state();

	function onPaste(event: ClipboardEvent) {
		const files = event.clipboardData?.files;
		if (!files || files.length == 0 || isSendingMessage) return;

		// pasting into some other field (e.g. a modal) shouldn't attach to the chat
		const target = event.target as HTMLElement | null;
		if (target !== textarea && target?.closest("input, textarea, [contenteditable]")) return;

		event.preventDefault();
		addFiles(files);
	}

	function isFileDrag(event: DragEvent) {
		return event.dataTransfer?.types.includes("Files") ?? false;
	}

	function onDragEnter(event: DragEvent) {
		if (!isFileDrag(event)) return;
		event.preventDefault();
		dragDepth++;
		isDraggingFiles = true;
	}

	function onDragOver(event: DragEvent) {
		if (!isFileDrag(event)) return;
		event.preventDefault();
		event.dataTransfer!.dropEffect = isSendingMessage ? "none" : "copy";
	}

	function onDragLeave(event: DragEvent) {
		if (!isFileDrag(event)) return;
		dragDepth = Math.max(0, dragDepth - 1);
		if (dragDepth == 0) isDraggingFiles = false;
	}

	function onDrop(event: DragEvent) {
		if (!isFileDrag(event)) return;
		event.preventDefault();
		dragDepth = 0;
		isDraggingFiles = false;

		if (isSendingMessage) return;
		addFiles(event.dataTransfer?.files ?? []);
	}

	function removeFile(index: number) {
		messageFiles = messageFiles.filter((_, i) => i !== index);
	}

	async function messageReceived(message: Message) {
		messages = [...messages, message];
		await scrollToBottom();
	}

	async function messageDeleted(message: Message) {
		messages = messages.filter((msg) => msg.id !== message.id);
	}

	async function messagesLoaded(msgs: Message[]) {
		hasMore = msgs.length > 0;

		if (scrollContainer) {
			const previousScrollHeight = scrollContainer.scrollHeight;
			const previousScrollTop = scrollContainer.scrollTop;

			messages = [...msgs, ...messages];

			await tick();

			scrollContainer.scrollTop =
				scrollContainer.scrollHeight - previousScrollHeight + previousScrollTop;
		} else {
			messages = [...msgs, ...messages];
		}
	}

	async function loadMessages() {
		if (isLoadingMessages || !hasMore) return;

		isLoadingMessages = true;

		const messages = await client.loadMessages(beforeId);

		await messagesLoaded(messages);

		isLoadingMessages = false;
	}

	async function deleteMessage(messageId: string) {
		await client.deleteMessage(messageId);
	}

	function onScroll() {
		if (!scrollContainer) return;

		if (scrollContainer.scrollTop <= 24) {
			loadMessages();
		}
	}

	onMount(() => {
		client.onMessageReceived = messageReceived;
		client.onMessageDeleted = messageDeleted;

		loadMessages();
	});
</script>

<!-- keep files dropped outside the chat from making the browser navigate away to open them -->
<svelte:window
	ondragover={(e) => isFileDrag(e) && e.preventDefault()}
	ondrop={(e) => isFileDrag(e) && e.preventDefault()}
	onpaste={onPaste}
/>

<div
	role="region"
	aria-label="Chat"
	class="relative flex h-full min-h-0 w-full min-w-0 flex-col"
	ondragenter={onDragEnter}
	ondragover={onDragOver}
	ondragleave={onDragLeave}
	ondrop={onDrop}
>
	{#if isDraggingFiles && !isSendingMessage}
		<div
			class="pointer-events-none absolute inset-2 z-10 flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed border-blue-500 bg-gray-900/90"
		>
			<Fa class="text-4xl text-blue-500" icon={faUpload} />
			<p>Drop files to attach them</p>
		</div>
	{/if}
	<div class="flex h-full w-full overflow-y-auto">
		<div
			bind:this={scrollContainer}
			onscroll={onScroll}
			class="flex h-full w-full overflow-y-auto font-mono"
		>
			<div class="flex w-full flex-col">
				{#each messages as message}
					{#if message.profile.public_key === client.profile?.public_key}
						<Msg
							{message}
							url={client.url}
							onDelete={() => deleteMessage(message.id)}
						/>
					{:else}
						<Msg {message} url={client.url} />
					{/if}
				{/each}
			</div>
		</div>
	</div>
	{#if isSendingMessage}
		<hr />
		<div class="flex h-20 w-full shrink-0 items-center justify-center">
			<Loading />
		</div>
	{:else}
		{#if messageFiles.length > 0}
			<hr />
			<div class="flex shrink-0 flex-row gap-2 overflow-x-auto overflow-y-hidden p-2">
				{#each messageFiles as file, i (file)}
					<div class="group relative shrink-0" title={file.name}>
						<FilePreview {file} />
						<div
							class="absolute top-1 right-1 flex items-center justify-center rounded-md bg-gray-900 p-1 opacity-0 transition-opacity *:cursor-pointer *:rounded-sm *:p-1 *:transition group-focus-within:opacity-100 group-hover:opacity-100 *:hover:bg-white/10 [&_svg]:transition-transform [&>*:hover>svg]:scale-110 [@media(hover:none)]:opacity-100"
						>
							<button
								type="button"
								class="text-red-400"
								aria-label={`Remove ${file.name}`}
								onclick={() => removeFile(i)}
							>
								<Fa icon={faTrash} />
							</button>
						</div>
					</div>
				{/each}
			</div>
			<hr />
		{:else}
			<hr />
		{/if}
		<form
			class="flex h-20 w-full shrink-0 flex-col items-center justify-center gap-2 p-2"
			onsubmit={(e) => {
				e.preventDefault();
				sendMessage();
			}}
		>
			<div class="flex w-full flex-row items-center">
				<label
					class="flex h-10 w-10 cursor-pointer items-center justify-center"
					for="file-upload"
				>
					<Fa class="text-2xl" icon={faUpload}></Fa>
					<input
						onchange={onFileInputChange}
						class="hidden"
						type="file"
						id="file-upload"
						multiple
					/>
				</label>
				<textarea
					bind:this={textarea}
					bind:value={messageContent}
					onkeydown={(event) => {
						if (event.key === "Enter" && !event.shiftKey) {
							event.preventDefault();
							sendMessage();
						}
					}}
					class="w-full rounded-md border border-inherit bg-inherit p-1.5"></textarea>
				<button type="submit" class="cursor-pointer text-3xl">▶️</button>
			</div>
		</form>
	{/if}
</div>
