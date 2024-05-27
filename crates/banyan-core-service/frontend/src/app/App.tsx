import { Suspense, useEffect, useState } from 'react';

import { Modal } from '@components/common/Modal';
import { Notifications } from '@components/common/Notifications';
import { FilePreview } from '@components/common/FilePreview';
import { MobilePlaceholder } from '@components/common/MobilePlaceholder';

import { Routes, RoutesConfig } from './routes';
import { FilePreviewProvider } from '@contexts/filesPreview';
import { FileUploadProvider } from '@contexts/filesUpload';
import { getLocalStorageItem, setLocalStorageItem } from '@app/utils/localStorage';
import { preventDefaultDragAction } from '@app/utils/dragHandlers';
import { useAppDispatch, useAppSelector } from '@app/store';
import { LANGUAGES, LANGUAGES_KEYS, changeLanguage } from '@store/locales/slice';
import ECCKeystore from '@utils/crypto/ecc/keystore';
import { getLocalKey } from '@app/utils';
import { setKeystore, setKeystoreInitialized } from '@store/keystore/slice';
import { useNavigate } from 'react-router-dom';
import { unwrapResult } from '@reduxjs/toolkit';
import { getApiKey, getEncryptionKey, getEscrowedKeyMaterial } from '@store/keystore/actions';
import { setEncryptionKey, setTomb } from '@store/tomb/slice';
import { BannerError, setError } from '@store/errors/slice';
import { getBuckets, updateStorageLimitsState, updateStorageUsageState } from '@store/tomb/actions';

const App = () => {
    const dispatch = useAppDispatch();
    const navigate = useNavigate();
    const { keystoreInitialized, escrowedKeyMaterial } = useAppSelector(state => state.keystore);
    const { tomb, buckets } = useAppSelector(state => state.tomb);

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
        if (!user.id || !keystoreInitialized || !escrowedKeyMaterial) { return; }

        (async () => {
            try {
                const apiKey = unwrapResult(await dispatch(getApiKey()));
                const TombWasm = (await import('tomb-wasm-experimental')).TombWasm;
                const tomb = await new TombWasm(
                    apiKey.privatePem,
                    user.id,
                    window.location.protocol + '//' + window.location.host,
                );
                const key = unwrapResult(await dispatch(getEncryptionKey()));
                dispatch(setEncryptionKey(key));
                dispatch(setTomb(tomb));
            } catch (error: any) {
                dispatch(setError(new BannerError(error.message)));
            };
        })()
    }, [user, keystoreInitialized, escrowedKeyMaterial]);

    useEffect(() => {
        if (!tomb) return;

        (async () => {
            try {
                unwrapResult(await dispatch(getBuckets()));
                unwrapResult(await dispatch(updateStorageUsageState()));
                unwrapResult(await dispatch(updateStorageLimitsState()));
            } catch (error: any) {
                dispatch(setError(new BannerError(error.message)));
            };
        })();
    }, [tomb]);

    useEffect(() => {
        if (!user.id) return;

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
            <MobilePlaceholder />
        </main>
    );
};

export default App;
