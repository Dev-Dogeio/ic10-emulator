<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import * as monaco from 'monaco-editor';
    import type { WasmDevice, WasmICChip } from '../../pkg/ic10_emulator';
    import { syncFromWasm, onTick } from '../stores/simulationState.svelte';

    interface Props {
        device: WasmDevice;
        onClose: () => void;
    }

    let { device, onClose }: Props = $props();

    let editorContainer: HTMLDivElement;
    let editor: monaco.editor.IStandaloneCodeEditor | null = null;
    let decorations: monaco.editor.IEditorDecorationsCollection | null = null;

    let unsubscribeTick: (() => void) | null = null;

    // Root element for inspector resize events
    let rootEl: HTMLElement | null = $state(null);

    // Chip state
    let chip: WasmICChip | undefined = $state(undefined);
    let registers: number[] = $state([]);
    let stack: number[] = $state([]);
    let pc: number = $state(0);
    let isHalted: boolean = $state(false);
    let errorMessage: string | null = $state(null);
    let lineCount: number = $state(0);
    let errorLine: number | null = $state(null);

    // Refresh chip state from the device
    function refreshChipState() {
        try {
            chip = device.get_chip();
            if (chip) {
                registers = Array.from(chip.get_all_registers());
                stack = Array.from(chip.get_all_stack());
                pc = chip.get_pc();
                isHalted = chip.is_halted();
                lineCount = chip.get_line_count();

                // Preserve load error. Use chip's error line if present. Keep message on recent load failure.
                const chipErrLine = chip.get_error_line();
                errorLine = chipErrLine ?? null;

                if (chipErrLine !== undefined && chipErrLine !== null) {
                    // Show generic message if none set
                    if (!errorMessage) {
                        errorMessage = `Error at line ${chipErrLine + 1}`;
                    }
                } else {
                    // Clear message only if not set by a recent load failure
                    if (!errorMessage) {
                        errorMessage = null;
                    }
                }
            } else {
                registers = [];
                stack = [];
                pc = 0;
                isHalted = false;
                lineCount = 0;
                errorLine = null;
            }
        } catch (e) {
            errorMessage = e?.toString() ?? 'Unknown error';
        }
    }

    // RAF-scheduled highlight for current PC
    function updateHighlight() {
        if (!editor || !chip || pendingRAF) return;

        pendingRAF = requestAnimationFrame(() => {
            try {
                if (!editor) return;
                const lineNumber = pc + 1;
                const model = editor.getModel();
                if (!model) return;

                const newDecorations: monaco.editor.IModelDeltaDecoration[] = [];

                // Highlight current line
                if (lineNumber >= 1 && lineNumber <= model.getLineCount()) {
                    newDecorations.push({
                        range: new monaco.Range(lineNumber, 1, lineNumber, 1),
                        options: {
                            isWholeLine: true,
                            className: 'current-line-highlight',
                            glyphMarginClassName: 'current-line-glyph',
                        },
                    });
                }

                // Highlight error line
                if (errorLine !== null) {
                    const errLineNum = errorLine + 1;
                    if (errLineNum >= 1 && errLineNum <= model.getLineCount()) {
                        newDecorations.push({
                            range: new monaco.Range(errLineNum, 1, errLineNum, 1),
                            options: {
                                isWholeLine: true,
                                className: 'error-line-highlight',
                                glyphMarginClassName: 'error-line-glyph',
                            },
                        });
                    }
                }

                // Apply decorations
                decorationIds = editor.deltaDecorations(decorationIds, newDecorations);
            } finally {
                pendingRAF = 0;
            }
        });
    }

    // Load editor code into chip
    function pushCode() {
        if (!editor || !chip) return;

        const code = editor.getValue();
        try {
            chip.load_program(code);
            syncFromWasm();
            refreshChipState();
            scheduleDecorationUpdate();
            errorMessage = null;
        } catch (e) {
            errorMessage = e?.toString() ?? 'Failed to load program';
            refreshChipState();
            scheduleDecorationUpdate();
        }
    }

    // Step one instruction
    function stepInstruction() {
        if (!chip) return;

        try {
            chip.step();
            syncFromWasm();
            refreshChipState();
            scheduleDecorationUpdate();
        } catch (e) {
            errorMessage = e?.toString() ?? 'Failed to load program';
            refreshChipState();
            scheduleDecorationUpdate();
        }
    }

    // Run one tick (128 instructions max)
    function runTick() {
        if (!chip) return;

        try {
            chip.run(128);
            syncFromWasm();
            refreshChipState();
            scheduleDecorationUpdate();
        } catch (e) {
            errorMessage = e?.toString() ?? 'Step failed';
            refreshChipState();
            scheduleDecorationUpdate();
        }
    }

    // Reset the chip
    function resetExecution() {
        if (!chip) return;
        pushCode();
        chip.clear_registers();
        chip.clear_stack();
    }

    // Get register name
    function getRegisterName(index: number): string {
        if (index === 16) return 'sp';
        if (index === 17) return 'ra';
        return `r${index}`;
    }

    // Filter non-zero stack values for display
    let nonZeroStack = $derived(
        stack.map((value, index) => ({ index, value })).filter(({ value }) => value !== 0)
    );

    // Refresh debugger on simulation ticks
    let decorationIds: string[] = [];
    let pendingRAF: number | null = null;
    let resizeObserver: ResizeObserver | null = null;
    let onWindowResize: (() => void) | null = null;

    function applyDecorations() {
        if (!editor) return;
        const model = editor.getModel();
        if (!model) return;

        // Model lines are 1-based; editor displays 0-based numbers
        const lineNumber = pc + 1;
        const newDecorations: monaco.editor.IModelDeltaDecoration[] = [];

        if (lineNumber >= 1 && lineNumber <= model.getLineCount()) {
            newDecorations.push({
                range: new monaco.Range(lineNumber, 1, lineNumber, 1),
                options: {
                    isWholeLine: true,
                    className: 'current-line-highlight',
                    glyphMarginClassName: 'current-line-glyph',
                },
            });
        }

        if (errorLine !== null) {
            const errLineNum = errorLine + 1;
            if (errLineNum >= 1 && errLineNum <= model.getLineCount()) {
                newDecorations.push({
                    range: new monaco.Range(errLineNum, 1, errLineNum, 1),
                    options: {
                        isWholeLine: true,
                        className: 'error-line-highlight',
                        glyphMarginClassName: 'error-line-glyph',
                    },
                });
            }
        }

        decorationIds = editor.deltaDecorations(decorationIds, newDecorations);
    }

    function scheduleDecorationUpdate() {
        // already scheduled via RAF
        if (pendingRAF) return;

        pendingRAF = requestAnimationFrame(() => {
            try {
                applyDecorations();
            } finally {
                pendingRAF = null;
            }
        });
    }

    // Handle tick events to refresh chip state and decorations
    function handleTick() {
        try {
            refreshChipState();

            // Update highlights and schedule decorations/layout
            try {
                updateHighlight();
            } catch (e) {
                // ignore errors
            }
            scheduleDecorationUpdate();
        } catch (e) {
            console.warn('Debugger tick refresh error', e);
        }
    }

    onMount(() => {
        // Subscribe to tick events
        unsubscribeTick = onTick(handleTick);

        // Initialize Monaco editor
        editor = monaco.editor.create(editorContainer, {
            value: device.get_chip_source() || '# Enter IC10 code here...\n',
            language: 'plaintext',
            theme: 'vs-dark',
            automaticLayout: true,
            minimap: { enabled: false },
            lineNumbers: (line) => String(line - 1),
            glyphMargin: true,
            fontSize: 13,
            fontFamily: "'JetBrains Mono', 'Consolas', monospace",
            scrollBeyondLastLine: false,
            wordWrap: 'off',
            tabSize: 4,
        });

        editor.onDidChangeModelContent(() => {
            pushCode();
        });

        // Add custom CSS for highlights
        const style = document.createElement('style');
        style.textContent = `
            .current-line-highlight {
                background-color: rgba(99, 102, 241, 0.3) !important;
            }
            .current-line-glyph {
                background-color: #818cf8;
                width: 4px !important;
                margin-left: 3px;
            }
            .error-line-highlight {
                background-color: rgba(239, 68, 68, 0.3) !important;
            }
            .error-line-glyph {
                background-color: #ef4444;
                width: 4px !important;
                margin-left: 3px;
            }
        `;
        document.head.appendChild(style);

        refreshChipState();
        scheduleDecorationUpdate();

        // Ensure Monaco resizes with container (handles shrinking)
        try {
            if (typeof ResizeObserver !== 'undefined') {
                resizeObserver = new ResizeObserver(() => {
                    try {
                        const w = editorContainer.clientWidth;
                        const h = editorContainer.clientHeight;
                        editor?.layout({ width: w, height: h });
                    } catch (e) {
                        // ignore errors
                    }
                });
                resizeObserver.observe(editorContainer);
            } else {
                onWindowResize = () => {
                    try {
                        const w = editorContainer.clientWidth;
                        const h = editorContainer.clientHeight;
                        editor?.layout({ width: w, height: h });
                    } catch (e) {
                        // ignore errors
                    }
                };
                window.addEventListener('resize', onWindowResize);
            }
            // Trigger initial layout with exact size
            const iw = editorContainer.clientWidth;
            const ih = editorContainer.clientHeight;
            editor?.layout({ width: iw, height: ih });
        } catch (e) {
            // ignore errors
        }

        // Request the inspector enforce a minimum size of 750x650
        try {
            rootEl?.dispatchEvent(
                new CustomEvent('inspector-enforce-min-size', {
                    detail: { minWidth: 750, minHeight: 650 },
                    bubbles: true,
                })
            );
        } catch (e) {
            // ignore errors
        }
    });

    onDestroy(() => {
        if (unsubscribeTick) unsubscribeTick();
        if (pendingRAF) cancelAnimationFrame(pendingRAF);
        try {
            if (resizeObserver) {
                resizeObserver.disconnect();
                resizeObserver = null;
            }
            if (onWindowResize) {
                window.removeEventListener('resize', onWindowResize);
                onWindowResize = null;
            }
        } catch (e) {
            // ignore errors
        }
        if (editor) {
            editor.dispose();
            editor = null;
        }
    });
</script>

<div class="debugger-root" bind:this={rootEl}>
    <div class="debugger-container">
        <!-- Header -->
        <div class="debugger-header">
            <h2>🔧 IC10 Debugger - {device.name()}</h2>
            <button class="close-btn" onclick={onClose}>✕</button>
        </div>

        <!-- Main content -->
        <div class="debugger-content">
            <!-- Left: Code Editor -->
            <div class="editor-panel">
                <div class="panel-header">
                    <span>Code Editor</span>
                    <div class="editor-actions"></div>
                </div>
                <div class="editor-container" bind:this={editorContainer}></div>
            </div>

            <!-- Right: Debug Info -->
            <div class="debug-panel">
                <!-- Execution Controls -->
                <div class="control-section">
                    <h3>Execution</h3>
                    <div class="execution-info">
                        <span class="info-label">PC:</span>
                        <span class="info-value">{pc}</span>
                        <span class="info-label">Lines:</span>
                        <span class="info-value">{lineCount}</span>
                        <span class="info-label">Status:</span>
                        <span class="info-value status" class:halted={isHalted}>
                            {isHalted ? 'HALTED' : 'RUNNING'}
                        </span>
                    </div>
                    <div class="control-buttons">
                        <button class="control-btn step" onclick={stepInstruction} disabled={!chip}>
                            ⏭️ Step
                        </button>
                        <button class="control-btn run" onclick={runTick} disabled={!chip}>
                            ▶️ Run
                        </button>
                        <button class="control-btn reset" onclick={resetExecution} disabled={!chip}>
                            🔄 Reset
                        </button>
                    </div>
                </div>

                <!-- Error Display -->
                {#if errorMessage}
                    <div class="error-section">
                        <h3>⚠️ Error</h3>
                        <div class="error-message">{errorMessage}</div>
                    </div>
                {/if}

                <!-- Registers -->
                <div class="registers-section">
                    <h3>Registers</h3>
                    <div class="registers-grid">
                        {#each registers as value, index}
                            <div class="register-item" class:nonzero={value !== 0}>
                                <span class="register-name">{getRegisterName(index)}</span>
                                <span class="register-value">{value.toFixed(2)}</span>
                            </div>
                        {/each}
                    </div>
                </div>

                <!-- Stack -->
                <div class="stack-section">
                    <h3>Stack (non-zero values)</h3>
                    <div class="stack-list">
                        {#if nonZeroStack.length === 0}
                            <div class="stack-empty">All stack values are 0</div>
                        {:else}
                            {#each nonZeroStack as { index, value }}
                                <div class="stack-item">
                                    <span class="stack-index">[{index}]</span>
                                    <span class="stack-value">{value.toFixed(2)}</span>
                                </div>
                            {/each}
                        {/if}
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

<style>
    .debugger-root {
        display: flex;
        flex: 1;
        min-height: 0;
    }

    .debugger-container {
        width: 100%;
        height: 100%;
        min-width: 500px;
        min-height: 500px;
        background: #1a1a2e;
        border-radius: 10px;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .debugger-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 12px 20px;
        background: rgba(0, 0, 0, 0.3);
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .debugger-header h2 {
        margin: 0;
        font-size: 16px;
        color: #e0e0e0;
    }

    .close-btn {
        background: rgba(239, 68, 68, 0.2);
        border: 1px solid rgba(239, 68, 68, 0.3);
        color: #f87171;
        padding: 6px 12px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 14px;
    }

    .close-btn:hover {
        background: rgba(239, 68, 68, 0.3);
    }

    .debugger-content {
        flex: 1;
        display: flex;
        overflow: hidden;
    }

    .editor-panel {
        flex: 1;
        display: flex;
        flex-direction: column;
        border-right: 1px solid rgba(255, 255, 255, 0.1);
    }

    .panel-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 8px 12px;
        background: rgba(0, 0, 0, 0.2);
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
        font-size: 12px;
        color: rgba(255, 255, 255, 0.6);
        text-transform: uppercase;
    }

    .editor-actions {
        display: flex;
        gap: 8px;
    }

    .editor-container {
        flex: 1;
        min-height: 0;
        height: 100%;
        position: relative;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    /* Ensure Monaco editor fills the container and respects shrinking */
    :global(.monaco-editor) {
        position: absolute;
        inset: 0;
        height: 100% !important;
        width: 100% !important;
    }

    :global(.monaco-editor) :global(.overflow-guard),
    :global(.monaco-editor) :global(.monaco-scrollable-element) {
        height: 100% !important;
        width: 100% !important;
    }

    .debug-panel {
        width: 400px;
        flex: 0 0 400px;
        min-width: 400px;
        max-width: 400px;
        display: flex;
        flex-direction: column;
        overflow-y: auto;
        background: rgba(0, 0, 0, 0.1);
    }

    .control-section,
    .registers-section,
    .stack-section,
    .error-section {
        padding: 12px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .control-section h3,
    .registers-section h3,
    .stack-section h3,
    .error-section h3 {
        margin: 0 0 10px 0;
        font-size: 12px;
        color: rgba(255, 255, 255, 0.5);
        text-transform: uppercase;
    }

    .execution-info {
        display: grid;
        grid-template-columns: auto 1fr auto 1fr;
        gap: 6px 12px;
        margin-bottom: 12px;
        font-size: 12px;
    }

    .info-label {
        color: rgba(255, 255, 255, 0.5);
    }

    .info-value {
        color: #e0e0e0;
        font-family: 'JetBrains Mono', monospace;
    }

    .info-value.status {
        color: #4ade80;
    }

    .info-value.status.halted {
        color: #fbbf24;
    }

    .control-buttons {
        display: flex;
        gap: 8px;
    }

    .control-btn {
        flex: 1;
        padding: 8px 12px;
        border-radius: 6px;
        border: 1px solid;
        cursor: pointer;
        font-size: 12px;
        transition: all 0.15s ease;
    }

    .control-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .control-btn.step {
        background: rgba(99, 102, 241, 0.2);
        border-color: rgba(99, 102, 241, 0.3);
        color: #818cf8;
    }

    .control-btn.step:hover:not(:disabled) {
        background: rgba(99, 102, 241, 0.3);
    }

    .control-btn.run {
        background: rgba(34, 197, 94, 0.2);
        border-color: rgba(34, 197, 94, 0.3);
        color: #4ade80;
    }

    .control-btn.run:hover:not(:disabled) {
        background: rgba(34, 197, 94, 0.3);
    }

    .control-btn.reset {
        background: rgba(251, 191, 36, 0.2);
        border-color: rgba(251, 191, 36, 0.3);
        color: #fbbf24;
    }

    .control-btn.reset:hover:not(:disabled) {
        background: rgba(251, 191, 36, 0.3);
    }

    .error-section {
        background: rgba(239, 68, 68, 0.1);
    }

    .error-message {
        font-family: 'JetBrains Mono', monospace;
        font-size: 12px;
        color: #f87171;
        word-break: break-word;
    }

    .registers-grid {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 4px;
    }

    .register-item {
        display: flex;
        justify-content: space-between;
        padding: 4px 8px;
        background: rgba(0, 0, 0, 0.2);
        border-radius: 4px;
        font-size: 11px;
        font-family: 'JetBrains Mono', monospace;
    }

    .register-item.nonzero {
        background: rgba(99, 102, 241, 0.15);
    }

    .register-name {
        color: rgba(255, 255, 255, 0.5);
    }

    .register-value {
        color: #e0e0e0;
    }

    .stack-list {
        max-height: 200px;
        overflow-y: auto;
    }

    .stack-empty {
        font-size: 12px;
        color: rgba(255, 255, 255, 0.4);
        font-style: italic;
    }

    .stack-item {
        display: flex;
        justify-content: space-between;
        padding: 4px 8px;
        background: rgba(0, 0, 0, 0.2);
        border-radius: 4px;
        margin-bottom: 2px;
        font-size: 11px;
        font-family: 'JetBrains Mono', monospace;
    }

    .stack-index {
        color: rgba(255, 255, 255, 0.5);
    }

    .stack-value {
        color: #e0e0e0;
    }
</style>
