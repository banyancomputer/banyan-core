import React from 'react';
import { useIntl } from 'react-intl';
import { useNavigate } from 'react-router-dom';

import { Action } from '@components/Bucket/BucketTable/FileActions';
import { HttpClient } from '@/api/http/client';
import { useKeystore } from '@app/contexts/keystore';

import { Key, LogoutAlternative, Settings } from '@static/images/common';

export const ProfileControls = () => {
    const navigate = useNavigate();
    const { messages } = useIntl();
    const { purgeKeystore } = useKeystore();

    const goTo = (path: string) => function () {
        navigate(path);
    };

    const logout = async () => {
        const api = new HttpClient();
        try {
            await purgeKeystore();
            await api.get('/auth/logout');
            window.location.href = '/login';
        }
        catch (err: any) {
            console.error('An Error occurred trying to logout: ', err.message);
        }
    };


    const options = [
        new Action(`${messages.settings}`, <Settings />, goTo('/account/settings')),
        new Action(`${messages.manageKeys}`, <Key />, goTo('/account/manage-keys')),
        new Action(`${messages.logout}`, <LogoutAlternative />, logout),
    ];

    return (
        <div
            className="absolute z-10 right-0 top-12 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden  bg-bucket-actionsBackground text-bucket-actionsText cursor-pointer border-1 border-border-darken"
        >
            {options.map(option =>
                <div
                    key={option.label}
                    className="flex items-center gap-2 py-2.5 px-3 whitespace-nowrap transition-all hover:bg-hover"
                    onClick={option.value}
                >
                    <span className="text-button-primary">
                        {option.icon}
                    </span>
                    {option.label}
                </div>
            )}
        </div>
    )
}