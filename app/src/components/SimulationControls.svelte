<script lang="ts">
    import {
        stepTicks,
        startAutoStep,
        stopAutoStep,
        isAutoStepping,
        getAutoStepRate,
        getTickCount,
    } from '../stores/simulationState.svelte';

    let autoRunning: boolean = $state(isAutoStepping());
    let autoRate: number = $state(getAutoStepRate() ?? 1);
    let tickCount: number = $derived(getTickCount());

    function handleStep(n: number) {
        stepTicks(n);
    }

    function handleToggleAuto() {
        if (autoRunning) {
            stopAutoStep();
            autoRunning = false;
        } else {
            startAutoStep(autoRate);
            autoRunning = true;
        }
    }

    function handleRateChange(e: Event) {
        const target = e.target as HTMLInputElement;
        const val = Number(target.value) || 1;
        autoRate = Math.max(1, Math.min(64, Math.round(val)));
        if (autoRunning) {
            startAutoStep(autoRate);
        }
    }
</script>

<div class="controls" role="group" aria-label="Simulation controls">
    <div class="left">
        <button class="btn small" onclick={() => handleStep(1)}>Step 1</button>
        <button class="btn small" onclick={() => handleStep(5)}>Step 5</button>
    </div>

    <div class="center">
        <div class="tick-display">
            <div class="tick-count">{tickCount}</div>
            <div class="tick-label">Ticks</div>
        </div>
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
        display: flex;
        gap: 14px;
        align-items: center;
        padding: 10px 14px;
        background: rgba(24, 26, 46, 0.92);
        border-radius: 14px;
        border: 1px solid rgba(255, 255, 255, 0.06);
        box-shadow: 0 10px 40px rgba(2, 6, 23, 0.6);
        backdrop-filter: blur(6px);
        min-width: 280px;
        max-width: calc(100% - 40px);
        width: min(520px, 88%);
    }

    .btn {
        background: rgba(255, 255, 255, 0.03);
        color: #e6eef8;
        border: 1px solid rgba(255, 255, 255, 0.02);
        padding: 8px 12px;
        border-radius: 10px;
        cursor: pointer;
        font-weight: 700;
        font-size: 13px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        transition:
            transform 0.12s ease,
            background 0.12s ease,
            box-shadow 0.12s ease;
    }

    .btn.small {
        padding: 6px 8px;
        border-radius: 8px;
        font-size: 12px;
    }

    .btn:hover {
        transform: translateY(-1px);
    }

    .btn.primary {
        background: linear-gradient(180deg, rgba(99, 102, 241, 0.95), rgba(79, 70, 229, 0.95));
        color: #fff;
        border-color: rgba(99, 102, 241, 0.25);
        box-shadow: 0 6px 18px rgba(79, 70, 229, 0.12);
        width: 44px;
        height: 44px;
        padding: 0;
        font-size: 14px;
        border-radius: 10px;
    }

    .btn.primary.active {
        background: linear-gradient(180deg, rgba(239, 68, 68, 0.95), rgba(220, 38, 38, 0.95));
        box-shadow:
            0 8px 22px rgba(239, 68, 68, 0.18) inset,
            0 8px 26px rgba(20, 20, 40, 0.35);
        transform: translateY(-1px) scale(1.02);
    }

    .rate-control {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .rate-control label {
        font-size: 12px;
        color: rgba(230, 238, 248, 0.9);
    }

    .rate-control input[type='range'] {
        width: 120px;
    }

    /* Layout helpers */
    .left {
        display: flex;
        gap: 6px;
        align-items: center;
    }
    .center {
        flex: 1;
        display: flex;
        justify-content: center;
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
        padding: 6px 12px;
        background: rgba(255, 255, 255, 0.02);
        border-radius: 8px;
        min-width: 86px;
    }

    .tick-count {
        font-size: 18px;
        font-weight: 800;
        color: #eaf2ff;
    }
    .tick-label {
        font-size: 11px;
        color: rgba(230, 238, 248, 0.7);
        margin-top: 2px;
    }
</style>
