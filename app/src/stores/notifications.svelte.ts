export type NotificationType = 'error' | 'warning' | 'info';

export interface Notification {
    id: string;
    type: NotificationType;
    message: string;
}

let _notifications: Notification[] = $state([]);

export function getNotifications() {
    return {
        get notifications() {
            return _notifications;
        },
    };
}

export function addNotification(type: NotificationType, message: string, durationMs: number | null = 3000): string {
    const id = `notif-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    _notifications = [..._notifications, { id, type, message }];

    if (durationMs !== null && durationMs > 0) {
        setTimeout(() => removeNotification(id), durationMs);
    }

    return id;
}

export function removeNotification(id: string): void {
    _notifications = _notifications.filter((n) => n.id !== id);
}
