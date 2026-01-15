<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import {
        stepTicks,
        startAutoStep,
        stopAutoStep,
        isAutoStepping,
        getAutoStepRate,
        getSimulationState,
    } from '../stores/simulationState.svelte';

    const simState = getSimulationState();

    let autoRate: number = $state(getAutoStepRate() ?? 1);
    let tickCount: number = $derived.by(() => simState.tickCount);
    let autoRunning: boolean = $state(isAutoStepping());

    function handleStep(n: number) {
        stepTicks(n);
    }

    function _onAutoStepChanged(e: Event) {
        try {
            const detail = (e as CustomEvent).detail as { running: boolean } | undefined;
            if (detail && typeof detail.running === 'boolean') {
                autoRunning = detail.running;
            } else {
                autoRunning = isAutoStepping();
            }
        } catch (e) {
            autoRunning = isAutoStepping();
        }
    }

    onMount(() => {
        window.addEventListener('sim:autoStepChanged', _onAutoStepChanged);
        autoRunning = isAutoStepping();
    });

    onDestroy(() => {
        window.removeEventListener('sim:autoStepChanged', _onAutoStepChanged);
    });

    function handleToggleAuto() {
        if (isAutoStepping()) {
            stopAutoStep();
        } else {
            startAutoStep(autoRate);
        }
        autoRunning = isAutoStepping();
    }

    function handleRateChange(e: Event) {
        const target = e.target as HTMLInputElement;
        const val = Number(target.value) || 1;
        autoRate = Math.max(1, Math.min(64, Math.round(val)));
        if (isAutoStepping()) {
            startAutoStep(autoRate);
        }
    }
</script>

<div class="controls" role="group" aria-label="Simulation controls">
    <div class="left">
        <div class="tick-display">
            <div class="tick-count">{tickCount}</div>
            <div class="tick-label">Ticks</div>
        </div>
    </div>

    <div class="center">
        <button class="btn small" onclick={() => handleStep(1)}>Step</button>
    </div>

    <div class="right">
        <button
            class="btn primary"
            class:active={autoRunning}
            onclick={handleToggleAuto}
            aria-pressed={autoRunning}
            title={autoRunning ? 'Stop auto-step' : 'Start auto-step'}
        >
            <span aria-hidden="true">{autoRunning ? '■' : '▶'}</span>
        </button>
        <div class="rate-control">
            <label for="rate">{autoRate} t/s</label>
            <input
                id="rate"
                type="range"
                min="1"
                max="64"
                bind:value={autoRate}
                oninput={handleRateChange}
            />
        </div>
    </div>
</div>

<style>
    .controls {
        position: fixed;
        bottom: 18px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 1000;
        display: inline-flex;
        gap: 10px;
        align-items: center;
        padding: 6px 10px;
        background: rgba(18, 20, 40, 0.9);
        border-radius: 10px;
        border: 1px solid rgba(255, 255, 255, 0.04);
        box-shadow: 0 8px 30px rgba(2, 6, 23, 0.55);
        backdrop-filter: blur(6px);
        max-width: calc(100% - 40px);
    }

    .btn {
        background: rgba(255, 255, 255, 0.03);
        color: #e6eef8;
        border: 1px solid rgba(255, 255, 255, 0.02);
        height: 36px;
        padding: 0 10px;
        border-radius: 8px;
        cursor: pointer;
        font-weight: 700;
        font-size: 13px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        min-width: 56px;
        transition:
            transform 0.12s ease,
            background 0.12s ease,
            box-shadow 0.12s ease;
    }

    .btn.small {
        height: 36px;
        min-width: 56px;
        padding: 0 8px;
        border-radius: 8px;
        font-size: 12px;
        font-weight: 800;
    }

    .btn:hover {
        transform: translateY(-1px);
    }

    .btn:focus-visible {
        outline: 2px solid rgba(96, 165, 250, 0.18);
        outline-offset: 2px;
    }

    .btn.primary {
        background: linear-gradient(180deg, rgba(99, 102, 241, 0.98), rgba(79, 70, 229, 0.98));
        color: #fff;
        border-color: rgba(99, 102, 241, 0.25);
        box-shadow: 0 8px 18px rgba(79, 70, 229, 0.12);
        width: 36px;
        height: 36px;
        padding: 0;
        font-size: 14px;
        border-radius: 8px;
        min-width: 36px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
    }

    .btn.primary.active {
        background: linear-gradient(180deg, rgba(239, 68, 68, 0.98), rgba(220, 38, 38, 0.98));
        box-shadow:
            0 8px 18px rgba(20, 20, 40, 0.35),
            0 4px 10px rgba(239, 68, 68, 0.08) inset;
        transform: translateY(-1px) scale(1.02);
    }

    .rate-control {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 4px 6px;
        background: rgba(255, 255, 255, 0.02);
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.02);
        height: 36px;
    }

    .rate-control label {
        font-size: 12px;
        color: rgba(230, 238, 248, 0.95);
        min-width: 48px;
        width: 48px;
        display: inline-block;
        text-align: right;
        font-family: 'JetBrains Mono', monospace;
        line-height: 1;
    }

    .rate-control input[type='range'] {
        width: 104px;
        -webkit-appearance: none;
        appearance: none;
        background: transparent;
        height: 12px;
        margin-left: 4px;
    }

    .rate-control input[type='range']::-webkit-slider-runnable-track {
        height: 6px;
        background: linear-gradient(90deg, rgba(99, 102, 241, 0.95), rgba(79, 70, 229, 0.95));
        border-radius: 999px;
    }
    .rate-control input[type='range']::-webkit-slider-thumb {
        -webkit-appearance: none;
        width: 12px;
        height: 12px;
        margin-top: -3px;
        background: #fff;
        border-radius: 50%;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        border: 2px solid rgba(79, 70, 229, 0.9);
    }

    .rate-control input[type='range']::-moz-range-track {
        height: 6px;
        background: linear-gradient(90deg, rgba(99, 102, 241, 0.95), rgba(79, 70, 229, 0.95));
        border-radius: 999px;
    }
    .rate-control input[type='range']::-moz-range-thumb {
        width: 12px;
        height: 12px;
        background: #fff;
        border-radius: 50%;
        border: 2px solid rgba(79, 70, 229, 0.9);
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
    }

    /* Layout helpers */
    .left {
        display: flex;
        gap: 8px;
        align-items: center;
    }
    .center {
        flex: 0 1 auto;
        display: flex;
        justify-content: center;
        align-items: center;
    }
    .right {
        display: flex;
        gap: 10px;
        align-items: center;
    }

    .tick-display {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 36px;
        padding: 0 6px;
        background: rgba(255, 255, 255, 0.02);
        border-radius: 8px;
        min-width: 64px;
        font-family: 'JetBrains Mono', monospace;
    }

    .tick-count {
        font-size: 14px;
        font-weight: 800;
        color: #eaf2ff;
        line-height: 1;
        margin: 0;
    }
    .tick-label {
        font-size: 10px;
        color: rgba(230, 238, 248, 0.7);
        margin: 0;
        line-height: 1;
    }
</style>
