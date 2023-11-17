export interface Notification {
    details: {
        type: string
    },
    id: string,
    msg: string,
    severity: string,
    triggered_at: string
};