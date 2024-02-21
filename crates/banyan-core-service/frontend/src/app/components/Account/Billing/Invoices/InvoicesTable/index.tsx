import React from 'react';

import { InvoiceDetails } from '../Invoice';

import { Invoice } from '@/entities/billing';
import { useAppDispatch, useAppSelector } from '@/app/store';
import { getDateLabel } from '@/app/utils/date';
import { selectInvoice } from '@/app/store/billing/slice';

export const InvoicesTable: React.FC<{ invoices: Invoice[] }> = ({ invoices }) => {
    const messages = useAppSelector(state => state.locales.messages.coponents.account.billing.invoices.invoicesTable);
    const dispatch = useAppDispatch();
    const { selectedInvoice } = useAppSelector(state => state.billing);

    const viewInvoice = (invoice: Invoice) => {
        dispatch(selectInvoice(invoice));
    };

    return (
        <>
            <table className="table table-fixed invoices-table border-1 border-border-regular">
                <thead>
                    <tr className="border-none  bg-gray-100">
                        <th className="p-3 text-text-600 font-medium text-xs">{messages.date}</th>
                        <th className="p-3 text-text-600 font-medium text-xs">{messages.status}</th>
                        <th className="p-3 text-text-600 font-medium text-xs">{messages.totalCost}</th>
                        <th className="p-3 w-[160px] text-text-600 font-medium text-xs">{messages.details}</th>
                    </tr>
                </thead>
                <tbody>
                    {invoices.map(invoice =>
                        <tr className="border-b-2 border-border-regular" >
                            <td className="px-3 py-4 text-text-600 text-xs">
                                {getDateLabel(new Date(invoice.created_at).getTime() / 1000)}
                            </td>
                            <td className="px-3 py-4 text-text-600 text-xs">
                                {invoice.status}
                            </td>
                            <td className="px-3 py-4 text-text-800 font-semibold text-sm">
                                ${(invoice.total_amount / 100).toFixed(2)}
                            </td>
                            <td className="px-3 py-4 text-text-600 text-xs" onClick={() => viewInvoice(invoice)}>
                                <div className="flex items-center justify-start font-semibold text-xs text-text-viewInvoiceText cursor-pointer">
                                    {messages.viewInvoice}
                                </div>
                            </td>
                        </tr>
                    )}
                </tbody>
            </table>
            {selectedInvoice && <InvoiceDetails invoice={selectedInvoice} />}
        </>
    )
}
