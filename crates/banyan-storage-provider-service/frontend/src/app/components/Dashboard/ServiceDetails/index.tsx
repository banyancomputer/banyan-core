import { useAppDispatch } from '@app/store';
import { convertFileSize } from '@app/utils/storage';

class MockProvider {
    constructor(
        public name: string = 'Provider 1',
        public usedStorage: number = 1000000000,
        public availiableStorage: number = 1000000000,
        public billingPeriod: string = '05 Oct 2023',
    ) { }
}

export const ServiceDetails = () => {
    const dispatch = useAppDispatch();

    const providers = [
        new MockProvider(),
        new MockProvider(),
        new MockProvider(),
        new MockProvider(),
        new MockProvider(),
    ];

    return (
        <section>
            <h2 className='mt-20 mb-6 text-28 text-darkText font-bold'>Service Details</h2>
            <div className='max-h-table overflow-y-scroll pr-2'>
                <table className="table w-full">
                    <thead className="bg-[#FFF8EB]">
                        <tr>
                            <td className="p-3 text-12">Storage Provider Identity</td>
                            <td className="p-3 text-12">Currently used storage</td>
                            <td className="p-3 text-12">Available/ allocated storage</td>
                            <td className="p-3 text-12">Billing period</td>
                        </tr>
                    </thead>
                    <tbody>
                        {providers.map((provider, index) =>
                            <tr className={`border-b-1 border-[#DDD] ${index % 2 ? '' : 'bg-[#F5F5F5]'}`}>
                                <td className="py-6 px-3 text-14">{provider.name}</td>
                                <td className="py-6 px-3 text-14">{convertFileSize(provider.usedStorage)}</td>
                                <td className="py-6 px-3 text-14">{convertFileSize(provider.availiableStorage)}</td>
                                <td className="py-6 px-3 text-14">{provider.billingPeriod}</td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </section >
    );
};
