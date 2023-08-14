import { useTomb } from '@/contexts/tomb';
import { Select } from '@chakra-ui/react'
import { Upload } from '@static/images/buckets';
import React, { useState } from 'react'

export const UploadFileModal = () => {
    const { buckets } = useTomb()
    const [selectedBucket, setSelectedBucket] = useState('');
    const [selectedFolder, setSelectedFolder] = useState('');

    const selectBucket = (id: string) => {
        setSelectedBucket(id);
    }

    const uploadFile = (event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files) return;

        const files = Array.from(event.target.files);
    };


    return (
        <div className='w-uploadFileModal flex flex-col gap-4'>
            <div>
                <h4 className='text-m font-semibold '>Upload files</h4>
                <p className='mt-2 text-gray-600'>
                    Choose files for uploading from your device, or use drag & drop
                </p>
            </div>
            <div>
                <span className='inline-block mb-1 text-xs font-normal'>Select bucket:</span>
                <Select variant='outline' placeholder='Select Bucket' className='font-normal text-sm' >
                    {buckets.map(bucket =>
                        <option onClick={() => selectBucket(bucket.id)}>{bucket.name}</option>
                    )}
                </Select>
            </div>
            <div>
                <span className='inline-block mb-1 text-xs font-normal'>Select folder:</span>
                <Select
                    variant='outline'
                    placeholder='Select folder'
                    value={selectedBucket}
                />
            </div>
            <label className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer">
                <Upload />
                <span className="text-gray-600">
                    <span className="font-semibold text-black">Click to upload </span>
                    or drag and drop
                </span>
                <input
                    type="file"
                    className="hidden"
                    onChange={uploadFile}
                />
            </label>
        </div>
    )
}
