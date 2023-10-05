import React from 'react';
import { UseFormRegisterReturn } from 'react-hook-form';

export const Input: React.FC<{
    value?: string;
    onChange?: React.Dispatch<React.SetStateAction<string>>;
    placeholder?: string;
    error?: string;
    register?: UseFormRegisterReturn<string>;
    type?: 'text' | 'password'
}> = ({
    value,
    placeholder,
    register,
    error,
    onChange = () => { },
    type = 'text'
}) =>
        <div>
            <input
                type={type}
                className="input w-full h-10 py-3 px-4 rounded-xl border-gray-200 focus:outline-none"
                value={value}
                onChange={event => onChange(event.target.value)}
                placeholder={placeholder}
                {...register}
            />
            {error &&
                <span className="mt-2 inline-block w-full text-error text-xs">{error}</span>
            }
        </div>;


