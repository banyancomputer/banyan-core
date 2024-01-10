import { Suspense, useEffect } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { Notifications } from '@components/common/Notifications';
import { Navigation } from '@components/common/Navigation';
import { Header } from '@components/common/Header';
import { ErrorBanner } from '@components/common/ErrorBanner';
import { MobilePlaceholder } from '@components/common/MobilePlaceholder';

import { Routes } from './routes';
import { ModalProvider } from './contexts/modals';
import { getLocalStorageItem, setLocalStorageItem } from './utils/localStorage';
import { SessionProvider } from './contexts/session';

const App = () => {

    useEffect(() => {
        const theme = getLocalStorageItem('theme');
        theme && document.documentElement.setAttribute('prefers-color-scheme', theme);

        window.addEventListener('storage', () => {
            const selectedLanguage = getLocalStorageItem('lang');
        });

        const selectedLanguage = getLocalStorageItem('lang');

        if (selectedLanguage) { return; }

        setLocalStorageItem('lang', navigator.language.includes('-') ? navigator.language.split('-')[0] : navigator.language);
    }, []);

    return (<>
          <main
            className="flex flex-col h-screen max-h-screen font-sans bg-mainBackground text-text-900 max-sm:hidden"
            onDragOver={() => ({})}
            onDrop={() => ({})}
          >
              <BrowserRouter basename="/">
                  <ModalProvider>
                      <SessionProvider>
                          <Notifications />
                          <Notifications />
                          <section className="flex flex-grow">
                              <Navigation />
                              <section className="flex-grow flex flex-col h-screen overflow-y-scroll">
                                  <Header />
                                  <ErrorBanner />
                                  <Suspense>
                                      <Routes />
                                  </Suspense>
                              </section>
                          </section>
                      </SessionProvider>
                  </ModalProvider>
              </BrowserRouter>
          </main>
          <MobilePlaceholder />
      </>

    );
};

export default App;
