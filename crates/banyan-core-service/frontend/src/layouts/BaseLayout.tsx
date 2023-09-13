import { ReactElement, useEffect, useMemo } from 'react';
import { useRouter } from 'next/router';

import { Header } from '@components/common/Header';
import { Navigation } from '@components/common/Navigation';
import { useKeystore } from '@/contexts/keystore';
import { useModal } from '@/contexts/modals';

export interface IBaseLayout {
    children: ReactElement;
}

const BaseLayout: React.FC<IBaseLayout> = ({ children }) => {
    const router = useRouter();
    const isNavigationVisible = useMemo(() => router.pathname === '/bucket/[id]' || router.pathname === '/' || router.pathname === '/trash', [router.pathname]);
    const { keystoreInitialized, isLoading, escrowedDevice } = useKeystore();
    const { openEscrowModal } = useModal();

    useEffect(() => {
        if (!keystoreInitialized && !isLoading) {
            openEscrowModal(!!escrowedDevice);
        };
    }, [keystoreInitialized, isLoading, escrowedDevice])

    return <main className="flex flex-col h-screen font-sans bg-white">
        <Header />
        <section className="flex flex-grow">
            {isNavigationVisible &&
                <Navigation />
            }
            <div className="flex-grow">
                {children}
            </div>
        </section>
    </main>;
};
export default BaseLayout;
