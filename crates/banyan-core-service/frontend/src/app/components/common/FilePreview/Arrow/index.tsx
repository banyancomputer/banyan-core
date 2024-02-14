import React from 'react';

import { ArrowDown, ChevronUp } from '@static/images/common';

export const PreviewArrow: React.FC<{ action: () => void; isVisible: boolean; className?: string }> = ({ action, isVisible, className }) =>
    <>
        {isVisible ?
            <button
                onClick={action}
                className={`rounded-full text-white z-40 transition-all ${className}`}
            >
                <ArrowDown width="24px" height="24px" />
            </button>
            : null
        }
    </>;

