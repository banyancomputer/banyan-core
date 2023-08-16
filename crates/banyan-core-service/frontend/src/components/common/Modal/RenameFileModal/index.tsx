import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { useModal } from '@/contexts/modals';
import { BucketFile } from '@/lib/interfaces/bucket'

export const RenameFileModal: React.FC<{ file: BucketFile }> = ({ file }) => {
    const { closeModal } = useModal();
    const { messages } = useIntl();
    const [newName, setNewName] = useState('');

    return (
        <div className='w-modal flex flex-col gap-8' >
            <div>
                <h4 className='text-m font-semibold '>{`${messages.renameFile}`}</h4>
            </div>
            <div>
                <label>
                    {`${messages.fileName}`}
                    <input
                        className='mt-2 input w-full h-11 py-3 px-4 rounded-lg border-gray-300 shadow-sm focus:outline-none'
                        type="text"
                        placeholder={`${messages.enterNewBucketName}`}
                        value={newName}
                        onChange={event => setNewName(event.target.value)}
                    />
                </label>
            </div>
            <div className='mt-3 flex items-center gap-3 text-xs' >
                <button
                    className='btn-secondary flex-grow py-3 px-4'
                    onClick={closeModal}
                >
                    {`${messages.cancel}`}
                </button>
                <button className='btn-primary flex-grow py-3 px-4'>{`${messages.save}`}</button>
            </div>
        </div >
    )
}