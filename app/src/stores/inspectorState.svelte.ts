import type { GridDevice, GridNetwork } from './simulationState.svelte';

export type InspectorType = 'device' | 'network';

export interface InspectorWindow {
    id: string;
    type: InspectorType;
    targetId: number | string; // device id or network id
    x: number;
    y: number;
    width: number;
    height: number;
    zIndex: number;
    minimized: boolean;
    activeTab: string; // 'overview' | 'settings' | 'ic' etc.
}

let _inspectorWindows: InspectorWindow[] = $state([]);
let _nextZIndex = $state(1000);

export function getInspectorState() {
    return {
        get windows() {
            return _inspectorWindows;
        },
    };
}

function _getViewport() {
    if (typeof window === 'undefined') {
        return { width: 1000, height: 1000 };
    }
    return { width: window.innerWidth, height: window.innerHeight };
}

function _clampPosition(x: number, y: number, width: number, height: number) {
    const margin = 16;
    const vp = _getViewport();
    const maxX = Math.max(margin, Math.min(x, Math.max(margin, vp.width - width - margin)));
    const maxY = Math.max(margin, Math.min(y, Math.max(margin, vp.height - height - margin)));
    return { x: maxX, y: maxY };
}

export function openInspector(
    type: InspectorType,
    targetId: number | string,
    initialX: number = 100,
    initialY: number = 100
): InspectorWindow {
    const existing = _inspectorWindows.find((w) => w.type === type && w.targetId === targetId);

    const defaultWidth = type === 'device' ? 380 : 340;
    const defaultHeight = type === 'device' ? 450 : 380;

    const spawnPos = _clampPosition(initialX, initialY, defaultWidth, defaultHeight);

    if (existing) {
        const { x: newX, y: newY } = _clampPosition(
            existing.x,
            existing.y,
            existing.width,
            existing.height
        );
        const idx = _inspectorWindows.findIndex((w) => w.id === existing.id);
        if (idx !== -1) {
            _inspectorWindows[idx] = {
                ..._inspectorWindows[idx],
                x: newX,
                y: newY,
                minimized: false,
                zIndex: _nextZIndex++,
            };
            return _inspectorWindows[idx];
        }
        bringToFront(existing.id);
        return existing;
    }

    const id = `${type}-${targetId}-${Date.now()}`;
    const newWindow: InspectorWindow = {
        id,
        type,
        targetId,
        x: spawnPos.x,
        y: spawnPos.y,
        width: defaultWidth,
        height: defaultHeight,
        zIndex: _nextZIndex++,
        minimized: false,
        activeTab: 'overview',
    };

    _inspectorWindows = [..._inspectorWindows, newWindow];
    return newWindow;
}

export function closeInspector(id: string): void {
    _inspectorWindows = _inspectorWindows.filter((w) => w.id !== id);
}

export function bringToFront(id: string): void {
    const idx = _inspectorWindows.findIndex((w) => w.id === id);
    if (idx === -1) return;

    _inspectorWindows[idx] = {
        ..._inspectorWindows[idx],
        zIndex: _nextZIndex++,
    };
}

export function updateInspectorPosition(id: string, x: number, y: number): void {
    const idx = _inspectorWindows.findIndex((w) => w.id === id);
    if (idx !== -1) {
        const clamped = _clampPosition(
            x,
            y,
            _inspectorWindows[idx].width,
            _inspectorWindows[idx].height
        );
        _inspectorWindows[idx] = { ..._inspectorWindows[idx], x: clamped.x, y: clamped.y };
    }
}

export function updateInspectorSize(id: string, width: number, height: number): void {
    const idx = _inspectorWindows.findIndex((w) => w.id === id);
    if (idx !== -1) {
        const MIN_WIDTH = 280;
        const MIN_HEIGHT = 200;
        const vp = _getViewport();

        const w = Math.max(MIN_WIDTH, Math.min(width, Math.max(MIN_WIDTH, vp.width - 32)));
        const h = Math.max(MIN_HEIGHT, Math.min(height, Math.max(MIN_HEIGHT, vp.height - 32)));

        let x = _inspectorWindows[idx].x;
        let y = _inspectorWindows[idx].y;
        const clamped = _clampPosition(x, y, w, h);
        x = clamped.x;
        y = clamped.y;

        _inspectorWindows[idx] = { ..._inspectorWindows[idx], width: w, height: h, x, y };
    }
}

export function toggleMinimized(id: string): void {
    const idx = _inspectorWindows.findIndex((w) => w.id === id);
    if (idx !== -1) {
        _inspectorWindows[idx] = {
            ..._inspectorWindows[idx],
            minimized: !_inspectorWindows[idx].minimized,
        };
    }
}

export function setActiveTab(id: string, tab: string): void {
    const idx = _inspectorWindows.findIndex((w) => w.id === id);
    if (idx !== -1) {
        _inspectorWindows[idx] = { ..._inspectorWindows[idx], activeTab: tab };
    }
}

export function closeAllInspectors(): void {
    _inspectorWindows = [];
}

export function getInspectorForTarget(
    type: InspectorType,
    targetId: number | string
): InspectorWindow | undefined {
    return _inspectorWindows.find((w) => w.type === type && w.targetId === targetId);
}
