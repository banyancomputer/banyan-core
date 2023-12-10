import React from 'react';
import { useIntl } from 'react-intl';

import { SubmitButton } from '@components/common/SubmitButton';

import { useModal } from '@/app/contexts/modals';
import { useTomb } from '@/app/contexts/tomb';
import { BucketKey } from '@/app/types/bucket';

export const RemoveBucketAccessModal: React.FC<{ bucketKey: BucketKey }> = ({ bucketKey }) => {
    const { messages } = useIntl();
    const { removeBucketAccess } = useTomb();
    const { closeModal } = useModal();

    const removeAccess = async () => {
        try {
            await removeBucketAccess(bucketKey.id);
            closeModal();
        } catch (error: any) { }
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.removeAccess}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.removeAccessDescription}`}
                </p>
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary w-1/2 py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <SubmitButton
                    text={`${messages.removeAccess}`}
                    action={removeAccess}
                    className="w-1/2"
                />
            </div>
        </div>
    );
};
