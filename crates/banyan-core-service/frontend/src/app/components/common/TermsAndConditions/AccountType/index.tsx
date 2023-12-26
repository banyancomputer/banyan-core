import React from 'react'

export const AccountType: React.FC<{ text: string, action:  React.Dispatch<React.SetStateAction<string>>, isActive: boolean, value: string }> = ({ text, action, isActive, value }) => {
    const selectAccountType = () => {
        action(value);
    };

    return (
        <div
            className={`py-3 px-6 border-1 border-border-darken rounded-md cursor-pointer font-mediuma transition-all hover:bg-hover ${isActive && 'text-termsAndConditions-activeAccountType border-termsAndConditions-activeAccountType bg-termsAndConditions-activeAccountTypeBackground'}`}
            onClick={selectAccountType}
        >
            {text}
        </div>
    )
};
