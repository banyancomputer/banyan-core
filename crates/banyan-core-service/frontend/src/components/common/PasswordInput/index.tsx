import React from 'react';
import { UseFormRegisterReturn } from 'react-hook-form';

export const PasswordInput: React.FC<{
    value?: string,
    onChange?: React.Dispatch<React.SetStateAction<string>>,
    placeholder?: string,
    error?: string,
    register?: UseFormRegisterReturn<string>;
}> = ({
    value,
    placeholder,
    register,
    error,
    onChange = () => { },
}) => {

        return (
            <div>
                <input
                    className={`input w-full h-10 py-3 px-4 rounded-xl border-gray-400 focus:outline-none`}
                    value={value}
                    onChange={event => onChange(event.target.value)}
                    placeholder={placeholder}
                    {...register}
                />
                {error &&
                    <span className='mt-2 inline-block w-full text-error text-xs'>{error}</span>
                }
            </div>
        );
    };

