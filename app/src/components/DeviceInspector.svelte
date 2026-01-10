<script lang="ts">
    import {
        type WasmDevice,
        type DevicePrefabInfo,
        LogicType,
        LogicSlotType,
        get_registered_item_prefabs,
        get_item_prefab_info,
    } from '../../pkg/ic10_emulator';
    import { type InspectorWindow, setActiveTab } from '../stores/inspectorState.svelte';
    import { getSimulationState, syncFromWasm } from '../stores/simulationState.svelte';

    interface Props {
        window: InspectorWindow;
        device: WasmDevice;
        prefabInfo: DevicePrefabInfo;
    }

    let { window, device, prefabInfo }: Props = $props();

    const simState = getSimulationState();

    // Tabs
    type TabId = 'overview' | 'logic' | 'slots' | 'ic';
    let activeTab: TabId = $derived(window.activeTab as TabId);

    function switchTab(tab: TabId) {
        setActiveTab(window.id, tab);
    }

    // Available tabs based on device capabilities
    let availableTabs = $derived(() => {
        const tabs: { id: TabId; label: string; icon: string }[] = [
            { id: 'overview', label: 'Overview', icon: '📋' },
            { id: 'logic', label: 'Logic', icon: '⚙️' },
        ];

        if (prefabInfo.is_slot_host) {
            tabs.push({ id: 'slots', label: 'Slots', icon: '📦' });
        }

        if (prefabInfo.is_ic_host) {
            tabs.push({ id: 'ic', label: 'IC Chip', icon: '🖥️' });
        }

        return tabs;
    });

    // Device name editing
    let isEditingName = $state(false);
    let editingName = $state('');

    function startEditingName() {
        editingName = device.name();
        isEditingName = true;
    }

    function saveName() {
        if (editingName.trim()) {
            device.rename(editingName.trim());
            syncFromWasm();
        }
        isEditingName = false;
    }

    function cancelEditingName() {
        isEditingName = false;
    }

    // Logic properties
    interface LogicProperty {
        type: LogicType;
        name: string;
        value: number | null;
        readable: boolean;
        writable: boolean;
    }

    function getLogicProperties(): LogicProperty[] {
        const props: LogicProperty[] = [];

        for (const prop of prefabInfo.properties) {
            let value: number | null = null;
            if (prop.readable) {
                try {
                    value = device.read(prop.logic);
                } catch (e) {
                    value = null;
                }
            }

            props.push({
                type: prop.logic,
                name: prop.logic_name,
                value,
                readable: prop.readable,
                writable: prop.writable,
            });
        }

        return props;
    }

    let logicProperties = $state<LogicProperty[]>([]);

    $effect(() => {
        // Refresh properties when tab is active
        if (activeTab === 'logic' || activeTab === 'overview') {
            logicProperties = getLogicProperties();
        }
    });

    function updateLogicValue(prop: LogicProperty, value: number) {
        if (prop.writable) {
            if (!Number.isFinite(value)) return;
            try {
                device.write(prop.type, value);
                syncFromWasm();
                try {
                    const readBack = prop.readable ? device.read(prop.type) : value;
                    prop.value = readBack;
                    logicProperties = getLogicProperties();
                } catch {
                    logicProperties = getLogicProperties();
                }
            } catch (e) {
                console.error('Failed to write logic value:', e);
            }
        }
    }

    // Slot properties
    interface SlotInfo {
        index: number;
        occupied: boolean;
        itemHash: number;
        itemName: string;
        quantity: number;
        maxQuantity: number;
    }

    function getSlotInfo(): SlotInfo[] {
        const slots: SlotInfo[] = [];

        // Check how many slots exist by trying to read them
        for (let i = 0; i < 10; i++) {
            try {
                const occupied = device.read_slot(i, LogicSlotType.Occupied) === 1;
                const itemHash = device.read_slot(i, LogicSlotType.OccupantHash);
                const quantity = device.read_slot(i, LogicSlotType.Quantity);
                const maxQuantity = device.read_slot(i, LogicSlotType.MaxQuantity);

                let itemName = '';
                if (itemHash && itemHash !== 0) {
                    try {
                        const info = get_item_prefab_info(itemHash);
                        itemName = info.name;
                    } catch {
                        itemName = `#${itemHash}`;
                    }
                }

                slots.push({
                    index: i,
                    occupied,
                    itemHash,
                    itemName,
                    quantity,
                    maxQuantity,
                });
            } catch (e) {
                // No more slots
                break;
            }
        }

        return slots;
    }

    let slotInfoList = $state<SlotInfo[]>([]);

    $effect(() => {
        if (activeTab === 'slots' && prefabInfo.is_slot_host) {
            slotInfoList = getSlotInfo();
        }
    });

    function removeItemFromSlot(index: number) {
        try {
            device.remove_item_from_slot(index);
            syncFromWasm();
            slotInfoList = getSlotInfo();
        } catch (e) {
            console.error('Failed to remove item from slot:', e);
        }
    }

    // Registered item prefabs for insertion menu
    let registeredItems = $derived(() => {
        try {
            const arr = get_registered_item_prefabs();
            const list: { name: string; prefab_hash: number; item_type: string }[] = [];
            for (let i = 0; i < arr.length; i++) {
                try {
                    const info = get_item_prefab_info(arr[i]);
                    list.push({
                        name: info.name,
                        prefab_hash: info.prefab_hash,
                        item_type: info.item_type,
                    });
                } catch {
                    // skip invalid entries
                }
            }
            return list;
        } catch {
            return [];
        }
    });

    // Insert menu state
    let insertIndexOpen = $state<number | null>(null);
    let selectedItemHash = $state<number | null>(null);

    function openInsertMenu(index: number) {
        insertIndexOpen = index;
        const items = registeredItems();
        selectedItemHash = items.length > 0 ? items[0].prefab_hash : null;
    }

    function closeInsertMenu() {
        insertIndexOpen = null;
        selectedItemHash = null;
    }

    function insertItemIntoSlot(index: number) {
        try {
            const manager = simState.simulationManager;
            if (!manager) return;
            if (selectedItemHash == null) return;

            const prefabHash = Number(selectedItemHash);
            if (Number.isNaN(prefabHash)) return;

            const item = manager.create_item(prefabHash);
            const leftover = device.insert_item_into_slot(index, item);
            if (leftover) {
                console.warn('Item could not be inserted into slot; leftover returned.');
            }
            syncFromWasm();
            slotInfoList = getSlotInfo();
            closeInsertMenu();
        } catch (e) {
            console.error('Failed to insert item into slot:', e);
        }
    }

    // IC Chip handling
    let icCode = $state('');
    let icHasChip = $derived(() => {
        if (!prefabInfo.is_ic_host) return false;
        try {
            return device.has_chip();
        } catch {
            return false;
        }
    });

    // Error state for code set failures
    let codeError = $state<string | null>(null);

    function pushCodeToChip() {
        if (!prefabInfo.is_ic_host) return;
        try {
            if (!icHasChip()) {
                codeError = 'No chip installed';
                return;
            }

            device.set_code(icCode ?? '');
            syncFromWasm();
            codeError = null;
        } catch (e) {
            console.error('Failed to set code on chip:', e);
            const msg = e?.toString ? e.toString() : String(e);
            codeError = `Failed to load program: ${msg}`;
        }
    }

    // Only push code once when entering the IC tab or when a chip becomes attached.
    // Use previous-state flags to detect transitions and avoid triggering on every keystroke.
    let _prevActiveIsIC = false;
    let _prevHasChip = false;

    $effect(() => {
        const activeIsIC = activeTab === 'ic' && prefabInfo.is_ic_host;
        const hasChipNow = icHasChip();

        // Entering IC tab
        if (activeIsIC && !_prevActiveIsIC) {
            if (hasChipNow && icCode && icCode.trim() && !codeError) {
                pushCodeToChip();
            }
        }

        // Chip was attached while IC tab is active
        if (hasChipNow && !_prevHasChip) {
            if (activeIsIC && icCode && icCode.trim() && !codeError) {
                pushCodeToChip();
            }
        }

        _prevActiveIsIC = activeIsIC;
        _prevHasChip = hasChipNow;
    });

    // Device pin configuration
    let devicePins = $state<(number | null)[]>([]);

    function loadDevicePins() {
        if (!prefabInfo.is_ic_host) return;
        const count = device.get_device_pin_count();
        const pins: (number | null)[] = [];
        for (let i = 0; i < count; i++) {
            const pin = device.get_device_pin(i);
            pins.push(pin ?? null);
        }
        devicePins = pins;
    }

    $effect(() => {
        if (activeTab === 'ic' && prefabInfo.is_ic_host) {
            loadDevicePins();
        }
    });

    let devicePinHeader = $derived(() => {
        if (!prefabInfo.is_ic_host) return 'Device Pins';
        try {
            const count = device.get_device_pin_count();
            if (count === 0) return 'Device Pins (none)';
            return `Device Pins (d0~d${Math.max(0, count - 1)})`;
        } catch {
            return 'Device Pins';
        }
    });

    function setDevicePin(index: number, refId: number | null) {
        try {
            device.set_device_pin(index, refId);
            loadDevicePins();
            syncFromWasm();
        } catch (e) {
            console.error('Failed to set device pin:', e);
        }
    }

    // Get list of all devices on the same network for pin selection
    function getNetworkDevices(): { id: number; name: string }[] {
        try {
            const network = device.get_network();
            if (!network) return [];

            const deviceIds = Array.from(network.device_ids());
            return deviceIds
                .filter((id) => id !== device.id())
                .map((id) => {
                    const d = simState.gridDevices.find((gd) => gd.id === id);
                    return {
                        id,
                        name: d?.device.name() ?? `Device ${id}`,
                    };
                });
        } catch {
            return [];
        }
    }

    let networkDevices = $derived(getNetworkDevices());

    function installChip() {
        try {
            const manager = simState.simulationManager;
            if (!manager) return;

            const chip = manager.create_chip();
            device.set_chip(chip);
            // If code is present in the editor, load it into the newly installed chip
            if (icCode && icCode.trim()) {
                pushCodeToChip();
            }

            syncFromWasm();
            slotInfoList = getSlotInfo();
        } catch (e) {
            console.error('Failed to install chip:', e);
        }
    }

    // Quick stats for overview
    let quickStats = $derived(() => {
        const stats: { label: string; value: string }[] = [
            { label: 'Reference ID', value: String(device.id()) },
            { label: 'Prefab Hash', value: String(prefabInfo.prefab_hash) },
            { label: 'Name Hash', value: String(device.name_hash()) },
            { label: 'Network', value: String(device.get_network()?.id?.() ?? 'None') },
        ];

        return stats;
    });
</script>

<div class="device-inspector">
    <!-- Tab bar -->
    <div class="tab-bar">
        {#each availableTabs() as tab (tab.id)}
            <button
                class="tab-btn"
                class:active={activeTab === tab.id}
                onclick={() => switchTab(tab.id)}
            >
                <span class="tab-icon">{tab.icon}</span>
                <span class="tab-label">{tab.label}</span>
            </button>
        {/each}
    </div>

    <!-- Tab content -->
    <div class="tab-content">
        {#if activeTab === 'overview'}
            <div class="overview-section">
                <!-- Name editing -->
                <div class="name-section">
                    {#if isEditingName}
                        <input
                            type="text"
                            class="name-input"
                            bind:value={editingName}
                            onkeydown={(e) => {
                                if (e.key === 'Enter') saveName();
                                if (e.key === 'Escape') cancelEditingName();
                            }}
                            onblur={saveName}
                        />
                    {:else}
                        <h3 class="device-name" ondblclick={startEditingName}>
                            {device.name()}
                            <button class="edit-btn" onclick={startEditingName}>✏️</button>
                        </h3>
                    {/if}
                    <span class="device-type">{prefabInfo.device_name}</span>
                </div>

                <!-- Quick stats -->
                <div class="stats-grid">
                    {#each quickStats() as stat}
                        <div class="stat-item">
                            <span class="stat-label">{stat.label}</span>
                            <span class="stat-value">{stat.value}</span>
                        </div>
                    {/each}
                </div>

                <!-- Capability badges -->
                <div class="capability-badges">
                    {#if prefabInfo.is_ic_host}
                        <span class="badge ic">IC Host</span>
                    {/if}
                    {#if prefabInfo.is_slot_host}
                        <span class="badge slot">Slot Host</span>
                    {/if}
                    {#if prefabInfo.is_atmospheric_device}
                        <span class="badge atmo">Atmospheric</span>
                    {/if}
                    {#if prefabInfo.properties && prefabInfo.properties.length > 0}
                        <span class="badge logic">Logic</span>
                    {/if}
                </div>

                <!-- Key logic values preview -->
                <div class="key-values">
                    <h4>Key Values</h4>
                    {#each logicProperties.slice(0, 5) as prop}
                        <div class="value-row">
                            <span class="value-name">{prop.name}</span>
                            <span class="value-data"
                                >{prop.value !== null ? prop.value.toFixed(2) : '—'}</span
                            >
                        </div>
                    {/each}
                </div>
            </div>
        {:else if activeTab === 'logic'}
            <div class="logic-section">
                <div class="logic-list">
                    {#each logicProperties as prop}
                        <div class="logic-row" class:readonly={!prop.writable}>
                            <span class="logic-name">{prop.name}</span>
                            <div class="logic-value-container">
                                {#if prop.writable}
                                    <input
                                        type="number"
                                        class="logic-input"
                                        value={prop.value ?? ''}
                                        step="any"
                                        onchange={(e) =>
                                            updateLogicValue(
                                                prop,
                                                parseFloat(e.currentTarget.value)
                                            )}
                                    />
                                {:else}
                                    <span class="logic-value"
                                        >{prop.value !== null ? prop.value.toFixed(4) : '—'}</span
                                    >
                                {/if}
                                <span class="logic-rw">
                                    {prop.readable ? 'R' : ''}{prop.writable ? 'W' : ''}
                                </span>
                            </div>
                        </div>
                    {/each}
                </div>
            </div>
        {:else if activeTab === 'slots'}
            <div class="slots-section">
                {#if slotInfoList.length === 0}
                    <div class="empty-state">No slots available</div>
                {:else}
                    <div class="slots-list">
                        {#each slotInfoList as slot}
                            <div class="slot-row" class:occupied={slot.occupied}>
                                <span class="slot-index">Slot {slot.index}</span>
                                {#if slot.occupied}
                                    <div class="slot-info">
                                        <span class="slot-hash">{slot.itemName}</span>
                                        <span class="slot-qty"
                                            >{slot.quantity}/{slot.maxQuantity}</span
                                        >
                                        <button
                                            class="slot-remove-btn"
                                            onclick={() => removeItemFromSlot(slot.index)}
                                        >
                                            ✕
                                        </button>
                                    </div>
                                {:else}
                                    <div class="slot-empty-container">
                                        {#if insertIndexOpen !== slot.index}
                                            <span class="slot-empty">Empty</span>
                                        {/if}
                                        <button
                                            class="slot-insert-btn"
                                            onclick={() => openInsertMenu(slot.index)}
                                            >Insert</button
                                        >
                                        {#if insertIndexOpen === slot.index}
                                            <div class="insert-menu">
                                                <select bind:value={selectedItemHash}>
                                                    {#each registeredItems() as item}
                                                        <option value={item.prefab_hash}
                                                            >{item.name} ({item.item_type})</option
                                                        >
                                                    {/each}
                                                </select>
                                                <div class="insert-actions">
                                                    <button
                                                        class="btn small"
                                                        onclick={() =>
                                                            insertItemIntoSlot(slot.index)}
                                                        >Insert</button
                                                    >
                                                    <button
                                                        class="btn small"
                                                        onclick={() => closeInsertMenu()}
                                                        >Cancel</button
                                                    >
                                                </div>
                                            </div>
                                        {/if}
                                    </div>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {:else if activeTab === 'ic'}
            <div class="ic-section">
                <!-- Chip status -->
                <div class="chip-status">
                    {#if icHasChip()}
                        <span class="chip-badge installed">IC10 Installed</span>
                    {:else}
                        <span class="chip-badge not-installed">No Chip</span>
                        <button class="install-btn" onclick={installChip}>Install IC10</button>
                    {/if}
                </div>

                <!-- Device Pins -->
                <div class="pins-section">
                    <h4>{devicePinHeader()}</h4>

                    {#if codeError}
                        <div class="code-error-popup">
                            <div class="popup-content">
                                <h4>Error loading code</h4>
                                <p>{codeError}</p>
                                <div class="popup-actions">
                                    <button class="btn" onclick={() => (codeError = null)}
                                        >OK</button
                                    >
                                    <button class="btn" onclick={() => pushCodeToChip()}
                                        >Retry</button
                                    >
                                </div>
                            </div>
                        </div>
                    {/if}

                    <div class="pins-list">
                        {#each devicePins as pin, i}
                            <div class="pin-row">
                                <span class="pin-label">d{i}</span>
                                <select
                                    class="pin-select"
                                    value={pin ?? ''}
                                    onchange={(e) => {
                                        const val = e.currentTarget.value;
                                        setDevicePin(i, val ? parseInt(val) : null);
                                    }}
                                >
                                    <option value="">Not Connected</option>
                                    {#each networkDevices as dev}
                                        <option value={dev.id}>{dev.name} (#{dev.id})</option>
                                    {/each}
                                </select>
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Code editor (simplified) -->
                {#if icHasChip()}
                    <div class="code-section">
                        <h4>IC10 Code</h4>
                        <textarea
                            class="code-editor"
                            bind:value={icCode}
                            placeholder="# Enter IC10 code here..."
                            rows="10"
                            onblur={() => pushCodeToChip()}
                        ></textarea>
                    </div>
                {/if}
            </div>
        {/if}
    </div>
</div>

<style>
    .device-inspector {
        display: flex;
        flex-direction: column;
        height: 100%;
        color: #e0e0e0;
    }

    .tab-bar {
        display: flex;
        gap: 2px;
        padding: 8px 8px 0;
        background: rgba(0, 0, 0, 0.2);
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    }

    .tab-btn {
        display: flex;
        align-items: center;
        gap: 6px;
        padding: 8px 12px;
        border: none;
        border-radius: 6px 6px 0 0;
        background: transparent;
        color: rgba(255, 255, 255, 0.5);
        font-size: 12px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .tab-btn:hover {
        background: rgba(255, 255, 255, 0.05);
        color: rgba(255, 255, 255, 0.8);
    }

    .tab-btn.active {
        background: #1a1a2e;
        color: #818cf8;
    }

    .tab-icon {
        font-size: 12px;
    }

    .tab-content {
        flex: 1;
        overflow: auto;
        padding: 12px;
    }

    /* Overview Section */
    .overview-section {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .name-section {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .device-name {
        display: flex;
        align-items: center;
        gap: 8px;
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #fff;
    }

    .edit-btn {
        background: none;
        border: none;
        padding: 2px;
        cursor: pointer;
        opacity: 0.5;
        font-size: 12px;
    }

    .edit-btn:hover {
        opacity: 1;
    }

    .name-input {
        padding: 6px 10px;
        border: 1px solid #818cf8;
        border-radius: 6px;
        background: #252542;
        color: #fff;
        font-size: 16px;
        font-weight: 600;
        outline: none;
    }

    .device-type {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.5);
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 8px;
    }

    .stat-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 8px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 6px;
    }

    .stat-label {
        font-size: 10px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .stat-value {
        font-size: 13px;
        font-weight: 500;
        font-family: 'JetBrains Mono', monospace;
    }

    .capability-badges {
        display: flex;
        gap: 6px;
        flex-wrap: wrap;
    }

    .badge {
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 10px;
        font-weight: 600;
        text-transform: uppercase;
    }

    .badge.ic {
        background: rgba(34, 197, 94, 0.2);
        color: #4ade80;
    }

    .badge.slot {
        background: rgba(251, 191, 36, 0.2);
        color: #fbbf24;
    }

    .badge.atmo {
        background: rgba(96, 165, 250, 0.2);
        color: #60a5fa;
    }

    .badge.logic {
        background: rgba(206, 204, 124, 0.08);
        color: #e2ed81;
    }

    .key-values {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .key-values h4 {
        margin: 0;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .value-row {
        display: flex;
        justify-content: space-between;
        padding: 6px 8px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 4px;
    }

    .value-name {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.7);
    }

    .value-data {
        font-size: 12px;
        font-family: 'JetBrains Mono', monospace;
        color: #818cf8;
    }

    /* Logic Section */
    .logic-section {
        display: flex;
        flex-direction: column;
    }

    .logic-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .logic-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 10px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 6px;
        transition: background 0.15s ease;
    }

    .logic-row:hover {
        background: rgba(255, 255, 255, 0.06);
    }

    .logic-row.readonly {
        opacity: 0.7;
    }

    .logic-name {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.8);
    }

    .logic-value-container {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .logic-input {
        width: 100px;
        padding: 4px 8px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 4px;
        background: #252542;
        color: #fff;
        font-size: 12px;
        font-family: 'JetBrains Mono', monospace;
        text-align: right;
    }

    .logic-input:focus {
        border-color: #818cf8;
        outline: none;
    }

    .logic-value {
        font-size: 12px;
        font-family: 'JetBrains Mono', monospace;
        color: #818cf8;
    }

    .logic-rw {
        font-size: 9px;
        padding: 2px 4px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 3px;
        color: rgba(255, 255, 255, 0.5);
    }

    /* Slots Section */
    .slots-section {
        display: flex;
        flex-direction: column;
    }

    .empty-state {
        padding: 24px;
        text-align: center;
        color: rgba(255, 255, 255, 0.4);
    }

    .slots-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .slot-row {
        position: relative;
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 10px 12px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 6px;
        border: 1px solid transparent;
        overflow: visible;
    }

    .slot-row.occupied {
        border-color: rgba(251, 191, 36, 0.3);
        background: rgba(251, 191, 36, 0.05);
    }

    .slot-index {
        font-size: 12px;
        font-weight: 500;
    }

    .slot-info {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .slot-hash {
        font-size: 11px;
        font-family: 'JetBrains Mono', monospace;
        color: rgba(255, 255, 255, 0.6);
    }

    .slot-qty {
        font-size: 11px;
        color: #fbbf24;
    }

    .slot-remove-btn {
        padding: 2px 6px;
        border: none;
        border-radius: 3px;
        background: rgba(239, 68, 68, 0.2);
        color: #f87171;
        cursor: pointer;
        font-size: 10px;
    }

    .slot-remove-btn:hover {
        background: rgba(239, 68, 68, 0.4);
    }

    .slot-empty {
        font-size: 11px;
        color: rgba(255, 255, 255, 0.4);
        font-style: italic;
    }

    .slot-empty-container {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .slot-insert-btn {
        padding: 4px 8px;
        border-radius: 4px;
        border: none;
        background: rgba(34, 197, 94, 0.12);
        color: #bbf7d0;
        cursor: pointer;
        font-size: 11px;
    }

    .slot-insert-btn:hover {
        background: rgba(34, 197, 94, 0.2);
    }

    .insert-menu {
        position: absolute;
        right: 8px;
        top: calc(100% + 6px);
        display: flex;
        align-items: center;
        gap: 8px;
        background: rgba(15, 23, 42, 0.98);
        padding: 6px;
        border-radius: 6px;
        box-shadow: 0 8px 20px rgba(2, 6, 23, 0.6);
        z-index: 40;
        max-width: min(360px, 70vw);
    }

    .insert-menu select {
        background: #0f1724;
        color: #fff;
        padding: 6px 8px;
        border-radius: 4px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        font-size: 12px;
        width: 160px;
        max-width: 220px;
    }

    .insert-actions .btn {
        padding: 4px 8px;
        border-radius: 4px;
        border: none;
        background: rgba(99, 102, 241, 0.12);
    }

    .insert-actions .btn.small {
        padding: 4px 6px;
        font-size: 11px;
        color: #a5b4fc;
        cursor: pointer;
        font-size: 11px;
    }

    .insert-actions .btn:hover {
        background: rgba(99, 102, 241, 0.2);
    }

    /* IC Section */
    .ic-section {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .chip-status {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .chip-badge {
        padding: 6px 12px;
        border-radius: 6px;
        font-size: 11px;
        font-weight: 600;
    }

    .chip-badge.installed {
        background: rgba(34, 197, 94, 0.2);
        color: #4ade80;
    }

    .chip-badge.not-installed {
        background: rgba(239, 68, 68, 0.2);
        color: #f87171;
    }

    .install-btn {
        padding: 6px 12px;
        border: none;
        border-radius: 6px;
        background: #818cf8;
        color: #fff;
        font-size: 11px;
        font-weight: 600;
        cursor: pointer;
    }

    .install-btn:hover {
        background: #6366f1;
    }

    .pins-section h4,
    .code-section h4 {
        margin: 0 0 8px 0;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .pins-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .pin-row {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .pin-label {
        width: 24px;
        font-size: 12px;
        font-family: 'JetBrains Mono', monospace;
        color: #818cf8;
    }

    .pin-select {
        flex: 1;
        padding: 6px 10px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        background: #252542;
        color: #fff;
        font-size: 12px;
        cursor: pointer;
    }

    .pin-select:focus {
        border-color: #818cf8;
        outline: none;
    }

    .code-editor {
        width: 100%;
        padding: 10px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        background: #0f0f1a;
        color: #e0e0e0;
    }

    .code-error-popup {
        margin: 8px 0 12px 0;
        border: 1px solid rgba(255, 68, 68, 0.15);
        font-family: 'JetBrains Mono', monospace;
        background: rgba(255, 68, 68, 0.04);
        padding: 8px;
        border-radius: 6px;
    }

    .code-error-popup .popup-content h4 {
        margin: 0 0 6px 0;
        font-size: 12px;
        color: #ff6b6b;
    }

    .code-error-popup .popup-content p {
        margin: 0 0 8px 0;
        font-size: 12px;
        color: rgba(255, 255, 255, 0.8);
        font-family: 'JetBrains Mono', monospace;
    }

    .code-error-popup .popup-actions {
        display: flex;
        gap: 8px;
    }

    .code-error-popup .btn {
        padding: 6px 10px;
        background: rgba(255, 68, 68, 0.08);
        border: 1px solid rgba(255, 68, 68, 0.15);
        border-radius: 6px;
        color: #ff9b9b;
        cursor: pointer;
        font-size: 12px;
    }

    .code-error-popup .btn:hover {
        background: rgba(255, 68, 68, 0.12);
    }

    .code-editor:focus {
        border-color: #818cf8;
        outline: none;
    }
</style>
