
import { useIntl } from 'react-intl';

import { NextPageWithLayout } from '@/pages/page';
import { useTomb } from '@/contexts/tomb';
import { KeyManagementTable } from '@/components/KeyManagement/KeyManagementTable';
import { Fallback } from '@/components/common/Fallback';

import getServerSideProps from '@/utils/session';
import SettingsLayout from '@/layouts/SettingsLayout';

export { getServerSideProps };

const ManageKeys: NextPageWithLayout = () => {
    const { buckets, areBucketsLoading } = useTomb();
    const { messages } = useIntl();

    return (
        <div className="flex flex-col gap-5 px-4">
            <h2 className="text-lg font-semibold">
                {`${messages.manageKeyAccess}`}
            </h2>
            <Fallback shouldRender={!areBucketsLoading}>
                <KeyManagementTable buckets={buckets} />
            </Fallback>
        </div>
    );
};

export default ManageKeys;

ManageKeys.getLayout = (page) => <SettingsLayout>{page}</SettingsLayout>;
