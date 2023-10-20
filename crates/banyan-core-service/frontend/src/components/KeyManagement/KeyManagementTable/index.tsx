import React, { useEffect, useRef, useState } from 'react';
import { useIntl } from 'react-intl';

import { KeyActions } from '@components/KeyManagement/KeyActions';
import { Bucket } from '@/lib/interfaces/bucket';
import { ActionsCell } from '@/components/common/ActionsCell';
import { fingerprintEcPem } from '@/lib/crypto/utils';

export const KeyManagementTable: React.FC<{ buckets: Bucket[] }> = ({ buckets }) => {
    const { messages } = useIntl();
    const tableRef = useRef<HTMLDivElement | null>(null);
    const [tableScroll, setTableScroll] = useState(0);

    useEffect(() => {
        /** Weird typescript issue with scrollTop which exist, but not for typescript */
        //@ts-ignore
        const listener = (event: Event) => setTableScroll(event.target!.scrollTop || 0);
        tableRef.current?.addEventListener("scroll", listener);

        return () => tableRef.current?.removeEventListener('scroll', listener);
    }, [tableRef]);

    return (
        <div
            ref={tableRef}
            className="max-h-[calc(100vh-320px)] overflow-x-auto border-2 border-border-regular bg-secondaryBackground rounded-xl"
        >
            <table className="table table-pin-rows w-full text-text-600 rounded-xl">
                <thead className="border-b-table-cellBackground text-xxs font-normal text-text-600">
                    <tr className="border-b-table-cellBackground bg-table-headBackground border-none">
                        <th className="py-3 px-6 w-44 whitespace-break-spaces text-left font-medium">{`${messages.locationForKey}`}</th>
                        <th className="py-3 px-6 text-left font-medium whitespace-pre">
                            {/* {`${messages.client}`} */
                                `bucket_key_id`
                            }
                        </th>
                        <th className="py-3 px-6 text-left font-medium">
                            {/* {`${messages.fingerprint}`} */
                                `PEM`
                            }
                        </th>
                        <th className="py-3 px-6 w-32 text-left font-medium">
                            {`${messages.status}`}
                        </th>
                        <th className="w-16"></th>
                    </tr>
                </thead>
                <tbody>
                    {buckets.map(bucket => {
                        // var bucket_id = bucket.id();

                        return (<React.Fragment key={bucket.id}>
                        <tr className="bg-table-cellBackground text-gray-900">
                            <td className="px-6 py-4">{bucket.name}</td>
                            <td className="px-6 py-4"></td>
                            <td className="px-6 py-4"></td>
                            <td className="px-6 py-4"></td>
                            <td className="px-6 py-4"></td>
                        </tr>
                        {
                            bucket?.keys?.map(bucketKey => {
                                var pem = bucketKey.pem();
                                var approved = bucketKey.approved();
                                var bucket_key_id = bucketKey.id();

                                return <tr key={bucket_key_id}>
                                <td className="px-6 py-4"></td>
                                <td className="px-6 py-4">{bucket_key_id}</td>
                                <td className="px-6 py-4">{pem}</td>
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

