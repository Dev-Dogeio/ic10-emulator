import init, {
    WasmSimulationManager,
    get_registered_device_prefabs,
    get_device_prefab_info,
    DeviceAtmosphericNetworkType,
    type WasmDevice,
    type WasmCableNetwork,
    type WasmAtmosphericNetwork,
    type DevicePrefabInfo,
} from '../../pkg/ic10_emulator';
import { addNotification } from './notifications.svelte';

export type { WasmDevice, WasmCableNetwork, WasmAtmosphericNetwork, DevicePrefabInfo };

export type ConnectorType =
    | 'cable'
    | 'atmo-input'
    | 'atmo-input2'
    | 'atmo-output'
    | 'atmo-output2'
    | 'network-cable'
    | 'network-atmo';

// A connection between a device and a network
export interface Connection {
    id: string;
    deviceId: number;
    deviceConnectorType: ConnectorType;
    networkId: string;
    networkType: 'cable' | 'atmospheric';
    atmoConnectionType?: DeviceAtmosphericNetworkType;
}

export interface GridDevice {
    id: number;
    device: WasmDevice;
    prefabInfo: DevicePrefabInfo;
    x: number;
    y: number;
}

export type NetworkType = 'cable' | 'atmospheric';

export interface NetworkNodeData {
    id: string;
    type: NetworkType;
    name: string;
    network: WasmCableNetwork | WasmAtmosphericNetwork;
    managerId: number;
}

export interface GridNetwork {
    id: string;
    data: NetworkNodeData;
    x: number;
    y: number;
}

let _wasmReady = $state(false);
let _simulationManager: WasmSimulationManager | null = $state(null);
let _devicePrefabs: DevicePrefabInfo[] = $state([]);

let _gridDevices: GridDevice[] = $state([]);
let _gridNetworks: GridNetwork[] = $state([]);
let _connections: Connection[] = $state([]);

let _cableNetworkCounter = $state(0);
let _atmosphericNetworkCounter = $state(0);

export function getSimulationState() {
    return {
        get wasmReady() {
            return _wasmReady;
        },
        get simulationManager() {
            return _simulationManager;
        },
        get devicePrefabs() {
            return _devicePrefabs;
        },
        get gridDevices() {
            return _gridDevices;
        },
        set gridDevices(value: GridDevice[]) {
            _gridDevices = value;
        },
        get gridNetworks() {
            return _gridNetworks;
        },
        set gridNetworks(value: GridNetwork[]) {
            _gridNetworks = value;
        },
        get connections() {
            return _connections;
        },
        set connections(value: Connection[]) {
            _connections = value;
        },
        get tickCount() {
            return _tickCount;
        },
    };
}

export async function initializeWasm(): Promise<void> {
    if (_wasmReady) return;

    try {
        await init();
        _simulationManager = new WasmSimulationManager();

        const prefabHashes = get_registered_device_prefabs();
        _devicePrefabs = Array.from(prefabHashes).map((hash) => get_device_prefab_info(hash));

        _wasmReady = true;
        syncFromWasm();
        console.log('WASM initialized, found', _devicePrefabs.length, 'device prefabs');
    } catch (error) {
        console.error('Failed to initialize WASM:', error);
        throw error;
    }
}

export function createDevice(prefabHash: number, x: number, y: number): GridDevice | null {
    if (!_simulationManager) return null;

    try {
        const device = _simulationManager.create_device(prefabHash);
        const prefabInfo = get_device_prefab_info(prefabHash);

        const gridDevice: GridDevice = {
            id: device.id(),
            device,
            prefabInfo,
            x,
            y,
        };

        _gridDevices = [..._gridDevices, gridDevice];
        console.log('Created device:', device.name(), 'at', x, y);
        syncFromWasm();

        return gridDevice;
    } catch (e) {
        console.error('Failed to create device via manager:', e);
        addNotification('error', `Failed to create device: ${String(e)}`, 5000);
        return null;
    }
}

export function createCableNetwork(x: number, y: number): GridNetwork | null {
    if (!_simulationManager) return null;

    try {
        const network = _simulationManager.create_cable_network();
        _cableNetworkCounter++;

        const networkData: NetworkNodeData = {
            id: `cable-${_cableNetworkCounter}`,
            type: 'cable',
            name: `Network ${network.id()}`,
            network,
            managerId: network.id(),
        };

        const gridNetwork: GridNetwork = {
            id: networkData.id,
            data: networkData,
            x,
            y,
        };

        _gridNetworks = [..._gridNetworks, gridNetwork];
        syncFromWasm();
        return gridNetwork;
    } catch (e) {
        console.error('Failed to create cable network via manager:', e);
        addNotification('error', `Failed to create cable network: ${String(e)}`, 5000);
        return null;
    }
}

export function createAtmosphericNetwork(
    x: number,
    y: number,
    volume: number = 250
): GridNetwork | null {
    if (!_simulationManager) return null;

    try {
        const network = _simulationManager.create_atmospheric_network(volume);
        _atmosphericNetworkCounter++;

        const networkData: NetworkNodeData = {
            id: `atmo-${_atmosphericNetworkCounter}`,
            type: 'atmospheric',
            name: `Atmosphere ${network.id()}`,
            network,
            managerId: network.id(),
        };

        const gridNetwork: GridNetwork = {
            id: networkData.id,
            data: networkData,
            x,
            y,
        };

        _gridNetworks = [..._gridNetworks, gridNetwork];
        syncFromWasm();
        return gridNetwork;
    } catch (e) {
        console.error('Failed to create atmospheric network via manager:', e);
        addNotification('error', `Failed to create atmospheric network: ${String(e)}`, 5000);
        return null;
    }
}

export function updateDevicePosition(deviceId: number, x: number, y: number): void {
    const idx = _gridDevices.findIndex((d) => d.id === deviceId);
    if (idx !== -1) {
        _gridDevices[idx] = { ..._gridDevices[idx], x, y };
    }
}

export function updateNetworkPosition(networkId: string, x: number, y: number): void {
    const idx = _gridNetworks.findIndex((n) => n.id === networkId);
    if (idx !== -1) {
        _gridNetworks[idx] = { ..._gridNetworks[idx], x, y };
    }
}

export function removeDevice(deviceId: number): boolean {
    if (!_simulationManager) return false;

    let success = false;
    try {
        success = _simulationManager.remove_device(deviceId);
    } catch (e) {
        console.warn('Failed to remove device via manager:', e);
    }

    syncFromWasm();
    return success;
}

export function removeNetwork(networkId: string): boolean {
    if (!_simulationManager) return false;

    const idx = _gridNetworks.findIndex((n) => n.id === networkId);
    if (idx === -1) return false;

    const net = _gridNetworks[idx];
    const managerId = net.data.managerId;
    let success = false;

    try {
        if (net.data.type === 'cable') {
            success = _simulationManager.remove_cable_network(managerId);
        } else {
            success = _simulationManager.remove_atmospheric_network(managerId);
        }
    } catch (e) {
        console.warn('Failed to remove network via manager:', e);
    }

    syncFromWasm();
    return success;
}

export function debugSimulation(): void {
    console.group('Simulation Manager Debug');

    if (!_simulationManager) {
        console.warn('No SimulationManager instance available');
        console.groupEnd();
        return;
    }

    try {
        const simDevices = _simulationManager.all_devices();
        console.log(`Devices (${simDevices.length}):`);
        for (const d of simDevices) {
            try {
                const deviceId = d.id();
                const deviceName = d.name();
                const prefabHash = d.prefab_hash();

                const conns: string[] = [];
                try {
                    const cableNet = d.get_network();
                    if (cableNet) conns.push(`cable:${cableNet.id()}`);
                } catch (_) {}

                const atmoTypesForDevice = [
                    DeviceAtmosphericNetworkType.Internal,
                    DeviceAtmosphericNetworkType.Input,
                    DeviceAtmosphericNetworkType.Input2,
                    DeviceAtmosphericNetworkType.Output,
                    DeviceAtmosphericNetworkType.Output2,
                ];

                for (const atType of atmoTypesForDevice) {
                    try {
                        const atmoNet = d.get_atmospheric_network(atType);
                        if (atmoNet) {
                            const name = DeviceAtmosphericNetworkType[atType];
                            conns.push(
                                `${name ? name.toLowerCase() : String(atType)}:${atmoNet.id()}`
                            );
                        }
                    } catch (_) {}
                }

                console.log(
                    ` - Device id=${deviceId} name=${deviceName} prefab_hash=${prefabHash} connections=[${conns.join(
                        ', '
                    )}] has_chip=${d.has_chip()}`
                );
            } catch (e) {
                console.log(' - Device (failed to read properties)', e);
            }
        }

        const cableNets = _simulationManager.all_cable_networks();
        console.log(`Cable networks (${cableNets.length}):`);
        for (const net of cableNets) {
            try {
                const ids = Array.from(net.device_ids());
                console.log(` - Cable id=${net.id()} devices=[${ids.join(', ')}]`);
            } catch (e) {
                console.warn(' - Cable network: failed to read devices', e);
            }
        }

        const atmoMap = new Map<number, Array<{ deviceId: number; conn: number }>>();
        for (const d of simDevices) {
            for (let conn = 0; conn <= 4; conn++) {
                try {
                    const res = d.get_atmospheric_network(conn);
                    if (res) {
                        const atmoNet = res as WasmAtmosphericNetwork;
                        if (atmoNet) {
                            const netId = atmoNet.id();
                            const arr = atmoMap.get(netId) ?? [];
                            arr.push({ deviceId: d.id(), conn });
                            atmoMap.set(netId, arr);
                        }
                    }
                } catch (err) {
                    // ignore
                }
            }
        }

        const atmoNets = _simulationManager.all_atmospheric_networks();
        console.log(`Atmospheric networks (${atmoNets.length}):`);
        for (const net of atmoNets) {
            try {
                const entries = atmoMap.get(net.id()) ?? [];
                const formatted = entries.map((e) => {
                    const name = DeviceAtmosphericNetworkType[e.conn];
                    return `${e.deviceId}(${name ? name.toLowerCase() : e.conn})`;
                });
                console.log(
                    ` - Atmo id=${net.id()} volume=${
                        net.total_volume?.() ?? 'n/a'
                    } connections=[${formatted.join(', ')}]`
                );
            } catch (e) {
                console.warn(' - Atmospheric network: failed to read properties', e);
            }
        }
    } catch (err) {
        console.error('Failed to read simulation manager state', err);
    }

    console.groupEnd();
}

// Rebuild entire UI state from the WASM simulation manager. Preserves UI positions when possible.
export function syncFromWasm(): void {
    if (!_simulationManager) return;

    try {
        // Map previous positions so we can preserve UX layout where possible
        const devicePosMap = new Map<number, { x: number; y: number }>();
        for (const d of _gridDevices) devicePosMap.set(d.id, { x: d.x, y: d.y });

        const networkPosMap = new Map<string, { id: string; x: number; y: number }>();
        for (const n of _gridNetworks) {
            const prefix = n.data.type === 'atmospheric' ? 'atmo' : 'cable';
            networkPosMap.set(`${prefix}-${n.data.managerId}`, { id: n.id, x: n.x, y: n.y });
        }

        // Rebuild devices
        const simDevices = _simulationManager.all_devices();

        // Free WASM JS wrappers for devices that no longer exist in the manager
        try {
            const simDeviceIds = new Set<number>(simDevices.map((d) => d.id()));
            for (const old of _gridDevices) {
                if (!simDeviceIds.has(old.id)) {
                    try {
                        if (old.device?.free) {
                            old.device.free();
                            console.log('Freed device wrapper for id=', old.id);
                        }
                    } catch (e) {
                        console.warn('Failed to free device wrapper for id=', old.id, e);
                    }
                }
            }
        } catch (e) {
            console.warn('Failed to cleanup old device wrappers:', e);
        }

        _gridDevices = simDevices.map((d) => {
            const id = d.id();
            const prev = devicePosMap.get(id);
            const prefabInfo = get_device_prefab_info(d.prefab_hash());
            return {
                id,
                device: d,
                prefabInfo,
                x: prev?.x ?? 0,
                y: prev?.y ?? 0,
            } as GridDevice;
        });

        // Free WASM JS wrappers for networks that no longer exist in the manager
        try {
            const currentCableIds = new Set<number>(
                _simulationManager.all_cable_networks().map((n) => n.id())
            );
            const currentAtmoIds = new Set<number>(
                _simulationManager.all_atmospheric_networks().map((n) => n.id())
            );

            for (const old of _gridNetworks) {
                const mid = old.data.managerId;
                const isCable = old.data.type === 'cable';
                const exists = isCable ? currentCableIds.has(mid) : currentAtmoIds.has(mid);
                if (!exists) {
                    try {
                        if (old.data.network?.free) {
                            old.data.network.free();
                            console.log('Freed network wrapper for id=', old.id);
                        }
                    } catch (e) {
                        console.warn('Failed to free network wrapper for id=', old.id, e);
                    }
                }
            }
        } catch (e) {
            console.warn('Failed to cleanup old network wrappers:', e);
        }

        // Rebuild networks (cable + atmospheric)
        const newNetworks: GridNetwork[] = [];
        for (const net of _simulationManager.all_cable_networks()) {
            const managerId = net.id();
            const key = `cable-${managerId}`;
            const prev = networkPosMap.get(key);
            let id = prev?.id ?? `cable-${_cableNetworkCounter + 1}`;
            if (!prev) _cableNetworkCounter++;
            const networkData: NetworkNodeData = {
                id,
                type: 'cable',
                name: `Network ${net.id()}`,
                network: net,
                managerId: net.id(),
            };
            newNetworks.push({ id, data: networkData, x: prev?.x ?? 0, y: prev?.y ?? 0 });
        }

        // Build set of internal atmospheric networks to skip
        const internalAtmoIds = new Set<number>();
        for (const d of simDevices) {
            try {
                const internal = d.get_atmospheric_network(DeviceAtmosphericNetworkType.Internal);
                if (internal) {
                    const nid = internal.id?.();
                    if (nid) internalAtmoIds.add(nid);
                }
            } catch (_) {}
        }

        for (const net of _simulationManager.all_atmospheric_networks()) {
            const managerId = net.id();

            // Skip display of networks that are internal to devices
            if (internalAtmoIds.has(managerId)) {
                console.info('Skipping internal atmospheric network id=', managerId);
                continue;
            }

            const key = `atmo-${managerId}`;
            const prev = networkPosMap.get(key);
            let id = prev?.id ?? `atmo-${_atmosphericNetworkCounter + 1}`;
            if (!prev) _atmosphericNetworkCounter++;
            const networkData: NetworkNodeData = {
                id,
                type: 'atmospheric',
                name: `Atmosphere ${net.id()}`,
                network: net,
                managerId: net.id(),
            };
            newNetworks.push({ id, data: networkData, x: prev?.x ?? 0, y: prev?.y ?? 0 });
        }

        _gridNetworks = newNetworks;

        rebuildConnectionsFromWasm();
        debugSimulation();

        try {
            // Update the local state tick count from the WASM manager
            _tickCount = Number(_simulationManager.current_tick());
        } catch (_) {
            // ignore failures reading tick counter from WASM
        }
    } catch (e) {
        console.warn('Failed to sync from WASM:', e);
    }
}

function connectorTypeToAtmoType(type: ConnectorType): DeviceAtmosphericNetworkType | null {
    switch (type) {
        case 'atmo-input':
            return DeviceAtmosphericNetworkType.Input;
        case 'atmo-input2':
            return DeviceAtmosphericNetworkType.Input2;
        case 'atmo-output':
            return DeviceAtmosphericNetworkType.Output;
        case 'atmo-output2':
            return DeviceAtmosphericNetworkType.Output2;
        default:
            return null;
    }
}

function atmoTypeToConnectorType(atmoType: DeviceAtmosphericNetworkType): ConnectorType | null {
    switch (atmoType) {
        case DeviceAtmosphericNetworkType.Input:
            return 'atmo-input';
        case DeviceAtmosphericNetworkType.Input2:
            return 'atmo-input2';
        case DeviceAtmosphericNetworkType.Output:
            return 'atmo-output';
        case DeviceAtmosphericNetworkType.Output2:
            return 'atmo-output2';
        default:
            return null;
    }
}

// Rebuild connections array from WASM state
function rebuildConnectionsFromWasm(): void {
    if (!_simulationManager) return;

    const newConnections: Connection[] = [];

    // Build a map of network manager IDs to grid network IDs
    const cableNetworkMap = new Map<number, string>();
    const atmoNetworkMap = new Map<number, string>();
    for (const n of _gridNetworks) {
        if (n.data.type === 'cable') {
            cableNetworkMap.set(n.data.managerId, n.id);
        } else {
            atmoNetworkMap.set(n.data.managerId, n.id);
        }
    }

    // Check each device for connections
    for (const gd of _gridDevices) {
        const device = gd.device;
        const deviceId = gd.id;

        // Check cable network connection
        try {
            const cableNet = device.get_network();
            if (cableNet) {
                const netManagerId = cableNet.id();
                const networkId = cableNetworkMap.get(netManagerId);
                if (networkId) {
                    newConnections.push({
                        id: `conn-${deviceId}-cable-${networkId}`,
                        deviceId,
                        deviceConnectorType: 'cable',
                        networkId,
                        networkType: 'cable',
                    });
                }
            }
        } catch (e) {
            // Device may not support cable networks
        }

        // Check atmospheric network connections
        const atmoTypes = [
            DeviceAtmosphericNetworkType.Input,
            DeviceAtmosphericNetworkType.Input2,
            DeviceAtmosphericNetworkType.Output,
            DeviceAtmosphericNetworkType.Output2,
        ];

        for (const atmoType of atmoTypes) {
            try {
                const atmoNet = device.get_atmospheric_network(atmoType);
                if (atmoNet) {
                    const netManagerId = atmoNet.id();
                    const networkId = atmoNetworkMap.get(netManagerId);
                    if (networkId) {
                        const connectorType = atmoTypeToConnectorType(atmoType);
                        if (connectorType) {
                            newConnections.push({
                                id: `conn-${deviceId}-${connectorType}-${networkId}`,
                                deviceId,
                                deviceConnectorType: connectorType,
                                networkId,
                                networkType: 'atmospheric',
                                atmoConnectionType: atmoType,
                            });
                        }
                    }
                }
            } catch (e) {
                // Device may not support this atmospheric connection type
            }
        }
    }

    _connections = newConnections;
}

export function connectDeviceToCableNetwork(deviceId: number, networkId: string): boolean {
    if (!_simulationManager) return false;

    const gridDevice = _gridDevices.find((d) => d.id === deviceId);
    const gridNetwork = _gridNetworks.find((n) => n.id === networkId);

    if (!gridDevice || !gridNetwork || gridNetwork.data.type !== 'cable') {
        console.warn('Invalid device or network for cable connection');
        return false;
    }

    try {
        const network = gridDevice.device.get_network();
        network?.remove_device?.(gridDevice.device.id());
    } catch (e) {
        console.warn('Failed to remove device from existing cable network:', e);
        addNotification(
            'warning',
            `Failed to remove device from existing cable network: ${String(e)}`,
            3000
        );
    }

    try {
        const cableNetwork = gridNetwork.data.network as WasmCableNetwork;
        cableNetwork.add_device(gridDevice.device);
        console.log(`Connected device ${deviceId} to cable network ${networkId}`);
        syncFromWasm();
        return true;
    } catch (e) {
        console.error('Failed to connect device to cable network:', e);
        addNotification(
            'error',
            `Failed to connect device ${deviceId} to cable network: ${String(e)}`,
            5000
        );
        return false;
    }
}

export function connectDeviceToAtmosphericNetwork(
    deviceId: number,
    connectorType: ConnectorType,
    networkId: string
): boolean {
    if (!_simulationManager) return false;

    const gridDevice = _gridDevices.find((d) => d.id === deviceId);
    const gridNetwork = _gridNetworks.find((n) => n.id === networkId);

    if (!gridDevice || !gridNetwork || gridNetwork.data.type !== 'atmospheric') {
        console.warn('Invalid device or network for atmospheric connection');
        return false;
    }

    const atmoType = connectorTypeToAtmoType(connectorType);
    if (atmoType === null) {
        console.warn('Invalid connector type for atmospheric connection:', connectorType);
        return false;
    }

    try {
        const atmoNetwork = gridNetwork.data.network as WasmAtmosphericNetwork;
        gridDevice.device.set_atmospheric_network(atmoType, atmoNetwork);
        console.log(
            `Connected device ${deviceId} (${connectorType}) to atmospheric network ${networkId}`
        );
        syncFromWasm();
        return true;
    } catch (e) {
        console.error('Failed to connect device to atmospheric network:', e);
        addNotification(
            'error',
            `Failed to connect device ${deviceId} (${connectorType}) to atmospheric network: ${String(
                e
            )}`,
            5000
        );
        return false;
    }
}

export function disconnectDeviceFromCableNetwork(deviceId: number): boolean {
    if (!_simulationManager) return false;

    const gridDevice = _gridDevices.find((d) => d.id === deviceId);
    if (!gridDevice) {
        console.warn('Device not found:', deviceId);
        return false;
    }

    try {
        const net = gridDevice.device.get_network();
        if (net) {
            const refId = gridDevice.device.id();
            const removed = net.remove_device(refId);
            console.log(`Disconnected device ${deviceId} from cable network:`, removed);
            syncFromWasm();
            return removed;
        } else {
            console.warn('Device not connected to any cable network:', deviceId);
            return false;
        }
    } catch (e) {
        console.error('Failed to disconnect device from cable network:', e);
        return false;
    }
}

export function disconnectDeviceFromAtmosphericNetwork(
    deviceId: number,
    connectorType: ConnectorType
): boolean {
    if (!_simulationManager) return false;

    const gridDevice = _gridDevices.find((d) => d.id === deviceId);
    if (!gridDevice) {
        console.warn('Device not found:', deviceId);
        return false;
    }

    const atmoType = connectorTypeToAtmoType(connectorType);
    if (atmoType === null) {
        console.warn('Invalid connector type for atmospheric disconnection:', connectorType);
        return false;
    }

    try {
        gridDevice.device.clear_atmospheric_network(atmoType);
        console.log(`Disconnected device ${deviceId} (${connectorType}) from atmospheric network`);
        syncFromWasm();
        return true;
    } catch (e) {
        console.error('Failed to disconnect device from atmospheric network:', e);
        return false;
    }
}

export function removeConnection(connectionId: string): boolean {
    const connection = _connections.find((c) => c.id === connectionId);
    if (!connection) {
        console.warn('Connection not found:', connectionId);
        return false;
    }

    if (connection.networkType === 'cable') {
        return disconnectDeviceFromCableNetwork(connection.deviceId);
    } else {
        return disconnectDeviceFromAtmosphericNetwork(
            connection.deviceId,
            connection.deviceConnectorType
        );
    }
}

export function getConnection(
    deviceId: number,
    connectorType: ConnectorType
): Connection | undefined {
    return _connections.find(
        (c) => c.deviceId === deviceId && c.deviceConnectorType === connectorType
    );
}

export function getNetworkConnections(networkId: string): Connection[] {
    return _connections.filter((c) => c.networkId === networkId);
}

export function getDeviceConnections(deviceId: number): Connection[] {
    return _connections.filter((c) => c.deviceId === deviceId);
}

export function stepTicks(ticks: number) {
    if (!_simulationManager) return null;
    try {
        for (let i = 0; i < ticks; i++) {
            const changes = _simulationManager.update();
            syncFromWasm();

            // If no changes occurred in this tick, warn the user and stop auto-stepping if active
            if (changes === 0) {
                if (_autoRunning) {
                    stopAutoStep();
                    addNotification(
                        'warning',
                        'No changes occurred during the last step; auto-stepping has been stopped.',
                        3000
                    );
                } else {
                    addNotification('warning', 'No changes occurred during the last step.', 3000);
                }
                break;
            }
        }
        return _tickCount;
    } catch (e) {
        console.error('Failed to step simulation:', e);
        // Stop auto-step if active and notify the user
        if (_autoRunning) {
            stopAutoStep();
        }
        addNotification('error', `Simulation step failed: ${String(e)}`, null);
        return null;
    }
}

// Tick counter and auto-step management
let _tickCount: number = $state(0);

// Auto-step management: allows starting/stopping a periodic stepping at a specified ticks-per-second rate (1..64)
let _autoStepTimer: number | null = $state(null);
let _autoStepRate: number = $state(1);
let _autoRunning: boolean = $state(false);

export function startAutoStep(rate: number): boolean {
    if (!_simulationManager) return false;
    if (rate < 1) rate = 1;
    if (rate > 64) rate = 64;

    // Clear existing timer if any
    if (_autoStepTimer !== null) {
        try {
            clearInterval(_autoStepTimer);
        } catch (e) {
            // ignore
        }
        _autoStepTimer = null;
    }

    _autoStepRate = Math.round(rate);
    _autoRunning = true;

    // Notify UI that auto-step changed
    try {
        window.dispatchEvent(
            new CustomEvent('sim:autoStepChanged', { detail: { running: _autoRunning } })
        );
    } catch (e) {
        // ignore (e.g., server-side or non-window environments)
    }

    // Interval such that we advance 1 tick per interval and hit approximately `rate` ticks per second.
    const intervalMs = Math.max(10, Math.round(1000 / _autoStepRate));

    _autoStepTimer = window.setInterval(() => {
        stepTicks(1);
    }, intervalMs);

    window.setTimeout(() => {
        stepTicks(1);
    }, 0);

    return true;
}

export function stopAutoStep(): void {
    if (_autoStepTimer !== null) {
        try {
            clearInterval(_autoStepTimer);
        } catch (e) {
            // ignore
        }
        _autoStepTimer = null;
    }
    _autoRunning = false;

    try {
        window.dispatchEvent(
            new CustomEvent('sim:autoStepChanged', { detail: { running: _autoRunning } })
        );
    } catch (e) {
        // ignore
    }
}

export function isAutoStepping(): boolean {
    return _autoRunning;
}

export function getAutoStepRate(): number {
    return _autoStepRate;
}
