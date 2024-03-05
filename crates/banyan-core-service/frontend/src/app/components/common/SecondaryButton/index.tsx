import React from 'react';

export const SecondaryButton: React.FC<{ text: string; action?: () => void; disabled?: boolean; className?: string; type?: 'button' | 'reset' | 'submit' | undefined }> =
    ({
        action = () => { },
        className,
        disabled = false,
        text,
    }) => {
        return (
            <button
                disabled={disabled}
                onClick={action}
                className={`btn-secondary py-2 px-4 ${className}`}
            >
                {text}
            </button>
        );
    };
