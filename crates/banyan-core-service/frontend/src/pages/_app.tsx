import type { AppProps } from 'next/app';
import { IntlProvider } from 'react-intl';
import { useRouter } from 'next/router';
import { ChakraProvider } from '@chakra-ui/react';
import { SessionProvider } from 'next-auth/react';
import Head from 'next/head';

import en from '@static/locales/en.json';
import fr from '@static/locales/fr.json';
import { NextPageWithLayout } from '@/pages/page';
import { KeystoreProvider } from '@/contexts/keystore';
import { TombProvider } from '@/contexts/tomb';
import { ModalProvider } from '@/contexts/modals';

import { Notifications } from '@/components/common/Notifications';
import { Modal } from '@/components/common/Modal';

import '@static/styles/globals.css';

const TRANSLATES: Record<string, Record<string, string>> = {
    en,
    fr,
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

    return (
        // Session provider for Authentication against NextAuth
        <SessionProvider session={session}>
            {/*  Tomb Provider for access to User's Keystore + TombFs */}
            <KeystoreProvider>
                <TombProvider>
                    <ModalProvider>
                        <ChakraProvider>
                            <IntlProvider locale={locale} messages={TRANSLATES[locale]} >
                                <Notifications />
                                <Modal />
                                <Head>
                                    <title>Banyan</title>
                                    <link rel="icon" href="/static/images/favicon.svg" sizes="any" />
                                </Head>
                                {/* Chakra Provider for access to Chakra UI components */}
                                {/* Get the layout and render the component :) */}
                                {getLayout(<Component {...pageProps} />)}
                            </IntlProvider>
                        </ChakraProvider>
                    </ModalProvider>
                </TombProvider>
            </KeystoreProvider>
        </SessionProvider>
    );
}
