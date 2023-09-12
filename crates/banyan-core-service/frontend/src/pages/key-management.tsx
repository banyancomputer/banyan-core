
import { useEffect } from 'react';
import BaseLayout from '@layouts/BaseLayout';
import { useIntl } from 'react-intl';

import { NextPageWithLayout } from '@/pages/page';
import { useTomb } from '@/contexts/tomb';
import { KeyManagementTable } from '@/components/KeyManagement/KeyManagementTable';
import { Fallback } from '@/components/common/Fallback';

import getServerSideProps from '@/utils/session';
import { IEscrowPage } from './escrow';
import { useKeystore } from '@/contexts/keystore';
import { useModal } from '@/contexts/modals';

export { getServerSideProps };

const HomePage: NextPageWithLayout<IEscrowPage> = ({ escrowedDevice }) => {
    const { buckets, areBucketsLoading } = useTomb();
    const { keystoreInitialized } = useKeystore();
    const { messages } = useIntl();
    const { openEscrowModal } = useModal();

    useEffect(() => {
        if (keystoreInitialized) return;
        openEscrowModal(!!escrowedDevice);
    }, [keystoreInitialized]);

    return (
        <div className="flex flex-col gap-6 px-4 py-keyManagement w-keyManagement mx-auto">
            <h2 className="text-xl font-semibold">
                {`${messages.manageKeyAccess}`}
            </h2>
            <Fallback shouldRender={!areBucketsLoading}>
                <KeyManagementTable buckets={buckets} />
            </Fallback>
        </div>
    );
};

export default HomePage;

HomePage.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
