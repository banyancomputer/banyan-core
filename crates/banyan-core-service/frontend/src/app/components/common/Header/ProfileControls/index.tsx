import React, { useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';

import { Action } from '@components/Bucket/Files/BucketTable/FileActions';
import { SubscriptionPlanModal } from '@components/common/Modal/SubscriptionPlanModal';

import { HttpClient } from '@/api/http/client';
import { useAppDispatch, useAppSelector } from '@app/store';
import { getSubscriptions } from '@store/billing/actions';
import { RoutesConfig } from '@/app/routes';
import { openModal } from '@store/modals/slice';
import { unwrapResult } from '@reduxjs/toolkit';
import { purgeKeystore } from '@store/keystore/actions';

import { Logout, Settings } from '@static/images/common';

export const ProfileControls = () => {
    const navigate = useNavigate();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.header.profileControls);
    const { displayName, email, profileImage } = useAppSelector(state => state.session.user);
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const dispatch = useAppDispatch();


    const goTo = (path: string) => function () {
        navigate(path);
    };

    const logout = async () => {
        const api = new HttpClient();
        try {
            unwrapResult(await dispatch(purgeKeystore()));
            await api.get('/auth/logout');
            window.location.href = '/login';
        }
        catch (err: any) {
            console.error('An Error occurred trying to logout: ', err.message);
        };
    };

    const upgragePlan = () => {
        dispatch(openModal({ content: <SubscriptionPlanModal /> }));
    };

    const options = [
        new Action(`${messages.settings}`, <Settings />, goTo('/account/profile')),
        new Action(`${messages.logout}`, <Logout />, logout),
    ];

    useEffect(() => {
        dispatch(getSubscriptions());
    }, []);

    return (
        <div
            className="absolute z-10 right-0 top-12 w-[270px] shadow-xl rounded-md text-xs font-medium overflow-hidden  bg-bucket-actionsBackground text-bucket-actionsText cursor-pointer border-1 border-border-darken"
        >
            <div className="flex items-stretch gap-3 flex-wrap p-4 border-b-1 border-border-regular">
                <img
                    className="rounded-full"
                    src={profileImage}
                    width={40}
                    height={40}
                    alt="User Avatar"
                />
                <div className="flex flex-col text-xs">
                    <span className="font-medium">{displayName}</span>
                    <span className="font-normal">{email}</span>
                </div>
                <div className="w-full flex gap-2">
                    <span className="text-text-900">
                        {selectedSubscription?.title}
                    </span>
                    {selectedSubscription?.service_key === 'starter' &&
                        <Link
                            onClick={upgragePlan}
                            to={RoutesConfig.Billing.fullPath}
                            className="font-semibold underline cursor-pointer"
                        >
                            {`${messages.upgrade}`}
                        </Link>
                    }
                </div>
            </div>
            <div className="flex flex-col items-stretch">
                {options.map(option =>
                    <div
                        key={option.label}
                        className="flex items-center gap-2 py-3 px-4 whitespace-nowrap transition-all hover:bg-hover"
                        onClick={option.value}
                    >
                        <span>
                            {option.icon}
                        </span>
                        {option.label}
                    </div>
                )}
            </div>
        </div>
    )
};
