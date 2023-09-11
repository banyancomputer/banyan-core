import { useEffect, useState } from 'react';
import { signIn, useSession } from 'next-auth/react';
import { FcGoogle } from 'react-icons/fc';

import Router from 'next/router';
import { NextPageWithLayout } from '@/pages/page';
import { useKeystore } from '@/contexts/keystore';
import { Logo } from '@static/images/common';
import { useIntl } from 'react-intl';

const Login: NextPageWithLayout = () => {
    const { data } = useSession();
    const [error, setError] = useState('');
    const { keystoreInitialized } = useKeystore();
    const { messages } = useIntl();

    // Redirect to home page if user is logged in
    useEffect(() => {
        if (data && keystoreInitialized) {
            Router.push('/').then(() => window.scrollTo(0, 0));
        } else if (data) {
            Router.push('/escrow').then(() => window.scrollTo(0, 0));
        }
    }, [data, keystoreInitialized]);

    const handleLoginWithProvider = (provider: any) => async () => {
        try {
            await signIn(provider, { redirect: false, });
        } catch (err: any) {
            setError(err.message);
        }
    };

    return (
        <div className='flex items-stretch h-screen bg-login'>
            <div className="w-2/3 flex items-center justify-center bg-navigation-primary svg:w-44">
                <Logo width="426px" height="88px" />
            </div>
            <div className="w-1/3 min-w-login flex items-center justify-center">
                <div className='flex flex-col justify-cente'>
                    <h3 className='mb-12 text-3xl font-medium'>Log in</h3>
                    <div
                        className='w-80 flex items-center justify-center bg-white text-sm font-medium py-select px-3 rounded-lg cursor-pointer'
                        onClick={handleLoginWithProvider('google')}
                    >
                        <span className='flex items-center justify-center gap-2'>
                            <FcGoogle size="24px" />
                            {`${messages.loginWithGoogle}`}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default Login;
