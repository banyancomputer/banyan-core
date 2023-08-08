import { ReactElement, useEffect } from 'react';
import { useSession } from 'next-auth/react';
import { useRouter } from 'next/router';

import { Header } from '@components/common/Header';
import { Navigation } from '@components/common/Navigation';

export interface IBaseLayout {
    children: ReactElement;
}

const BaseLayout: React.FC<IBaseLayout> = ({ children }) => {
    const { data, status } = useSession();
    const router = useRouter();

    // Redirect to login page if user is not logged in
    useEffect(() => {
        if (!data && status !== String('loading')) {
            router.push('/login').then(() => window.scrollTo(0, 0));
        }
    }, [data, router]);

    return <main className="flex flex-col h-screen font-sans">
        <Header />
        <section className="flex flex-grow">
            {router.pathname !== '/key-management' &&
                <Navigation />
            }
            <div className="flex-grow">
                {children}
            </div>
        </section>
    </main>;
};
export default BaseLayout;
