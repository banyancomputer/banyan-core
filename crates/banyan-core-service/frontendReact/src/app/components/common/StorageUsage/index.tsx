import React from 'react';
import { useIntl } from 'react-intl';

import { convertFileSize } from '@/app/utils/storage';
import { useTomb } from '@/app/contexts/tomb';

export const StorageUsage = () => {
    const { usedStorage, usageLimit } = useTomb();
    const { messages } = useIntl();

    return (
        <div className="w-full bg-mainBackground rounded-lg p-4">
            <span className="flex justify-between items-center font-semibold">
                {`${messages.storage}`}
            </span>
            <span className="text-xs font-normal">{` ${messages.youHaveUsed} `}
                <span className="uppercase">{convertFileSize(usedStorage)}</span>
                {` ${messages.outOf} `}
                <span className="uppercase">{convertFileSize(usageLimit)}</span>.
            </span>
            <progress className="progress w-full" value={usedStorage} max={usageLimit}></progress>
        </div>
    );
};
