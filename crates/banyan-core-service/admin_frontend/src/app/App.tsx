import { Suspense, useEffect } from 'react';
import { BrowserRouter } from 'react-router-dom';

import { Notifications } from '@components/common/Notifications';
import { Navigation } from '@components/common/Navigation';
import { Header } from '@components/common/Header';
import { ErrorBanner } from '@components/common/ErrorBanner';

import { Routes } from './routes';
import { ModalProvider } from './contexts/modals';
import { getLocalStorageItem } from './utils/localStorage';
import { SessionProvider } from './contexts/session';
import { Modal } from '@components/common/Modal';

const App = () => {
	useEffect(() => {
		const theme = getLocalStorageItem('theme');
		theme &&
			document.documentElement.setAttribute('prefers-color-scheme', theme);
	}, []);

	return (
		<main
			className="flex flex-col h-screen max-h-screen font-sans bg-mainBackground text-text-900 max-sm:hidden"
			onDragOver={() => ({})}
			onDrop={() => ({})}
		>
			<BrowserRouter basename="/">
				<ModalProvider>
					<SessionProvider>
						<Modal />
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
	);
};

export default App;
