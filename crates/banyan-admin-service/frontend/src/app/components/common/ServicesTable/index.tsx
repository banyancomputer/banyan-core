import React, { useEffect, useState } from 'react';

class Storage {
    constructor(
        public service: string,
        public serviceCost: string = '$24/mo',
        public dataQty: string = '1 TB',
        public cost: string = '$24/mo'
    ) { }
}

export const ServicesTable = () => {

    const MOCK_DATA = [
        new Storage("coldStorage"),
        new Storage("hotStorage"),
        new Storage("downloads"),
        new Storage("egress"),
        new Storage("migrationFees"),
    ];

    return (
        <div className="max-h-[calc(100vh-290px)] overflow-x-auto border-1 border-border-regular rounded-xl" >
            <table className="table table-pin-rows w-full text-text-600 rounded-xl ">
                <thead className="border-b-table-cellBackground text-xxs font-normal text-text-600">
                    <tr className="border-b-table-cellBackground bg-table-headBackground">
                        <th className="p-3 whitespace-break-spaces text-left font-medium">
                            {"service"}
                        </th>
                        <th className="p-3 text-left font-medium whitespace-pre">
                            {"serviceCost"}
                        </th>
                        <th className="p-3 text-left font-medium">
                            {"dataQty"}
                        </th>
                        <th className="p-3 w-32 text-left font-medium">
                            {"cost"}
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {MOCK_DATA.map(storage =>
                        <tr>
                            <td className="px-3 py-6 border-t-1 border-r-1 border-border-regular">{storage.service}</td>
                            <td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">{storage.serviceCost}</td>
                            <td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">{storage.dataQty}</td>
                            <td className="px-3 py-6 border-t-1 border-l-1 border-border-regular">{storage.cost}</td>
                        </tr>
                    )}
                </tbody>
            </table >
        </div>
    );
};

