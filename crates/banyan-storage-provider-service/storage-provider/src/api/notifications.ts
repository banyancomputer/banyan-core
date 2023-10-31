import { APIClient } from ".";
import { Notification } from "@/entities/notifications";

export class NotificationsClient extends APIClient {
    public async getNotificationsHistory():Promise<Notification[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/alerts/history`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };

    public async getNotifications(): Promise<Notification[]> {
        const response = await this.http.get(`${this.ROOT_PATH}/api/v1/alerts`)

        if (!response.ok) {
            await this.handleError(response);
        }

        return await response.json();
    };
}