import { StorageUsage } from './StorageUsage';
import { BandwidthUsage } from './BandwidthUsage';

export const Charts = () => {

    return (
        <section>
            <div className='flex items-center gap-6'>
                <StorageUsage />
                <BandwidthUsage />
            </div>
        </section>
    )
}
