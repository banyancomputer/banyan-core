import React, { useEffect, useState } from 'react';

import { AdminClient } from '@/api/admin';
import { NotFoundError } from '@/api/http';
import { HttpClient } from '@/api/http/client';
import { StorageProvider } from '@app/types';
import { ServicesTable } from '@components/common/ServicesTable';

const client = new AdminClient();

const Home = () => {
    const [providers, setProviders] =useState<StorageProvider[]>([])
    useEffect(() => {
        (async () => {
            try {
                const providers =await client.getStorageProviders();
                setProviders(providers)
            } catch (error: any) {
                if (error instanceof NotFoundError) {
                    const api = new HttpClient;
                    await api.get('/auth/logout');
                    window.location.href = '/login';
                }
            }
        })();
    }, [providers]);


    return (
        <section className="py-9 pt-14 px-4" id="buckets">
            {/*{*/}
            {/*}*/}
            <ServicesTable/>
        </section>
    );
};

export default Home;
