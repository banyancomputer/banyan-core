import React from 'react';
import { useIntl } from 'react-intl';
import { Switch } from '@chakra-ui/react';
import { NextPageWithLayout } from '../page';
import SettingsLayout from '@/layouts/SettingsLayout';
import { LanguageSelect } from '@/components/common/LanguageSelect';

export const Billing: NextPageWithLayout = () => {
    const { messages } = useIntl();

    return (
        <div className="flex flex-col gap-5 px-4">
            <h2 className="text-lg font-semibold">
                {`${messages.settings}`}
            </h2>
            <div className="flex justify-between items-center py-5 px-4 border-1 rounded-lg text-gray-800 border-gray-200">
                <div>
                    <h5 className="font-semibold">{`${messages.theme}`}</h5>
                    <p>{`${messages.selectTheme}`}</p>
                </div>
                <Switch />
            </div>
            <div className="flex justify-between items-center py-5 px-4 border-1 rounded-lg text-gray-800 border-gray-200">
                <div>
                    <h5 className="font-semibold">{`${messages.language}`}</h5>
                    <p>{`${messages.chooseLanguage}`}</p>
                </div>
                <LanguageSelect />
            </div>
        </div>
    );
};

export default Billing;

Billing.getLayout = (page) => <SettingsLayout>{page}</SettingsLayout>;
