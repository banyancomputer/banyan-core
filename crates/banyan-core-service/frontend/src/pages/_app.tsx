import type { AppProps } from 'next/app';
import { IntlProvider } from 'react-intl';
import { useRouter } from 'next/router';
import { ChakraProvider } from '@chakra-ui/react';
import { SessionProvider } from 'next-auth/react';
import Script from 'next/script';

import en from '@static/locales/en.json';
import fr from '@static/locales/fr.json';
import { NextPageWithLayout } from '@/pages/page';
import { KeystoreProvider } from '@/contexts/keystore';
import { TombProvider } from '@/contexts/tomb';
import { ModalProvider } from '@/contexts/modals';
import { FilePreviewProvider } from '@/contexts/filesPreview';
import { FileUploadProvider } from '@/contexts/filesUpload';

import { Notifications } from '@/components/common/Notifications';
import { Modal } from '@/components/common/Modal';
import { FilePreview } from '@/components/common/FilePreview';

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
                <Script>
                    {`
                            window['_fs_host'] = 'fullstory.com';
                            window['_fs_script'] = 'edge.fullstory.com/s/fs.js';
                            window['_fs_org'] = 'o-1QMN2D-na1';
                            window['_fs_namespace'] = 'FS';
                            (function(m,n,e,t,l,o,g,y){
                                if (e in m) {if(m.console && m.console.log) { m.console.log('FullStory namespace conflict. Please set window["_fs_namespace"].');} return;}
                                g=m[e]=function(a,b,s){g.q?g.q.push([a,b,s]):g._api(a,b,s);};g.q=[];
                                o=n.createElement(t);o.async=1;o.crossOrigin='anonymous';o.src='https://'+_fs_script;
                                y=n.getElementsByTagName(t)[0];y.parentNode.insertBefore(o,y);
                                g.identify=function(i,v,s){g(l,{uid:i},s);if(v)g(l,v,s)};g.setUserVars=function(v,s){g(l,v,s)};g.event=function(i,v,s){g('event',{n:i,p:v},s)};
                                g.anonymize=function(){g.identify(!!0)};
                                g.shutdown=function(){g("rec",!1)};g.restart=function(){g("rec",!0)};
                                g.log = function(a,b){g("log",[a,b])};
                                g.consent=function(a){g("consent",!arguments.length||a)};
                                g.identifyAccount=function(i,v){o='account';v=v||{};v.acctId=i;g(o,v)};
                                g.clearUserCookie=function(){};
                                g.setVars=function(n, p){g('setVars',[n,p]);};
                                g._w={};y='XMLHttpRequest';g._w[y]=m[y];y='fetch';g._w[y]=m[y];
                                if(m[y])m[y]=function(){return g._w[y].apply(this,arguments)};
                                g._v="1.3.0";
                            })(window,document,window['_fs_namespace'],'script','user');
                        `}
                </Script>
                <TombProvider>
                    <ModalProvider>
                        <FileUploadProvider>
                            <FilePreviewProvider>
                                <ChakraProvider>
                                    <IntlProvider locale={locale} messages={TRANSLATES[locale]} >
                                        <Notifications />
                                        <Modal />
                                        <FilePreview />
                                        {/* Chakra Provider for access to Chakra UI components */}
                                        {/* Get the layout and render the component :) */}
                                        {getLayout(<Component {...pageProps} />)}
                                    </IntlProvider>
                                </ChakraProvider>
                            </FilePreviewProvider>
                        </FileUploadProvider>
                    </ModalProvider>
                </TombProvider>
            </KeystoreProvider>
        </SessionProvider>
    );
}
