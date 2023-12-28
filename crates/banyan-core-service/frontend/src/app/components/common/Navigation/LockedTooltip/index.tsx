import React from 'react';
import { useIntl } from 'react-intl';

import { RequestBucketAccessModal } from '@components/common/Modal/RequestBucketAccessModal';

import { Bucket } from '@app/types/bucket';
import { useModal } from '@app/contexts/modals';

import { Lock } from '@static/images/buckets';

export const LockedTooltip: React.FC<{ bucket: Bucket, className?: string, size?: string }> = ({ bucket, className, size }) => {
    const { messages } = useIntl();
    const { openModal } = useModal();
    const stopPopagation = (event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
        event.stopPropagation();
        event.preventDefault();
    };

    const requestAccess = () => {
        openModal(<RequestBucketAccessModal bucket={bucket} />);
    };

    return (
        <div className={`absolute  group text-xxs text-bucket-actionsText ${className}`}>
            <Lock width={size || '20px'} height={size || '20px'} />
            <div
                className="absolute top-5 left-0 hidden flex-col px-3 py-1 border-2 rounded-lg border-border-regular bg-bucket-actionsBackground cursor-default whitespace-nowrap group-hover:flex"
                onClick={stopPopagation}
            >
                {`${messages.youHaveNoAccess};`}
                <span
                    className="font-semibold underline text-button-primary cursor-pointer"
                    onClick={requestAccess}
                >
                    {`${messages.requestAccess} ${messages.here}`}
                </span>
            </div>
        </div>
    );
};
