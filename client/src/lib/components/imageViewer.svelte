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
	let image = $state<HTMLImageElement>();
	let scale = $state(1);
	let x = $state(0);
	let y = $state(0);

	// every finger/mouse button currently down, by pointerId, at its last known position
	const pointers = new Map<number, { x: number; y: number }>();
	// whether the current gesture panned or pinched; if not, releasing it counts as a tap
	let gestureMoved = false;
	let tapTarget: EventTarget | null = null;

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

	function fromCentre(point: { clientX: number; clientY: number }) {
		const rect = container!.getBoundingClientRect();
		return [
			point.clientX - rect.left - rect.width / 2,
			point.clientY - rect.top - rect.height / 2
		] as const;
	}

	function pinchState() {
		const [a, b] = [...pointers.values()];
		return {
			distance: Math.hypot(a.x - b.x, a.y - b.y),
			centre: fromCentre({ clientX: (a.x + b.x) / 2, clientY: (a.y + b.y) / 2 })
		};
	}

	function onWheel(event: WheelEvent) {
		event.preventDefault();
		zoomTo(scale * (event.deltaY < 0 ? 1.2 : 1 / 1.2), ...fromCentre(event));
	}

	function onPointerDown(event: PointerEvent) {
		// leave the toolbar's buttons alone so their clicks still work
		if ((event.target as Element).closest("[data-toolbar]")) return;
		if (event.pointerType === "mouse" && event.button !== 0) return;

		if (pointers.size == 0) {
			gestureMoved = false;
			tapTarget = event.target;
		}
		pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
		container!.setPointerCapture(event.pointerId);
	}

	function onPointerMove(event: PointerEvent) {
		const previous = pointers.get(event.pointerId);
		if (!previous) return;

		if (pointers.size >= 2) {
			const before = pinchState();
			pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
			const after = pinchState();
			gestureMoved = true;

			if (before.distance > 0) {
				zoomTo(scale * (after.distance / before.distance), ...before.centre);
			}
			// moving both fingers together also pans
			if (scale > 1) {
				x += after.centre[0] - before.centre[0];
				y += after.centre[1] - before.centre[1];
			}
			return;
		}

		const dx = event.clientX - previous.x;
		const dy = event.clientY - previous.y;
		if (!gestureMoved && Math.hypot(dx, dy) < 4) return;

		gestureMoved = true;
		pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
		if (scale > 1) {
			x += dx;
			y += dy;
		}
	}

	function onPointerUp(event: PointerEvent) {
		if (!pointers.delete(event.pointerId)) return;
		if (pointers.size > 0 || gestureMoved) return;

		if (tapTarget === image) {
			// a tap on the image toggles between fit and 2x at the tapped point
			zoomTo(scale > 1 ? 1 : 2, ...fromCentre(event));
		} else if (tapTarget === container) {
			onClose();
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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	bind:this={container}
	role="dialog"
	aria-modal="true"
	aria-label={name}
	tabindex="-1"
	class="fixed inset-0 z-40 flex touch-none items-center justify-center overflow-hidden bg-black/90"
	onwheel={onWheel}
	onpointerdown={onPointerDown}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	onpointercancel={(event) => {
		pointers.delete(event.pointerId);
		gestureMoved = true;
	}}
	oncontextmenu={(event) => event.stopPropagation()}
>
	<img
		bind:this={image}
		{src}
		alt={name}
		draggable="false"
		class="max-h-[90dvh] max-w-[90vw] object-contain select-none {scale > 1
			? 'cursor-grab active:cursor-grabbing'
			: 'cursor-zoom-in'}"
		style="transform: translate({x}px, {y}px) scale({scale});"
	/>

	<div
		data-toolbar
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
