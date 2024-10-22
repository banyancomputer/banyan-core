import React from 'react';

import { Invoice } from '@/entities/billing';
import { getDateLabel } from '@/app/utils/date';
import { useAppDispatch, useAppSelector } from '@store/index';
import { convertSubscriptionsSizes } from '@/app/utils/storage';
import { selectInvoice } from '@store/billing/slice';
import { getHotStorageAmount } from '@/app/utils/subscritions';

import { Close } from '@/app/static/images/common';

export const InvoiceDetails: React.FC<{ invoice: Invoice }> = ({ invoice }) => {
  const messages = useAppSelector(state => state.locales.messages.coponents.account.billing.invoices.invoice);
  const dispatch = useAppDispatch();
  const { displayName } = useAppSelector(state => state.session.user)
  const { subscriptions, selectedSubscription } = useAppSelector(state => state.billing);
  const subscription = subscriptions.find(subscription => subscription.pricing?.plan_base === invoice.total_amount / 100) || selectedSubscription;

  const close = () => {
    dispatch(selectInvoice(null));
  };

  return (
    <div className="fixed top-0 left-0 w-screen h-screen bg-[#0A0B0C80] z-20" onClick={close}>
      <div
        onClick={event => event.stopPropagation()}
        className="absolute right-0 top-0 w-modal h-full bg-mainBackground"
      >
        <div className="flex flex-col gap-2 p-4 font-semibold">
          <button
            onClick={close}
          >
            <Close />
          </button>
          <div>{messages.title}</div>
          <div>{getDateLabel(new Date(invoice.created_at).getTime() / 1000)}</div>
        </div>
        <div className="px-4 py-2.5 bg-invoiceHeadingBackground text-text-600 font-medium">{messages.summary}</div>
        <div className="py-2 px-4 flex flex-col gap-4">
          <div className="flex items-center justify-between w-full">
            <span className="font-medium">To</span>
            <span className="font-normal">{displayName}</span>
          </div>
          <div className="flex items-center justify-between w-full">
            <span className="font-medium">From</span>
            <span className="font-normal">Banyan</span>
          </div>
          <div className="flex items-center justify-between w-full">
            <span className="font-medium">Subscribed Plan</span>
            <span className="font-normal">{subscription?.title}</span>
          </div>
        </div>
        <div className="px-4 py-2.5 bg-invoiceHeadingBackground text-text-600 font-medium">{messages.items}</div>
        <div className="py-2 px-4 flex flex-col gap-4">
          <div className="flex items-center justify-between w-full">
            <span className="font-medium">{messages.driveSrorage}</span>
            <span className="font-normal">{convertSubscriptionsSizes(getHotStorageAmount(subscription))}</span>
          </div>
          <div className="flex items-center justify-between w-full">
            <span className="font-medium">{messages.dataEggress}</span>
            <span className="font-normal">{convertSubscriptionsSizes(subscription!.features.included_bandwidth)}</span>
          </div>
        </div>
        <div className="px-4 py-2.5 bg-invoiceHeadingBackground text-text-600 font-medium">{messages.payment}</div>
        <div className="py-2 px-4 flex flex-col gap-4">
          <div className="flex justify-between">
            <span className="font-medium">{messages.total}</span>
            <span className="font-normal">${subscription?.pricing?.plan_base.toFixed(2)}</span>
          </div>
        </div>
      </div>
    </div>
  )
}
