import React from 'react';
import { useIntl } from 'react-intl';
import { FiArrowLeft } from "react-icons/fi"
import { useModal } from '@/contexts/modals';
import { Select } from '@chakra-ui/react';

export const MoveToModal = () => {
    const { messages } = useIntl();
    const { closeModal } = useModal();

    return (
        <div className='w-modal flex flex-col gap-6' >
            <div>
                <button
                    onClick={closeModal}
                    className='mb-3'
                >
                    <FiArrowLeft size="24px" />
                </button>
                <h4 className='text-m font-semibold '>{`${messages.moveTo}`}</h4>
                <p className='mt-2 text-gray-600'>
                    {`${messages.selectWhereToMove}`}
                </p>
            </div>
            <div>
                <label className='inline-block mb-1 text-xs font-normal'>{`${messages.selectInTheList}`}:</label>
                <Select
                    variant='outline'
                    placeholder={`${messages.selectInTheList}`}
                    className='font-normal text-sm'
                >
                </Select>
            </div>
            <div>
                <label className='inline-block mb-1 text-xs font-normal'>{`${messages.folder}`}:</label>
                <Select
                    variant='outline'
                    placeholder={`${messages.selectFolder}`}
                />
            </div>
        </div>
    )
}
