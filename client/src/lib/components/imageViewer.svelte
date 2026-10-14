<script lang="ts">
	import {
		faClose,
		faDownload,
		faMagnifyingGlassMinus,
		faMagnifyingGlassPlus
	} from "@fortawesome/free-solid-svg-icons";
	import Fa from "svelte-fa";

	const { src, name, onClose }: { src: string; name: string; onClose: () => void } = $props();

	const MIN_SCALE = 1;
	const MAX_SCALE = 8;

	let container = $state<HTMLDivElement>();
	let scale = $state(1);
	let x = $state(0);
	let y = $state(0);

	let drag: { pointerId: number; startX: number; startY: number; moved: boolean } | null = null;

	// zoom keeping the point under (cx, cy) fixed; coordinates are relative to the viewport centre
	function zoomTo(next: number, cx = 0, cy = 0) {
		next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, next));
		if (next == MIN_SCALE) {
			x = 0;
			y = 0;
		} else {
			x = cx - (cx - x) * (next / scale);
			y = cy - (cy - y) * (next / scale);
		}
		scale = next;
	}

	function fromCentre(event: MouseEvent) {
		const rect = container!.getBoundingClientRect();
		return [
			event.clientX - rect.left - rect.width / 2,
			event.clientY - rect.top - rect.height / 2
		] as const;
	}

	function onWheel(event: WheelEvent) {
		event.preventDefault();
		zoomTo(scale * (event.deltaY < 0 ? 1.2 : 1 / 1.2), ...fromCentre(event));
	}

	function onPointerDown(event: PointerEvent) {
		if (event.button !== 0) return;
		drag = {
			pointerId: event.pointerId,
			startX: event.clientX,
			startY: event.clientY,
			moved: false
		};
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
	}

	function onPointerMove(event: PointerEvent) {
		if (!drag || drag.pointerId !== event.pointerId) return;
		const dx = event.clientX - drag.startX;
		const dy = event.clientY - drag.startY;
		if (!drag.moved && Math.hypot(dx, dy) < 4) return;
		drag.moved = true;
		if (scale > 1) {
			x += dx;
			y += dy;
		}
		drag.startX = event.clientX;
		drag.startY = event.clientY;
	}

	function onPointerUp(event: PointerEvent) {
		if (!drag || drag.pointerId !== event.pointerId) return;
		const wasClick = !drag.moved;
		drag = null;
		if (wasClick) {
			// a plain click on the image toggles between fit and 2x at the clicked point
			zoomTo(scale > 1 ? 1 : 2, ...fromCentre(event));
		}
	}
</script>

<svelte:window
	onkeydown={(event) => {
		if (event.key === "Escape") onClose();
		else if (event.key === "+" || event.key === "=") zoomTo(scale * 1.5);
		else if (event.key === "-") zoomTo(scale / 1.5);
		else if (event.key === "0") zoomTo(1);
	}}
/>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={container}
	role="dialog"
	aria-modal="true"
	aria-label={name}
	tabindex="-1"
	class="fixed inset-0 z-40 flex items-center justify-center overflow-hidden bg-black/90"
	onwheel={onWheel}
	onclick={(event) => {
		if (event.target === event.currentTarget) onClose();
	}}
	oncontextmenu={(event) => event.stopPropagation()}
>
	<img
		{src}
		alt={name}
		draggable="false"
		class="max-h-[90dvh] max-w-[90vw] touch-none object-contain select-none {scale > 1
			? 'cursor-grab active:cursor-grabbing'
			: 'cursor-zoom-in'}"
		style="transform: translate({x}px, {y}px) scale({scale});"
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={() => (drag = null)}
	/>

	<div
		class="absolute top-3 right-3 flex items-center gap-1 rounded-md bg-gray-900 p-1 text-white *:flex *:h-8 *:w-8 *:cursor-pointer *:items-center *:justify-center *:rounded-sm *:transition *:hover:bg-white/10"
	>
		<button type="button" aria-label="Zoom out" onclick={() => zoomTo(scale / 1.5)}>
			<Fa icon={faMagnifyingGlassMinus} />
		</button>
		<button
			type="button"
			aria-label="Reset zoom"
			class="w-auto! px-2 text-sm"
			onclick={() => zoomTo(1)}
		>
			{Math.round(scale * 100)}%
		</button>
		<button type="button" aria-label="Zoom in" onclick={() => zoomTo(scale * 1.5)}>
			<Fa icon={faMagnifyingGlassPlus} />
		</button>
		<a
			href={src}
			download={name}
			target="_blank"
			rel="noopener noreferrer"
			aria-label="Download"
		>
			<Fa icon={faDownload} />
		</a>
		<button type="button" aria-label="Close" onclick={onClose}>
			<Fa icon={faClose} />
		</button>
	</div>
</div>
