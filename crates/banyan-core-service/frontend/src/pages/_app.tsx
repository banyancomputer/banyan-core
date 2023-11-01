import { useEffect } from 'react';
import type { AppProps } from 'next/app';
import { IntlProvider } from 'react-intl';
import { useRouter } from 'next/router';
import { SessionProvider } from 'next-auth/react';

import en from '@static/locales/en.json';
import fr from '@static/locales/fr.json';
import de from '@static/locales/de.json';
import ja from '@static/locales/ja.json';
import zh from '@static/locales/zh.json';
import { NextPageWithLayout } from '@/pages/page';
import { KeystoreProvider } from '@/contexts/keystore';
import { TombProvider } from '@/contexts/tomb';
import { ModalProvider } from '@/contexts/modals';
import { FilePreviewProvider } from '@/contexts/filesPreview';
import { FileUploadProvider } from '@/contexts/filesUpload';
import { getLocalStorageItem } from '@/utils/localStorage';

import { Notifications } from '@/components/common/Notifications';
import { Modal } from '@/components/common/Modal';
import { FilePreview } from '@/components/common/FilePreview';

import '@static/styles/globals.css';

const TRANSLATES: Record<string, Record<string, string>> = {
    en,
    fr,
    de,
    ja,
    zh
};

interface AppPropsWithLayout extends AppProps {
    Component: NextPageWithLayout;
}
export default function App({
    Component,
    pageProps: { session, ...pageProps },
}: AppPropsWithLayout) {
    const getLayout = Component.getLayout || ((page) => page);
    const { locale = '' } = useRouter();

    useEffect(() => {
        const theme = getLocalStorageItem('theme');

        if (!theme) return;
        document.documentElement.setAttribute('prefers-color-scheme', theme);
    }, []);

    return (
        // Session provider for Authentication against NextAuth
        <SessionProvider session={session}>
            {/*  Tomb Provider for access to User's Keystore + TombFs */}
            <KeystoreProvider>
                <TombProvider>
                    <ModalProvider>
                        <FileUploadProvider>
                            <FilePreviewProvider>
                                <IntlProvider locale={locale} messages={TRANSLATES[locale]} >
                                    <Notifications />
                                    <Modal />
                                    <FilePreview />
                                    <>
                                        {/* Get the layout and render the component :) */}
                                        {getLayout(<Component {...pageProps} />)}
                                    </>
                                </IntlProvider>
                            </FilePreviewProvider>
                        </FileUploadProvider>
                    </ModalProvider>
                </TombProvider>
            </KeystoreProvider>
        </SessionProvider>
    );
}
