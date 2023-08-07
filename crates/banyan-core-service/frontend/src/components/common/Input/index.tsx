import React, { ReactElement } from 'react'

export const Input: React.FC<{
    placeholder: string,
    onChange: () => void
    icon?: ReactElement,
}> = ({
    placeholder,
    onChange,
    icon,
}) => {
        return (
            <div className='flex relative flex-grow max-w-xl'>
                <span className='absolute left-4 top-1/2 -translate-y-1/2'>
                    {icon}
                </span>
                <input
                    className={`input w-full h-10 py-3 px-4 rounded-xl border-gray-400  ${icon ? 'pl-12' : ''} focus:outline-none`}
                    onChange={onChange}
                    placeholder={placeholder}
                />
            </div>
        )
    }
