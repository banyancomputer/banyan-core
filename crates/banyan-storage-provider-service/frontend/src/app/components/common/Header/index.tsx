import React, { useEffect, useRef, useState } from 'react';
import { NavLink, useLocation } from 'react-router-dom';

import { Bolt, Headphones, Logo, Mail, Question } from '@static/images'
import { popupClickHandler } from '@app/utils/clickHandlers';
import { NotificationsHistory } from '@components/common/NotificationsHistory';

export const Header = () => {
    const [isNotificationsVisible, setIsNotificationsVisible] = useState(false);
    const [isFAQVisible, setIsFAQVisible] = useState(false);
    const faqRef = useRef<HTMLDivElement | null>(null);
    const notificationsRef = useRef<HTMLDivElement | null>(null);
    const location = useLocation();

    useEffect(() => {
        const listener = popupClickHandler(faqRef.current!, setIsFAQVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [faqRef]);

    useEffect(() => {
        const listener = popupClickHandler(notificationsRef.current!, setIsNotificationsVisible);
        document.addEventListener('click', listener);

        return () => {
            document.removeEventListener('click', listener);
        };
    }, [notificationsRef]);

    return (
        <header className='mx-auto max-w-wrapper flex items-center gap-10 justify-between px-12 py-10 bg-mainBackground text-lightText'>
            <Logo />
            <nav className="flex items-center flex-grow gap-4 text-black">
                <NavLink
                    to="/"
                    className={`px-4 py-2 text-16 font-semibold ${location.pathname === '/' ? 'text-lightText border-b-1 border-lightText' : ''}`}
                >
                    Dashboard
                </NavLink>
                <NavLink
                    to="/leaderboard"
                    className={`px-4 py-2 text-16 font-semibold ${location.pathname === '/leaderboard' ? 'text-lightText border-b-1 border-lightText' : ''}`}
                >
                    Leaderboard
                </NavLink>
            </nav>
            <div className='flex items-center gap-3 text-darkText'>
                <div
                    className='relative p-2.5 cursor-pointer'
                    onClick={() => setIsNotificationsVisible(prev => !prev)}
                    ref={notificationsRef}
                >
                    <Bolt />
                    {isNotificationsVisible &&
                        <NotificationsHistory />
                    }
                </div>
                <div
                    className='p-2.5 relative cursor-pointer'
                    onClick={() => setIsFAQVisible(prev => !prev)}
                    ref={faqRef}
                >
                    <Headphones />
                    {isFAQVisible &&
                        <div
                            className="absolute z-10 right-0 top-10 w-36 flex flex-col items-stretch shadow-xl rounded-xl text-xs font-semibold overflow-hidden bg-contextMenuBackground cursor-pointer text-text-900"
                        >
                            <a
                                className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
                                href="https://banyan8674.zendesk.com/hc/en-us"
                                target="_blank"
                            >
                                <span className="text-button-primary">
                                    <Question />
                                </span>
                                FAQ
                            </a>
                            <a
                                href="mailto:support@banyan8674.zendesk.com"
                                className="flex items-center gap-2 py-2.5 px-3 transition-all hover:bg-hover"
                                target="_blank"
                            >
                                <span className="text-button-primary">
                                    <Mail />
                                </span>
                                Contact Us
                            </a>
                        </div>
                    }
                </div>
            </div>
        </header>
    )
}
