import React from 'react';

import { useAppSelector } from '@store/index';

import { Mail, Question } from '@static/images/common'

export const HelpControls = () => {
    const messages = useAppSelector(state => state.locales.messages.coponents.common.header.helpControls);

    return (
        <div
            className="absolute right-0 top-10  flex flex-col items-stretch shadow-xl rounded-md overflow-hidden text-xs font-semibold overflow-hiddenaa bg-bucket-actionsBackground cursor-pointer text-bucket-actionsText"
        >
            <a
                className="flex items-center gap-2 py-2.5 px-3 transition-all whitespace-nowrap hover:bg-hover"
                href="https://banyan8674.zendesk.com/hc/en-us/sections/19265371237517-FAQ"
                target="_blank"
            >
                <Question />
                FAQ
            </a>
            <a
                href="mailto:support@banyan8674.zendesk.com"
                className="flex items-center gap-2 py-2.5 px-3 transition-all whitespace-nowrap hover:bg-hover"
                target="_blank"
            >
                <Mail />
                {`${messages.contactUs}`}
            </a>
        </div>
    )
}
