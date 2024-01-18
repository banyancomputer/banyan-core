import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { Invoice } from '..';
import { InvoiceDetails } from '../Invoice';

export const InvoicesTable: React.FC<{ billingHistory: Invoice[] }> = ({ billingHistory }) => {
    const { messages } = useIntl();
    const [selectedInvoice, setSelectedInvoice] = useState<Invoice | null>(null);

    return (
        <>
            <table className="table invoices-table border-1 border-border-regular">
                <thead>
                    <tr className="border-none  bg-gray-100">
                        <th className="p-3 text-text-600 font-medium text-xs">{`${messages.date}`}</th>
                        <th className="p-3 text-text-600 font-medium text-xs">{`${messages.storageUsed}`}</th>
                        <th className="p-3 text-text-600 font-medium text-xs">{`${messages.description}`}</th>
                        <th className="p-3 text-text-600 font-medium text-xs">{`${messages.totalCost}`}</th>
                        <th className="p-3 text-text-600 font-medium text-xs">{`${messages.details}`}</th>
                    </tr>
                </thead>
                <tbody>
                    {billingHistory.map(billing =>
                        <tr className="border-b-2 border-border-regular" >
                            <td className="px-3 py-4 text-text-600 text-xs">
                                {billing.date}
                            </td>
                            <td className="px-3 py-4 text-text-600 text-xs">
                                {billing.storageUsed}
                            </td>
                            <td className="px-3 py-4 text-text-600 text-xs">
                                {billing.description}
                            </td>
                            <td className="px-3 py-4 text-text-800 font-semibold text-sm">
                                {billing.price}
                            </td>
                            <td className="px-3 py-4 text-text-600 text-xs" onClick={() => setSelectedInvoice(billing)}>
                                <div className="flex items-center justify-start font-semibold text-xs text-text-viewInvoiceText cursor-pointer">
                                    {`${messages.viewInvoice}`}
                                </div>
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
            {selectedInvoice && <InvoiceDetails invoice={selectedInvoice} setSelectedInvoice={setSelectedInvoice} />}
        </>
    )
}
