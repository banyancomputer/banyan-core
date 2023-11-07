
import { Suspense, useEffect, useState } from 'react';
import { BrowserRouter, useLocation } from 'react-router-dom';

import { Modal } from '@components/common/Modal';
import { Notifications } from '@components/common/Notifications';
import { Routes } from './routes';
import { KeystoreProvider, useKeystore } from './contexts/keystore';
import { ModalProvider } from './contexts/modals';
import { FilePreviewProvider } from './contexts/filesPreview';
import { IntlProvider } from 'react-intl';
import { FilePreview } from '@components/common/FilePreview';
import { FileUploadProvider } from './contexts/filesUpload';
import { TombProvider } from './contexts/tomb';
import { Navigation } from '@components/common/Navigation';
import { Header } from '@components/common/Header';

import en from '@static/locales/en.json';
import fr from '@static/locales/fr.json';
import de from '@static/locales/de.json';
import ja from '@static/locales/ja.json';
import zh from '@static/locales/zh.json';
import { getLocalStorageItem, setLocalStorageItem } from './utils/localStorage';
import { SessionProvider } from './contexts/session';

const TRANSLATES: Record<string, Record<string, string>> = {
	en,
	fr,
	de,
	ja,
	zh,
};

export const locales = Object.keys(TRANSLATES);

const App = () => {
	// const { openEscrowModal } = useModal();
	const { keystoreInitialized, isLoading, escrowedKeyMaterial } = useKeystore();
	const [locale, setLocale] = useState('en');

	useEffect(() => {
		window.addEventListener('storage', () => {
			const selectedLanguage = getLocalStorageItem('lang');
			setLocale(selectedLanguage || 'en');
		});

		const selectedLanguage = getLocalStorageItem('lang');
		setLocale(selectedLanguage || 'en');

		if (selectedLanguage) { return; }

		setLocalStorageItem('lang', navigator.language.includes('-') ? navigator.language.split('-')[0] : navigator.language);
	}, []);

	return (
		<main className="flex flex-col h-screen font-sans bg-mainBackground text-text-900">
			<BrowserRouter basename="/" >
				<SessionProvider>
					<KeystoreProvider>
						<ModalProvider>
							<TombProvider>
								<FileUploadProvider>
									<FilePreviewProvider>
										<IntlProvider locale={locale} messages={TRANSLATES[locale]}>
											<Notifications />
											<Modal />
											<FilePreview />
											<Modal />
											<Notifications />
											<section className="flex flex-grow">
												<Navigation />
												<section className="flex-grow h-screen overflow-y-scroll">
													<Header />
													<Suspense>
														<Routes />
													</Suspense>
												</section>
											</section>
										</IntlProvider>
									</FilePreviewProvider>
								</FileUploadProvider>
							</TombProvider>
						</ModalProvider>
					</KeystoreProvider>
				</SessionProvider>
			</BrowserRouter>
		</main>
	);
};

export default App;
