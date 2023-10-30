import React, { useEffect, useRef } from 'react';

import { useAppDispatch, useAppSelector } from '@app/store'
import { getNotifications, getNotificationsHistory } from '@app/store/notifications/actions';
import { NotificationBolt } from '@static/images';
import { getNotiifcationDateLabel } from '@app/utils/time';

export const NotificationsHistory = () => {
    const dispatch = useAppDispatch();
    const { notificationsHistory } = useAppSelector(state => state.notifications);
    const { activeDeals, availiableDeals } = useAppSelector(state => state.deals);

    const scrollIntoView = (id: string) => {
        const deal = document.getElementById('12');

        deal?.scrollIntoView({ block: 'center' });
        deal!.style.border = '1px solid #F94144';

        setTimeout(() => {
            deal!.style.border = '1px solid #AAAA';
        }, 3000);
    }

    useEffect(() => {
        try {
            (async () => {
                await dispatch(getNotificationsHistory());
                await dispatch(getNotifications());
            })()
        } catch (error: any) { }
    }, []);

    return (
        <div className="absolute top-10 right-0 rounded-xl overflow-hidden shadow-xl">
            <div className=' w-96 max-h-notifications bg-contextMenuBackground overflow-y-scroll'>
                {
                    notificationsHistory.map(notification =>
                        <div
                            className="flex items-center gap-5 p-4 border-b-1 border-b-tableBorder cursor-pointer transition-all last:border-0 hover:bg-slate-200"
                            onClick={() => scrollIntoView(notification.id)}
                        >
                            <div className='w-10 h-10'>
                                <NotificationBolt />
                            </div>
                            <div className="flex flex-col items-start">
                                <span className="font-semibold text-left">{notification.msg}</span>
                                <span className="text-10 text-tableBorder">{getNotiifcationDateLabel(new Date(notification.triggered_at))}</span>
                            </div>
                        </div>
                    )
                }
            </div>
        </div>
    )
}
