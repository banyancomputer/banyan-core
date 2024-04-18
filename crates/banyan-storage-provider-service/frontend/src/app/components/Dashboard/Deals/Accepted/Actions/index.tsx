import { Decline, Download } from '@static/images';
import React from 'react'
import { useAppDispatch } from '@app/store';
import { rejectDeal } from '@app/store/deals/actions';

interface ActiveDealsActionsProps {
    dealId: string;
    onDealAccepted: () => Promise<void>;
}
export const ActiveDealsActions: React.FC<ActiveDealsActionsProps> = ({ dealId, onDealAccepted }) => {
    const dispatch = useAppDispatch();

    const handleDecline = async () => {
        await dispatch(rejectDeal(dealId));
        await onDealAccepted();
    };

    return (
        <div className="flex items-center gap-6">
            <div className="cursor-pointer" onClick={handleDecline}><Decline /></div>
        </div>
    )
};
