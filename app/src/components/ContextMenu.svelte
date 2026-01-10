<script lang="ts">
    export interface MenuItem {
        id: string;
        label: string;
        icon?: string;
        disabled?: boolean;
        divider?: boolean;
    }

    interface Props {
        x: number;
        y: number;
        items: MenuItem[];
        visible: boolean;
        onSelect?: (itemId: string) => void;
        onClose?: () => void;
    }

    let { x, y, items, visible, onSelect, onClose }: Props = $props();

    function handleItemClick(item: MenuItem) {
        if (item.disabled || item.divider) return;

        if (onSelect) {
            onSelect(item.id);
        }
        if (onClose) {
            onClose();
        }
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Escape' && onClose) {
            onClose();
        }
    }

    function getAdjustedPosition(): { x: number; y: number } {
        const menuWidth = 200;
        const menuHeight = items.length * 36;
        const padding = 10;

        let adjustedX = x;
        let adjustedY = y;

        if (typeof window !== 'undefined') {
            if (x + menuWidth > window.innerWidth - padding) {
                adjustedX = window.innerWidth - menuWidth - padding;
            }
            if (y + menuHeight > window.innerHeight - padding) {
                adjustedY = window.innerHeight - menuHeight - padding;
            }
        }

        return { x: adjustedX, y: adjustedY };
    }

    let position = $derived(getAdjustedPosition());

    let menuEl: HTMLElement | null = $state(null);

    function onPointerDown(e: PointerEvent) {
        if (!menuEl || !menuEl.contains(e.target as Node)) {
            if (onClose) onClose();
        }
    }

    $effect(() => {
        if (visible) {
            window.addEventListener('pointerdown', onPointerDown, true);
            return () => {
                window.removeEventListener('pointerdown', onPointerDown, true);
            };
        }
    });
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if visible}
    <div class="context-menu-backdrop" role="presentation"></div>
    <div
        class="context-menu"
        bind:this={menuEl}
        style:left="{position.x}px"
        style:top="{position.y}px"
        role="menu"
    >
        {#each items as item}
            {#if item.divider}
                <div class="divider"></div>
            {:else}
                <button
                    class="menu-item"
                    class:disabled={item.disabled}
                    role="menuitem"
                    disabled={item.disabled}
                    onclick={() => handleItemClick(item)}
                >
                    {#if item.icon}
                        <span class="icon">{item.icon}</span>
                    {/if}
                    <span class="label">{item.label}</span>
                </button>
            {/if}
        {/each}
    </div>
{/if}

<style>
    .context-menu-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 999;
        pointer-events: none;
    }

    .context-menu {
        position: fixed;
        z-index: 1000;
        min-width: 180px;
        background: #252542;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 6px;
        box-shadow:
            0 8px 32px rgba(0, 0, 0, 0.4),
            0 2px 8px rgba(0, 0, 0, 0.2);
        backdrop-filter: blur(8px);
    }

    .menu-item {
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        padding: 8px 12px;
        border: none;
        background: transparent;
        color: #e0e0e0;
        font-size: 13px;
        text-align: left;
        cursor: pointer;
        border-radius: 6px;
        transition: all 0.1s ease;
    }

    .menu-item:hover:not(.disabled) {
        background: rgba(99, 102, 241, 0.2);
        color: #fff;
    }

    .menu-item:active:not(.disabled) {
        background: rgba(99, 102, 241, 0.3);
    }

    .menu-item.disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .menu-item:focus-visible {
        outline: 2px solid #818cf8;
        outline-offset: -2px;
    }

    .icon {
        font-size: 14px;
        width: 20px;
        text-align: center;
    }

    .label {
        flex: 1;
    }

    .divider {
        height: 1px;
        background: rgba(255, 255, 255, 0.1);
        margin: 6px 8px;
    }
</style>
