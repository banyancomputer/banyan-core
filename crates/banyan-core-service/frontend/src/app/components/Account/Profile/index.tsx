import React, { useEffect, useState } from 'react';

import { LanguageSelect } from '@components/common/LanguageSelect';

import { getLocalStorageItem, setLocalStorageItem } from '@/app/utils/localStorage';
import { useAppSelector } from '@/app/store';

export const Profile = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.profile);
    const [isDarkModaActive, setIsDarkModeActive] = useState(false);

    /** Uncomment when dark theme will be updated. */
    const toggleTheme = () => {
        const currentTheme = document.documentElement.getAttribute('prefers-color-scheme');
        const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
        document.documentElement.setAttribute('prefers-color-scheme', newTheme);
        setLocalStorageItem('theme', newTheme);
        setIsDarkModeActive(newTheme === 'dark');
    };

    useEffect(() => {
        const theme = getLocalStorageItem('theme');

        setIsDarkModeActive(theme === 'dark');
    }, []);

    return (
        <div className="flex flex-col gap-5 px-10">
            <h2 className="text-lg font-semibold">
                {messages.title}
            </h2>
            {/* <div className="flex justify-between items-center py-5 px-4 border-1 rounded-lg bg-secondaryBackground text-text-800 border-border-regular">
                <div>
                    <h5 className="font-semibold">{`${messages.theme}`}</h5>
                    <p>{messages.selectTheme}</p>
                </div>
                <input
                    type="checkbox"
                    className="toggle"
                    checked={isDarkModaActive}
                    onChange={toggleTheme}
                />
            </div> */}
            <div className="flex justify-between items-center py-5 px-4 border-1 rounded-lg bg-secondaryBackground text-text-800 border-border-regular">
                <div>
                    <h5 className="font-semibold">{`${messages.language}`}</h5>
                    <p>{messages.chooseYourLanguage}</p>
                </div>
                <LanguageSelect />
            </div>
        </div>
    );
};

export default Profile;
