import { useEffect, useState } from 'react';
import { signIn, useSession } from 'next-auth/react';
import { FcGoogle } from 'react-icons/fc';
import { useSearchParams } from 'next/navigation';
import { useIntl } from 'react-intl';
import Router from 'next/router';
import { destroyCookie } from 'nookies';

import { NextPageWithLayout } from '@/pages/page';

import { Logo } from '@static/images/common';

const Login: NextPageWithLayout = () => {
    const { data } = useSession();
    const [errorMessage, setErrorMessage] = useState('');
    const { messages } = useIntl();
    const searchParams = useSearchParams();
    const error = searchParams.get('error');

    const handleLoginWithProvider = (provider: any) => async () => {
        try {
            destroyCookie(null, 'banyan-key-cookie', { path: '/' });
            await signIn(provider, { redirect: false });
        } catch (err: any) {
            setErrorMessage(err.message);
        }
    };

    // Redirect to home page if user is logged in
    useEffect(() => {
        if (data) {
            Router.push('/').then(() => window.scrollTo(0, 0));
        }
    }, [data]);

    useEffect(() => {
        if (!error) { return; };

        setErrorMessage(`${messages[error]}` || '');
    }, [error]);

    return (
        <div className="flex items-stretch h-screen bg-login text-text-900">
            <div className="w-2/3 flex items-center justify-center bg-navigation-primary text-navigation-text">
                <Logo width="426px" height="88px" />
            </div>
            <div className="w-1/3 min-w-login flex items-center justify-center">
                <div className="flex flex-col justify-cente">
                    <h3 className="mb-12 text-3xl font-medium">Log in</h3>
                    <button
                        className="w-80 flex items-center justify-center bg-secondaryBackground text-sm font-medium py-2.5 px-3 rounded-lg cursor-pointer"
                        onClick={handleLoginWithProvider('google')}
                    >
                        <span className="flex items-center justify-center gap-2">
                            <FcGoogle size="24px" />
                            {`${messages.loginWithGoogle}`}
                        </span>
                    </button>
                    {errorMessage &&
                        <span className="w-80 mt-3 text-error text-xxs font-medium">{errorMessage}</span>
                    }
                </div>
            </div>
        </div>
    );
};

export default Login;
