<script lang="ts">
    import {
        type WasmDevice,
        type DevicePrefabInfo,
        DeviceAtmosphericNetworkType,
    } from '../../pkg/ic10_emulator';
    import type { ConnectorType } from '../stores/simulationState.svelte';
    import Connector, { type ConnectorSide } from './Connector.svelte';
    import Node from './Node.svelte';
    import { NODE_W, NODE_H } from '../lib/constants';

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
        onInspect?: (deviceId: number) => void;
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
        onInspect,
        onStartConnect,
        onEndConnect,
    }: Props = $props();

    let deviceId = $derived(device.id());
    let deviceName = $derived(device.name());

    let deviceIcon = $derived.by(() => {
        if (prefabInfo.is_ic_host) return '🖥️';
        if (prefabInfo.is_atmospheric_device) return '💨';
        return '📊';
    });

    let deviceColor = $derived.by(() => {
        if (prefabInfo.is_ic_host) return 'rgb(74, 222, 128)';
        if (prefabInfo.is_atmospheric_device) return 'rgb(96, 165, 250)';
        if (prefabInfo.is_slot_host) return 'rgb(251, 191, 36)';
        if (prefabInfo.properties && prefabInfo.properties.length > 0) return 'rgb(226, 237, 129)';
        return 'rgba(255, 255, 255, 0.12)';
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

    // Whether the device supports attaching to the cable network
    let hasCable = $derived.by(() => prefabInfo.supports_cable_network);

    interface Badge {
        label: string;
        bgColor: string;
        textColor: string;
    }

    let badges = $derived.by(() => {
        const result: Badge[] = [];
        if (prefabInfo.is_ic_host) {
            result.push({
                label: 'IC HOST',
                bgColor: 'rgba(34, 197, 94, 0.2)',
                textColor: '#4ade80',
            });
        }
        if (prefabInfo.is_atmospheric_device) {
            result.push({
                label: 'ATMO',
                bgColor: 'rgba(96, 165, 250, 0.12)',
                textColor: '#60a5fa',
            });
        }
        if (prefabInfo.is_slot_host) {
            result.push({
                label: 'SLOT',
                bgColor: 'rgba(251, 191, 36, 0.2)',
                textColor: '#fbbf24',
            });
        }
        if (prefabInfo.properties && prefabInfo.properties.length > 0) {
            result.push({
                label: 'LOGIC',
                bgColor: 'rgba(206, 204, 124, 0.08)',
                textColor: '#e2ed81',
            });
        }
        if (
            prefabInfo.atmospheric_connections &&
            prefabInfo.atmospheric_connections.some((c) => c.connection_type === 0)
        ) {
            result.push({
                label: 'INTRNL',
                bgColor: 'rgba(148, 163, 184, 0.12)',
                textColor: '#94a3b8',
            });
        }
        return result;
    });

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
    {onInspect}
    width={NODE_W}
    height={NODE_H}
    nodeClass={`device-node ${prefabInfo.is_ic_host ? 'ic' : ''} ${prefabInfo.is_atmospheric_device ? 'atmo' : ''} ${prefabInfo.is_slot_host ? 'slot' : ''} ${prefabInfo.properties && prefabInfo.properties.length > 0 ? 'logic' : ''}`}
>
    <!-- Background rect with gradient -->
    <defs>
        <linearGradient id="device-bg-{deviceId}" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#252542"></stop>
            <stop offset="100%" stop-color="#1a1a32"></stop>
        </linearGradient>
        <filter id="device-shadow-{deviceId}" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="0" dy="4" stdDeviation="6" flood-color="rgba(0,0,0,0.3)"
            ></feDropShadow>
        </filter>
    </defs>

    <rect
        x="0"
        y="0"
        width={NODE_W}
        height={NODE_H}
        rx="10"
        ry="10"
        fill="url(#device-bg-{deviceId})"
        stroke={selected ? '#818cf8' : deviceColor}
        stroke-width="1"
        filter="url(#device-shadow-{deviceId})"
        class="device-background"
    ></rect>

    <!-- Inner highlight -->
    <rect
        x="1"
        y="1"
        width={NODE_W - 2}
        height={NODE_H - 2}
        rx="9"
        ry="9"
        fill="none"
        stroke="rgba(255, 255, 255, 0.03)"
        stroke-width="1"
    ></rect>
    <rect
        x="0"
        y="0"
        width={NODE_W}
        height={NODE_H}
        rx="10"
        ry="10"
        fill="rgba(255, 255, 255, 0.02)"
        class="node-hover"
        pointer-events="none"
    ></rect>

    {#if hasCable}
        <!-- Cable network connector -->
        <Connector
            id={`device-${deviceId}-cable`}
            type="cable"
            side="left"
            label="Network"
            offsetY={-22}
            nodeWidth={NODE_W}
            nodeHeight={NODE_H}
            compatible={isCompatible('cable')}
            connecting={connectingType !== null}
            onStartConnect={handleConnectorStart}
            onEndConnect={handleConnectorEnd}
        />
    {/if}

    <!-- Atmospheric input connectors (left side) -->
    {#if hasAtmoInput}
        <Connector
            id={`device-${deviceId}-atmo-input`}
            type="atmo-input"
            side="left"
            label="Input"
            offsetY={hasAtmoInput2 ? -8 : 0}
            nodeWidth={NODE_W}
            nodeHeight={NODE_H}
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
            offsetY={hasAtmoInput ? 14 : 0}
            nodeWidth={NODE_W}
            nodeHeight={NODE_H}
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
            offsetY={hasAtmoOutput2 ? -8 : 0}
            nodeWidth={NODE_W}
            nodeHeight={NODE_H}
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
            offsetY={hasAtmoOutput ? 14 : 0}
            nodeWidth={NODE_W}
            nodeHeight={NODE_H}
            compatible={isCompatible('atmo-output2')}
            connecting={connectingType !== null}
            onStartConnect={handleConnectorStart}
            onEndConnect={handleConnectorEnd}
        />
    {/if}

    <!-- Device icon -->
    <text x="12" y="20" font-size="14" class="device-icon">{deviceIcon}</text>

    <!-- Device name -->
    <text x="32" y="20" font-size="12" font-weight="600" fill="#fff" class="device-name">
        {deviceName.length > 14 ? deviceName.slice(0, 14) + '…' : deviceName}
    </text>

    <!-- Device ID -->
    <text
        x={NODE_W - 10}
        y="14"
        font-size="9"
        fill="rgba(255, 255, 255, 0.6)"
        text-anchor="end"
        font-family="'JetBrains Mono', monospace"
        class="device-id"
    >
        ID: {deviceId}
    </text>

    <!-- Device type -->
    <text x="12" y="36" font-size="10" fill="rgba(255, 255, 255, 0.5)" class="device-type">
        {prefabInfo.device_name.length > 20
            ? prefabInfo.device_name.slice(0, 20) + '…'
            : prefabInfo.device_name}
    </text>

    <!-- Badges -->
    <g class="device-badges" transform="translate(12, 48)">
        {#each badges as badge, i}
            <g transform="translate({i * 38}, 0)">
                <rect x="0" y="0" width="34" height="12" rx="3" fill={badge.bgColor}></rect>
                <text
                    x="17"
                    y="9"
                    font-size="7"
                    font-weight="600"
                    fill={badge.textColor}
                    text-anchor="middle"
                >
                    {badge.label}
                </text>
            </g>
        {/each}
    </g>
</Node>

<style>
    .device-background {
        transition: stroke 0.15s ease;
    }

    .device-icon {
        pointer-events: none;
    }

    .device-name {
        pointer-events: none;
    }

    .device-type {
        pointer-events: none;
    }

    .device-id {
        pointer-events: none;
    }

    .device-badges {
        pointer-events: none;
    }
</style>
