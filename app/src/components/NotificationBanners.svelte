<script lang="ts">
    import { getNotifications, removeNotification } from '../stores/notifications.svelte';

    const notifState = getNotifications();

    function dismiss(id: string) {
        removeNotification(id);
    }
</script>

<div class="banner-container">
    {#each notifState.notifications as n (n.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div class="banner {n.type}" role="status" aria-live="polite" onclick={() => dismiss(n.id)}>
            <div class="message">{n.message}</div>
            <button
                class="close"
                onclick={(e) => {
                    e.stopPropagation();
                    dismiss(n.id);
                }}
                aria-label="Dismiss"
            >
                ✕
            </button>
        </div>
    {/each}
</div>

<style>
    .banner-container {
        position: fixed;
        top: 18px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 2000;
        display: flex;
        flex-direction: column;
        gap: 8px;
        pointer-events: none;
    }

    :global(.banner) {
        pointer-events: auto;
        min-width: 300px;
        max-width: 500px;
        padding: 12px 16px;
        border-radius: 6px;
        display: flex;
        align-items: center;
        gap: 12px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        cursor: pointer;
        transition:
            transform 0.15s ease,
            box-shadow 0.15s ease;
        border: 1px solid rgba(255, 255, 255, 0.2);
    }

    :global(.banner:hover) {
        transform: translateY(-1px);
        box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
    }

    :global(.banner.info) {
        background: #3b82f6;
        color: white;
    }

    :global(.banner.warning) {
        background: #f59e0b;
        color: #1f2937;
    }

    :global(.banner.error) {
        background: #ef4444;
        color: white;
    }

    :global(.banner .message) {
        flex: 1 1 auto;
        font-size: 14px;
        line-height: 1.4;
        font-weight: 500;
    }

    :global(.banner .close) {
        background: rgba(0, 0, 0, 0.1);
        border: none;
        color: inherit;
        font-weight: 700;
        cursor: pointer;
        padding: 4px 8px;
        border-radius: 4px;
        font-size: 16px;
        line-height: 1;
        transition: background 0.15s ease;
        flex-shrink: 0;
    }

    :global(.banner .close:hover) {
        background: rgba(0, 0, 0, 0.2);
    }
</style>
