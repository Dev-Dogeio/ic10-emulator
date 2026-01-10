<script lang="ts">
    import {
        type WasmAtmosphericNetwork,
        GasType,
        gas_type_all_gases,
        gas_type_all_liquids,
        gas_type_display_name,
        gas_type_symbol,
    } from '../../pkg/ic10_emulator';
    import { type InspectorWindow, setActiveTab } from '../stores/inspectorState.svelte';
    import { syncFromWasm } from '../stores/simulationState.svelte';
    import { onMount } from 'svelte';

    interface Props {
        window: InspectorWindow;
        network: WasmAtmosphericNetwork;
        networkName: string;
    }

    let { window, network, networkName }: Props = $props();

    type TabId = 'overview' | 'gases' | 'settings';
    let activeTab: TabId = $derived(window.activeTab as TabId);

    function switchTab(tab: TabId) {
        setActiveTab(window.id, tab);
    }

    // Cached gas metadata to avoid repeated WASM calls
    let _cachedAllGasTypes: GasType[] | null = null;
    let _cachedLiquidSet: Set<number> | null = null;
    const _cachedGasNames: Map<number, string> = new Map();
    const _cachedGasSymbols: Map<number, string> = new Map();

    // Gas type info
    interface GasInfo {
        type: GasType;
        name: string;
        symbol: string;
        moles: number;
        ratio: number;
        isLiquid: boolean;
    }

    function getAllGasTypes(): GasType[] {
        if (_cachedAllGasTypes) return _cachedAllGasTypes;
        const gases = gas_type_all_gases() as number[];
        const liquids = gas_type_all_liquids() as number[];

        const s = new Set<number>();
        for (const g of gases) s.add(g);
        for (const l of liquids) s.add(l);

        _cachedAllGasTypes = Array.from(s).map((n) => n as GasType);
        _cachedLiquidSet = new Set(liquids);
        return _cachedAllGasTypes;
    }

    function getGasName(type: GasType): string {
        const key = type as number;
        if (_cachedGasNames.has(key)) return _cachedGasNames.get(key)!;
        const name = gas_type_display_name(type);
        _cachedGasNames.set(key, name);
        return name;
    }

    function getGasSymbol(type: GasType): string {
        const key = type as number;
        if (_cachedGasSymbols.has(key)) return _cachedGasSymbols.get(key)!;
        const sym = gas_type_symbol(type);
        _cachedGasSymbols.set(key, sym);
        return sym;
    }

    function isLiquidType(type: GasType): boolean {
        if (_cachedLiquidSet) return _cachedLiquidSet.has(type as number);
        const liquids = gas_type_all_liquids() as number[];
        _cachedLiquidSet = new Set(liquids);
        return _cachedLiquidSet.has(type as number);
    }

    function getGasInfoList(): GasInfo[] {
        const allTypes = getAllGasTypes();
        const gases: GasInfo[] = [];

        for (const type of allTypes) {
            try {
                const moles = network.get_moles(type);
                if (moles > 0.0001) {
                    gases.push({
                        type,
                        name: getGasName(type),
                        symbol: getGasSymbol(type),
                        moles,
                        ratio: network.gas_ratio(type),
                        isLiquid: isLiquidType(type),
                    });
                }
            } catch (e) {
                // Skip types that fail
                console.warn('Failed to get gas info for type', type);
            }
        }

        return gases.sort((a, b) => b.moles - a.moles);
    }

    let gasInfoList = $state<GasInfo[]>([]);
    let refreshCounter = $state(0);

    function refreshData() {
        gasInfoList = getGasInfoList();
        refreshCounter++;
    }

    onMount(() => {
        refreshData();
    });

    let pressure = $derived(() => {
        try {
            return network.pressure();
        } catch {
            return 0;
        }
    });

    let temperature = $derived(() => {
        try {
            return network.temperature();
        } catch {
            return 0;
        }
    });

    let totalMoles = $derived(() => {
        try {
            return network.total_moles();
        } catch {
            return 0;
        }
    });

    let volume = $derived(() => {
        try {
            return network.total_volume();
        } catch {
            return 0;
        }
    });

    $effect(() => {
        if (activeTab === 'overview' || activeTab === 'gases') {
            void pressure();
            void temperature();
            void totalMoles();
            void volume();

            gasInfoList = getGasInfoList();
        }
    });

    let addGasType = $state<GasType>(GasType.Oxygen);
    let addGasMoles = $state(1000);
    let addGasTemp = $state(293.15);

    function addGas() {
        if (addGasMoles <= 0) return;
        try {
            network.add_gas(addGasType, addGasMoles, addGasTemp);
            syncFromWasm();
            refreshData();
        } catch (e) {
            console.error('Failed to add gas:', e);
        }
    }

    function removeGas(type: GasType) {
        try {
            network.remove_all_gas(type);
            syncFromWasm();
            refreshData();
        } catch (e) {
            console.error('Failed to remove gas:', e);
        }
    }

    function removeGasMoles(type: GasType, moles: number) {
        try {
            network.remove_gas(type, moles);
            syncFromWasm();
            refreshData();
        } catch (e) {
            console.error('Failed to remove gas moles:', e);
        }
    }

    // Settings
    let newVolume = $state(0);
    let newTemperature = $state(0);

    $effect(() => {
        newVolume = volume();
        newTemperature = temperature();
    });

    function applyVolume() {
        if (newVolume <= 0) return;
        try {
            network.set_volume(newVolume);
            syncFromWasm();
            refreshData();
        } catch (e) {
            console.error('Failed to set volume:', e);
        }
    }

    function applyTemperature() {
        if (newTemperature <= 0) return;
        try {
            network.set_temperature(newTemperature);
            syncFromWasm();
            refreshData();
        } catch (e) {
            console.error('Failed to set temperature:', e);
        }
    }

    function clearAll() {
        try {
            network.clear();
            syncFromWasm();
            refreshData();
        } catch (e) {
            console.error('Failed to clear network:', e);
        }
    }

    // Color for gas types
    function getGasColor(type: GasType): string {
        const colors: Record<number, string> = {
            [GasType.Oxygen]: '#60a5fa',
            [GasType.Nitrogen]: '#a78bfa',
            [GasType.CarbonDioxide]: '#9ca3af',
            [GasType.Volatiles]: '#f97316',
            [GasType.Pollutant]: '#84cc16',
            [GasType.NitrousOxide]: '#ec4899',
            [GasType.Steam]: '#e0e7ff',
            [GasType.Hydrogen]: '#fbbf24',
            [GasType.Water]: '#38bdf8',
            [GasType.LiquidNitrogen]: '#8b5cf6',
            [GasType.LiquidOxygen]: '#3b82f6',
            [GasType.LiquidVolatiles]: '#ea580c',
            [GasType.LiquidCarbonDioxide]: '#6b7280',
            [GasType.LiquidPollutant]: '#65a30d',
            [GasType.LiquidNitrousOxide]: '#db2777',
            [GasType.LiquidHydrogen]: '#f59e0b',
            [GasType.PollutedWater]: '#78716c',
        };
        return colors[type] || '#818cf8';
    }

    // Format helpers
    function formatNumber(n: number, decimals: number = 2): string {
        if (Math.abs(n) < 0.01 && n !== 0) {
            return n.toExponential(decimals);
        }
        return n.toFixed(decimals);
    }

    function formatMoles(moles: number): string {
        if (moles >= 1000000) {
            return (moles / 1000000).toFixed(2) + 'M';
        } else if (moles >= 1000) {
            return (moles / 1000).toFixed(2) + 'k';
        }
        return moles.toFixed(2);
    }
</script>

<div class="atmo-inspector">
    <!-- Tab bar -->
    <div class="tab-bar">
        <button
            class="tab-btn"
            class:active={activeTab === 'overview'}
            onclick={() => switchTab('overview')}
        >
            <span class="tab-icon">📊</span>
            <span class="tab-label">Overview</span>
        </button>
        <button
            class="tab-btn"
            class:active={activeTab === 'gases'}
            onclick={() => switchTab('gases')}
        >
            <span class="tab-icon">🧪</span>
            <span class="tab-label">Gases</span>
        </button>
        <button
            class="tab-btn"
            class:active={activeTab === 'settings'}
            onclick={() => switchTab('settings')}
        >
            <span class="tab-icon">⚙️</span>
            <span class="tab-label">Settings</span>
        </button>
    </div>

    <!-- Tab content -->
    <div class="tab-content">
        {#if activeTab === 'overview'}
            <div class="overview-section">
                <h3 class="network-name">{networkName}</h3>

                <!-- Stats cards -->
                <div class="stats-grid">
                    <div class="stat-card pressure">
                        <span class="stat-icon">🔵</span>
                        <div class="stat-info">
                            <span class="stat-label">Pressure</span>
                            <span class="stat-value">{formatNumber(pressure())} kPa</span>
                        </div>
                    </div>
                    <div class="stat-card temperature">
                        <span class="stat-icon">🌡️</span>
                        <div class="stat-info">
                            <span class="stat-label">Temperature</span>
                            <span class="stat-value">{formatNumber(temperature())} K</span>
                        </div>
                    </div>
                    <div class="stat-card moles">
                        <span class="stat-icon">⚛️</span>
                        <div class="stat-info">
                            <span class="stat-label">Total Moles</span>
                            <span class="stat-value">{formatMoles(totalMoles())}</span>
                        </div>
                    </div>
                    <div class="stat-card volume">
                        <span class="stat-icon">📦</span>
                        <div class="stat-info">
                            <span class="stat-label">Volume</span>
                            <span class="stat-value">{formatNumber(volume())} L</span>
                        </div>
                    </div>
                </div>

                <!-- Composition bar -->
                {#if gasInfoList.length > 0}
                    <div class="composition-section">
                        <h4>Composition</h4>
                        <div class="composition-bar">
                            {#each gasInfoList as gas}
                                <div
                                    class="composition-segment"
                                    style="width: {gas.ratio * 100}%; background: {getGasColor(
                                        gas.type
                                    )}"
                                    title="{gas.name}: {(gas.ratio * 100).toFixed(1)}%"
                                ></div>
                            {/each}
                        </div>
                        <div class="composition-legend">
                            {#each gasInfoList as gas}
                                <div class="legend-item">
                                    <span
                                        class="legend-color"
                                        style="background: {getGasColor(gas.type)}"
                                    ></span>
                                    <span class="legend-name">{gas.symbol}</span>
                                    <span class="legend-percent"
                                        >{(gas.ratio * 100).toFixed(1)}%</span
                                    >
                                </div>
                            {/each}
                        </div>
                    </div>
                {:else}
                    <div class="empty-state">No gases in network</div>
                {/if}

                <button class="refresh-btn" onclick={refreshData}> 🔄 Refresh </button>
            </div>
        {:else if activeTab === 'gases'}
            <div class="gases-section">
                <!-- Add gas form -->
                <div class="add-gas-form">
                    <h4>Add Gas</h4>
                    <div class="form-row">
                        <label>
                            Type
                            <select bind:value={addGasType}>
                                {#each getAllGasTypes() as type}
                                    <option value={type}>{getGasName(type)}</option>
                                {/each}
                            </select>
                        </label>
                    </div>
                    <div class="form-row double">
                        <label>
                            Moles
                            <input type="number" bind:value={addGasMoles} min="0" step="100" />
                        </label>
                        <label>
                            Temp (K)
                            <input type="number" bind:value={addGasTemp} min="0" step="1" />
                        </label>
                    </div>
                    <button class="add-btn" onclick={addGas}>Add Gas</button>
                </div>

                <!-- Gas list -->
                <div class="gas-list">
                    <h4>Current Gases</h4>
                    {#if gasInfoList.length === 0}
                        <div class="empty-state">No gases present</div>
                    {:else}
                        {#each gasInfoList as gas}
                            <div class="gas-row" class:liquid={gas.isLiquid}>
                                <div class="gas-info">
                                    <span
                                        class="gas-color"
                                        style="background: {getGasColor(gas.type)}"
                                    ></span>
                                    <span class="gas-name">{gas.name}</span>
                                    <span class="gas-symbol">{gas.symbol}</span>
                                </div>
                                <div class="gas-stats">
                                    <span class="gas-moles">{formatMoles(gas.moles)} mol</span>
                                    <span class="gas-ratio">{(gas.ratio * 100).toFixed(1)}%</span>
                                </div>
                                <div class="gas-actions">
                                    <button
                                        class="remove-partial-btn"
                                        onclick={() => removeGasMoles(gas.type, gas.moles * 0.5)}
                                        title="Remove 50%"
                                    >
                                        -50%
                                    </button>
                                    <button
                                        class="remove-btn"
                                        onclick={() => removeGas(gas.type)}
                                        title="Remove all"
                                    >
                                        ✕
                                    </button>
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        {:else if activeTab === 'settings'}
            <div class="settings-section">
                <div class="setting-group">
                    <h4>Volume</h4>
                    <div class="setting-row">
                        <input type="number" bind:value={newVolume} min="1" step="10" />
                        <span class="setting-unit">L</span>
                        <button class="apply-btn" onclick={applyVolume}>Apply</button>
                    </div>
                </div>

                <div class="setting-group">
                    <h4>Temperature</h4>
                    <div class="setting-row">
                        <input type="number" bind:value={newTemperature} min="0" step="1" />
                        <span class="setting-unit">K</span>
                        <button class="apply-btn" onclick={applyTemperature}>Apply</button>
                    </div>
                    <div class="temp-presets">
                        <button
                            onclick={() => {
                                newTemperature = 273.15;
                                applyTemperature();
                            }}>0°C</button
                        >
                        <button
                            onclick={() => {
                                newTemperature = 293.15;
                                applyTemperature();
                            }}>20°C</button
                        >
                        <button
                            onclick={() => {
                                newTemperature = 295.15;
                                applyTemperature();
                            }}>22°C</button
                        >
                    </div>
                </div>

                <div class="danger-zone">
                    <h4>Danger Zone</h4>
                    <button class="danger-btn" onclick={clearAll}> 🗑️ Clear All Gases </button>
                </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .atmo-inspector {
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
        color: #60a5fa;
    }

    .tab-icon {
        font-size: 12px;
    }

    .tab-content {
        flex: 1;
        overflow: auto;
        padding: 12px;
    }

    /* Overview */
    .overview-section {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .network-name {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #60a5fa;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 8px;
    }

    .stat-card {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.06);
    }

    .stat-icon {
        font-size: 18px;
    }

    .stat-info {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }

    .stat-label {
        font-size: 10px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .stat-value {
        font-size: 13px;
        font-weight: 600;
        font-family: 'JetBrains Mono', monospace;
    }

    .composition-section {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .composition-section h4 {
        margin: 0;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .composition-bar {
        display: flex;
        height: 20px;
        border-radius: 10px;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.05);
    }

    .composition-segment {
        min-width: 2px;
        transition: width 0.3s ease;
    }

    .composition-legend {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
    }

    .legend-item {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 10px;
    }

    .legend-color {
        width: 8px;
        height: 8px;
        border-radius: 2px;
    }

    .legend-name {
        color: rgba(255, 255, 255, 0.7);
    }

    .legend-percent {
        color: rgba(255, 255, 255, 0.5);
    }

    .empty-state {
        padding: 24px;
        text-align: center;
        color: rgba(255, 255, 255, 0.4);
    }

    .refresh-btn {
        padding: 8px 16px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        background: rgba(255, 255, 255, 0.05);
        color: rgba(255, 255, 255, 0.7);
        font-size: 12px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .refresh-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }

    /* Gases Tab */
    .gases-section {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .add-gas-form {
        padding: 12px;
        background: rgba(96, 165, 250, 0.05);
        border: 1px solid rgba(96, 165, 250, 0.2);
        border-radius: 8px;
    }

    .add-gas-form h4 {
        margin: 0 0 10px 0;
        font-size: 11px;
        color: #60a5fa;
        text-transform: uppercase;
    }

    .form-row {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 10px;
    }

    .form-row.double {
        flex-direction: row;
        gap: 10px;
    }

    .form-row.double label {
        flex: 1;
        min-width: 120px;
        max-width: 260px;
    }

    .form-row label {
        display: flex;
        flex-direction: column;
        gap: 4px;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.6);
    }

    .form-row select,
    .form-row input {
        padding: 8px 10px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        background: #252542;
        color: #fff;
        font-size: 12px;
        width: 100%;
        box-sizing: border-box;
    }

    .form-row.double input {
        max-width: 220px;
    }

    .form-row select:focus,
    .form-row input:focus {
        border-color: #60a5fa;
        outline: none;
    }

    .add-btn {
        width: 100%;
        padding: 8px;
        border: none;
        border-radius: 6px;
        background: #60a5fa;
        color: #fff;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
    }

    .add-btn:hover {
        background: #3b82f6;
    }

    .gas-list {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .gas-list h4 {
        margin: 0 0 6px 0;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .gas-row {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 10px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 6px;
    }

    .gas-row.liquid {
        background: rgba(56, 189, 248, 0.05);
        border: 1px solid rgba(56, 189, 248, 0.15);
    }

    .gas-info {
        display: flex;
        align-items: center;
        gap: 8px;
        flex: 1;
    }

    .gas-color {
        width: 10px;
        height: 10px;
        border-radius: 3px;
    }

    .gas-name {
        font-size: 12px;
        font-weight: 500;
    }

    .gas-symbol {
        font-size: 10px;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'JetBrains Mono', monospace;
    }

    .gas-stats {
        display: flex;
        gap: 10px;
        font-size: 11px;
        font-family: 'JetBrains Mono', monospace;
    }

    .gas-moles {
        color: #60a5fa;
    }

    .gas-ratio {
        color: rgba(255, 255, 255, 0.5);
    }

    .gas-actions {
        display: flex;
        gap: 4px;
    }

    .remove-partial-btn {
        padding: 4px 6px;
        border: none;
        border-radius: 4px;
        background: rgba(251, 191, 36, 0.2);
        color: #fbbf24;
        font-size: 9px;
        cursor: pointer;
    }

    .remove-partial-btn:hover {
        background: rgba(251, 191, 36, 0.4);
    }

    .remove-btn {
        padding: 4px 6px;
        border: none;
        border-radius: 4px;
        background: rgba(239, 68, 68, 0.2);
        color: #f87171;
        font-size: 10px;
        cursor: pointer;
    }

    .remove-btn:hover {
        background: rgba(239, 68, 68, 0.4);
    }

    /* Settings Tab */
    .settings-section {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }

    .setting-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .setting-group h4 {
        margin: 0;
        font-size: 11px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .setting-row {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .setting-row input {
        flex: 1;
        padding: 8px 10px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 6px;
        background: #252542;
        color: #fff;
        font-size: 12px;
        font-family: 'JetBrains Mono', monospace;
    }

    .setting-row input:focus {
        border-color: #60a5fa;
        outline: none;
    }

    .setting-unit {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.5);
        min-width: 20px;
    }

    .apply-btn {
        padding: 8px 12px;
        border: none;
        border-radius: 6px;
        background: #60a5fa;
        color: #fff;
        font-size: 11px;
        font-weight: 600;
        cursor: pointer;
    }

    .apply-btn:hover {
        background: #3b82f6;
    }

    .temp-presets {
        display: flex;
        gap: 6px;
    }

    .temp-presets button {
        padding: 6px 10px;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 4px;
        background: rgba(255, 255, 255, 0.05);
        color: rgba(255, 255, 255, 0.7);
        font-size: 11px;
        cursor: pointer;
    }

    .temp-presets button:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }

    .danger-zone {
        padding: 12px;
        background: rgba(239, 68, 68, 0.05);
        border: 1px solid rgba(239, 68, 68, 0.2);
        border-radius: 8px;
    }

    .danger-zone h4 {
        margin: 0 0 10px 0;
        font-size: 11px;
        color: #f87171;
        text-transform: uppercase;
    }

    .danger-btn {
        padding: 8px 16px;
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: 6px;
        background: rgba(239, 68, 68, 0.1);
        color: #f87171;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .danger-btn:hover {
        background: rgba(239, 68, 68, 0.2);
        border-color: rgba(239, 68, 68, 0.5);
    }
</style>
