import { ReactElement, useEffect, useMemo } from 'react';
import { useRouter } from 'next/router';

import { Header } from '@components/common/Header';
import { Navigation } from '@components/common/Navigation';
import { ErrorBanner } from '@/components/common/ErrorBanner';

import { useKeystore } from '@/contexts/keystore';
import { useModal } from '@/contexts/modals';

export interface IBaseLayout {
    children: ReactElement;
}

const BaseLayout: React.FC<IBaseLayout> = ({ children }) => {
    const router = useRouter();
    const { keystoreInitialized, isLoading, escrowedDevice } = useKeystore();
    const { openEscrowModal } = useModal();

    useEffect(() => {
        if (!keystoreInitialized && !isLoading) {
            openEscrowModal(!!escrowedDevice);
        };
    }, [keystoreInitialized, isLoading, escrowedDevice]);

    return <main className="flex flex-col h-screen font-sans bg-mainBackground text-text-900">
        <section className="flex flex-grow">
            <Navigation />
            <div className="flex-grow">
                <Header />
                <ErrorBanner />
                {children}
            </div>
        </section>
    </main>;
};
export default BaseLayout;
