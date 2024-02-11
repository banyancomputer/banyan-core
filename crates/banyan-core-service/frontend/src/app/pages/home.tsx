import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';

import { UploadFileModal } from '@components/common/Modal/UploadFileModal';
import { Fallback } from '@components/common/Fallback';
import { Bucket } from '@components/Home/Bucket';
import { CreateBucketModal } from '@components/common/Modal/CreateBucketModal';

import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';
import { ToastNotifications } from '../utils/toastNotifications';

import { PlusBold, Upload } from '@static/images/common';

const Home = () => {
    const { openModal } = useModal();
    const { buckets, areBucketsLoading, getBucketsFiles, tomb } = useTomb();
    const { messages } = useIntl();

    const uploadFile = () => {
        openModal(<UploadFileModal path={[]} />);
    };

    const createDrive = () => {
        openModal(<CreateBucketModal />);
    }

    useEffect(() => {
        if (!tomb) { return; }

        const getFiles = async () => {
            try {
                await getBucketsFiles();
            } catch (error: any) {
                ToastNotifications.error('Error on files loading', 'Try again', getFiles)
            };
        };

        getFiles();
    }, [buckets.length, tomb]);

    return (
        <section className="py-9 pt-14 px-4" id="buckets">
            <div className="mb-4 flex flex-col w-full justify-between gap-4">
                <h2 className="text-lg font-semibold">
                    {`${messages.allDrives}`}
                </h2>
                <div className="flex items-stretch gap-2">
                    <button
                        className="btn-highlighted gap-2 w-[138px] py-2 px-4 text-sm"
                        onClick={uploadFile}
                    >
                        <Upload />
                        {`${messages.upload}`}
                    </button>
                    <button
                        className="flex items-center gap-2 py-2 px-4 border-1 border-border-regular rounded-md text-text-900 font-semibold"
                        onClick={createDrive}
                    >
                        <PlusBold width="20px" height="20px" />
                        {`${messages.newDrive}`}
                    </button>
                </div>
            </div>
            <Fallback shouldRender={!areBucketsLoading}>
                <div className="grid grid-cols-3 gap-3 xl:grid-cols-4 ">
                    {
                        buckets.map(bucket =>
                            <Bucket bucket={bucket} key={bucket.id} />
                        )
                    }
                </div>
            </Fallback>
        </section>
    );
};

export default Home;
