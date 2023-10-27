import { HealthStatus } from '@static/images';
import { StorageUsage } from './StorageUsage';
import { BandwidthUsage } from './BandwidthUsage';
import { CurrentlyUsedStorage } from './CurrentlyUsedStorage';

export const Charts = () => {

    return (
        <section>
            <h3 className='mb-6 text-42 text-darkText'>Storage provider #134159</h3>
            <p className='mb-12 flex items-center gap-3'><HealthStatus />  Health Status</p>
            <div className='flex items-center gap-6'>
                <StorageUsage />
                <BandwidthUsage />
            </div>
            <CurrentlyUsedStorage />
        </section>
    )
}
