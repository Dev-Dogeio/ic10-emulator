<script lang="ts">
    import {
        type WasmDevice,
        type DevicePrefabInfo,
        DeviceAtmosphericNetworkType,
    } from '../../pkg/ic10_emulator';
    import type { ConnectorType } from '../stores/simulationState.svelte';
    import Connector, { type ConnectorSide } from './Connector.svelte';

    interface Props {
        device: WasmDevice;
        prefabInfo: DevicePrefabInfo;
        x: number;
        y: number;
        gridX?: number;
        gridY?: number;
        scale: number;
        selected?: boolean;
        connectingType?: ConnectorType | null;
        onMove?: (deviceId: number, x: number, y: number) => void;
        onSelect?: (deviceId: number) => void;
        onStartConnect?: (
            deviceId: number,
            connectorId: string,
            connectorType: ConnectorType,
            side: ConnectorSide,
            event: PointerEvent
        ) => void;
        onEndConnect?: (
            deviceId: number,
            connectorId: string,
            connectorType: ConnectorType,
            side: ConnectorSide
        ) => void;
    }

    let {
        device,
        prefabInfo,
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
    }: Props = $props();

    import Node from './Node.svelte';

    let deviceId = $derived(device.id());
    let deviceName = $derived(device.name());

    function getDeviceIcon(): string {
        if (prefabInfo.is_ic_host) return '🖥️';
        if (prefabInfo.is_atmospheric_device) return '💨';
        return '📊';
    }

    let hasAnyTag = $derived(() => {
        return (
            prefabInfo.is_ic_host ||
            prefabInfo.is_slot_host ||
            prefabInfo.is_atmospheric_device ||
            (prefabInfo.atmospheric_connections &&
                prefabInfo.atmospheric_connections.some(
                    (c) => c.connection_type === DeviceAtmosphericNetworkType.Internal
                ))
        );
    });

    let hasAtmoInput = $derived(
        prefabInfo.atmospheric_connections.some(
            (c) => c.connection_type === DeviceAtmosphericNetworkType.Input
        )
    );
    let hasAtmoInput2 = $derived(
        prefabInfo.atmospheric_connections.some(
            (c) => c.connection_type === DeviceAtmosphericNetworkType.Input2
        )
    );
    let hasAtmoOutput = $derived(
        prefabInfo.atmospheric_connections.some(
            (c) => c.connection_type === DeviceAtmosphericNetworkType.Output
        )
    );
    let hasAtmoOutput2 = $derived(
        prefabInfo.atmospheric_connections.some(
            (c) => c.connection_type === DeviceAtmosphericNetworkType.Output2
        )
    );

    function isCompatible(type: ConnectorType): boolean {
        if (!connectingType) return false;
        if (connectingType === 'network-cable' && type === 'cable') return true;
        if (
            connectingType === 'network-atmo' &&
            (type === 'atmo-input' ||
                type === 'atmo-input2' ||
                type === 'atmo-output' ||
                type === 'atmo-output2')
        )
            return true;
        return false;
    }

    function handleConnectorStart(
        connectorId: string,
        type: ConnectorType,
        side: ConnectorSide,
        event: PointerEvent
    ) {
        if (onStartConnect) {
            onStartConnect(deviceId, connectorId, type, side, event);
        }
    }

    function handleConnectorEnd(connectorId: string, type: ConnectorType, side: ConnectorSide) {
        if (onEndConnect) {
            onEndConnect(deviceId, connectorId, type, side);
        }
    }
</script>

<Node
    id={deviceId}
    {gridX}
    {gridY}
    {scale}
    {selected}
    {onMove}
    {onSelect}
    nodeClass={`device-node ${prefabInfo.is_ic_host ? 'ic' : ''} ${prefabInfo.is_atmospheric_device ? 'atmo' : ''} ${prefabInfo.is_slot_host ? 'slot' : ''} ${!hasAnyTag() ? 'logic' : ''}`}
>
    <!-- Cable network connector -->
    <Connector
        id={`device-${deviceId}-cable`}
        type="cable"
        side="left"
        label="Network"
        offsetY={-30}
        compatible={isCompatible('cable')}
        connecting={connectingType !== null}
        onStartConnect={handleConnectorStart}
        onEndConnect={handleConnectorEnd}
    />

    <!-- Atmospheric input connectors (left side) -->
    {#if hasAtmoInput}
        <Connector
            id={`device-${deviceId}-atmo-input`}
            type="atmo-input"
            side="left"
            label="Input"
            offsetY={hasAtmoInput2 ? -10 : 0}
            compatible={isCompatible('atmo-input')}
            connecting={connectingType !== null}
            onStartConnect={handleConnectorStart}
            onEndConnect={handleConnectorEnd}
        />
    {/if}
    {#if hasAtmoInput2}
        <Connector
            id={`device-${deviceId}-atmo-input2`}
            type="atmo-input2"
            side="left"
            label="Input 2"
            offsetY={hasAtmoInput ? 20 : 0}
            compatible={isCompatible('atmo-input2')}
            connecting={connectingType !== null}
            onStartConnect={handleConnectorStart}
            onEndConnect={handleConnectorEnd}
        />
    {/if}

    <!-- Atmospheric output connectors (right side) -->
    {#if hasAtmoOutput}
        <Connector
            id={`device-${deviceId}-atmo-output`}
            type="atmo-output"
            side="right"
            label="Output"
            offsetY={hasAtmoOutput2 ? -10 : 0}
            compatible={isCompatible('atmo-output')}
            connecting={connectingType !== null}
            onStartConnect={handleConnectorStart}
            onEndConnect={handleConnectorEnd}
        />
    {/if}
    {#if hasAtmoOutput2}
        <Connector
            id={`device-${deviceId}-atmo-output2`}
            type="atmo-output2"
            side="right"
            label="Output 2"
            offsetY={hasAtmoOutput ? 20 : 0}
            compatible={isCompatible('atmo-output2')}
            connecting={connectingType !== null}
            onStartConnect={handleConnectorStart}
            onEndConnect={handleConnectorEnd}
        />
    {/if}

    <div class="device-header">
        <span class="device-icon">{getDeviceIcon()}</span>
        <span class="device-name">{deviceName}</span>
    </div>
    <div class="device-type">{prefabInfo.device_name}</div>
    <div class="device-id">ID: {deviceId}</div>

    <div class="device-badges">
        {#if prefabInfo.is_ic_host}
            <span class="mini-badge ic">IC</span>
        {/if}
        {#if prefabInfo.is_atmospheric_device}
            <span class="mini-badge atmo">ATMO</span>
        {/if}
        {#if prefabInfo.atmospheric_connections && prefabInfo.atmospheric_connections.some((c) => c.connection_type === 0)}
            <span class="mini-badge internal">INTERNAL</span>
        {/if}
        {#if prefabInfo.is_slot_host}
            <span class="mini-badge slot">SLOT</span>
        {/if}
        {#if !hasAnyTag()}
            <span class="mini-badge logic">LOGIC</span>
        {/if}
    </div>
</Node>

<style>
    :global(.device-node.ic) {
        --device-color: rgb(74, 222, 128);
    }
    :global(.device-node.atmo) {
        --device-color: rgb(96, 165, 250);
    }
    :global(.device-node.slot) {
        --device-color: rgb(251, 191, 36);
    }
    :global(.device-node.logic) {
        --device-color: rgb(226, 237, 129);
    }

    :global(.device-node) {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 4px;
        cursor: move;
        user-select: none;
        filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.3));
        background: linear-gradient(135deg, #252542 0%, #1a1a32 100%);
        border: 1px solid var(--device-color, rgba(255, 255, 255, 0.12));
        border-radius: 10px;
        padding: 13px 12px 10px 12px;
        width: min(220px, 100%);
        height: min(140px, 100%);
        max-width: 220px;
        max-height: 140px;
        margin: auto;
        box-sizing: border-box;
        min-width: 0;
        min-height: 0;
        transition:
            background 0.15s ease,
            border-color 0.15s ease,
            box-shadow 0.15s ease;
        overflow: visible;
        box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.03);
    }

    :global(.device-node:focus-visible) {
        outline: 2px solid #818cf8;
        outline-offset: 2px;
    }

    :global(.device-node.dragging) {
        cursor: grabbing;
        z-index: 100;
        transition: none !important;
        filter: none !important;
        box-shadow: none !important;
    }

    :global(.device-node:hover):not(.selected):not(.dragging) {
        background: linear-gradient(135deg, #2d2d50 0%, #1e1e38 100%);
    }

    :global(.device-node.selected),
    :global(.device-node.dragging) {
        border-color: #818cf8;
    }

    :global(.device-node .device-type) {
        color: rgba(var(--device-color, 129, 140, 248), 0.9);
    }

    .device-header {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 4px;
        width: 100%;
        justify-content: flex-start;
    }

    .device-icon {
        font-size: 16px;
    }

    .device-name {
        font-size: 14px;
        font-weight: 600;
        color: #fff;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 100%;
    }

    .device-type {
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        margin-bottom: 2px;
    }

    .device-id {
        position: absolute;
        top: 6px;
        right: 10px;
        font-size: 10px;
        color: rgba(255, 255, 255, 0.6);
        font-family: 'JetBrains Mono', monospace;
        opacity: 0.95;
        pointer-events: none;
    }

    .device-badges {
        display: flex;
        gap: 6px;
        margin-top: 4px;
        flex-wrap: wrap;
    }

    .mini-badge {
        font-size: 8px;
        font-weight: 600;
        padding: 2px 4px;
        border-radius: 3px;
        text-transform: uppercase;
    }

    .mini-badge.ic {
        background: rgba(34, 197, 94, 0.2);
        color: #4ade80;
    }

    .mini-badge.slot {
        background: rgba(251, 191, 36, 0.2);
        color: #fbbf24;
    }

    .mini-badge.atmo {
        background: rgba(96, 165, 250, 0.12);
        color: #60a5fa;
    }

    .mini-badge.internal {
        background: rgba(148, 163, 184, 0.12);
        color: #94a3b8;
    }

    .mini-badge.logic {
        background: rgba(206, 204, 124, 0.08);
        color: #e2ed81;
    }
</style>
