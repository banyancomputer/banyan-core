import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';

import { KeyActions } from '@components/KeyManagement/KeyActions';
import { Bucket } from '@/lib/interfaces/bucket';
import { ActionsCell } from '@/components/common/ActionsCell';
import { fingerprintEcPem } from '@/lib/crypto/utils';
import Bucket from '@/pages/bucket/[id]';
import { prettyFingerprintApiKeyPem } from '@/utils/fingerprint';

export const KeyManagementTable: React.FC<{ buckets: Bucket[] }> = ({ buckets }) => {
    const { messages } = useIntl();
    const tableRef = useRef<HTMLDivElement | null>(null);
    const [tableScroll, setTableScroll] = useState(0);
    const [fingerprints, setFingerprints] = useState(new Map([]));

    useEffect(() => {
        /** Weird typescript issue with scrollTop which exist, but not for typescript */
        // @ts-ignore
        const listener = (event: Event) => setTableScroll(event.target!.scrollTop || 0);
        tableRef.current?.addEventListener('scroll', listener);

        return () => tableRef.current?.removeEventListener('scroll', listener);
    }, [tableRef]);

    useEffect(() => {
        async function getFingerprints() {
            let fingerprintMap = new Map([]);
            for (const bucket of buckets) {
                for (const index in bucket.keys) {
                    const key = bucket.keys[index];
                    const pem = key.pem();
                    const id = key.id();
                    const fingerprint = await prettyFingerprintApiKeyPem(pem);
                    fingerprintMap.set(id, fingerprint);
                }
            };
            setFingerprints(fingerprintMap);
        }

        if (fingerprints.size == 0) {
            getFingerprints();
        }
    }, []);

    return (
        <div
            ref={tableRef}
            className="max-h-[calc(100vh-320px)] overflow-x-auto border-2 border-border-regular bg-secondaryBackground rounded-xl"
        >
            <table className="table table-pin-rows w-full text-text-600 rounded-xl">
                <thead className="border-b-reg text-xxs font-normal text-text-600 border-b-2 border-border-regular">
                    <tr className="border-b-table-cellBackground bg-table-headBackground border-none">
                        <th className="py-3 px-6 w-44 whitespace-break-spaces text-left font-medium">{`${messages.locationForKey}`}</th>
                        <th className="py-3 px-6 text-left font-medium whitespace-pre">
                            {`${messages.client}`}
                        </th>
                        <th className="py-3 px-6 text-left font-medium">
                            {`${messages.fingerprint}`}
                        </th>
                        <th className="py-3 px-6 w-32 text-left font-medium">
                            {`${messages.status}`}
                        </th>
                        <th className="w-16"></th>
                    </tr>
                </thead>
                <tbody>
                    {buckets.map(bucket => {
                        return (<React.Fragment key={bucket.id}>
                            <tr className="bg-table-cellBackground text-gray-900 border-b-2 border-y-border-regular ">
                                <td className="px-6 py-4">{bucket.name}</td>
                                <td className="px-6 py-4"></td>
                                <td className="px-6 py-4"></td>
                                <td className="px-6 py-4"></td>
                                <td className="px-6 py-4"></td>
                            </tr>
                            {
                                bucket?.keys?.map(bucketKey => {
                                    const approved = bucketKey.approved();
                                    const bucket_key_id = bucketKey.id();
                                    const fingerprint = fingerprints.get(bucket_key_id);

                                    return <tr key={bucket_key_id}>
                                        <td className="px-6 py-4"></td>
                                        <td className="px-6 py-4">{bucket_key_id}</td>
                                        <td className="px-6 py-4">{fingerprint}</td>
                                        <td className="px-6 py-4">{approved ? `${messages.approved}` : `${messages.noAccess}`}</td>
                                        <td className="px-6 py-4">
                                            <ActionsCell
                                                actions={<KeyActions bucket={bucket} bucketKey={bucketKey} />}
                                                offsetTop={tableScroll}
                                                tableRef={tableRef}
                                            />
                                        </td>
                                    </tr>
                                })
                            }
                        </React.Fragment>);
                    }

                    )}
                </tbody>
            </table >
        </div>
    );
};

