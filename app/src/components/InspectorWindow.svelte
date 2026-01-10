<script lang="ts">
    import { onMount } from 'svelte';
    import {
        type InspectorWindow,
        bringToFront,
        updateInspectorPosition,
        updateInspectorSize,
        closeInspector,
        toggleMinimized,
    } from '../stores/inspectorState.svelte';

    interface Props {
        window: InspectorWindow;
        title: string;
        icon?: string;
        children?: import('svelte').Snippet;
    }

    let { window, title, icon = '📋', children }: Props = $props();

    let containerEl: HTMLElement | null = $state(null);

    // Dragging state
    let isDragging = $state(false);
    let dragStartX = 0;
    let dragStartY = 0;
    let dragOffsetX = $state(0);
    let dragOffsetY = $state(0);

    // Resizing state
    let isResizing = $state(false);
    let resizeStartX = 0;
    let resizeStartY = 0;
    let resizeStartW = 0;
    let resizeStartH = 0;

    const MIN_WIDTH = 280;
    const MIN_HEIGHT = 200;

    function handleMouseDown(e: MouseEvent) {
        bringToFront(window.id);
    }

    function handleHeaderPointerDown(e: PointerEvent) {
        if (e.button !== 0) return;
        e.preventDefault();
        e.stopPropagation();

        isDragging = true;
        dragStartX = e.clientX;
        dragStartY = e.clientY;
        dragOffsetX = 0;
        dragOffsetY = 0;

        (e.target as HTMLElement).setPointerCapture(e.pointerId);
        bringToFront(window.id);
    }

    function handleHeaderPointerMove(e: PointerEvent) {
        if (!isDragging) return;
        dragOffsetX = e.clientX - dragStartX;
        dragOffsetY = e.clientY - dragStartY;
    }

    function handleHeaderPointerUp(e: PointerEvent) {
        if (!isDragging) return;
        isDragging = false;

        try {
            (e.target as HTMLElement).releasePointerCapture(e.pointerId);
        } catch {}

        const newX = window.x + dragOffsetX;
        const newY = window.y + dragOffsetY;
        updateInspectorPosition(window.id, newX, newY);
        dragOffsetX = 0;
        dragOffsetY = 0;
    }

    function handleResizePointerDown(e: PointerEvent) {
        if (e.button !== 0) return;
        e.preventDefault();
        e.stopPropagation();

        isResizing = true;
        resizeStartX = e.clientX;
        resizeStartY = e.clientY;
        resizeStartW = window.width;
        resizeStartH = window.height;

        (e.target as HTMLElement).setPointerCapture(e.pointerId);
        bringToFront(window.id);
    }

    function handleResizePointerMove(e: PointerEvent) {
        if (!isResizing) return;
        const dx = e.clientX - resizeStartX;
        const dy = e.clientY - resizeStartY;
        const newW = Math.max(MIN_WIDTH, resizeStartW + dx);
        const newH = Math.max(MIN_HEIGHT, resizeStartH + dy);
        updateInspectorSize(window.id, newW, newH);
    }

    function handleResizePointerUp(e: PointerEvent) {
        if (!isResizing) return;
        isResizing = false;
        try {
            (e.target as HTMLElement).releasePointerCapture(e.pointerId);
        } catch {}
    }

    function handleClose() {
        closeInspector(window.id);
    }

    function handleMinimize() {
        toggleMinimized(window.id);
    }

    let visualX = $derived(window.x + (isDragging ? dragOffsetX : 0));
    let visualY = $derived(window.y + (isDragging ? dragOffsetY : 0));
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    bind:this={containerEl}
    class="inspector-window"
    class:minimized={window.minimized}
    class:dragging={isDragging}
    class:resizing={isResizing}
    style="
        left: {visualX}px;
        top: {visualY}px;
        width: {window.width}px;
        height: {window.minimized ? 'auto' : window.height + 'px'};
        z-index: {window.zIndex};
    "
    onmousedown={handleMouseDown}
>
    <!-- Header bar -->
    <div
        class="inspector-header"
        onpointerdown={handleHeaderPointerDown}
        onpointermove={handleHeaderPointerMove}
        onpointerup={handleHeaderPointerUp}
    >
        <span class="inspector-icon">{icon}</span>
        <span class="inspector-title">{title}</span>
        <div class="inspector-controls">
            <button class="control-btn minimize" onclick={handleMinimize} title="Minimize">
                <svg width="10" height="2" viewBox="0 0 10 2">
                    <rect width="10" height="2" fill="currentColor"></rect>
                </svg>
            </button>
            <button class="control-btn close" onclick={handleClose} title="Close">
                <svg width="10" height="10" viewBox="0 0 10 10">
                    <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" fill="none"
                    ></path>
                </svg>
            </button>
        </div>
    </div>

    <!-- Content area -->
    {#if !window.minimized}
        <div class="inspector-content">
            {#if children}
                {@render children()}
            {/if}
        </div>

        <!-- Resize handle -->
        <div
            class="resize-handle"
            onpointerdown={handleResizePointerDown}
            onpointermove={handleResizePointerMove}
            onpointerup={handleResizePointerUp}
        ></div>
    {/if}
</div>

<style>
    .inspector-window {
        position: fixed;
        display: flex;
        flex-direction: column;
        background: rgba(26, 26, 46, 0.88);
        backdrop-filter: blur(6px);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        box-shadow:
            0 18px 60px rgba(0, 0, 0, 0.48),
            0 6px 24px rgba(0, 0, 0, 0.32);
        overflow: hidden;
        font-family:
            'Inter',
            -apple-system,
            BlinkMacSystemFont,
            sans-serif;
    }

    .inspector-window.dragging,
    .inspector-window.resizing {
        user-select: none;
    }

    .inspector-window.minimized {
        height: auto !important;
    }

    .inspector-header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 12px;
        background: linear-gradient(180deg, rgba(255, 255, 255, 0.06) 0%, transparent 100%);
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
        cursor: grab;
        flex-shrink: 0;
    }

    .inspector-header:active {
        cursor: grabbing;
    }

    .inspector-icon {
        font-size: 14px;
    }

    .inspector-title {
        flex: 1;
        font-size: 13px;
        font-weight: 600;
        color: #e0e0e0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .inspector-controls {
        display: flex;
        gap: 6px;
    }

    .control-btn {
        width: 20px;
        height: 20px;
        border: none;
        border-radius: 4px;
        background: rgba(255, 255, 255, 0.08);
        color: rgba(255, 255, 255, 0.6);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s ease;
    }

    .control-btn:hover {
        background: rgba(255, 255, 255, 0.15);
        color: #fff;
    }

    .control-btn.close:hover {
        background: rgba(239, 68, 68, 0.7);
        color: #fff;
    }

    .inspector-content {
        flex: 1;
        overflow: auto;
        min-height: 0;
        scrollbar-width: thin;
        scrollbar-color: rgba(129, 140, 248, 0.9) transparent;
    }

    .inspector-content::-webkit-scrollbar {
        width: 10px;
    }
    .inspector-content::-webkit-scrollbar-track {
        background: transparent;
    }
    .inspector-content::-webkit-scrollbar-thumb {
        background: linear-gradient(180deg, rgba(129, 140, 248, 0.95), rgba(96, 165, 250, 0.95));
        border-radius: 8px;
        border: 2px solid rgba(26, 26, 46, 0.6);
    }
    .inspector-content::-webkit-scrollbar-thumb:hover {
        background: linear-gradient(180deg, rgba(99, 102, 241, 1), rgba(59, 130, 246, 1));
    }

    .inspector-window :global(input[type='number'])::-webkit-outer-spin-button,
    .inspector-window :global(input[type='number'])::-webkit-inner-spin-button {
        -webkit-appearance: none;
        appearance: none;
        margin: 0;
    }
    .inspector-window :global(input[type='number']) {
        -moz-appearance: textfield;
        appearance: textfield;
    }

    .resize-handle {
        position: absolute;
        bottom: 0;
        right: 0;
        width: 16px;
        height: 16px;
        cursor: se-resize;
        background: linear-gradient(
            135deg,
            transparent 50%,
            rgba(255, 255, 255, 0.1) 50%,
            rgba(255, 255, 255, 0.15) 100%
        );
        border-radius: 0 0 10px 0;
    }

    .resize-handle:hover {
        background: linear-gradient(
            135deg,
            transparent 50%,
            rgba(255, 255, 255, 0.2) 50%,
            rgba(255, 255, 255, 0.3) 100%
        );
    }
</style>
