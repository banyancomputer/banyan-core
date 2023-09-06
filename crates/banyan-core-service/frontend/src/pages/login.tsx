import { useEffect, useState } from 'react';
import { signIn, useSession } from 'next-auth/react';
import { Button } from '@chakra-ui/react';

import Router from 'next/router';
import { NextPageWithLayout } from '@/pages/page';
import { useKeystore } from '@/contexts/keystore';

const Login: NextPageWithLayout = () => {
    const { data } = useSession();
    const [error, setError] = useState('');
    const { keystoreInitialized } = useKeystore();

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
            await signIn(provider);
        } catch (err: any) {
            setError(err.message);
        }
    };

    return (
        <div>
            <div className="text-6xl font-semibold align-left mb-2">Log in</div>
            {error &&
                // Error when login fails
                <label htmlFor="registration" className="label">
                    <span className="text-xxs !p-0 text-error text-left">
                        There was an issue logging you in. Please try again.
                    </span>
                </label>
            }
            <div className="flex items-center mt-4">
                <Button
                    colorScheme="blue"
                    variant="solid"
                    ml={4}
                    w={40}
                    onClick={handleLoginWithProvider('google')}
                >
                    Log in with Google
                </Button>
            </div>
        </div>
    );
};

export default Login;
