import { useState } from 'react';

import { GoogleIcon, Logo } from '@static/images';

const Login = () => {
    const [errorMessage, setErrorMessage] = useState('');

    const handleLoginWithProvider = (provider: any) => async () => {
        try {
        } catch (err: any) {
            setErrorMessage(err.message);
        }
    };

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
                            <GoogleIcon />
                            Continue with Google
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
