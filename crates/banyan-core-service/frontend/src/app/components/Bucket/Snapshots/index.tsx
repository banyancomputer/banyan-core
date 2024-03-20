import React, { useEffect } from 'react';
import { Link } from 'react-router-dom';

import { useTomb } from '@app/contexts/tomb';
import { ArrowDown } from '@app/static/images/common';
import { SnapshotsTable } from './Table';
import { useAppSelector } from '@app/store';

const Snapshots = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.bucket.snapshots);
    const { selectedBucket } = useTomb();

    return (
        <div className="flex flex-col gap-3 flex-grow">
            <div className="flex items-center gap-3 py-4 px-3">
                <Link to={`/drive/${selectedBucket?.id}`} className="rotate-90">
                    <ArrowDown width="20px" height="20px" />
                </Link>
                <div className="flex flex-col">
                    <div className="flex gap-1 text-xs">
                        <span className="font-semibold">{selectedBucket?.name}</span>
                        /
                        <span className="">{messages.title}</span>
                    </div>
                </div>
            </div>
            <SnapshotsTable />
        </div>
    )
};

export default Snapshots;
