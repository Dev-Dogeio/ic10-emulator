<script lang="ts">
    import type { DevicePrefabInfo } from '../../pkg/ic10_emulator';

    interface Props {
        prefabs: DevicePrefabInfo[];
        onDragStart?: (prefab: DevicePrefabInfo) => void;
    }

    let { prefabs, onDragStart }: Props = $props();

    function handleDragStart(e: DragEvent, prefab: DevicePrefabInfo) {
        if (e.dataTransfer) {
            e.dataTransfer.setData(
                'application/json',
                JSON.stringify({
                    type: 'device-prefab',
                    prefabHash: prefab.prefab_hash,
                    deviceName: prefab.device_name,
                })
            );
            e.dataTransfer.effectAllowed = 'copy';
        }
        if (onDragStart) {
            onDragStart(prefab);
        }
    }

    function getDeviceCategory(prefab: DevicePrefabInfo): string {
        if (prefab.is_ic_host) return 'IC Hosts';
        if (prefab.is_atmospheric_device) return 'Atmospheric';
        return 'Logic';
    }

    function getDeviceIcon(prefab: DevicePrefabInfo): string {
        if (prefab.is_ic_host) return '🖥️';
        if (prefab.is_atmospheric_device) return '💨';
        return '📊';
    }

    let groupedPrefabs = $derived(() => {
        const groups: Record<string, DevicePrefabInfo[]> = {};
        for (const prefab of prefabs) {
            const category = getDeviceCategory(prefab);
            if (!groups[category]) {
                groups[category] = [];
            }
            groups[category].push(prefab);
        }
        return groups;
    });
</script>

<div class="device-list">
    <div class="header">
        <h2>Devices</h2>
        <span class="count">{prefabs.length}</span>
    </div>

    <div class="scroll-area">
        {#each Object.entries(groupedPrefabs()) as [category, categoryPrefabs]}
            <div class="category">
                <h3 class="category-header">{category}</h3>
                <div class="category-items">
                    {#each categoryPrefabs as prefab}
                        <div
                            class="device-item"
                            draggable="true"
                            role="button"
                            tabindex="0"
                            ondragstart={(e) => handleDragStart(e, prefab)}
                            title={`Hash: ${prefab.prefab_hash}\nProperties: ${prefab.properties.length}\nAtmo Connections: ${prefab.atmospheric_connections.length}`}
                        >
                            <span class="icon">{getDeviceIcon(prefab)}</span>
                            <span class="name">{prefab.device_name}</span>
                            <div class="badges">
                                {#if prefab.is_ic_host}
                                    <span class="badge ic">IC</span>
                                {/if}
                                {#if prefab.is_atmospheric_device}
                                    <span class="badge atmo">ATMO</span>
                                {/if}
                                {#if prefab.is_slot_host}
                                    <span class="badge slot">SLOT</span>
                                {/if}
                                {#if prefab.properties && prefab.properties.length > 0}
                                    <span class="badge logic">LOGIC</span>
                                {/if}
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {/each}
    </div>
</div>

<style>
    .device-list {
        display: flex;
        flex-direction: column;
        width: 310px;
        height: 100%;
        background: #16162a;
        border-right: 1px solid rgba(255, 255, 255, 0.1);
        color: #e0e0e0;
    }

    .badge.logic {
        background: rgba(206, 204, 124, 0.08);
        color: #e2ed81;
    }

    .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 16px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .header h2 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #fff;
    }

    .count {
        background: rgba(99, 102, 241, 0.2);
        color: #818cf8;
        padding: 2px 8px;
        border-radius: 12px;
        font-size: 12px;
        font-weight: 500;
    }

    .scroll-area {
        flex: 1;
        overflow-y: auto;
        padding: 8px;
    }

    .scroll-area::-webkit-scrollbar {
        width: 8px;
    }

    .scroll-area::-webkit-scrollbar-track {
        background: transparent;
    }

    .scroll-area::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.2);
        border-radius: 4px;
    }

    .scroll-area::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.3);
    }

    .category {
        margin-bottom: 16px;
    }

    .category-header {
        font-size: 11px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: rgba(255, 255, 255, 0.5);
        padding: 8px 8px 4px 8px;
        margin: 0;
    }

    .category-items {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .device-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 12px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 8px;
        cursor: grab;
        transition: all 0.15s ease;
        border: 1px solid transparent;
    }

    .device-item:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(99, 102, 241, 0.3);
    }

    .device-item:active {
        cursor: grabbing;
        transform: scale(0.98);
    }

    .device-item:focus-visible {
        outline: 2px solid #818cf8;
        outline-offset: 2px;
    }

    .icon {
        font-size: 18px;
        flex-shrink: 0;
    }

    .name {
        flex: 1;
        font-size: 13px;
        font-weight: 500;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .badges {
        display: flex;
        gap: 4px;
        flex-shrink: 0;
    }

    .badge {
        font-size: 9px;
        font-weight: 600;
        padding: 2px 5px;
        border-radius: 4px;
        text-transform: uppercase;
    }

    .badge.ic {
        background: rgba(34, 197, 94, 0.2);
        color: #4ade80;
    }

    .badge.atmo {
        background: rgba(59, 130, 246, 0.2);
        color: #60a5fa;
    }

    .badge.slot {
        background: rgba(251, 191, 36, 0.2);
        color: #fbbf24;
    }
</style>
