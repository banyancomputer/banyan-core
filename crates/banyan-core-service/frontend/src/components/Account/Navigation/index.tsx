import React from 'react';
import { useIntl } from 'react-intl';
import { useRouter } from 'next/router';
import Link from 'next/link';

class SettingsLink {
    constructor(
        public label: string,
        public path: string
    ) { }
};

export const AccountNavigation = () => {
    const { messages } = useIntl();
    const { pathname } = useRouter();

    const links = [
        // new SettingsLink(`${messages.profile}`, '/account/profile'),
        new SettingsLink(`${messages.appSettings}`, '/account/settings'),
        new SettingsLink(`${messages.manageKeys}`, '/account/manage-keys'),
        new SettingsLink(`${messages.billingAndPayments}`, '/account/billing'),
        // new SettingsLink(`${messages.services}`, '/account/services'),
    ];

    return (
        <section className="py-5 px-4" id="buckets">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {`${messages.account}`}
                </h2>
            </div>
            <ul className="flex justify-between p-1.5 rounded-lg bg-gray-200">
                {links.map(link =>
                    <li className="flex-grow" key={link.label}>
                        <Link
                            className={`flex justify-center w-full py-3 rounded-lg text-xs transition-all ${pathname == link.path && 'bg-secondaryBackground'} `}
                            href={link.path}
                        >
                            {link.label}
                        </Link>
                    </li>
                )}
            </ul>
        </section>
    );
};

