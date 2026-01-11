<script lang="ts">
    import { onDestroy, type Snippet } from 'svelte';
    import { NODE_W, NODE_H } from '../lib/constants';

    interface Props {
        id: string | number;
        x?: number;
        y?: number;
        width?: number;
        height?: number;
        gridX?: number;
        gridY?: number;
        scale?: number;
        selected?: boolean;
        nodeClass?: string;
        onMove?: (id: any, x: number, y: number) => void;
        onSelect?: (id: any) => void;
        onInspect?: (id: any) => void;
        onClick?: (id: any) => void;
        children?: Snippet;
    }

    let {
        id,
        x = 0,
        y = 0,
        gridX = undefined,
        gridY = undefined,
        scale = 1,
        selected = false,
        nodeClass = '',
        onMove = undefined,
        onSelect = undefined,
        onInspect = undefined,
        onClick = undefined,
        children,
    }: Props = $props();

    let root: SVGGElement | null = null;

    let isDragging = $state(false);
    let dragStartX = 0;
    let dragStartY = 0;
    let initialX = 0;
    let initialY = 0;

    let rafId: number | null = null;
    let pointerDx = 0;
    let pointerDy = 0;
    let visualDx = $state(0);
    let visualDy = $state(0);
    let dragScale = 1;

    let lastClientX = 0;
    let lastClientY = 0;

    function rafLoop() {
        rafId = requestAnimationFrame(rafLoop);
    }

    function handlePointerDown(e: PointerEvent) {
        if (e.button !== 0) return;
        (e.target as Element).setPointerCapture?.(e.pointerId);
        isDragging = true;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
        lastClientX = e.clientX;
        lastClientY = e.clientY;
        pointerDx = 0;
        pointerDy = 0;
        visualDx = 0;
        visualDy = 0;
        initialX = typeof gridX === 'number' ? gridX : x;
        initialY = typeof gridY === 'number' ? gridY : y;
        dragScale = scale;

        if (rafId == null) rafId = requestAnimationFrame(rafLoop);

        if (onSelect) onSelect(id);
        e.stopPropagation();
        e.preventDefault();
    }

    function handlePointerMove(e: PointerEvent) {
        if (!isDragging) return;
        lastClientX = e.clientX;
        lastClientY = e.clientY;
        pointerDx = e.clientX - dragStartX;
        pointerDy = e.clientY - dragStartY;
        visualDx = pointerDx / (dragScale || 1);
        visualDy = pointerDy / (dragScale || 1);

        if (onMove) {
            const dxGrid = visualDx;
            const dyGrid = visualDy;
            const newGridX = initialX + dxGrid;
            const newGridY = initialY + dyGrid;

            onMove(id, newGridX, newGridY);

            initialX = newGridX;
            initialY = newGridY;
            dragStartX = e.clientX;
            dragStartY = e.clientY;
            pointerDx = 0;
            pointerDy = 0;
            visualDx = 0;
            visualDy = 0;
        }
    }

    $effect(() => {
        if (isDragging && scale !== dragScale) {
            const vx = visualDx;
            const vy = visualDy;
            dragStartX = lastClientX - vx * scale;
            dragStartY = lastClientY - vy * scale;
            dragScale = scale;
        }
    });

    function handlePointerUp(e: PointerEvent) {
        if (!isDragging) return;
        isDragging = false;
        try {
            (e.target as Element).releasePointerCapture?.(e.pointerId);
        } catch {}
        if (rafId != null) cancelAnimationFrame(rafId);
        rafId = null;

        const dxGrid = (pointerDx || 0) / (dragScale || 1);
        const dyGrid = (pointerDy || 0) / (dragScale || 1);
        const newGridX = initialX + dxGrid;
        const newGridY = initialY + dyGrid;

        if (onMove) onMove(id, newGridX, newGridY);

        pointerDx = 0;
        pointerDy = 0;
        visualDx = 0;
        visualDy = 0;
    }

    function handleDoubleClick(e: MouseEvent) {
        e.stopPropagation();
        e.preventDefault();
        if (onInspect) onInspect(id);
        if (onClick) onClick(id);
    }

    onDestroy(() => {
        if (rafId != null) cancelAnimationFrame(rafId);
    });
</script>

<svelte:window onpointermove={handlePointerMove} onpointerup={handlePointerUp} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<g
    bind:this={root}
    class="svg-node {nodeClass}"
    class:dragging={isDragging}
    class:selected
    transform="translate({visualDx}, {visualDy})"
    onpointerdown={handlePointerDown}
    ondblclick={handleDoubleClick}
>
    {#if children}
        {@render children()}
    {/if}
</g>

<style>
    .svg-node {
        cursor: move;
    }

    .svg-node.dragging {
        cursor: grabbing;
    }

    .svg-node :global(.node-hover) {
        opacity: 0;
        transition: opacity 120ms ease;
        pointer-events: none;
    }

    .svg-node:hover :global(.node-hover) {
        opacity: 1;
    }

    .svg-node.dragging :global(.node-hover) {
        opacity: 0 !important;
    }
</style>
