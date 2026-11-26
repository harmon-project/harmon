<script module lang="ts">
	import type { IconDefinition } from "@fortawesome/fontawesome-common-types";
	import Fa from "svelte-fa";

	export interface MenuItem {
		label: string;
		icon?: IconDefinition;
		danger?: boolean;
		onSelect: () => any | Promise<any>;
	}

	export interface ActiveMenu {
		position: { x: number; y: number };
		items: MenuItem[];
	}

	let activeMenu = $state<ActiveMenu | null>(null);
	let menuElement = $state<HTMLDivElement>();
	let width = $state(0);
	let height = $state(0);

	const left = $derived(
		activeMenu ? Math.min(activeMenu.position.x, window.innerWidth - width - 8) : 0
	);
	const top = $derived(
		activeMenu ? Math.min(activeMenu.position.y, window.innerHeight - height - 8) : 0
	);

	function onpointerdown(event: PointerEvent) {
		if (activeMenu && !menuElement?.contains(event.target as Node)) {
			setActiveMenu(null);
		}
	}

	function onkeydown(event: KeyboardEvent) {
		if (activeMenu && event.key === "Escape") {
			setActiveMenu(null);
		}
	}

	function onclick(event: MouseEvent) {
		if (event.target !== event.currentTarget) {
			setActiveMenu(null);
		}
	}

	function onwheel(event: WheelEvent) {
		event.preventDefault();

		setActiveMenu(null);
	}

	export function setActiveMenu(menu: ActiveMenu | null = null) {
		activeMenu = menu;
	}

	export function contextMenu(node: HTMLElement, items: MenuItem[]) {
		const handle = (event: MouseEvent) => {
			event.preventDefault();

			const position = { x: event.clientX, y: event.clientY };

			setActiveMenu({
				position,
				items
			});
		};

		node.addEventListener("contextmenu", handle);

		return {
			destroy() {
				node.removeEventListener("contextmenu", handle);
			}
		};
	}
</script>

<svelte:window {onpointerdown} {onwheel} />

{#if activeMenu}
	<div
		role="menu"
		tabindex="-1"
		style="left: {left}px; top: {top}px;"
		class="fixed z-50 flex w-max max-w-[50vw] flex-col gap-0.5 rounded-md border border-gray-700 bg-gray-900 p-1 text-sm text-white shadow-lg shadow-black/50 outline-none"
		bind:this={menuElement}
		bind:clientWidth={width}
		bind:clientHeight={height}
		{onkeydown}
		{onclick}
	>
		{#each activeMenu.items as item}
			<button
				type="button"
				role="menuitem"
				onclick={item.onSelect}
				class:text-red-400={item.danger}
				class="flex w-full cursor-pointer items-center gap-2 rounded-sm px-2 py-1.5 text-left transition hover:bg-white/10"
			>
				{#if item.icon}
					<Fa icon={item.icon} class="shrink-0" />
				{/if}
				{item.label}
			</button>
		{/each}
	</div>
{/if}
