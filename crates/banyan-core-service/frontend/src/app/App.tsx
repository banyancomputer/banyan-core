import { Suspense, useEffect, useState } from 'react';
import { BrowserRouter } from 'react-router-dom';
import { Provider } from 'react-redux';

import { Modal } from '@components/common/Modal';
import { Notifications } from '@components/common/Notifications';
import { FilePreview } from '@components/common/FilePreview';
import { MobilePlaceholder } from '@components/common/MobilePlaceholder';

import { Routes } from './routes';
import { KeystoreProvider } from './contexts/keystore';
import { FilePreviewProvider } from './contexts/filesPreview';
import { ModalProvider } from './contexts/modals';
import { FileUploadProvider } from './contexts/filesUpload';
import { TombProvider } from './contexts/tomb';
import { getLocalStorageItem, setLocalStorageItem } from './utils/localStorage';
import { SessionProvider } from './contexts/session';
import { preventDefaultDragAction } from './utils/dragHandlers';
import { store, useAppDispatch } from '@app/store';
import { LANGUAGES_KEYS, changeLanguage } from '@app/store/locales/slice';


const App = () => {
    const dispatch = useAppDispatch();

    useEffect(() => {
        const theme = getLocalStorageItem('theme');
        theme ? document.documentElement.setAttribute('prefers-color-scheme', theme) :
            document.documentElement.setAttribute('prefers-color-scheme', 'light');
    }, []);

    useEffect(() => {
        window.addEventListener('storage', () => {
            const selectedLanguage = getLocalStorageItem('lang');
            dispatch(changeLanguage(selectedLanguage as LANGUAGES_KEYS || 'en'));
        });

        const selectedLanguage = getLocalStorageItem('lang');
        dispatch(changeLanguage(selectedLanguage as LANGUAGES_KEYS || 'en'));

        if (selectedLanguage) { return; }

        setLocalStorageItem('lang', navigator.language.includes('-') ? navigator.language.split('-')[0] : navigator.language);
    }, []);

    return (
        <Provider store={store}>
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
        </Provider>
    );
};

export default App;
