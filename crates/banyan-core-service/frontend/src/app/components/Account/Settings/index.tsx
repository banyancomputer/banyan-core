import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';
import { LanguageSelect } from '@/app/components/common/LanguageSelect';
import { getLocalStorageItem, setLocalStorageItem } from '@/app/utils/localStorage';

export const Settings = () => {
    const { messages } = useIntl();
    const [isDarkModaActive, setIsDarkModeActive] = useState(false);

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
                {`${messages.settings}`}
            </h2>
            <div className="flex justify-between items-center py-5 px-4 border-1 rounded-lg bg-secondaryBackground text-text-800 border-border-regular">
                <div>
                    <h5 className="font-semibold">{`${messages.theme}`}</h5>
                    <p>{`${messages.selectTheme}`}</p>
                </div>
                <input
                    type="checkbox"
                    className="toggle"
                    checked={isDarkModaActive}
                    onChange={toggleTheme}
                />
            </div>
            <div className="flex justify-between items-center py-5 px-4 border-1 rounded-lg bg-secondaryBackground text-text-800 border-border-regular">
                <div>
                    <h5 className="font-semibold">{`${messages.language}`}</h5>
                    <p>{`${messages.chooseLanguage}`}</p>
                </div>
                <LanguageSelect />
            </div>
        </div>
    );
};

export default Settings;
