import React, { useEffect, useState } from 'react';
import Papa from 'papaparse';

export const SpreadsheetViewer: React.FC<{ data: File }> = ({ data }) => {
    const [spreadSheetData, setSpreadSheetData] = useState<Papa.ParseResult<string[]> | null>(null);
    useEffect(() => {
        Papa.parse(data, {
            complete(results: Papa.ParseResult<string[]>) {
                setSpreadSheetData({ ...results, data: results.data.filter(row => row.join('')) });
            },
        });
    }, [data]);

    return (
        <div
            className="h-full w-filePreview max-w-filePreview p-8 rounded-xl bg-white"
            onClick={event => event.stopPropagation()}
        >
            <div className="h-full overflow-x-auto border-2 border-border-regular rounded-xl">
                <table className="table table-pin-rows table-pin-cols text-text-900">
                    <thead>
                        <tr className="bg-white">
                            <th className="border-1 border-border-regular bg-white text-text-900"></th>
                            {spreadSheetData?.data[0]?.map((header: string, index: number) => (
                                <th
                                    key={index}
                                    className="border-1 border-border-regular bg-white text-text-900"
                                >
                                    {header}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {spreadSheetData?.data.slice(1)?.map((rowData, index) =>
                            <tr
                                key={index}
                                className=" border-none"
                            >
                                <th>{index + 1}</th>
                                {rowData?.map((data: string, index: number) =>
                                    <td
                                        key={index}
                                        className="border-1 border-border-regular bg-white"
                                    >
                                        {data}
                                    </td>
                                )}
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    )
};
