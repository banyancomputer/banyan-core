import React from 'react';

import { Folders, Logo } from '@static/images/common'

export const MobilePlaceholder = () => {

    return (
        <section className="hidden w-screen h-screen flex-col items-stretch text-[#57221E] max-sm:flex">
            <div className="flex justify-center items-center px-16 py-8 bg-termsAndConditions-header">
                <Logo width="195px" height="40px" />
            </div>
            <div className="flex-grow flex flex-col items-center justify-center bg-white">
                <Folders />
                <div className="mt-6 flex flex-col items-center text-text-900">
                    <h4 className="mb-3 font-semibold text-[24px]">{}</h4>
                </div>
            </div>
        </section>
    )
}
