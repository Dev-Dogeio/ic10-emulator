<script lang="ts">
    import type { WasmCableNetwork, WasmAtmosphericNetwork } from '../../pkg/ic10_emulator';
    import type { ConnectorType } from '../stores/simulationState.svelte';
    import Connector, { type ConnectorSide } from './Connector.svelte';

    export type NetworkType = 'cable' | 'atmospheric';

    export interface NetworkNodeData {
        id: string;
        type: NetworkType;
        name: string;
        network: WasmCableNetwork | WasmAtmosphericNetwork;
        managerId: number;
    }

    interface Props {
        data: NetworkNodeData;
        x: number;
        y: number;
        gridX?: number;
        gridY?: number;
        scale: number;
        selected?: boolean;
        connectingType?: ConnectorType | null;
        onMove?: (id: string, x: number, y: number) => void;
        onSelect?: (id: string) => void;
        onStartConnect?: (
            networkId: string,
            connectorId: string,
            connectorType: ConnectorType,
            side: ConnectorSide,
            event: PointerEvent
        ) => void;
        onEndConnect?: (
            networkId: string,
            connectorId: string,
            connectorType: ConnectorType,
            side: ConnectorSide
        ) => void;
        onInspect?: (id: string) => void;
    }
    let {
        data,
        x = $bindable(0),
        y = $bindable(0),
        gridX = undefined,
        gridY = undefined,
        scale = $bindable(1),
        selected = false,
        connectingType = null,
        onMove,
        onSelect,
        onStartConnect,
        onEndConnect,
        onInspect,
    }: Props = $props();

    import Node from './Node.svelte';

    function getNetworkInfo(): string {
        if (data.type === 'cable') {
            const cable = data.network as WasmCableNetwork;
            return `${cable.device_count()} devices`;
        } else {
            const atmo = data.network as WasmAtmosphericNetwork;
            const pressure = atmo.pressure().toFixed(1);
            const temp = atmo.temperature().toFixed(1);
            const volume = atmo.total_volume().toFixed(1);
            return `${pressure} kPa, ${temp} K, ${volume} L`;
        }
    }

    function getIcon(): string {
        return data.type === 'cable' ? '🔌' : '💨';
    }

    let networkConnectorType: ConnectorType = $derived(
        data.type === 'cable' ? 'network-cable' : 'network-atmo'
    );

    function isCompatible(): boolean {
        if (!connectingType) return false;
        if (data.type === 'cable') {
            return connectingType === 'cable';
        } else {
            return (
                connectingType === 'atmo-input' ||
                connectingType === 'atmo-input2' ||
                connectingType === 'atmo-output' ||
                connectingType === 'atmo-output2'
            );
        }
    }

    function handleConnectorStart(
        connectorId: string,
        type: ConnectorType,
        side: ConnectorSide,
        event: PointerEvent
    ) {
        if (onStartConnect) {
            onStartConnect(data.id, connectorId, type, side, event);
        }
    }

    function handleConnectorEnd(connectorId: string, type: ConnectorType, side: ConnectorSide) {
        if (onEndConnect) {
            onEndConnect(data.id, connectorId, type, side);
        }
    }
</script>

<Node
    id={data.id}
    {gridX}
    {gridY}
    {scale}
    {selected}
    {onMove}
    {onSelect}
    {onInspect}
    nodeClass={`network-node ${data.type === 'cable' ? 'cable' : 'atmospheric'}`}
>
    <!-- Left connector -->
    <Connector
        id={`network-${data.id}-left`}
        type={networkConnectorType}
        side="left"
        label={data.type === 'cable' ? 'Network' : 'Atmo'}
        offsetY={0}
        compatible={isCompatible()}
        connecting={connectingType !== null}
        onStartConnect={handleConnectorStart}
        onEndConnect={handleConnectorEnd}
    />

    <!-- Right connector -->
    <Connector
        id={`network-${data.id}-right`}
        type={networkConnectorType}
        side="right"
        label={data.type === 'cable' ? 'Network' : 'Atmo'}
        offsetY={0}
        compatible={isCompatible()}
        connecting={connectingType !== null}
        onStartConnect={handleConnectorStart}
        onEndConnect={handleConnectorEnd}
    />

    <div class="network-header">
        <span class="network-icon">{getIcon()}</span>
        <span class="network-name"
            >{data.type === 'cable'
                ? `Network ${data.managerId}`
                : `Atmosphere ${data.managerId}`}</span
        >
    </div>
    <div class="network-type">
        {data.type === 'cable' ? 'Cable Network' : 'Atmospheric Network'}
    </div>
    <div class="network-info">{getNetworkInfo()}</div>
</Node>

<style>
    :global(.network-node.cable) {
        --network-color: #fbbf24;
    }

    :global(.network-node.atmospheric) {
        --network-color: #60a5fa;
    }

    :global(.network-node) {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 4px;
        cursor: move;
        user-select: none;
        filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.3));
        background: linear-gradient(
            135deg,
            rgba(255, 255, 255, 0.08) 0%,
            rgba(255, 255, 255, 0.03) 100%
        );
        border: 2px solid var(--network-color);
        border-radius: 12px;
        padding: 11px 16px;
        width: 100%;
        height: 100%;
        max-width: none;
        max-height: none;
        margin: 0;
        box-sizing: border-box;
        min-width: 0;
        min-height: 0;
        transition:
            background 0.15s ease,
            border-color 0.15s ease,
            box-shadow 0.15s ease;
        overflow: visible;
        box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.02);
    }

    :global(.network-node:focus-visible) {
        outline: 2px solid var(--network-color);
        outline-offset: 2px;
    }

    :global(.network-node.dragging) {
        cursor: grabbing;
        z-index: 100;
        transition: none !important;
        filter: none !important;
        box-shadow: none !important;
    }

    :global(.network-node.selected),
    :global(.network-node.dragging) {
        border-color: #818cf8;
    }

    :global(.network-node:hover):not(.selected):not(.dragging) {
        background: linear-gradient(
            135deg,
            rgba(255, 255, 255, 0.12) 0%,
            rgba(255, 255, 255, 0.05) 100%
        );
    }

    .network-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 4px;
        width: 100%;
    }

    .network-icon {
        font-size: 18px;
    }

    .network-name {
        font-size: 14px;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }

    .network-type {
        font-size: 10px;
        color: var(--network-color);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: 4px;
    }

    .network-info {
        font-size: 9px;
        color: rgba(255, 255, 255, 0.6);
        font-family: 'JetBrains Mono', monospace;
    }
</style>
