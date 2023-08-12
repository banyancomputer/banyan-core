import type { AppProps } from 'next/app';
import { ChakraProvider } from '@chakra-ui/react';
import { SessionProvider } from 'next-auth/react';
import { NextPageWithLayout } from '@/pages/page';
import { KeystoreProvider } from '@/contexts/keystore';
import { TombProvider } from '@/contexts/tomb';
import { ModalProvider } from '@/contexts/modals';
import { Modal } from '@/components/common/Modal';

import '@static/styles/globals.css';

interface AppPropsWithLayout extends AppProps {
    Component: NextPageWithLayout;
}
export default function App({
    Component,
    pageProps: { session, ...pageProps },
}: AppPropsWithLayout) {
    const getLayout = Component.getLayout || ((page) => page);

    return (
        // Session provider for Authentication against NextAuth
        <SessionProvider session={session}>
            {/*  Tomb Provider for access to User's Keystore + TombFs */}
            <KeystoreProvider>
                <TombProvider>
                    <ModalProvider>
                        <ChakraProvider>
                            <Modal />
                            {/* Chakra Provider for access to Chakra UI components */}
                            {/* Get the layout and render the component :) */}
                            {getLayout(<Component {...pageProps} />)}
                        </ChakraProvider>
                    </ModalProvider>
                </TombProvider>
            </KeystoreProvider>
        </SessionProvider>
    );
}
