<script lang="ts">
    import { onMount, type Snippet } from 'svelte';

    interface Props {
        offsetX?: number;
        offsetY?: number;
        scale?: number;
        gridSize?: number;
        svgContent?: Snippet;
        nodeContent?: Snippet;
        onContextMenu?: (event: { x: number; y: number; gridX: number; gridY: number }) => void;
    }

    let {
        offsetX = $bindable(0),
        offsetY = $bindable(0),
        scale = $bindable(1),
        gridSize = 45,
        svgContent,
        nodeContent,
        onContextMenu,
    }: Props = $props();

    let container: HTMLDivElement;
    let svgEl: SVGSVGElement | null = null;
    let isPanning = false;
    let startX = 0;
    let startY = 0;
    let startOffsetX = 0;
    let startOffsetY = 0;
    let pendingOffsetX = 0;
    let pendingOffsetY = 0;
    let pendingScale = scale;
    let rafId: number | null = null;
    let cachedRect: DOMRect | null = null;
    let wheelTimeout: number | null = null;

    function updateTransforms() {
        offsetX = pendingOffsetX;
        offsetY = pendingOffsetY;
        scale = pendingScale;
        rafId = null;
    }

    function commitTransforms() {
        offsetX = pendingOffsetX;
        offsetY = pendingOffsetY;
        scale = pendingScale;

        if (wheelTimeout !== null) {
            clearTimeout(wheelTimeout);
            wheelTimeout = null;
        }
    }

    let activePointerId: number | null = null;

    function handlePointerDown(e: PointerEvent) {
        if (e.button === 1 || (e.button === 0 && !e.ctrlKey && !e.metaKey && !e.shiftKey)) {
            isPanning = true;
            startX = e.clientX;
            startY = e.clientY;
            startOffsetX = offsetX;
            startOffsetY = offsetY;
            // Initialize pending values so DOM updates are consistent during drag
            pendingOffsetX = offsetX;
            pendingOffsetY = offsetY;
            pendingScale = scale;
            cachedRect = container?.getBoundingClientRect() ?? null;
            activePointerId = e.pointerId;
            // Capture the pointer so we continue to get move events even if the cursor leaves the element
            try {
                container?.setPointerCapture(activePointerId);
            } catch {}
            e.preventDefault();
        }
    }

    function handlePointerMove(e: PointerEvent) {
        if (isPanning && e.pointerId === activePointerId) {
            pendingOffsetX = startOffsetX + (e.clientX - startX);
            pendingOffsetY = startOffsetY + (e.clientY - startY);

            if (rafId === null) {
                rafId = requestAnimationFrame(updateTransforms);
            }
        }
    }

    function handlePointerUp(e: PointerEvent) {
        if (isPanning && e.pointerId === activePointerId) {
            isPanning = false;
            if (rafId !== null) {
                cancelAnimationFrame(rafId);
                rafId = null;
            }
            commitTransforms();
            try {
                container?.releasePointerCapture(e.pointerId);
            } catch {}
            activePointerId = null;
        }
    }

    function handleWheel(e: WheelEvent) {
        e.preventDefault();

        // Use cached rect to avoid layout reads on every wheel event
        if (!cachedRect) cachedRect = container?.getBoundingClientRect() ?? null;
        const rect = cachedRect!;
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        const gridX = (mouseX - pendingOffsetX) / pendingScale;
        const gridY = (mouseY - pendingOffsetY) / pendingScale;

        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        const newScale = Math.min(Math.max(pendingScale * zoomFactor, 0.1), 5);

        pendingScale = newScale;
        pendingOffsetX = mouseX - gridX * newScale;
        pendingOffsetY = mouseY - gridY * newScale;

        if (rafId === null) {
            rafId = requestAnimationFrame(updateTransforms);
        }

        if (wheelTimeout !== null) clearTimeout(wheelTimeout);
        wheelTimeout = window.setTimeout(() => commitTransforms(), 120);
    }

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();

        const rect = container.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        const gridX = (mouseX - offsetX) / scale;
        const gridY = (mouseY - offsetY) / scale;

        if (onContextMenu) {
            onContextMenu({
                x: e.clientX,
                y: e.clientY,
                gridX,
                gridY,
            });
        }
    }

    let width = $state(0);
    let height = $state(0);

    function updateSize() {
        const r = container.getBoundingClientRect();
        width = r.width;
        height = r.height;
        cachedRect = r;
    }

    function getGridPattern(): string {
        return `
            linear-gradient(
                var(--grid-line-color) 1px,
                transparent 1px
            ),
            linear-gradient(
                90deg,
                var(--grid-line-color) 0px,
                var(--grid-line-color) 1px,
                transparent 1px
            )
        `;
    }

    function getBackgroundPosition(): string {
        const gridPixelSize = Math.max(1, Math.round(gridSize * scale));
        const rx = Math.round(offsetX) % gridPixelSize;
        const ry = Math.round(offsetY) % gridPixelSize;
        return `${rx}px ${ry}px`;
    }

    function getBackgroundSize(): string {
        const gridPixelSize = Math.max(1, Math.round(gridSize * scale));
        return `${gridPixelSize}px ${gridPixelSize}px`;
    }

    onMount(() => {
        updateSize();
        pendingOffsetX = offsetX;
        pendingOffsetY = offsetY;
        pendingScale = scale;
        cachedRect = container?.getBoundingClientRect() ?? null;

        window.addEventListener('resize', updateSize);

        return () => {
            window.removeEventListener('resize', updateSize);
        };
    });
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    class="grid-container"
    bind:this={container}
    role="application"
    aria-label="Grid canvas"
    tabindex="0"
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    onpointerleave={handlePointerUp}
    onwheel={handleWheel}
    oncontextmenu={handleContextMenu}
    style:background-image={getGridPattern()}
    style:background-position={getBackgroundPosition()}
    style:background-size={getBackgroundSize()}
>
    <!-- SVG layer for paths and nodes -->
    <svg
        class="grid-viewport"
        bind:this={svgEl}
        viewBox={`${-offsetX / scale} ${-offsetY / scale} ${width / scale} ${height / scale}`}
        preserveAspectRatio="xMinYMin meet"
        xmlns="http://www.w3.org/2000/svg"
    >
        {#if svgContent}
            {@render svgContent()}
        {/if}
        {#if nodeContent}
            {@render nodeContent()}
        {/if}
    </svg>
</div>

<style>
    .grid-container {
        --grid-line-color: rgba(255, 255, 255, 0.08);

        position: relative;
        width: 100%;
        height: 100%;
        background-color: #1a1a2e;
        overflow: hidden;
        cursor: grab;
        user-select: none;
    }

    .grid-container:active {
        cursor: grabbing;
    }

    .grid-viewport {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        overflow: visible;
        will-change: transform;
        backface-visibility: hidden;
        image-rendering: -webkit-optimize-contrast;
        image-rendering: crisp-edges;
        -webkit-font-smoothing: antialiased;
    }

    .grid-viewport :global(text) {
        text-rendering: geometricPrecision;
    }

    .grid-viewport :global(rect),
    .grid-viewport :global(circle),
    .grid-viewport :global(line),
    .grid-viewport :global(path) {
        shape-rendering: geometricPrecision;
        vector-effect: non-scaling-stroke;
    }
</style>
