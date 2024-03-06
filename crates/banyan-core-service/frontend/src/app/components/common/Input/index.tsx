import React from 'react';
import { UseFormRegisterReturn } from 'react-hook-form';

export const Input: React.FC<{
    value?: string;
    onChange?: React.Dispatch<React.SetStateAction<string>>;
    placeholder?: string;
    label?: string;
    error?: string;
    register?: UseFormRegisterReturn<string>;
    containerClassName?: string;
    inputClassName?: string;
    mandatory?: boolean;
    labelClassName?: string;
    type?: 'text' | 'password';
}> = ({
    value,
    placeholder,
    register,
    error,
    label,
    containerClassName,
    inputClassName,
    labelClassName,
    mandatory = false,
    onChange = () => { },
    type = 'text',
}) =>
        <div className={containerClassName}>
            {label ?
                <label className={`inline-block mb-2 text-xs font-normal ${labelClassName}`}>{`${label}`} {mandatory && <span className="text-error">*</span>}</label>
                :
                null
            }
            <input
                type={type}
                className={`input w-full h-10 p-3 rounded-md border-border-darken focus:outline-none ${inputClassName}`}
                value={value}
                onChange={event => onChange(event.target.value)}
                placeholder={placeholder}
                {...register}
            />
            {error &&
                <span className="mt-2 inline-block w-full text-error text-xs">{error}</span>
            }
        </div>;
