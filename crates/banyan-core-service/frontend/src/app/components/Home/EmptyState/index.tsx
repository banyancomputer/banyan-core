import { useIntl } from 'react-intl';

import { CreateBucketModal } from '@components/common/Modal/CreateBucketModal';

import { useModal } from '@/app/contexts/modals';

import { ActiveDirectory, PlusBold } from '@/app/static/images/common';

export const EmptyState = () => {
    const { openModal } = useModal();
    const { messages } = useIntl();

    const createDrive = () => {
        openModal(<CreateBucketModal />);
    };

    return (
        <div
            className="mt-12 flex-grow flex flex-col justify-center items-center h-full border-1 border-border-darken border-dashed text-text-900"
        >
            <div className="mb-11">
                <ActiveDirectory width="64px" height="64px" />
            </div>
            <span className="mb-4">
                {`${messages.createYourFirstDrive}`}
            </span>
            <button
                className="btn-secondary flex items-center justify-center gap-2 py-2 px-4 cursor-pointer"
                onClick={createDrive}
            >
                <PlusBold />
                {`${messages.newDrive}`}
            </button>
        </div>
    )
};
