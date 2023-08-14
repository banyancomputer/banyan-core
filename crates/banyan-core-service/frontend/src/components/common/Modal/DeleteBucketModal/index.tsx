import React from 'react';

import { Trash } from '@static/images/common';
import { Bucket } from '@/lib/interfaces/bucket';
import { useModal } from '@/contexts/modals';

export const DeleteBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
  const { closeModal } = useModal();

  return (
    <div className='w-uploadFileModal flex flex-col gap-5'>
      <Trash />
      <div>
        <h4 className='text-m font-semibold '>Delete bucket</h4>
        <p className='mt-2 text-gray-600'>
          Are you sure you want to empty <b className='text-gray-900'>Bucket ABC</b>? Files will be deleted permanently.
        </p>
      </div>
      <div className='mt-3 flex items-center gap-3 text-xs' >
        <button
          className='btn-secondary flex-grow py-3 px-4'
          onClick={closeModal}
        >
          Cancel
        </button>
        <button className='btn-primary flex-grow py-3 px-4'>Delete</button>
      </div>
    </div>
  )
}
