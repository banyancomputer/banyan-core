import React from 'react';
import { useIntl } from 'react-intl';

import { Invoice } from '..';

import { Close } from '@/app/static/images/common';

export const InvoiceDetails: React.FC<{ invoice: Invoice, setSelectedInvoice: React.Dispatch<React.SetStateAction<Invoice | null>> }> = ({ invoice, setSelectedInvoice }) => {
  const { messages } = useIntl();

  return (
    <div className="fixed top-0 left-0 w-screen h-screen bg-[#0A0B0C80] z-20" onClick={() => setSelectedInvoice(null)}>
      <div className="absolute right-0 top-0 w-modal h-full bg-mainBackground">
        <div className="flex flex-col gap-2 p-4 font-semibold">
          <button><Close /></button>
          <div>{`${messages.invoice}`}</div>
          <div>{invoice.date}</div>
        </div>
        <div className="px-4 py-2.5 bg-invoiceHeadingBackground text-text-600 font-medium">{`${messages.summary}`}</div>
        <div className="py-2 px-4 flex flex-col gap-4">
        </div>
        <div className="px-4 py-2.5 bg-invoiceHeadingBackground text-text-600 font-medium">{`${messages.items}`}</div>
        <div className="py-2 px-4 flex flex-col gap-4">
        </div>
        <div className="px-4 py-2.5 bg-invoiceHeadingBackground text-text-600 font-medium">{`${messages.payment}`}</div>
        <div className="py-2 px-4 flex flex-col gap-4">
        </div>
      </div>
    </div>
  )
}
