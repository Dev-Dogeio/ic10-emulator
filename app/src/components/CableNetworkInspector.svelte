<script lang="ts">
    import { type WasmCableNetwork } from '../../pkg/ic10_emulator';
    import { type InspectorWindow } from '../stores/inspectorState.svelte';
    import { getSimulationState } from '../stores/simulationState.svelte';

    interface Props {
        window: InspectorWindow;
        network: WasmCableNetwork;
        networkName: string;
    }

    let { window, network, networkName }: Props = $props();

    const simState = getSimulationState();

    // Get connected devices
    function getConnectedDevices(): { id: number; name: string; prefabName: string }[] {
        try {
            const deviceIds = Array.from(network.device_ids());
            return deviceIds.map((id) => {
                const gridDevice = simState.gridDevices.find((d) => d.id === id);
                return {
                    id,
                    name: gridDevice?.device.name() ?? `Device ${id}`,
                    prefabName: gridDevice?.prefabInfo.device_name ?? 'Unknown',
                };
            });
        } catch {
            return [];
        }
    }

    let connectedDevices = $derived(getConnectedDevices());
    let deviceCount = $derived(() => {
        try {
            return network.device_count();
        } catch {
            return 0;
        }
    });
</script>

<div class="cable-inspector">
    <div class="header-section">
        <h3>{networkName}</h3>
        <span class="network-id">ID: {network.id()}</span>
    </div>

    <div class="stats-section">
        <div class="stat">
            <span class="stat-icon">🔌</span>
            <span class="stat-label">Connected Devices</span>
            <span class="stat-value">{deviceCount()}</span>
        </div>
    </div>

    <div class="devices-section">
        <h4>Connected Devices</h4>
        {#if connectedDevices.length === 0}
            <div class="empty-state">No devices connected</div>
        {:else}
            <div class="device-list">
                {#each connectedDevices as device}
                    <div class="device-item">
                        <span class="device-icon">📊</span>
                        <div class="device-info">
                            <span class="device-name">{device.name}</span>
                            <span class="device-type">{device.prefabName}</span>
                        </div>
                        <span class="device-id">#{device.id}</span>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>

<style>
    .cable-inspector {
        display: flex;
        flex-direction: column;
        gap: 16px;
        padding: 16px;
        color: #e0e0e0;
    }

    .header-section {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .header-section h3 {
        margin: 0;
        font-size: 16px;
        color: #fbbf24;
    }

    .network-id {
        font-size: 11px;
        font-family: 'JetBrains Mono', monospace;
        color: rgba(255, 255, 255, 0.5);
    }

    .stats-section {
        display: flex;
        gap: 12px;
    }

    .stat {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 14px;
        background: rgba(251, 191, 36, 0.1);
        border: 1px solid rgba(251, 191, 36, 0.2);
        border-radius: 8px;
        flex: 1;
    }

    .stat-icon {
        font-size: 18px;
    }

    .stat-label {
        font-size: 11px;
        color: rgba(255, 255, 255, 0.6);
        flex: 1;
    }

    .stat-value {
        font-size: 18px;
        font-weight: 600;
        color: #fbbf24;
    }

    .devices-section {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .devices-section h4 {
        margin: 0;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .empty-state {
        padding: 20px;
        text-align: center;
        color: rgba(255, 255, 255, 0.4);
        font-size: 13px;
    }

    .device-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .device-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 12px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .device-icon {
        font-size: 16px;
    }

    .device-info {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .device-name {
        font-size: 13px;
        font-weight: 500;
    }

    .device-type {
        font-size: 10px;
        color: rgba(255, 255, 255, 0.5);
    }

    .device-id {
        font-size: 11px;
        font-family: 'JetBrains Mono', monospace;
        color: rgba(255, 255, 255, 0.4);
    }
</style>
