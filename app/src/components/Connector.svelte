<script lang="ts">
    import type { ConnectorType } from '../stores/simulationState.svelte';
    import { NODE_W, NODE_H, CONNECTION_COLORS, UI_COLORS, LABEL_BG } from '../lib/constants';

    export type ConnectorSide = 'left' | 'right';

    interface Props {
        id: string;
        type: ConnectorType;
        side: ConnectorSide;
        label?: string;
        offsetY?: number;
        nodeWidth?: number;
        nodeHeight?: number;
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
        nodeWidth = NODE_W,
        nodeHeight = NODE_H,
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
                return CONNECTION_COLORS.cable;
            case 'atmo-input':
            case 'atmo-input2':
            case 'network-atmo':
                return CONNECTION_COLORS.atmoInput;
            case 'atmo-output':
            case 'atmo-output2':
                return CONNECTION_COLORS.atmoOutput;
            default:
                return CONNECTION_COLORS.default;
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
    const icon = $derived(getConnectorIcon());

    const cx = $derived(side === 'left' ? 0 : nodeWidth);
    const cy = $derived(nodeHeight / 2 + offsetY);
    const radius = 8;

    const labelX = $derived(side === 'left' ? cx - 15 : cx + 15);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<g
    class="connector"
    class:active
    class:connecting={connecting && compatible}
    class:compatible
    class:hovered={isHovered}
    onpointerdown={handlePointerDown}
    onpointerup={handlePointerUp}
    onpointerenter={handlePointerEnter}
    onpointerleave={handlePointerLeave}
>
    <defs>
        <filter id="glow-filter-{id}" x="-50%" y="-50%" width="200%" height="200%">
            <feGaussianBlur stdDeviation="4" result="blur"></feGaussianBlur>
            <feMerge>
                <feMergeNode in="blur"></feMergeNode>
                <feMergeNode in="SourceGraphic"></feMergeNode>
            </feMerge>
        </filter>
    </defs>

    {#if connecting && compatible}
        <circle
            {cx}
            {cy}
            r={radius * 1.4}
            fill={UI_COLORS.success}
            fill-opacity="0.18"
            filter="url(#glow-filter-{id})"
            class="connector-glow"
            pointer-events="none"
        >
            <animate
                attributeName="r"
                values={`${radius * 1.4};${radius * 1.6};${radius * 1.4}`}
                dur="900ms"
                repeatCount="indefinite"
            ></animate>
            <animate
                attributeName="fill-opacity"
                values="0.18;0.42;0.18"
                dur="900ms"
                repeatCount="indefinite"
            ></animate>
        </circle>
    {/if}

    <!-- Connector dot -->
    <circle
        {cx}
        {cy}
        r={radius}
        fill={isHovered || active ? color : 'rgba(0, 0, 0, 0.25)'}
        stroke={compatible ? UI_COLORS.success : color}
        stroke-width="2"
        class="connector-dot"
    >
        {#if connecting && compatible}
            <animate
                attributeName="r"
                values={`${radius};${radius * 1.2};${radius}`}
                dur="900ms"
                repeatCount="indefinite"
            ></animate>
        {/if}
    </circle>

    <!-- Icon text -->
    <text
        x={cx}
        y={cy}
        text-anchor="middle"
        dominant-baseline="central"
        font-size="8"
        fill={isHovered || active ? '#000' : color}
        class="connector-icon"
        pointer-events="none"
    >
        {icon}
    </text>

    <!-- Label on hover -->
    {#if label && isHovered}
        <g class="connector-label">
            <rect
                x={side === 'left' ? labelX - 40 : labelX - 5}
                y={cy - 8}
                width="45"
                height="16"
                rx="7"
                fill={LABEL_BG}
            ></rect>
            <text
                x={side === 'left' ? labelX - 17.5 : labelX + 17.5}
                y={cy}
                text-anchor="middle"
                dominant-baseline="central"
                font-size="9"
                fill="#fff"
                pointer-events="none"
            >
                {label}
            </text>
        </g>
    {/if}
</g>

<style>
    .connector {
        cursor: pointer;
    }

    .connector-dot {
        transition: fill 0.15s ease;
        shape-rendering: geometricPrecision;
        vector-effect: non-scaling-stroke;
        will-change: r;
    }

    .connector-glow {
        pointer-events: none;
        shape-rendering: geometricPrecision;
        will-change: r, fill-opacity;
    }
</style>
