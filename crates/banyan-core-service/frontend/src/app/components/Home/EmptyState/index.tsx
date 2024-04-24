import { CreateDriveModal } from '@components/common/Modal/CreateDriveModal';

import { openModal } from '@store/modals/slice';

import { ActiveDirectory, PlusBold } from '@/app/static/images/common';
import { useAppDispatch, useAppSelector } from '@/app/store';

export const EmptyState = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.home.emptyState);
    const dispatch = useAppDispatch();

    const createDrive = () => {
        dispatch(openModal({ content: <CreateDriveModal /> }));
    };

    return (
        <div
            className="mt-12 flex-grow flex flex-col justify-center items-center h-full border-1 border-border-darken border-dashed text-text-900"
        >
            <div className="mb-11">
                <ActiveDirectory width="64px" height="64px" />
            </div>
            <span className="mb-4">
                {`${messages.title}`}
            </span>
            <button
                className="btn-secondary flex items-center justify-center gap-2 py-2 px-4 cursor-pointer"
                onClick={createDrive}
            >
                <PlusBold />
                {`${messages.newDriveButton}`}
            </button>
        </div>
    )
};
