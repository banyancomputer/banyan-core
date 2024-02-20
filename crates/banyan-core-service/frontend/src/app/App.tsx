import { Suspense, useEffect, useState } from 'react';
import { IntlProvider } from 'react-intl';
import { BrowserRouter } from 'react-router-dom';
import { Provider } from 'react-redux';

import { Modal } from '@components/common/Modal';
import { Notifications } from '@components/common/Notifications';
import { FilePreview } from '@components/common/FilePreview';
import { MobilePlaceholder } from '@components/common/MobilePlaceholder';

import { Routes } from '@app/routes';
import { KeystoreProvider } from '@app/contexts/keystore';
import { FilePreviewProvider } from '@app/contexts/filesPreview';
import { ModalProvider } from '@app/contexts/modals';
import { FileUploadProvider } from '@app/contexts/filesUpload';
import { TombProvider } from '@app/contexts/tomb';
import { getLocalStorageItem, setLocalStorageItem } from '@app/utils/localStorage';
import { SessionProvider } from '@app/contexts/session';
import { preventDefaultDragAction } from '@app/utils/dragHandlers';
import { ErrorProvider } from '@app/contexts/error';
import { store } from '@app/store';

import en from '@static/locales/en.json';
import fr from '@static/locales/fr.json';
import de from '@static/locales/de.json';
import ja from '@static/locales/ja.json';
import zh from '@static/locales/zh.json';


const TRANSLATES: Record<string, Record<string, string>> = {
    en,
    fr,
    de,
    ja,
    zh,
};

export const locales = Object.keys(TRANSLATES);

const App = () => {
    const [locale, setLocale] = useState('en');

    useEffect(() => {
        const theme = getLocalStorageItem('theme');
        theme && document.documentElement.setAttribute('prefers-color-scheme', theme);

        window.addEventListener('storage', () => {
            const selectedLanguage = getLocalStorageItem('lang');
            setLocale(selectedLanguage || 'en');
        });

        const selectedLanguage = getLocalStorageItem('lang');
        setLocale(selectedLanguage || 'en');

        if (selectedLanguage) { return; }

        setLocalStorageItem('lang', navigator.language.includes('-') ? navigator.language.split('-')[0] : navigator.language);
    }, []);

    return (
        <Provider store={store}>
            <ErrorProvider>
                <IntlProvider locale={locale} messages={TRANSLATES[locale]}>
                    <main
                        className="flex flex-col h-screen max-h-screen font-sans bg-mainBackground text-text-900 max-sm:hidden"
                        onDragOver={preventDefaultDragAction}
                        onDrop={preventDefaultDragAction}
                    >
                        <BrowserRouter basename="/" >
                            <ModalProvider>
                                <SessionProvider>
                                    <KeystoreProvider>
                                        <TombProvider>
                                            <FileUploadProvider>
                                                <FilePreviewProvider>
                                                    <Modal />
                                                    <FilePreview />
                                                    <Notifications />
                                                    <Suspense>
                                                        <Routes />
                                                    </Suspense>
                                                </FilePreviewProvider>
                                            </FileUploadProvider>
                                        </TombProvider>
                                    </KeystoreProvider>
                                </SessionProvider>
                            </ModalProvider>
                        </BrowserRouter>
                    </main>
                    <MobilePlaceholder />
                </IntlProvider>
            </ErrorProvider>
        </Provider>
    );
};

export default App;
