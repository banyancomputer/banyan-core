import React from 'react';
import { useIntl } from 'react-intl';
import { Link, useLocation } from 'react-router-dom';

class SettingsLink {
    constructor(
        public label: string,
        public path: string
    ) { }
};

export const AccountNavigation = () => {
    const { messages } = useIntl();
    const { pathname } = useLocation();

    const links = [
        // new SettingsLink(`${messages.profile}`, '/account/profile'),
        new SettingsLink(`${messages.appSettings}`, '/account/settings'),
        new SettingsLink(`${messages.manageKeys}`, '/account/manage-keys'),
        new SettingsLink(`${messages.billingAndPayments}`, '/account/billing'),
        // new SettingsLink(`${messages.services}`, '/account/services'),
    ];

    return (
        <section className="py-5 px-10" id="buckets">
            <div className="mb-4 flex w-full justify-between items-center">
                <h2 className="text-xl font-semibold">
                    {`${messages.account}`}
                </h2>
            </div>
            <div className="border-b-1 border-border-regular">
                <ul className="w-max flex justify-between rounded-md bg-secondaryBackground">
                    {links.map(link =>
                        <li className="flex-grow" key={link.label}>
                            <Link
                                className={`relative flex justify-center w-full py-2 px-6 rounded-md text-xs transition-all ${pathname == link.path && 'font-semibold'} `}
                                to={link.path}
                            >
                                {link.label}
                                {pathname == link.path &&
                                    <span className="absolute bottom-0 left-0 w-full h-[2px] bg-text-900" />
                                }
                            </Link>
                        </li>
                    )}
                </ul>
            </div>
        </section>
    );
};

