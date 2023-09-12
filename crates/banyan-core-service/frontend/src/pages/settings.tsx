import React, { useEffect } from 'react';

import getServerSideProps from '@/utils/session';
import { NextPageWithLayout } from './page';
import { IEscrowPage } from './escrow';
import { useModal } from '@/contexts/modals';
import { useKeystore } from '@/contexts/keystore';

export { getServerSideProps };

const Settings: NextPageWithLayout<IEscrowPage> = ({ escrowedDevice }) => {
    const { openEscrowModal, closeModal } = useModal();
    const { keystoreInitialized, isLoading } = useKeystore();

    useEffect(() => {
        if (!keystoreInitialized && !isLoading) {
            openEscrowModal(!!escrowedDevice);
        } else {
            closeModal();
        };
    }, [keystoreInitialized, isLoading]);

    return (
        <div>Settings</div>
    )
}


export default Settings;
