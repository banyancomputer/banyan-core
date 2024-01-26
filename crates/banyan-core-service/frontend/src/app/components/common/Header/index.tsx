import { useEffect, useRef, useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useIntl } from 'react-intl';

import { ProfileControls } from './ProfileControls';
import { HelpControls } from './HelpControls';
import { SubscriptionPlanModal } from '../Modal/SubscriptionPlanModal';

import { useSession } from '@app/contexts/session';
import { popupClickHandler } from '@/app/utils';
import { useKeystore } from '@/app/contexts/keystore';
import { HttpClient } from '@/api/http/client';
<<<<<<< Updated upstream
import { NotFoundError } from '@/api/http';
import { useAppDispatch, useAppSelector } from '@/app/store';
=======
import { useAppDispatch } from '@/app/store';
>>>>>>> Stashed changes
import { unwrapResult } from '@reduxjs/toolkit';
import { getUserInfo } from '@/app/store/user/actions';
import { RoutesConfig } from '@/app/routes';
import { useModal } from '@/app/contexts/modals';

import { Logo, Question } from '@static/images/common';

export const Header: React.FC<{ logo?: boolean, className?: string }> = ({ logo = false, className = '' }) => {
    const dispatch = useAppDispatch();
    const { messages } = useIntl();
    const { selectedSubscription } = useAppSelector(state => state.billing);
    const profileOptionsRef = useRef<HTMLDivElement | null>(null);
    const helpOptionsRef = useRef<HTMLDivElement | null>(null);
    const { purgeKeystore } = useKeystore();
    const location = useLocation();
    const { openModal } = useModal();

    const { userData } = useSession();
    const [areProfileOptionsVisible, setAreProfileOptionsVisible] = useState(false);
    const [areHelpOptionsVisible, setAreHelpOptionsVisible] = useState(false);

    const toggleHelpOptionsVisibility = () => {
        setAreHelpOptionsVisible(prev => !prev);
    };

    const toggleProfileOptionsVisibility = () => {
        setAreProfileOptionsVisible(prev => !prev);
    };

    const upgragePlan = () => {
        openModal(<SubscriptionPlanModal />);
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
                unwrapResult(await dispatch(getUserInfo()));
            } catch (error: any) {
                if (error.message === 'Unauthorized') {
                    const api = new HttpClient;
                    await purgeKeystore();
                    await api.get('/auth/logout');
                    window.location.href = '/login';
                }
            }
        })();
    }, [location]);

    return (
        <header className={`flex items-center justify-between p-4 bg-mainBackground border-b-1 border-border-regular ${className}`}>
            <span className="text-logo">
                {logo && <Logo />}
            </span>
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
                {!selectedSubscription &&
                    <Link
                        onClick={upgragePlan}
                        to={RoutesConfig.Billing.fullPath}
                        className="px-4 py-2 text-xs font-semibold rounded-md bg-text-200 text-button-primary cursor-pointer"
                    >
                        {`${messages.upgrade}`}
                    </Link>
                }
                <div
                    className="relative w-10 h-10 rounded-full cursor-pointer "
                    onClick={toggleProfileOptionsVisibility}
                    ref={profileOptionsRef}
                >
                    {userData?.user?.profileImage ?
                        <img
                            className="rounded-full"
                            src={userData?.user.profileImage}
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
