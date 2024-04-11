import React from 'react';

import { Logo } from '@static/images/common';

export const AuthLayout: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    return (
        <section className="flex items-stretch h-screen">
            <div className="w-1/2 flex items-center justify-center bg-navigation-primary text-text-900">
                <Logo />
            </div>
            <div className="w-1/2 flex items-center justify-center bg-mainBackground">
                {children}
            </div>
        </section>
    )
};
