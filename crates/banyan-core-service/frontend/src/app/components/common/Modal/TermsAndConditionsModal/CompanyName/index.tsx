import React, { useState } from 'react';
import { useIntl } from 'react-intl';

import { useModal } from '@app/contexts/modals';

export const CompanyNameModal = () => {
    const { closeModal } = useModal()
    const { messages } = useIntl();
    const [companyName, setCompanyName] = useState('');

    const submit = () => {
        /** TODO: add logic when api will be merged. */
        closeModal();
    };

    return (
        <div className="w-snapshotsModal">
            <h3 className="mb-8 text-m font-semibold">
                {`${messages.isThisAWorkAccount}`}
            </h3>
            <label>
                {`${messages.organizationName}`}
                <input
                    className="mt-2 input w-full h-11 py-3 px-4 rounded-lg border-border-darken focus:outline-none"
                    type="text"
                    placeholder={`${messages.enterName}`}
                    value={companyName}
                    onChange={event => setCompanyName(event.target.value)}
                />
            </label>
            <div className=" mt-8 flex items-center gap-3 text-xs">
                <button
                    className="btn-secondary w-1/2 py-3 px-4"
                    onClick={() => closeModal()}
                >
                    {`${messages.thisIsNotWorkingAccount}`}
                </button>
                <button
                    className="btn-primary w-1/2 py-3 px-4"
                    disabled={!companyName}
                    onClick={submit}
                >
                    {`${messages.done}`}
                </button>
            </div>
        </div>
    )
}
