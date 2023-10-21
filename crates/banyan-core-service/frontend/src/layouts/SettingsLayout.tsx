import { ReactElement, useEffect, useMemo } from 'react';

import { Header } from '@components/common/Header';
import { Navigation } from '@components/common/Navigation';
import { useKeystore } from '@/contexts/keystore';
import { useModal } from '@/contexts/modals';
import { AccountNavigation } from '@/components/Account/Navigation';

export interface IBaseLayout {
    children: ReactElement;
}

const SettingsLayout: React.FC<IBaseLayout> = ({ children }) => {
    const { keystoreInitialized, isLoading, escrowedDevice } = useKeystore();
    const { openEscrowModal } = useModal();

    useEffect(() => {
        if (!keystoreInitialized && !isLoading) {
            openEscrowModal(!!escrowedDevice);
        };
    }, [keystoreInitialized, isLoading, escrowedDevice]);

    return <main className="flex flex-col min-h-screen font-sans bg-mainBackground text-text-900">
        <section className="flex flex-grow">
            <Navigation />
            <div className="flex-grow">
                <Header />
                <AccountNavigation />
                {children}
            </div>
        </section>
    </main>;
};
export default SettingsLayout;
