import React from 'react';
import { useIntl } from 'react-intl';

import { SubmitButton } from '@components/common/SubmitButton';

import { useModal } from '@app/contexts/modals';
import { Bucket } from '@app/types/bucket';

export const RequestBucketAccessModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { messages } = useIntl();
    const { closeModal } = useModal();

    const requestAccess = async () => {
        try {

            closeModal();
        } catch (error: any) { }
    };

    return (
        <div className="w-modal flex flex-col gap-8" >
            <div>
                <h4 className="text-m font-semibold ">{`${messages.requestAccess}`}</h4>
                <p className="mt-2 text-text-600">
                    {`${messages.requestAccessDescription}`}
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
                    text={`${messages.requestAccess}`}
                    action={requestAccess}
                    className="w-1/2"
                />
            </div>
        </div>
    )
}
