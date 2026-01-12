<script lang="ts">
    import type { Connection } from '../stores/simulationState.svelte';
    import type { ConnectorSide } from './Connector.svelte';
    import { CONNECTION_COLORS } from '../lib/constants';

    interface ConnectorPosition {
        x: number;
        y: number;
    }

    interface Props {
        connection: Connection;
        sourcePos: ConnectorPosition;
        targetPos: ConnectorPosition;
        sourceSide?: ConnectorSide;
        targetSide?: ConnectorSide;
        selected?: boolean;
        onContextMenu?: (connection: Connection, event: MouseEvent) => void;
    }

    let {
        connection,
        sourcePos,
        targetPos,
        sourceSide = 'left',
        targetSide = 'left',
        selected = false,
        onContextMenu,
    }: Props = $props();

    function handleContextMenu(e: MouseEvent) {
        e.preventDefault();
        e.stopPropagation();
        if (onContextMenu) {
            onContextMenu(connection, e);
        }
    }

    const path = $derived.by(() => {
        const offset = 2;

        const sX = sourcePos.x + (sourceSide === 'left' ? offset : -offset);
        const sY = sourcePos.y;
        const tX = targetPos.x + (targetSide === 'left' ? offset : -offset);
        const tY = targetPos.y;

        const dx2 = tX - sX;
        const controlPointOffset = Math.min(Math.abs(dx2) * 0.5, 100);

        const c1x = sX + (sourceSide === 'left' ? -controlPointOffset : controlPointOffset);
        const c1y = sY;
        const c2x = tX + (targetSide === 'left' ? -controlPointOffset : controlPointOffset);
        const c2y = tY;

        return `M ${sX} ${sY} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${tX} ${tY}`;
    });

    const color = $derived.by(() => {
        switch (connection.networkType) {
            case 'cable':
                return CONNECTION_COLORS.cable;
            case 'atmospheric':
                if (
                    connection.deviceConnectorType === 'atmo-output' ||
                    connection.deviceConnectorType === 'atmo-output2'
                ) {
                    return CONNECTION_COLORS.atmoOutput;
                }
                return CONNECTION_COLORS.atmoInput;
            default:
                return CONNECTION_COLORS.default;
        }
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- Invisible wider path for easier clicking -->
<path
    d={path}
    stroke="transparent"
    stroke-width="20"
    fill="none"
    class="connection-hitbox"
    oncontextmenu={handleContextMenu}
></path>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- Visible connection line -->
<path
    d={path}
    stroke={color}
    stroke-width={selected ? 4 : 3}
    fill="none"
    class="connection-line"
    class:selected
    stroke-linecap="round"
    oncontextmenu={handleContextMenu}
></path>

<path
    d={path}
    stroke={color}
    stroke-width="3"
    fill="none"
    class="connection-flow"
    stroke-dasharray="8 4"
    stroke-linecap="round"
></path>

<style>
    .connection-hitbox {
        cursor: pointer;
        pointer-events: stroke;
    }

    .connection-line {
        pointer-events: stroke;
        cursor: pointer;
        transition: stroke-width 0.15s ease;
        filter: drop-shadow(0 0 4px rgba(0, 0, 0, 0.5));
    }

    .connection-line:hover {
        stroke-width: 5;
        filter: drop-shadow(0 0 8px currentColor);
    }

    .connection-hitbox:hover + .connection-line {
        stroke-width: 5;
        filter: drop-shadow(0 0 8px currentColor);
    }

    .connection-flow {
        pointer-events: none;
        opacity: 0.6;
        animation: flow 1s linear infinite;
    }

    @keyframes flow {
        from {
            stroke-dashoffset: 0;
        }
        to {
            stroke-dashoffset: -12;
        }
    }
</style>
