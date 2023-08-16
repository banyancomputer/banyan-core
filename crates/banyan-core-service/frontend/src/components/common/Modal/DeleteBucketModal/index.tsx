import React from 'react';
import { useIntl } from 'react-intl';
import { FiTrash2 } from "react-icons/fi"

import { Bucket } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';

export const DeleteBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
  const { closeModal } = useModal();
  const { messages } = useIntl();

  return (
    <div className='w-uploadFileModal flex flex-col gap-5'>
      <FiTrash2 size="24px" stroke='#5e6c97' />
      <div>
        <h4 className='text-m font-semibold'>{`${messages.deleteBucket}`}</h4>
        <p className='mt-2 text-gray-600'>
          {`${messages.wantToEmpty}`} <b className='text-gray-900'>{bucket.name}</b>? {`${messages.filesWillBeDeletedPermanently}`}.
        </p>
      </div>
      <div className='mt-3 flex items-center gap-3 text-xs' >
        <button
          className='btn-secondary flex-grow py-3 px-4'
          onClick={closeModal}
        >
          {`${messages.cancel}`}
        </button>
        <button className='btn-primary flex-grow py-3 px-4'>{`${messages.delete}`}</button>
      </div>
    </div>
  )
}
