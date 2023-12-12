import React from 'react';

import { ChevronUp } from '@static/images/common';

export const PreviewArrow: React.FC<{ action: () => void, isVisible: boolean, className?: string }> = ({ action, isVisible, className }) => {
    return (
        <>
            {isVisible ?
                <button
                    onClick={action}
                    className={`fixed top-1/2 -translate-y-1/2 p-4 rounded-full bg-black text-white z-40 transition-all hover:bg-gray-800 ${className}`}
                >
                    <ChevronUp width="40px" height="40px" />
                </button>
                : null
            }
        </>
    )
}
