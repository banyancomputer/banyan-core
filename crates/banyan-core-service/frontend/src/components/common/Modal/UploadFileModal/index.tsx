import React, { useState } from 'react';
import { useIntl } from 'react-intl';
import { Select } from '@chakra-ui/react';

import { useTomb } from '@/contexts/tomb';

import { Upload } from '@static/images/buckets';

export const UploadFileModal = () => {
    const { buckets, uploadFile } = useTomb();
    const { messages } = useIntl();
    const [selectedBucket, setSelectedBucket] = useState('');
    const [selectedFolder, setSelectedFolder] = useState('');

    const selectBucket = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setSelectedBucket(event.target.value);
    };

    const selectFolder = (event: React.ChangeEvent<HTMLSelectElement>) => {
        setSelectedFolder(event.target.value);
    };
    console.log(selectedBucket);


    const upload = async(event: React.ChangeEvent<HTMLInputElement>) => {
        if (!event.target.files || !selectedBucket) { return; }

        try {
            const file = Array.from(event.target.files)[0];
            await uploadFile(selectedBucket, '/', file);
        } catch (error: any) {
            console.log(error);
        }
    };

    return (
        <div className="w-modal flex flex-col gap-4">
            <div>
                <h4 className="text-m font-semibold ">{`${messages.uploadFiles}`}</h4>
                <p className="mt-2 text-gray-600">
                    {`${messages.chooseFilesToUpload}`}
                </p>
            </div>
            <div>
                <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectBucket}`}:</span>
                <Select
                    variant="outline"
                    placeholder={`${messages.selectBucket}`}
                    className="font-normal text-sm"
                    onChange={selectBucket}
                >
                    {buckets.map(bucket =>
                        <option
                            key={bucket.id}
                            value={bucket.id}
                        >
                            {bucket.name}
                        </option>
                    )}
                </Select>
            </div>
            <div>
                <span className="inline-block mb-1 text-xs font-normal">{`${messages.selectFolder}`}:</span>
                <Select
                    variant="outline"
                    placeholder={`${messages.selectFolder}`}
                    value={selectedFolder}
                    onChange={selectFolder}
                />
            </div>
            <label className="mt-10 flex flex-col items-center justify-center gap-4 px-6 py-4 border-2 border-c rounded-xl  text-xs cursor-pointer">
                <Upload />
                <span className="text-gray-600">
                    <b className="text-gray-900">{`${messages.clickToUpload}`} </b>
                    {`${messages.orDragAndDrop}`}
                </span>
                <input
                    type="file"
                    className="hidden"
                    onChange={upload}
                />
            </label>
        </div>
    );
};
