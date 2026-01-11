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
    let isPanning = false;
    let startX = 0;
    let startY = 0;
    let startOffsetX = 0;
    let startOffsetY = 0;

    function handleMouseDown(e: MouseEvent) {
        if (e.button === 1 || (e.button === 0 && !e.ctrlKey && !e.metaKey && !e.shiftKey)) {
            isPanning = true;
            startX = e.clientX;
            startY = e.clientY;
            startOffsetX = offsetX;
            startOffsetY = offsetY;
            e.preventDefault();
        }
    }

    function handleMouseMove(e: MouseEvent) {
        if (isPanning) {
            offsetX = startOffsetX + (e.clientX - startX);
            offsetY = startOffsetY + (e.clientY - startY);
        }
    }

    function handleMouseUp(e: MouseEvent) {
        if (e.button === 1 || e.button === 0) {
            isPanning = false;
        }
    }

    function handleWheel(e: WheelEvent) {
        e.preventDefault();

        const rect = container.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        const gridX = (mouseX - offsetX) / scale;
        const gridY = (mouseY - offsetY) / scale;

        const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
        const newScale = Math.min(Math.max(scale * zoomFactor, 0.1), 5);

        offsetX = mouseX - gridX * newScale;
        offsetY = mouseY - gridY * newScale;
        scale = newScale;
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

    export function screenToGrid(screenX: number, screenY: number): { x: number; y: number } {
        const rect = container?.getBoundingClientRect();
        if (!rect) return { x: 0, y: 0 };

        const x = (screenX - rect.left - offsetX) / scale;
        const y = (screenY - rect.top - offsetY) / scale;
        return { x, y };
    }

    let width = $state(0);
    let height = $state(0);

    function updateSize() {
        const r = container.getBoundingClientRect();
        width = r.width;
        height = r.height;
    }

    function getGridPattern(): string {
        const gridPixelSize = Math.max(1, Math.round(gridSize * scale));
        return `
            repeating-linear-gradient(
                0deg,
                var(--grid-line-color) 0px,
                var(--grid-line-color) 1px,
                transparent 1px,
                transparent ${gridPixelSize}px
            ),
            repeating-linear-gradient(
                90deg,
                var(--grid-line-color) 0px,
                var(--grid-line-color) 1px,
                transparent 1px,
                transparent ${gridPixelSize}px
            )
        `;
    }

    function getBackgroundPosition(): string {
        const gridPixelSize = Math.max(1, Math.round(gridSize * scale));
        const rx = ((offsetX % gridPixelSize) + gridPixelSize) % gridPixelSize;
        const ry = ((offsetY % gridPixelSize) + gridPixelSize) % gridPixelSize;
        return `${rx}px ${ry}px`;
    }

    let resizeObserver: ResizeObserver | null = null;

    onMount(() => {
        updateSize();
        if (container && typeof ResizeObserver !== 'undefined') {
            resizeObserver = new ResizeObserver(updateSize);
            resizeObserver.observe(container);
        }
        window.addEventListener('resize', updateSize);

        return () => {
            resizeObserver?.disconnect();
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
    onmousedown={handleMouseDown}
    onmousemove={handleMouseMove}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
    onwheel={handleWheel}
    oncontextmenu={handleContextMenu}
    style:background-image={getGridPattern()}
    style:background-position={getBackgroundPosition()}
>
    <!-- SVG layer for paths -->
    <svg
        class="grid-viewport"
        viewBox="{-offsetX / scale} {-offsetY / scale} {width / scale} {height / scale}"
        preserveAspectRatio="xMinYMin meet"
        xmlns="http://www.w3.org/2000/svg"
    >
        {#if svgContent}
            {@render svgContent()}
        {/if}
    </svg>

    <!-- DOM layer for nodes -->
    <div
        class="nodes-layer"
        style:transform="translate({offsetX}px, {offsetY}px) scale({scale})"
        style:transform-origin="0 0"
    >
        {#if nodeContent}
            {@render nodeContent()}
        {/if}
    </div>
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
        pointer-events: none;
    }

    .nodes-layer {
        position: absolute;
        top: 0;
        left: 0;
        width: 0;
        height: 0;
        pointer-events: none;
    }

    .nodes-layer > :global(*) {
        pointer-events: auto;
    }
</style>
