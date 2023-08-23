
import BaseLayout from '@layouts/BaseLayout';
import { NextPageWithLayout } from '@/pages/page';
import { useTomb } from '@/contexts/tomb';
import { useIntl } from 'react-intl';
import { KeyManagementTable } from '@/components/KeyManagement/KeyManagementTable';

const HomePage: NextPageWithLayout = () => {
    const { buckets } = useTomb();
    const { messages } = useIntl();

    return (
        <div className="flex flex-col gap-6 px-4 py-keyManagement w-keyManagement mx-auto">
            <h2 className="text-xl font-semibold">
                {`${messages.manageKeyAccess}`}
            </h2>
            <KeyManagementTable buckets={buckets} />
        </div>
    );
};

export default HomePage;

HomePage.getLayout = (page) => <BaseLayout>{page}</BaseLayout>;
