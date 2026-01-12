<script lang="ts">
    import { tick } from 'svelte';

    export interface ConfigField {
        id: string;
        label: string;
        type: 'number' | 'text' | 'select';
        value: number | string;
        min?: number;
        max?: number;
        step?: number;
        options?: { value: string | number; label: string }[];
        placeholder?: string;
    }

    interface Props {
        visible: boolean;
        title: string;
        fields: ConfigField[];
        onConfirm: (values: Record<string, number | string>) => void;
        onCancel: () => void;
    }

    let { visible, title, fields = $bindable([]), onConfirm, onCancel }: Props = $props();

    let localFields = $state<ConfigField[]>([]);

    $effect(() => {
        if (visible) {
            localFields = fields.map((f) => ({ ...f }));
            tick().then(focusFirstField);
        }
    });

    function handleConfirm() {
        const values: Record<string, number | string> = {};
        for (const field of localFields) {
            values[field.id] = field.value;
        }
        onConfirm(values);
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (!visible) return;
        if (e.key === 'Escape') {
            onCancel();
        } else if (e.key === 'Enter') {
            handleConfirm();
        }
    }

    function updateFieldValue(fieldId: string, value: number | string) {
        const idx = localFields.findIndex((f) => f.id === fieldId);
        if (idx !== -1) {
            localFields[idx] = { ...localFields[idx], value };
        }
    }

    function focusFirstField() {
        if (!containerEl) return;
        const el = containerEl.querySelector('input, select, textarea') as HTMLElement | null;
        if (!el) return;
        try {
            el.focus();
            if (el instanceof HTMLInputElement) el.select();
        } catch {}
    }

    let containerEl: HTMLElement | null = $state(null);

    function onPointerDownOutside(e: PointerEvent) {
        if (!containerEl || !containerEl.contains(e.target as Node)) {
            onCancel();
        }
    }

    $effect(() => {
        if (visible) {
            window.addEventListener('pointerdown', onPointerDownOutside, true);
            return () => window.removeEventListener('pointerdown', onPointerDownOutside, true);
        }
    });
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if visible}
    <div class="popup-backdrop"></div>
    <div
        class="popup-container"
        bind:this={containerEl}
        role="dialog"
        aria-modal="true"
        aria-labelledby="popup-title"
    >
        <div class="popup-header">
            <h2 id="popup-title">{title}</h2>
        </div>
        <div class="popup-body">
            {#each localFields as field (field.id)}
                <div class="field-group">
                    <label for={field.id}>{field.label}</label>
                    {#if field.type === 'number'}
                        <input
                            id={field.id}
                            type="number"
                            value={field.value}
                            min={field.min}
                            max={field.max}
                            step={field.step ?? 1}
                            placeholder={field.placeholder}
                            oninput={(e) =>
                                updateFieldValue(field.id, parseFloat(e.currentTarget.value) || 0)}
                        />
                    {:else if field.type === 'text'}
                        <input
                            id={field.id}
                            type="text"
                            value={field.value}
                            placeholder={field.placeholder}
                            oninput={(e) => updateFieldValue(field.id, e.currentTarget.value)}
                        />
                    {:else if field.type === 'select'}
                        <select
                            id={field.id}
                            value={field.value}
                            onchange={(e) => updateFieldValue(field.id, e.currentTarget.value)}
                        >
                            {#each field.options ?? [] as option}
                                <option value={option.value}>{option.label}</option>
                            {/each}
                        </select>
                    {/if}
                </div>
            {/each}
        </div>
        <div class="popup-footer">
            <button class="btn btn-secondary" onclick={onCancel}>Cancel</button>
            <button class="btn btn-primary" onclick={handleConfirm}>Confirm</button>
        </div>
    </div>
{/if}

<style>
    .popup-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(4px);
        z-index: 1000000;
        pointer-events: auto;
    }

    .popup-container {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        display: flex;
        flex-direction: column;
        background: #1e1e2e;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 12px;
        min-width: 320px;
        max-width: 90vw;
        width: min(480px, 90vw);
        z-index: 1000001;
        overflow: hidden;
        box-shadow:
            0 20px 60px rgba(0, 0, 0, 0.5),
            0 8px 24px rgba(0, 0, 0, 0.3);
    }

    .popup-header {
        padding: 16px 20px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .popup-header h2 {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        color: #e0e0e0;
    }

    .popup-body {
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 16px;
        flex: 1 1 auto;
        min-height: 0;
        overflow: auto;
    }

    .field-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .field-group label {
        font-size: 13px;
        font-weight: 500;
        color: rgba(255, 255, 255, 0.7);
    }

    .field-group input,
    .field-group select {
        padding: 10px 12px;
        font-size: 14px;
        color: #e0e0e0;
        background: #252542;
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        outline: none;
        transition: all 0.15s ease;
    }

    .field-group input[type='number']::-webkit-outer-spin-button,
    .field-group input[type='number']::-webkit-inner-spin-button {
        -webkit-appearance: none;
        margin: 0;
    }

    .field-group input[type='number'] {
        -moz-appearance: textfield;
        appearance: textfield;
    }

    .field-group input:focus,
    .field-group select:focus {
        border-color: #818cf8;
        box-shadow: 0 0 0 3px rgba(129, 140, 248, 0.2);
    }

    .field-group input::placeholder {
        color: rgba(255, 255, 255, 0.3);
    }

    .popup-footer {
        padding: 16px 20px;
        border-top: 1px solid rgba(255, 255, 255, 0.1);
        display: flex;
        justify-content: flex-end;
        gap: 10px;
    }

    .btn {
        padding: 8px 16px;
        font-size: 13px;
        font-weight: 500;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        transition: all 0.15s ease;
    }

    .btn-secondary {
        background: rgba(255, 255, 255, 0.1);
        color: #e0e0e0;
    }

    .btn-secondary:hover {
        background: rgba(255, 255, 255, 0.15);
    }

    .btn-primary {
        background: #818cf8;
        color: white;
    }

    .btn-primary:hover {
        background: #6366f1;
    }

    .btn:focus-visible {
        outline: 2px solid #818cf8;
        outline-offset: 2px;
    }
</style>
