import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';

import { UploadFileModal } from '@/app/components/common/Modal/UploadFileModal';
import { Fallback } from '@/app/components/common/Fallback';
import { Bucket } from '@components/Home/Bucket';

import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';

import { EmptyIcon, PlusBold, Upload } from '@static/images/common';
import { CreateBucketModal } from '@components/common/Modal/CreateBucketModal';

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

        (async () => {
            await getBucketsFiles();
        })();
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
                {buckets.length ?
                    <div className="grid grid-cols-3 gap-3">
                        {
                            buckets.map(bucket =>
                                <Bucket bucket={bucket} key={bucket.id} />
                            )
                        }
                    </div>
                    :
                    <div className="h-full flex flex-col items-center justify-center saturate-0">
                        <EmptyIcon />
                        <p className="mt-4">{`${messages.noDrives}`}</p>
                    </div>
                }
            </Fallback>
        </section>
    );
};

export default Home;
