import React, { useEffect, useRef } from 'react';
import { IoMdClose } from 'react-icons/io';
import { useModal } from '@/contexts/modals';
import { FiArrowLeft } from 'react-icons/fi';

export const Modal = () => {
    const modalRef = useRef<HTMLDivElement | null>(null);
    const { modalState: { content, onBack }, closeModal } = useModal();

    const close = (event: React.MouseEvent<HTMLDivElement>) => {
        if (!modalRef.current!.contains(event.target as Node)) {
            closeModal();
        };
    };

    return (
        <>
            {content &&
                <div
                    className="absolute w-screen h-screen bg flex items-center justify-center z-10 bg-slate-800 bg-opacity-80 backdrop-blur-sm"
                    onClick={close}
                >
                    <div
                        className="relative p-6 bg-white rounded-xl"
                        ref={modalRef}
                    >
                        {onBack &&
                            <button onClick={onBack}>
                                <FiArrowLeft size="24px" />
                            </button>
                        }
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
