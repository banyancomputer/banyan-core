import React, { useEffect } from 'react';
import { IoMdClose } from 'react-icons/io';
import { useModal } from '@/contexts/modals';

export const Modal = () => {
    const { modalState: { content }, closeModal } = useModal();

    const stopPropagation = (event: React.MouseEvent<HTMLDivElement>) => {
        event.stopPropagation();
    };

    return (
        <>
            {content &&
                <div
                    className="absolute w-screen h-screen bg flex items-center justify-center z-10 bg-slate-800 bg-opacity-80 backdrop-blur-sm"
                    onClick={closeModal}
                >
                    <div
                        className="relative p-6 bg-white rounded-xl"
                        onClick={stopPropagation}
                    >
                        <button
                            className="absolute right-6 top-6"
                            onClick={closeModal}
                        >
                            <IoMdClose fill="#4A5578" size="24px" />
                        </button>
                        {content}
                    </div>
                </div>
            }
        </>
    );
};
