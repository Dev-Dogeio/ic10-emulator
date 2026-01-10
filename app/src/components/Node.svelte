<script lang="ts">
    import { onDestroy } from 'svelte';

    export let id: string | number;
    export let x: number = 0;
    export let y: number = 0;
    export let gridX: number | undefined = undefined;
    export let gridY: number | undefined = undefined;
    export let scale: number = 1;
    export let selected: boolean = false;
    export let nodeClass: string = '';

    export let onMove: ((id: any, x: number, y: number) => void) | undefined = undefined;
    export let onSelect: ((id: any) => void) | undefined = undefined;
    export let onInspect: ((id: any) => void) | undefined = undefined;
    export let onClick: ((id: any) => void) | undefined = undefined;

    let root: HTMLElement | null = null;

    let isDragging = false;
    let dragStartX = 0;
    let dragStartY = 0;
    let initialX = 0;
    let initialY = 0;

    let rafId: number | null = null;
    let pointerDx = 0;
    let pointerDy = 0;
    let visualDx = 0;
    let visualDy = 0;
    let dragScale = 1;

    let lastClientX = 0;
    let lastClientY = 0;

    function rafLoop() {
        if (root) root.style.transform = `translate(${visualDx}px, ${visualDy}px) translateZ(0)`;
        rafId = requestAnimationFrame(rafLoop);
    }

    function handlePointerDown(e: PointerEvent) {
        if (e.button !== 0) return;
        (root || (e.target as Element)).setPointerCapture?.(e.pointerId);
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

    $: if (isDragging && scale !== dragScale) {
        const vx = visualDx;
        const vy = visualDy;
        dragStartX = lastClientX - vx * scale;
        dragStartY = lastClientY - vy * scale;
        dragScale = scale;
    }

    function handlePointerUp(e: PointerEvent) {
        if (!isDragging) return;
        isDragging = false;
        try {
            (root || (e.target as Element)).releasePointerCapture?.(e.pointerId);
        } catch {}
        if (rafId != null) cancelAnimationFrame(rafId);
        rafId = null;

        const dxGrid = (pointerDx || 0) / (dragScale || 1);
        const dyGrid = (pointerDy || 0) / (dragScale || 1);
        const newGridX = initialX + dxGrid;
        const newGridY = initialY + dyGrid;

        if (root) {
            root.style.transition = '';
            root.style.transform = '';
        }

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

<div
    bind:this={root}
    class={nodeClass}
    class:dragging={isDragging}
    class:selected
    onpointerdown={handlePointerDown}
    ondblclick={handleDoubleClick}
    {...$$restProps}
>
    <slot />
</div>
