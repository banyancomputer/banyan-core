import { useEffect, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';

import { ProfileControls } from './ProfileControls';
import { HelpControls } from './HelpControls';
import { SubscriptionPlanModal } from '@components/common/Modal/SubscriptionPlanModal';

import { popupClickHandler } from '@utils/index';
import { HttpClient } from '@/api/http/client';
import { useAppDispatch, useAppSelector } from '@app/store';
import { unwrapResult } from '@reduxjs/toolkit';
import { getUser } from '@store/session/actions';
import { RoutesConfig } from '@app/routes';
import { openModal } from '@store/modals/slice';
import { getSubscriptionById } from '@store/billing/actions';
import { purgeKeystore } from '@store/keystore/actions';

import { Question } from '@static/images/common';

export const Header: React.FC<{ className?: string }> = ({ className = '' }) => {
    const dispatch = useAppDispatch();
    const messages = useAppSelector(state => state.locales.messages.coponents.common.header);
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const { user } = useAppSelector(state => state.session);
    const profileOptionsRef = useRef<HTMLDivElement | null>(null);
    const helpOptionsRef = useRef<HTMLDivElement | null>(null);
    const location = useLocation();

    const [areProfileOptionsVisible, setAreProfileOptionsVisible] = useState(false);
    const [areHelpOptionsVisible, setAreHelpOptionsVisible] = useState(false);

    const toggleHelpOptionsVisibility = () => {
        setAreHelpOptionsVisible(prev => !prev);
    };

    const toggleProfileOptionsVisibility = () => {
        setAreProfileOptionsVisible(prev => !prev);
    };

    const upgragePlan = () => {
        dispatch(openModal({ content: <SubscriptionPlanModal /> }));
    };

    useEffect(() => {
        const profileOptionsListener = popupClickHandler(profileOptionsRef.current!, setAreProfileOptionsVisible);
        const helpOptionsListener = popupClickHandler(helpOptionsRef.current!, setAreHelpOptionsVisible);
        document.addEventListener('click', profileOptionsListener);
        document.addEventListener('click', helpOptionsListener);

        return () => {
            document.removeEventListener('click', profileOptionsListener);
            document.removeEventListener('click', helpOptionsListener);
        };
    }, [profileOptionsRef, helpOptionsRef]);

    useEffect(() => {
        (async () => {
            try {
                const userData = unwrapResult(await dispatch(getUser()));
                dispatch(getSubscriptionById(userData.subscriptionId));
            } catch (error: any) {
                if (error.message === 'Unauthorized') {
                    const api = new HttpClient;
                    unwrapResult(await dispatch(purgeKeystore()));
                    await api.get('/auth/logout');
                    window.location.href = '/login';
                }
            }
        })();
    }, [location]);

    return (
        <header className={`flex items-center justify-between p-4 bg-headerBackground border-b-1 border-headerBorder ${className}`}>
            {/* <SearchInput /> */}
            <div className="flex flex-grow items-center justify-end gap-6">
                <div
                    className="relative cursor-pointer"
                    ref={helpOptionsRef}
                    onClick={toggleHelpOptionsVisibility}
                >
                    <Question width="24px" height="24px" />
                    {areHelpOptionsVisible &&
                        <HelpControls />
                    }
                </div>
                {selectedSubscription?.service_key === 'starter' &&
                    <Link
                        onClick={upgragePlan}
                        to={RoutesConfig.Billing.fullPath}
                        className="btn-secondary px-4 py-2 text-xs font-semibold rounded-md cursor-pointer"
                    >
                        {`${messages.upgrade}`}
                    </Link>
                }
                <div
                    className="relative w-10 h-10 rounded-full cursor-pointer "
                    onClick={toggleProfileOptionsVisibility}
                    ref={profileOptionsRef}
                >
                    {user?.profileImage ?
                        <img
                            className="rounded-full"
                            src={user.profileImage}
                            width={40}
                            height={40}
                            alt="User Avatar"
                        />
                        :
                        null
                    }
                    {areProfileOptionsVisible &&
                        <ProfileControls />
                    }
                </div>
            </div>
        </header >
    );
};
