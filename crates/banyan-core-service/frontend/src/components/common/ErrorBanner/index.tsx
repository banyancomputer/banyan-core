import React from 'react';
import { AiOutlineWarning } from 'react-icons/ai';

export const ErrorBanner: React.FC<{ message: string }> = ({ message }) =>
    <div className="flex justify-center items-center gap-3 py-4 px-2.5 bg-errorBanner border-2 border-navigation-border text-sm font-medium">
        <AiOutlineWarning size="20px" />
        {message}
    </div>;

