import React from 'react';
import { useIntl } from 'react-intl';

import { Mail, Question } from '@static/images/common'

export const HelpControls = () => {
    const { messages } = useIntl();
    return (
        <div
            className="absolute right-0 top-10  flex flex-col items-stretch shadow-xl rounded-xl overflow-hidden text-xs font-semibold overflow-hiddenaa bg-bucket-actionsBackground cursor-pointer text-bucket-actionsText"
        >
            <a
                className="flex items-center gap-2 py-2.5 px-3 transition-all whitespace-nowrap hover:bg-hover"
                href="https://banyan8674.zendesk.com/hc/en-us"
                target="_blank"
            >
                <span className="text-button-primary">
                    <Question />
                </span>
                FAQ
            </a>
            <a
                href="mailto:support@banyan8674.zendesk.com"
                className="flex items-center gap-2 py-2.5 px-3 transition-all whitespace-nowrap hover:bg-hover"
                target="_blank"
            >
                <span className="text-button-primary">
                    <Mail />
                </span>
                {`${messages.contactUs}`}
            </a>
        </div>
    )
}
