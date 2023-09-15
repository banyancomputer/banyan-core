import { useTomb } from '@/contexts/tomb';
import { convertFileSize } from '@/utils/storage';
import React, { useState } from 'react'
import { IoMdClose } from 'react-icons/io';
import { useIntl } from 'react-intl';

export const StorageUsage: React.FC<{ canBeClosed?: boolean }> = ({ canBeClosed = false }) => {
    const { usedStorage, usageLimit } = useTomb();
    const [isVisible, setIsVisible] = useState(true);
    const { messages } = useIntl();

    const toggleStorageVisibility = () => {
        setIsVisible(prev => !prev);
    };

    return (
        <div className={`w-full bg-white rounded-lg p-4 ${isVisible ? '' : 'hidden'}`}>
            <span className="flex justify-between items-center font-semibold">
                {`${messages.storage}`}
                {canBeClosed &&
                    <button onClick={toggleStorageVisibility}>
                        <IoMdClose size="20px" />
                    </button>
                }
            </span>
            <span className="text-xs font-normal">{` ${messages.youHaveUsed} `}
                <span className="uppercase">{convertFileSize(usedStorage)}</span>
                {` ${messages.outOf} `}
                <span className="uppercase">{convertFileSize(usageLimit)}</span>.
            </span>
            <progress className="progress w-full" value={usedStorage} max={usageLimit}></progress>
        </div>
    )
}
