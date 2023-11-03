import React, { useState } from 'react';
import { IoMdClose } from 'react-icons/io';

export const ErrorBanner: React.FC<{ title: string; description: string }> = ({ description, title }) => {
    const [isVisible, setIsVisible] = useState(true);

    const toggleStorageVisibility = () => {
        setIsVisible(prev => !prev);
    };

    return (
        <>
            {isVisible ?
                <div className="relative px-4 py-5 rounded-lg bg-errorBanner">
                    <button
                        onClick={toggleStorageVisibility}
                        className="absolute right-4 top-5"
                    >
                        <IoMdClose size="20px" />
                    </button>
                    <h4 className="mb-1 text-xs font-semibold">{title}</h4>
                    <p className="text-xs">{description}</p>
                </div>
                :
                null
            }
        </>
    );
};
