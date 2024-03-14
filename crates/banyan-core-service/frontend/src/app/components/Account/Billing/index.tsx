
import { NextBillingDate } from './NextBillingDate';
import { Invoices } from './Invoices';

export const Billing = () => {
    return (
        <div className="flex flex-col gap-10 p-6">
            <div className='flex items-stretch gap-2'>
                <NextBillingDate />
            </div>
            <Invoices />
        </div>
    );
};

export default Billing;
