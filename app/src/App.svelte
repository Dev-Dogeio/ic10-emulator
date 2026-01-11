<script lang="ts">
    import { onMount } from 'svelte';
    import {
        DeviceAtmosphericNetworkType,
        type WasmAtmosphericNetwork,
        type WasmCableNetwork,
    } from '../pkg/ic10_emulator';

    import Grid from './components/Grid.svelte';
    import DeviceList from './components/DeviceList.svelte';
    import DeviceNode from './components/DeviceNode.svelte';
    import NetworkNode from './components/NetworkNode.svelte';
    import ContextMenu, { type MenuItem } from './components/ContextMenu.svelte';
    import ConfigPopup, { type ConfigField } from './components/ConfigPopup.svelte';
    import ConnectionLine from './components/ConnectionLine.svelte';
    import InspectorWindow from './components/InspectorWindow.svelte';
    import DeviceInspector from './components/DeviceInspector.svelte';
    import AtmosphericNetworkInspector from './components/AtmosphericNetworkInspector.svelte';
    import CableNetworkInspector from './components/CableNetworkInspector.svelte';
    import SimulationControls from './components/SimulationControls.svelte';
    import type { ConnectorSide } from './components/Connector.svelte';

    import {
        getSimulationState,
        initializeWasm,
        createDevice,
        createCableNetwork,
        createAtmosphericNetwork,
        updateDevicePosition,
        updateNetworkPosition,
        removeDevice,
        removeNetwork,
        syncFromWasm,
        connectDeviceToCableNetwork,
        connectDeviceToAtmosphericNetwork,
        removeConnection,
        type Connection,
        type GridDevice,
        type ConnectorType,
    } from './stores/simulationState.svelte';

    import { getInspectorState, openInspector } from './stores/inspectorState.svelte';

    import { NODE_W, NODE_H, CONNECTION_COLORS } from './lib/constants';

    const snapToPixel = (v: number, scale: number) => Math.round(v * scale) / scale;

    // Get reactive state
    const simState = getSimulationState();
    const inspectorState = getInspectorState();

    let gridOffsetX = $state(0);
    let gridOffsetY = $state(0);
    let gridScale = $state(1);

    let selectedDeviceId: number | null = $state(null);
    let selectedNetworkId: string | null = $state(null);

    // Connection dragging state
    interface PendingConnection {
        sourceType: 'device' | 'network';
        sourceId: number | string;
        connectorId: string;
        connectorType: ConnectorType;
        connectorSide: ConnectorSide;
        startX: number;
        startY: number;
        currentX: number;
        currentY: number;
    }
    let pendingConnection: PendingConnection | null = $state(null);

    // Context menu state
    let contextMenuVisible = $state(false);
    let contextMenuX = $state(0);
    let contextMenuY = $state(0);
    let contextMenuGridX = $state(0);
    let contextMenuGridY = $state(0);
    let contextMenuTargetConnection: Connection | null = $state(null);

    // Config popup state (for creating atmospheric networks)
    let configPopupVisible = $state(false);
    let configPopupTitle = $state('');
    let configPopupFields: ConfigField[] = $state([]);
    let pendingAtmoNetworkPosition: { x: number; y: number } | null = $state(null);

    // Base context menu items for creating networks
    const BASE_CONTEXT_MENU_ITEMS: MenuItem[] = [
        { id: 'create-cable-network', label: 'Create Cable Network', icon: '🔌' },
        { id: 'create-atmo-network', label: 'Create Atmospheric Network', icon: '💨' },
        { id: 'divider1', label: '', divider: true },
        { id: 'cancel', label: 'Cancel', icon: '✕' },
    ];

    let contextMenuItems: MenuItem[] = $state([...BASE_CONTEXT_MENU_ITEMS]);
    onMount(async () => {
        await initializeWasm();
    });

    function getTopmostDeviceAt(gridX: number, gridY: number) {
        for (let i = simState.gridDevices.length - 1; i >= 0; i--) {
            const d = simState.gridDevices[i];
            if (gridX >= d.x && gridX <= d.x + NODE_W && gridY >= d.y && gridY <= d.y + NODE_H) {
                return d;
            }
        }
        return undefined;
    }

    function getTopmostNetworkAt(gridX: number, gridY: number) {
        for (let i = simState.gridNetworks.length - 1; i >= 0; i--) {
            const n = simState.gridNetworks[i];
            if (gridX >= n.x && gridX <= n.x + NODE_W && gridY >= n.y && gridY <= n.y + NODE_H) {
                return n;
            }
        }
        return undefined;
    }

    function handleGridContextMenu(event: { x: number; y: number; gridX: number; gridY: number }) {
        contextMenuX = event.x;
        contextMenuY = event.y;
        contextMenuGridX = event.gridX;
        contextMenuGridY = event.gridY;

        const clickedDevice = getTopmostDeviceAt(event.gridX, event.gridY);
        const clickedNetwork = getTopmostNetworkAt(event.gridX, event.gridY);

        const items: MenuItem[] = [];

        if (clickedDevice) {
            items.push({ id: 'remove-device', label: 'Remove Device', icon: '🗑️' });
            if (!clickedNetwork) {
                items.push({ id: 'divider-actions', label: '', divider: true });
            }
        }

        if (clickedNetwork) {
            items.push({ id: 'remove-network', label: 'Remove Network', icon: '🗑️' });
            items.push({ id: 'divider-actions', label: '', divider: true });
        }

        contextMenuItems = [...items, ...BASE_CONTEXT_MENU_ITEMS];
        contextMenuVisible = true;
    }

    function handleContextMenuSelect(itemId: string) {
        switch (itemId) {
            case 'create-cable-network':
                createCableNetwork(contextMenuGridX, contextMenuGridY);
                syncFromWasm();
                break;
            case 'create-atmo-network':
                // Show config popup for volume
                pendingAtmoNetworkPosition = { x: contextMenuGridX, y: contextMenuGridY };
                configPopupTitle = 'Create Atmospheric Network';
                configPopupFields = [
                    {
                        id: 'volume',
                        label: 'Volume (Liters)',
                        type: 'number',
                        value: 250,
                        min: 1,
                        max: 100000,
                        step: 1,
                        placeholder: 'Enter volume in liters',
                    },
                ];
                configPopupVisible = true;
                break;
            case 'remove-device': {
                const clickedDevice = getTopmostDeviceAt(contextMenuGridX, contextMenuGridY);
                if (clickedDevice) {
                    removeDevice(clickedDevice.id);
                }
                break;
            }
            case 'remove-network': {
                const clickedNetwork = getTopmostNetworkAt(contextMenuGridX, contextMenuGridY);
                if (clickedNetwork) {
                    removeNetwork(clickedNetwork.id);
                }
                break;
            }
            case 'remove-connection': {
                if (contextMenuTargetConnection) {
                    removeConnection(contextMenuTargetConnection.id);
                    contextMenuTargetConnection = null;
                }
                break;
            }
        }
        contextMenuVisible = false;
    }

    function handleConfigPopupConfirm(values: Record<string, number | string>) {
        if (pendingAtmoNetworkPosition) {
            const volume =
                typeof values.volume === 'number'
                    ? values.volume
                    : parseFloat(String(values.volume)) || 250;
            createAtmosphericNetwork(
                pendingAtmoNetworkPosition.x,
                pendingAtmoNetworkPosition.y,
                volume
            );
            syncFromWasm();
            pendingAtmoNetworkPosition = null;
        }
        configPopupVisible = false;
    }

    function handleConfigPopupCancel() {
        configPopupVisible = false;
        pendingAtmoNetworkPosition = null;
    }

    function handleDrop(e: DragEvent) {
        e.preventDefault();

        if (!simState.simulationManager || !e.dataTransfer) return;

        const data = e.dataTransfer.getData('application/json');
        if (!data) return;

        try {
            const dropData = JSON.parse(data);
            if (dropData.type !== 'device-prefab') return;

            const rect = (e.target as HTMLElement)
                .closest('.grid-container')
                ?.getBoundingClientRect();
            if (!rect) return;

            const screenX = e.clientX;
            const screenY = e.clientY;
            const gridX = (screenX - rect.left - gridOffsetX) / gridScale;
            const gridY = (screenY - rect.top - gridOffsetY) / gridScale;

            createDevice(dropData.prefabHash, gridX, gridY);
            syncFromWasm();
        } catch (error) {
            console.error('Failed to create device:', error);
        }
    }

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = 'copy';
        }
    }

    function handleDeviceSelect(deviceId: number) {
        selectedDeviceId = deviceId;
        selectedNetworkId = null;
    }

    function handleNetworkSelect(networkId: string) {
        selectedNetworkId = networkId;
        selectedDeviceId = null;
    }

    function handleDeviceMove(deviceId: number, x: number, y: number) {
        updateDevicePosition(deviceId, x, y);
    }

    function handleNetworkMove(networkId: string, x: number, y: number) {
        updateNetworkPosition(networkId, x, y);
    }

    function handleDeviceInspect(deviceId: number) {
        // Open inspector window for this device
        // Position near the device but offset so it doesn't cover it
        const device = simState.gridDevices.find((d) => d.id === deviceId);
        if (device && gridContainer) {
            const rect = gridContainer.getBoundingClientRect();
            const screenX =
                rect.left + gridOffsetX + device.x * gridScale + NODE_W * gridScale + 20;
            const screenY = rect.top + gridOffsetY + device.y * gridScale;
            openInspector('device', deviceId, screenX, screenY);
        } else {
            openInspector('device', deviceId, 150, 100);
        }
    }

    function handleNetworkInspect(networkId: string) {
        // Open inspector window for this network
        const network = simState.gridNetworks.find((n) => n.id === networkId);
        if (network && gridContainer) {
            const rect = gridContainer.getBoundingClientRect();
            const screenX =
                rect.left + gridOffsetX + network.x * gridScale + NODE_W * gridScale + 20;
            const screenY = rect.top + gridOffsetY + network.y * gridScale;
            openInspector('network', networkId, screenX, screenY);
        } else {
            openInspector('network', networkId, 150, 100);
        }
    }

    function handleGridClick(e: MouseEvent) {
        selectedDeviceId = null;
        selectedNetworkId = null;
    }

    // Connection handling functions
    let gridContainer: HTMLElement | null = $state(null);

    function screenToGrid(screenX: number, screenY: number): { x: number; y: number } {
        if (!gridContainer) return { x: 0, y: 0 };
        const rect = gridContainer.getBoundingClientRect();
        return {
            x: (screenX - rect.left - gridOffsetX) / gridScale,
            y: (screenY - rect.top - gridOffsetY) / gridScale,
        };
    }

    function getConnectorPosition(
        nodeType: 'device' | 'network',
        nodeId: number | string,
        connectorType: ConnectorType,
        side: ConnectorSide
    ): { x: number; y: number } {
        // Node dimensions
        const nodeWidth = NODE_W;
        const nodeHeight = NODE_H;
        let node: { x: number; y: number } | undefined;

        if (nodeType === 'device') {
            const device = simState.gridDevices.find((d) => d.id === nodeId);
            node = device;
        } else {
            const network = simState.gridNetworks.find((n) => n.id === nodeId);
            node = network;
        }

        if (!node) return { x: 0, y: 0 };

        // Base position (center of node)
        let x = node.x + nodeWidth / 2;
        let y = node.y + nodeHeight / 2;

        // Adjust for side
        if (side === 'left') {
            x = node.x;
        } else {
            x = node.x + nodeWidth;
        }

        // Adjust for connector type offsets (respect which connectors exist on the device)
        if (nodeType === 'device') {
            // Use the device prefab info to determine which atmospheric ports exist so single ports
            // can be centered while pairs are offset to avoid overlap.
            const prefabInfo = (node as GridDevice).prefabInfo;
            const hasAtmoInput = prefabInfo.atmospheric_connections.some(
                (c) => c.connection_type === DeviceAtmosphericNetworkType.Input
            );
            const hasAtmoInput2 = prefabInfo.atmospheric_connections.some(
                (c) => c.connection_type === DeviceAtmosphericNetworkType.Input2
            );
            const hasAtmoOutput = prefabInfo.atmospheric_connections.some(
                (c) => c.connection_type === DeviceAtmosphericNetworkType.Output
            );
            const hasAtmoOutput2 = prefabInfo.atmospheric_connections.some(
                (c) => c.connection_type === DeviceAtmosphericNetworkType.Output2
            );

            if (connectorType === 'cable') {
                y = node.y + nodeHeight / 2 - 22;
            } else if (connectorType === 'atmo-input') {
                y = node.y + nodeHeight / 2 + (hasAtmoInput2 ? -8 : 0);
            } else if (connectorType === 'atmo-input2') {
                y = node.y + nodeHeight / 2 + (hasAtmoInput ? 14 : 0);
            } else if (connectorType === 'atmo-output') {
                y = node.y + nodeHeight / 2 + (hasAtmoOutput2 ? -8 : 0);
            } else if (connectorType === 'atmo-output2') {
                y = node.y + nodeHeight / 2 + (hasAtmoOutput ? 14 : 0);
            }
        }

        return { x, y };
    }

    function handleDeviceStartConnect(
        deviceId: number,
        connectorId: string,
        connectorType: ConnectorType,
        side: ConnectorSide,
        event: PointerEvent
    ) {
        const pos = getConnectorPosition('device', deviceId, connectorType, side);
        pendingConnection = {
            sourceType: 'device',
            sourceId: deviceId,
            connectorId,
            connectorType,
            connectorSide: side,
            startX: pos.x,
            startY: pos.y,
            currentX: pos.x,
            currentY: pos.y,
        };
    }

    function handleDeviceEndConnect(
        deviceId: number,
        connectorId: string,
        connectorType: ConnectorType,
        side: ConnectorSide
    ) {
        if (!pendingConnection) return;

        // Only allow network -> device connections
        if (pendingConnection.sourceType === 'network') {
            const networkId = pendingConnection.sourceId as string;
            const network = simState.gridNetworks.find((n) => n.id === networkId);

            if (network) {
                if (
                    pendingConnection.connectorType === 'network-cable' &&
                    connectorType === 'cable'
                ) {
                    connectDeviceToCableNetwork(deviceId, networkId);
                } else if (
                    pendingConnection.connectorType === 'network-atmo' &&
                    (connectorType === 'atmo-input' ||
                        connectorType === 'atmo-input2' ||
                        connectorType === 'atmo-output' ||
                        connectorType === 'atmo-output2')
                ) {
                    connectDeviceToAtmosphericNetwork(deviceId, connectorType, networkId);
                }
            }
        }

        pendingConnection = null;
    }

    function handleNetworkStartConnect(
        networkId: string,
        connectorId: string,
        connectorType: ConnectorType,
        side: ConnectorSide,
        event: PointerEvent
    ) {
        const pos = getConnectorPosition('network', networkId, connectorType, side);
        pendingConnection = {
            sourceType: 'network',
            sourceId: networkId,
            connectorId,
            connectorType,
            connectorSide: side,
            startX: pos.x,
            startY: pos.y,
            currentX: pos.x,
            currentY: pos.y,
        };
    }

    function handleNetworkEndConnect(
        networkId: string,
        connectorId: string,
        connectorType: ConnectorType,
        side: ConnectorSide
    ) {
        if (!pendingConnection) return;

        // Only allow device -> network connections
        if (pendingConnection.sourceType === 'device') {
            const deviceId = pendingConnection.sourceId as number;
            const network = simState.gridNetworks.find((n) => n.id === networkId);

            if (network) {
                if (network.data.type === 'cable' && pendingConnection.connectorType === 'cable') {
                    connectDeviceToCableNetwork(deviceId, networkId);
                } else if (
                    network.data.type === 'atmospheric' &&
                    (pendingConnection.connectorType === 'atmo-input' ||
                        pendingConnection.connectorType === 'atmo-input2' ||
                        pendingConnection.connectorType === 'atmo-output' ||
                        pendingConnection.connectorType === 'atmo-output2')
                ) {
                    connectDeviceToAtmosphericNetwork(
                        deviceId,
                        pendingConnection.connectorType,
                        networkId
                    );
                }
            }
        }

        pendingConnection = null;
    }

    function handlePointerMove(e: PointerEvent) {
        if (!pendingConnection) return;
        const gridPos = screenToGrid(e.clientX, e.clientY);
        pendingConnection = {
            ...pendingConnection,
            currentX: gridPos.x,
            currentY: gridPos.y,
        };
    }

    function handlePointerUp(e: PointerEvent) {
        // Cancel pending connection if released on empty space
        pendingConnection = null;
    }

    function handleConnectionContextMenu(connection: Connection, event: MouseEvent) {
        event.preventDefault();
        event.stopPropagation();

        contextMenuX = event.clientX;
        contextMenuY = event.clientY;
        contextMenuTargetConnection = connection;

        contextMenuItems = [
            { id: 'remove-connection', label: 'Remove Connection', icon: '🔗' },
            { id: 'divider1', label: '', divider: true },
            { id: 'cancel', label: 'Cancel', icon: '✕' },
        ];

        contextMenuVisible = true;
    }

    // Get pending connection line path
    function getPendingConnectionPath(): string {
        if (!pendingConnection) return '';
        const { startX, startY, currentX, currentY } = pendingConnection;
        const offset = 2;

        const sX = startX + (pendingConnection.connectorSide === 'left' ? offset : -offset);
        const sY = startY;

        const dx2 = currentX - sX;
        const controlPointOffset = Math.min(Math.abs(dx2) * 0.5, 100);

        const c1x =
            sX +
            (pendingConnection.connectorSide === 'left' ? -controlPointOffset : controlPointOffset);
        const c1y = sY;
        const c2x = currentX + (dx2 >= 0 ? -controlPointOffset : controlPointOffset);
        const c2y = currentY;

        return `M ${sX} ${sY} C ${c1x} ${c1y}, ${c2x} ${c2y}, ${currentX} ${currentY}`;
    }

    function getPendingConnectionColor(): string {
        if (!pendingConnection) return CONNECTION_COLORS.default;
        const type = pendingConnection.connectorType;
        if (type === 'cable' || type === 'network-cable') return CONNECTION_COLORS.cable;
        if (type === 'atmo-output' || type === 'atmo-output2') return CONNECTION_COLORS.atmoOutput;
        return CONNECTION_COLORS.atmoInput;
    }

    // Calculate connection line positions
    // Memoize connection positions to avoid recalculating on every render
    const connectionPositionsCache = new Map<
        string,
        {
            source: { x: number; y: number; side: ConnectorSide };
            target: { x: number; y: number; side: ConnectorSide };
            deviceX: number;
            deviceY: number;
            networkX: number;
            networkY: number;
        }
    >();

    function getConnectionPositions(connection: Connection): {
        source: { x: number; y: number; side: ConnectorSide };
        target: { x: number; y: number; side: ConnectorSide };
    } {
        const device = simState.gridDevices.find((d) => d.id === connection.deviceId);
        const network = simState.gridNetworks.find((n) => n.id === connection.networkId);

        if (!device || !network) {
            return { source: { x: 0, y: 0, side: 'left' }, target: { x: 0, y: 0, side: 'left' } };
        }

        // Check cache
        const cached = connectionPositionsCache.get(connection.id);
        if (
            cached &&
            cached.deviceX === device.x &&
            cached.deviceY === device.y &&
            cached.networkX === network.x &&
            cached.networkY === network.y
        ) {
            return { source: cached.source, target: cached.target };
        }

        // Determine the device connector side based on connector type so outputs appear on the right
        const deviceSide: ConnectorSide =
            connection.deviceConnectorType === 'atmo-output' ||
            connection.deviceConnectorType === 'atmo-output2'
                ? 'right'
                : 'left';

        const sourcePos = getConnectorPosition(
            'device',
            connection.deviceId,
            connection.deviceConnectorType,
            deviceSide
        );

        // Find closest network connector side
        const networkLeftPos = getConnectorPosition(
            'network',
            connection.networkId,
            connection.networkType === 'cable' ? 'network-cable' : 'network-atmo',
            'left'
        );
        const networkRightPos = getConnectorPosition(
            'network',
            connection.networkId,
            connection.networkType === 'cable' ? 'network-cable' : 'network-atmo',
            'right'
        );

        const distLeft = Math.hypot(sourcePos.x - networkLeftPos.x, sourcePos.y - networkLeftPos.y);
        const distRight = Math.hypot(
            sourcePos.x - networkRightPos.x,
            sourcePos.y - networkRightPos.y
        );

        const useLeft = distLeft < distRight;
        const targetPos = useLeft ? networkLeftPos : networkRightPos;
        const targetSide: ConnectorSide = useLeft ? 'left' : 'right';

        const result = {
            source: { ...sourcePos, side: deviceSide },
            target: { ...targetPos, side: targetSide },
        };

        connectionPositionsCache.set(connection.id, {
            ...result,
            deviceX: device.x,
            deviceY: device.y,
            networkX: network.x,
            networkY: network.y,
        });

        return result;
    }
</script>

<svelte:window onpointermove={handlePointerMove} onpointerup={handlePointerUp} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="app">
    {#if !simState.wasmReady}
        <div class="loading">
            <div class="spinner"></div>
            <p>Loading WASM module...</p>
        </div>
    {:else}
        <DeviceList prefabs={simState.devicePrefabs} />

        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
            class="grid-area"
            ondrop={handleDrop}
            ondragover={handleDragOver}
            onclick={handleGridClick}
            role="application"
            bind:this={gridContainer}
        >
            <Grid
                bind:offsetX={gridOffsetX}
                bind:offsetY={gridOffsetY}
                bind:scale={gridScale}
                onContextMenu={handleGridContextMenu}
            >
                {#snippet svgContent()}
                    <!-- Connection lines -->
                    {#each simState.connections as connection (connection.id)}
                        {@const positions = getConnectionPositions(connection)}
                        <ConnectionLine
                            {connection}
                            sourcePos={{ x: positions.source.x, y: positions.source.y }}
                            targetPos={{ x: positions.target.x, y: positions.target.y }}
                            sourceSide={positions.source.side}
                            targetSide={positions.target.side}
                            onContextMenu={handleConnectionContextMenu}
                        />
                    {/each}

                    <!-- Pending connection line while dragging -->
                    {#if pendingConnection}
                        <path
                            d={getPendingConnectionPath()}
                            stroke={getPendingConnectionColor()}
                            stroke-width="2"
                            fill="none"
                            stroke-dasharray="8 4"
                            stroke-linecap="round"
                            opacity="0.8"
                            class="pending-connection"
                        ></path>
                    {/if}
                {/snippet}

                {#snippet nodeContent()}
                    {#each simState.gridNetworks as network (network.id)}
                        <g
                            transform="translate({snapToPixel(network.x, gridScale)}, {snapToPixel(
                                network.y,
                                gridScale
                            )})"
                        >
                            <NetworkNode
                                data={network.data}
                                x={0}
                                y={0}
                                gridX={network.x}
                                gridY={network.y}
                                scale={gridScale}
                                selected={selectedNetworkId === network.id}
                                connectingType={pendingConnection?.connectorType ?? null}
                                onMove={handleNetworkMove}
                                onSelect={handleNetworkSelect}
                                onInspect={handleNetworkInspect}
                                onStartConnect={handleNetworkStartConnect}
                                onEndConnect={handleNetworkEndConnect}
                            />
                        </g>
                    {/each}

                    {#each simState.gridDevices as gridDevice (gridDevice.id)}
                        <g
                            transform="translate({snapToPixel(
                                gridDevice.x,
                                gridScale
                            )}, {snapToPixel(gridDevice.y, gridScale)})"
                        >
                            <DeviceNode
                                device={gridDevice.device}
                                prefabInfo={gridDevice.prefabInfo}
                                x={0}
                                y={0}
                                gridX={gridDevice.x}
                                gridY={gridDevice.y}
                                scale={gridScale}
                                selected={selectedDeviceId === gridDevice.id}
                                connectingType={pendingConnection?.connectorType ?? null}
                                onMove={handleDeviceMove}
                                onSelect={handleDeviceSelect}
                                onInspect={handleDeviceInspect}
                                onStartConnect={handleDeviceStartConnect}
                                onEndConnect={handleDeviceEndConnect}
                            />
                        </g>
                    {/each}
                {/snippet}
            </Grid>
        </div>

        <ContextMenu
            x={contextMenuX}
            y={contextMenuY}
            visible={contextMenuVisible}
            items={contextMenuItems}
            onSelect={handleContextMenuSelect}
            onClose={() => (contextMenuVisible = false)}
        />

        <ConfigPopup
            visible={configPopupVisible}
            title={configPopupTitle}
            bind:fields={configPopupFields}
            onConfirm={handleConfigPopupConfirm}
            onCancel={handleConfigPopupCancel}
        />

        <!-- Inspector Windows -->
        {#each inspectorState.windows as inspectorWindow (inspectorWindow.id)}
            {#if inspectorWindow.type === 'device'}
                {@const gridDevice = simState.gridDevices.find(
                    (d) => d.id === inspectorWindow.targetId
                )}
                {#if gridDevice}
                    <InspectorWindow
                        window={inspectorWindow}
                        title={gridDevice.device.name()}
                        icon="🖥️"
                    >
                        <DeviceInspector
                            window={inspectorWindow}
                            device={gridDevice.device}
                            prefabInfo={gridDevice.prefabInfo}
                        />
                    </InspectorWindow>
                {/if}
            {:else if inspectorWindow.type === 'network'}
                {@const gridNetwork = simState.gridNetworks.find(
                    (n) => n.id === inspectorWindow.targetId
                )}
                {#if gridNetwork && gridNetwork.data.type === 'atmospheric'}
                    <InspectorWindow
                        window={inspectorWindow}
                        title={gridNetwork.data.name}
                        icon="💨"
                    >
                        <AtmosphericNetworkInspector
                            window={inspectorWindow}
                            network={gridNetwork.data.network as WasmAtmosphericNetwork}
                            networkName={gridNetwork.data.name}
                        />
                    </InspectorWindow>
                {:else if gridNetwork && gridNetwork.data.type === 'cable'}
                    <InspectorWindow
                        window={inspectorWindow}
                        title={gridNetwork.data.name}
                        icon="🔌"
                    >
                        <CableNetworkInspector
                            window={inspectorWindow}
                            network={gridNetwork.data.network as WasmCableNetwork}
                            networkName={gridNetwork.data.name}
                        />
                    </InspectorWindow>
                {/if}
            {/if}
        {/each}

        <SimulationControls />
    {/if}
</div>

<style>
    :global(*) {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    :global(body) {
        font-family:
            'Inter',
            -apple-system,
            BlinkMacSystemFont,
            'Segoe UI',
            Roboto,
            sans-serif;
        background: #0f0f1a;
        color: #e0e0e0;
        overflow: hidden;
    }

    .app {
        display: flex;
        width: 100vw;
        height: 100vh;
        background: #0f0f1a;
    }

    .loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        width: 100%;
        height: 100%;
        gap: 16px;
    }

    .spinner {
        width: 48px;
        height: 48px;
        border: 3px solid rgba(255, 255, 255, 0.1);
        border-top-color: #818cf8;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .loading p {
        color: rgba(255, 255, 255, 0.6);
        font-size: 14px;
    }

    .grid-area {
        flex: 1;
        position: relative;
        overflow: hidden;
        contain: layout style paint;
    }

    :global(.pending-connection) {
        pointer-events: none;
        animation: dash 0.5s linear infinite;
    }

    @keyframes dash {
        from {
            stroke-dashoffset: 0;
        }
        to {
            stroke-dashoffset: -12;
        }
    }
</style>
