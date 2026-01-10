<script lang="ts">
    import type { ConnectorType } from '../stores/simulationState.svelte';

    export type ConnectorSide = 'left' | 'right';

    interface Props {
        id: string;
        type: ConnectorType;
        side: ConnectorSide;
        label?: string;
        offsetY?: number;
        active?: boolean;
        connecting?: boolean;
        compatible?: boolean;
        onStartConnect?: (
            id: string,
            type: ConnectorType,
            side: ConnectorSide,
            event: PointerEvent
        ) => void;
        onEndConnect?: (id: string, type: ConnectorType, side: ConnectorSide) => void;
    }

    let {
        id,
        type,
        side,
        label = '',
        offsetY = 0,
        active = false,
        connecting = false,
        compatible = false,
        onStartConnect,
        onEndConnect,
    }: Props = $props();

    let isHovered = $state(false);

    function getConnectorColor(): string {
        switch (type) {
            case 'cable':
            case 'network-cable':
                return '#fbbf24';
            case 'atmo-input':
            case 'atmo-input2':
            case 'network-atmo':
                return '#60a5fa';
            case 'atmo-output':
            case 'atmo-output2':
                return '#f472b6';
            default:
                return '#818cf8';
        }
    }

    function getConnectorIcon(): string {
        switch (type) {
            case 'cable':
            case 'network-cable':
                return '⚡';
            default:
                return '💨';
        }
    }

    function handlePointerDown(e: PointerEvent) {
        e.stopPropagation();
        e.preventDefault();
        if (onStartConnect) {
            onStartConnect(id, type, side, e);
        }
    }

    function handlePointerUp(e: PointerEvent) {
        e.stopPropagation();
        if (onEndConnect) {
            onEndConnect(id, type, side);
        }
    }

    function handlePointerEnter() {
        isHovered = true;
    }

    function handlePointerLeave() {
        isHovered = false;
    }

    const color = $derived(getConnectorColor());
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
    class="connector"
    class:left={side === 'left'}
    class:right={side === 'right'}
    class:active
    class:connecting={connecting && compatible}
    class:compatible
    class:hovered={isHovered}
    style:--connector-color={color}
    style:--offset-y="{offsetY}px"
    onpointerdown={handlePointerDown}
    onpointerup={handlePointerUp}
    onpointerenter={handlePointerEnter}
    onpointerleave={handlePointerLeave}
    title={label || type}
>
    <div class="connector-dot">
        <span class="connector-icon">{getConnectorIcon()}</span>
    </div>
    {#if label && isHovered}
        <div
            class="connector-label"
            class:label-left={side === 'right'}
            class:label-right={side === 'left'}
        >
            {label}
        </div>
    {/if}
</div>

<style>
    .connector {
        position: absolute;
        top: 50%;
        transform: translateY(calc(-50% + var(--offset-y, 0px)));
        cursor: pointer;
        z-index: 10;
        display: flex;
        align-items: center;
    }

    .connector.left {
        left: -8px;
        flex-direction: row-reverse;
    }

    .connector.right {
        right: -8px;
        flex-direction: row;
    }

    .connector-dot {
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background: rgba(0, 0, 0, 0.6);
        border: 2px solid var(--connector-color, #818cf8);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s ease;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    .connector-icon {
        font-size: 8px;
        color: var(--connector-color, #818cf8);
        line-height: 1;
    }

    .connector:hover .connector-dot,
    .connector.hovered .connector-dot {
        transform: scale(1.2);
        background: var(--connector-color, #818cf8);
        box-shadow: 0 0 8px var(--connector-color, #818cf8);
    }

    .connector:hover .connector-icon,
    .connector.hovered .connector-icon {
        color: #000;
    }

    .connector.active .connector-dot {
        background: var(--connector-color, #818cf8);
        box-shadow: 0 0 12px var(--connector-color, #818cf8);
    }

    .connector.connecting .connector-dot {
        animation: pulse 0.8s ease-in-out infinite;
    }

    .connector.compatible .connector-dot {
        border-color: #4ade80;
        box-shadow: 0 0 8px #4ade80;
    }

    .connector-label {
        position: absolute;
        background: rgba(0, 0, 0, 0.85);
        color: #fff;
        font-size: 10px;
        padding: 2px 6px;
        border-radius: 4px;
        white-space: nowrap;
        pointer-events: none;
        z-index: 20;
    }

    .connector-label.label-left {
        right: 100%;
        margin-right: 8px;
    }

    .connector-label.label-right {
        left: 100%;
        margin-left: 8px;
    }

    @keyframes pulse {
        0%,
        100% {
            transform: scale(1);
            opacity: 1;
        }
        50% {
            transform: scale(1.3);
            opacity: 0.7;
        }
    }
</style>
