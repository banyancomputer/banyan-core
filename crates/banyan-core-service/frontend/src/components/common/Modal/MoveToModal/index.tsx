import React from 'react';
import { useIntl } from 'react-intl';
import { FiArrowLeft } from 'react-icons/fi';
import { MdDone } from 'react-icons/md';
import { Select } from '@chakra-ui/react';
import { useModal } from '@/contexts/modals';
import { BucketFile } from '@/lib/interfaces/bucket';
import { ToastNotifications } from '@/utils/toastNotifications';

export const MoveToModal: React.FC<{ file: BucketFile }> = ({ file }) => {
    const { messages } = useIntl();
    const { closeModal } = useModal();

    const move = async() => {
        try {
            ToastNotifications.notify(`${messages.fileWasMoved}`, <MdDone size="20px" />);
        } catch (error: any) { };
    };

    return (
        <div className="w-modal flex flex-col gap-6" >
            <div>
                <button
                    onClick={closeModal}
                    className="mb-3"
                >
                    <FiArrowLeft size="24px" />
                </button>
                <h4 className="text-m font-semibold ">{`${messages.moveTo}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.selectWhereToMove}`}
                </p>
            </div>
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.selectInTheList}`}:</label>
                <Select
                    variant="outline"
                    placeholder={`${messages.selectInTheList}`}
                    className="font-normal text-sm"
                >
                </Select>
            </div>
            <div>
                <label className="inline-block mb-1 text-xs font-normal">{`${messages.folder}`}:</label>
                <Select
                    variant="outline"
                    placeholder={`${messages.selectFolder}`}
                />
            </div>
            <div className="mt-3 flex items-center gap-3 text-xs" >
                <button
                    className="btn-secondary flex-grow py-3 px-4"
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button
                    className="btn-primary flex-grow py-3 px-4"
                    onClick={move}
                >
                    {`${messages.moveTo}`}
                </button>
            </div>
        </div>
    );
};
