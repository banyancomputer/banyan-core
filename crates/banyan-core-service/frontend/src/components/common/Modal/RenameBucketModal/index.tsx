import { useModal } from '@/contexts/modals';
import { Bucket } from '@/lib/interfaces/bucket'
import React from 'react'

export const RenameBucketModal: React.FC<{ bucket: Bucket }> = ({ bucket }) => {
    const { closeModal } = useModal();

    return <div></div>
}