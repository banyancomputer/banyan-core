import React, { useEffect } from 'react';
import { useIntl } from 'react-intl';

import { UploadFileModal } from '@/app/components/common/Modal/UploadFileModal';
import { Fallback } from '@/app/components/common/Fallback';
import { Bucket } from '@/app/components/Buckets/Bucket';

import { useTomb } from '@/app/contexts/tomb';
import { useModal } from '@/app/contexts/modals';

//@ts-ignore
import emptyIcon from '@static/images/common/emptyIcon.png';
import { PlusBold } from '@static/images/common';

const Buckets = () => {
    const { openModal } = useModal();
    const { buckets, areBucketsLoading, getBucketsFiles, tomb } = useTomb();
    const { messages } = useIntl();

    const uploadFile = () => {
        openModal(<UploadFileModal path={[]} />);
    };

    useEffect(() => {
        if (!tomb) return;

        (async () => {
            await getBucketsFiles();
        })();
    }, [buckets.length, tomb]);

    return (
        <section className="py-9 px-4" id="buckets">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {`${messages.myDrives}`}
                </h2>
                <button
                    className="btn-highlighted gap-2 w-40 py-2 px-4"
                    onClick={uploadFile}
                >
                    <PlusBold />
                    {`${messages.upload}`}
                </button>
            </div>
            <Fallback shouldRender={!areBucketsLoading}>
                {buckets.length ?
                    <div className='grid grid-cols-3 gap-3'>
                        {
                            buckets.map(bucket =>
                                <Bucket bucket={bucket} key={bucket.id} />
                            )
                        }
                    </div>
                    :
                    <div className="h-full flex flex-col items-center justify-center saturate-0">
                        <img src={emptyIcon} alt="emptyIcon" />
                        <p className="mt-4">{`${messages.noDrives}`}</p>
                    </div>
                }
            </Fallback>
        </section>
    );
};

export default Buckets;
