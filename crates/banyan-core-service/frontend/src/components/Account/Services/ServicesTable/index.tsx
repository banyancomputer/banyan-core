import React, { useEffect, useState } from 'react';
import { useIntl } from 'react-intl';

class Storage {
    constructor(
        public service: string,
        public serviceCost: string = '$24/mo',
        public dataQty: string = '1 TB',
        public cost: string = '$24/mo'
    ) { }
}

export const ServicesTable = () => {
    const { messages } = useIntl();

    const MOCK_DATA = [
        new Storage(`${messages.coldStorage}`),
        new Storage(`${messages.hotStorage}`),
        new Storage(`${messages.downloads}`),
        new Storage(`${messages.egress}`),
        new Storage(`${messages.migrationFees}`),
    ]

    return (
        <div className="max-h-[calc(100vh-290px)] overflow-x-auto border-1 border-border rounded-xl" >
            <table className="table table-pin-rows w-full text-gray-600 rounded-xl ">
                <thead className="border-b-table-cellBackground text-xxs font-normal text-gray-600">
                    <tr className="border-b-table-cellBackground bg-table-headBackground">
                        <th className="p-3 whitespace-break-spaces text-left font-medium">
                            {`${messages.service}`}
                        </th>
                        <th className="p-3 text-left font-medium whitespace-pre">
                            {`${messages.serviceCost}`}
                        </th>
                        <th className="p-3 text-left font-medium">
                            {`${messages.dataQty}`}
                        </th>
                        <th className="p-3 w-32 text-left font-medium">
                            {`${messages.cost}`}
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {MOCK_DATA.map(storage =>
                        <tr>
                            <td className='px-3 py-6 border-t-1 border-r-1 border-border'>{storage.service}</td>
                            <td className='px-3 py-6 border-t-1 border-x-1 border-border'>{storage.serviceCost}</td>
                            <td className='px-3 py-6 border-t-1 border-x-1 border-border'>{storage.dataQty}</td>
                            <td className='px-3 py-6 border-t-1 border-l-1 border-border'>{storage.cost}</td>
                        </tr>
                    )}
                </tbody>
            </table >
        </div>
    );
};

