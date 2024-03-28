import { Suspense, useEffect } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { Modal } from '@components/common/Modal';
import { Notifications } from '@components/common/Notifications';
import { FilePreview } from '@components/common/FilePreview';
import { MobilePlaceholder } from '@components/common/MobilePlaceholder';

import { Routes } from './routes';
import { FilePreviewProvider } from '@app/contexts/filesPreview';
import { FileUploadProvider } from '@app/contexts/filesUpload';
import { TombProvider } from '@app/contexts/tomb';
import { getLocalStorageItem, setLocalStorageItem } from '@app/utils/localStorage';
import { preventDefaultDragAction } from '@app/utils/dragHandlers';
import { useAppDispatch } from '@app/store';
import { LANGUAGES, LANGUAGES_KEYS, changeLanguage } from '@app/store/locales/slice';
import ECCKeystore from '@utils/crypto/ecc/keystore';
import { getLocalKey } from '@app/utils';
import { setKeystore, setKeystoreInitialized } from '@app/store/keystore/slice';

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

        const currentLanguage = navigator.language.includes('-') ? navigator.language.split('-')[0] : navigator.language;
        const languagesKeys = Object.keys(LANGUAGES);

        setLocalStorageItem('lang', languagesKeys.includes(currentLanguage) ? currentLanguage : 'en');
    }, []);

    useEffect(() => {
        (async () => {
            try {
                const ks = await ECCKeystore.init({
                    storeName: 'banyan-key-cache',
                });
                ks.clear();
                dispatch(setKeystore(ks));
                let localKey = getLocalKey();
                try {
                    await ks.retrieveCachedPrivateKeyMaterial(
                        localKey.key, localKey.id
                    );
                    dispatch(setKeystoreInitialized(true));
                    console.log("createKeystore: using cached key");
                } catch (err) {
                    console.log("No valid cached key material found for this session");
                };
            } catch (error: any) {
                throw new Error(error.message);
            }
        })();
    }, []);

    return (
        <main
            className="flex flex-col h-screen max-h-screen font-sans bg-mainBackground text-text-900 max-sm:hidden"
            onDragOver={preventDefaultDragAction}
            onDrop={preventDefaultDragAction}
        >
            <BrowserRouter basename="/" >
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
            </BrowserRouter>
            <MobilePlaceholder />
        </main>
    );
};

export default App;
