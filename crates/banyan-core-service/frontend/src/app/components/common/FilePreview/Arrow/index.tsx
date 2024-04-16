import React from 'react';

import { ArrowDown, ChevronUp } from '@static/images/common';

export const PreviewArrow: React.FC<{ action: () => void; isVisible: boolean; className?: string }> = ({ action, isVisible, className }) =>
    <>
        {isVisible ?
            <button
                onClick={event => {
                    event.stopPropagation();
                    action()
                }}
                className={`absolute top-1/2 -translate-y-1/2 p-3 rounded-full bg-gray-900 text-white z-40 transition-all ${className}`}
            >
                <ArrowDown width="24px" height="24px" />
            </button>
            : null
        }
    </>;

