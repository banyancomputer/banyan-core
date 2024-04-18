import { Decline, Download } from '@static/images';
import React from 'react'

export const ActiveDealsActions = () => {
    return (
        <div className="flex items-center gap-6">
            <div className="cursor-pointer"><Download /></div>
            <div className="cursor-pointer"><Decline /></div>
        </div>
    )
};
