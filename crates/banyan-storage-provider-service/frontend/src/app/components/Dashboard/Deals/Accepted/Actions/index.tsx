import { Decline, Download } from '@static/images';
import React from 'react'
import { useAppDispatch } from '@app/store';
import { cancelDeal, sealDeal } from '@app/store/deals/actions';

interface ActiveDealsActionsProps {
    dealId: string;
    onDealCancelled: () => Promise<void>;
}
export const ActiveDealsActions: React.FC<ActiveDealsActionsProps> = ({ dealId, onDealCancelled }) => {
    const dispatch = useAppDispatch();

    const handleDecline = async () => {
        await dispatch(cancelDeal(dealId));
        await onDealCancelled();
    };

    const handleSeal = async () => {
        await dispatch(sealDeal(dealId));
    };

    return (
      <div className="flex items-center gap-6">
          <div className="cursor-pointer" onClick={handleSeal}><Download /></div>
          {/*<div className="cursor-pointer" onClick={handleSeal}><Decline /></div>*/}
      </div>
    )
};
