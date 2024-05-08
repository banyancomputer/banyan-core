import { Accept, Decline, Download } from '@static/images';
import React from 'react'
import { useAppDispatch } from '@app/store';
import { acceptDeal } from '@app/store/deals/actions';

export const REJECTED_DEALS_LOCAL_STORAGE_KEY = 'rejectedDeals';

interface AvailiableDealsActionsProps {
    dealId: string;
    onDealAccepted: () => Promise<void>;
}
export const ActiveDealsActions: React.FC<AvailiableDealsActionsProps> = ({ dealId, onDealAccepted }) => {
    const dispatch = useAppDispatch();

    const handleAccept = async () => {
        await dispatch(acceptDeal(dealId));
        await onDealAccepted();
    };

    const handleReject = async () => {
        let rejectedDeals = JSON.parse(localStorage.getItem(REJECTED_DEALS_LOCAL_STORAGE_KEY) || '[]');
        if (!rejectedDeals.includes(dealId)) {
            rejectedDeals.push(dealId);
            localStorage.setItem('rejectedDeals', JSON.stringify(rejectedDeals));
        }
        await onDealAccepted();
    };

    return (
      <div className="flex items-center gap-6">
          <div className="cursor-pointer" onClick={handleAccept}><Accept /></div>
          <div className="cursor-pointer" onClick={handleReject}><Decline /></div>
      </div>
    )
};
