import { Accept, Decline, Download } from '@static/images';
import React from 'react'
import { useAppDispatch } from '@app/store';
import { acceptDeal } from '@app/store/deals/actions';


interface AvailiableDealsActionsProps {
    dealId: string;
    onDealAccepted: () => Promise<void>;
}
export const AvailableDealsActions: React.FC<AvailiableDealsActionsProps> = ({ dealId, onDealAccepted }) => {
    const dispatch = useAppDispatch();

    const handleAccept = async () => {
        await dispatch(acceptDeal(dealId));
        await onDealAccepted();
    };

    return (
        <div className="flex items-center gap-6">
            <div className="cursor-pointer" onClick={handleAccept}><Accept /></div>
        </div>
    )
};
