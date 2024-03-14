import { useEffect, useState } from 'react';

import { LanguageSelect } from '@components/common/LanguageSelect';

import { getLocalStorageItem, setLocalStorageItem } from '@/app/utils/localStorage';
import { useAppSelector } from '@/app/store';

export const Profile = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.profile);
    const { email, displayName } = useAppSelector(state => state.user);
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
        <div className="flex flex-col px-6 text-text-900 text-xs">
            <div className="flex justify-between items-center p-4 border-b-1 border-border-regular">
                <span>{messages.name}</span>
                <span className="font-medium">{displayName}</span>
            </div>
            <div className="flex justify-between items-center p-4 border-b-1 border-border-regular">
                <span>{messages.email}</span>
                <span className="font-medium">{email}</span>
            </div>
            <div className="flex justify-between items-center p-4 border-b-1 border-border-regular">
                <span>{messages.darkMode}</span>
                <span className="flex items-center gap-4 font-medium">
                    {isDarkModaActive ? "On" : "Off"}
                    <input
                        type="checkbox"
                        className="toggle"
                        checked={isDarkModaActive}
                        onChange={toggleTheme}
                    />
                </span>
            </div>
            <div className="flex justify-between items-center py-2 px-4 border-b-1 border-border-regular">
                <span>{messages.language}</span>
                <LanguageSelect />
            </div>
        </div>
    );
};

export default Profile;
