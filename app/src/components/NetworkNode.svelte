<script lang="ts">
    import type { WasmCableNetwork, WasmAtmosphericNetwork } from '../../pkg/ic10_emulator';
    import type { ConnectorType } from '../stores/simulationState.svelte';
    import Connector, { type ConnectorSide } from './Connector.svelte';
    import Node from './Node.svelte';
    import { NODE_W, NODE_H, CONNECTION_COLORS } from '../lib/constants';

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

    let networkInfo = $derived.by(() => {
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
    });

    let icon = $derived(data.type === 'cable' ? '🔌' : '💨');

    let networkColor = $derived(
        data.type === 'cable' ? CONNECTION_COLORS.cable : CONNECTION_COLORS.atmoInput
    );

    let networkConnectorType: ConnectorType = $derived(
        data.type === 'cable' ? 'network-cable' : 'network-atmo'
    );

    let networkTitle = $derived(
        data.type === 'cable' ? `Network ${data.managerId}` : `Atmosphere ${data.managerId}`
    );

    let networkTypeLabel = $derived(
        data.type === 'cable' ? 'Cable Network' : 'Atmospheric Network'
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
    width={NODE_W}
    height={NODE_H}
    nodeClass={`network-node ${data.type === 'cable' ? 'cable' : 'atmospheric'}`}
>
    <!-- Background rect with gradient -->
    <defs>
        <linearGradient id="network-bg-{data.id}" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="rgba(255, 255, 255, 0.08)"></stop>
            <stop offset="100%" stop-color="rgba(255, 255, 255, 0.03)"></stop>
        </linearGradient>
        <filter id="network-shadow-{data.id}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="rgba(0,0,0,0.3)"
            ></feDropShadow>
        </filter>
    </defs>

    <rect
        x="0"
        y="0"
        width={NODE_W}
        height={NODE_H}
        rx="12"
        ry="12"
        fill="url(#network-bg-{data.id})"
        stroke={selected ? '#818cf8' : networkColor}
        stroke-width="2"
        filter="url(#network-shadow-{data.id})"
        class="network-background"
    ></rect>

    <!-- Inner highlight -->
    <rect
        x="2"
        y="2"
        width={NODE_W - 4}
        height={NODE_H - 4}
        rx="10"
        ry="10"
        fill="none"
        stroke="rgba(255, 255, 255, 0.02)"
        stroke-width="1"
    ></rect>
    <rect
        x="0"
        y="0"
        width={NODE_W}
        height={NODE_H}
        rx="12"
        ry="12"
        fill="rgba(255, 255, 255, 0.02)"
        class="node-hover"
        pointer-events="none"
    ></rect>

    <!-- Left connector -->
    <Connector
        id={`network-${data.id}-left`}
        type={networkConnectorType}
        side="left"
        label={data.type === 'cable' ? 'Network' : 'Atmo'}
        offsetY={0}
        nodeWidth={NODE_W}
        nodeHeight={NODE_H}
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
        nodeWidth={NODE_W}
        nodeHeight={NODE_H}
        compatible={isCompatible()}
        connecting={connectingType !== null}
        onStartConnect={handleConnectorStart}
        onEndConnect={handleConnectorEnd}
    />

    <!-- Network icon -->
    <text x="16" y="24" font-size="16" class="network-icon">{icon}</text>

    <!-- Network name -->
    <text x="40" y="24" font-size="12" font-weight="600" fill="#fff" class="network-name">
        {networkTitle}
    </text>

    <!-- Network type label -->
    <text x="16" y="40" font-size="9" fill={networkColor} class="network-type">
        {networkTypeLabel.toUpperCase()}
    </text>

    <!-- Network info -->
    <text
        x="16"
        y="56"
        font-size="8"
        fill="rgba(255, 255, 255, 0.6)"
        font-family="'JetBrains Mono', monospace"
        class="network-info"
    >
        {networkInfo}
    </text>
</Node>

<style>
    .network-background {
        transition: stroke 0.15s ease;
    }

    .network-icon {
        pointer-events: none;
    }

    .network-name {
        pointer-events: none;
    }

    .network-type {
        pointer-events: none;
    }

    .network-info {
        pointer-events: none;
    }
</style>
