import React, { useEffect } from 'react';

export const PrimaryButton: React.FC<{ text: string; action?: () => void; disabled?: boolean; className?: string; type?: 'button' | 'reset' | 'submit' | undefined }> =
    ({
        action = () => { },
        className,
        disabled = false,
        text,
        type = 'submit',
    }) => {

        useEffect(() => {
            const listener = async (event: KeyboardEvent) => {
                if (event.key !== 'Enter' || disabled) { return; }
                action();
            };

            window.addEventListener('keypress', listener);

            return () => {
                window.removeEventListener('keypress', listener);
            };
        });

        return (
            <button
                type={type}
                disabled={disabled}
                onClick={action}
                className={`btn-primary py-2 px-4 ${className}`}
            >
                {text}
            </button>
        );
    };
