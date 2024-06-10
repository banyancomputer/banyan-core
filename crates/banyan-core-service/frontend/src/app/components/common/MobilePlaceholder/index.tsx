import React from 'react';

import { useAppSelector } from '@store/index';

import { Folders, Logo } from '@static/images/common'

export const MobilePlaceholder = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.mobilePlaceholder);

    return (
        <section className="hidden w-screen h-screen flex-col items-stretch text-[#57221E] max-sm:flex">
            <div className="flex justify-center items-center px-16 py-8 bg-termsAndConditions-header">
                <Logo />
            </div>
            <div className="flex-grow flex flex-col items-center justify-center bg-white">
                <Folders />
                <div className="mt-6 flex flex-col items-center text-text-900">
                    <h4 className="mb-3 font-semibold text-[24px]">{`${messages.title}`}</h4>
                    <p>{`${messages.subtitle}`}</p>
                </div>
            </div>
        </section>
    )
}
