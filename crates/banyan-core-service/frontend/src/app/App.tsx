import { Suspense, useEffect, useState } from 'react';

import { Modal } from '@components/common/Modal';
import { Notifications } from '@components/common/Notifications';
import { FilePreview } from '@components/common/FilePreview';
import { MobilePlaceholder } from '@components/common/MobilePlaceholder';

import { Routes, RoutesConfig } from './routes';
import { FilePreviewProvider } from '@contexts/filesPreview';
import { FileUploadProvider } from '@contexts/filesUpload';
import { TombProvider } from '@contexts/tomb';
import { getLocalStorageItem, setLocalStorageItem } from '@app/utils/localStorage';
import { preventDefaultDragAction } from '@app/utils/dragHandlers';
import { useAppDispatch, useAppSelector } from '@app/store';
import { LANGUAGES, LANGUAGES_KEYS, changeLanguage } from '@store/locales/slice';
import ECCKeystore from '@utils/crypto/ecc/keystore';
import { getLocalKey } from '@app/utils';
import { setKeystore, setKeystoreInitialized } from '@store/keystore/slice';
import { useNavigate } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';
import { getEscrowedKeyMaterial } from '@store/keystore/actions';

const App = () => {
    const dispatch = useAppDispatch();
    const navigate = useNavigate();
    const { keystoreInitialized } = useAppSelector(state => state.keystore);
    const { user } = useAppSelector(state => state.session);
    const [isKeystorageLoading, setIsKeystorageLoading] = useState(true);

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
                    setIsKeystorageLoading(false);
                } catch (err) {
                    setIsKeystorageLoading(false);
                };
            } catch (error: any) {
                throw new Error(error.message);
            }
        })();
    }, []);

    useEffect(() => {
        if(!user.id) return;

        (async () => {
            try {
                const escrowedKeyMaterial = unwrapResult(await dispatch(getEscrowedKeyMaterial()));
                if (isKeystorageLoading || keystoreInitialized) return;

                navigate(escrowedKeyMaterial ? RoutesConfig.EnterEncryptionKey.path : RoutesConfig.CreateEncryptionKey.path);

            } catch (error: any) {
                navigate(RoutesConfig.CreateEncryptionKey.path);
            }
        })()
    }, [isKeystorageLoading, keystoreInitialized, user.id]);

    return (
        <main
            className="flex flex-col h-screen max-h-screen font-sans bg-mainBackground text-text-900 max-sm:hidden"
            onDragOver={preventDefaultDragAction}
            onDrop={preventDefaultDragAction}
        >
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
            <MobilePlaceholder />
        </main>
    );
};

export default App;
