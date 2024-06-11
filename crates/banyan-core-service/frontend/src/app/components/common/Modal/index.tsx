import React, { useRef } from 'react';

import { closeModal } from '@store/modals/slice';

import { ArrowDown, Close } from '@static/images/common';
import { useAppDispatch, useAppSelector } from '@store/index';

export const Modal = () => {
    const dispatch = useAppDispatch();
    const modalRef = useRef<HTMLDivElement | null>(null);
    const { content, onBack, path } = useAppSelector(state => state.modals);

    const close = (event: React.MouseEvent<HTMLDivElement | HTMLButtonElement>) => {
        if (!modalRef.current!.contains(event.target as Node)) {
            dispatch(closeModal());
        };
    };

    return (
        <>
            {content &&
                <div
                    className="absolute w-screen h-screen bg flex items-center justify-center z-max bg-[#0d0d0dcc] text-text-900"
                    onClick={close}
                >
                    <div
                        className={`relative bg-modalBackground p-6 rounded-md`}
                        ref={modalRef}
                    >
                        {onBack &&
                            <button onClick={onBack} className="rotate-90">
                                <ArrowDown width="24px" height="24px" />
                            </button>
                        }
                        {path &&
                            <div className="mb-2 text-xs text-text-600">
                                {
                                    path.map((pathPart, index) =>
                                        <span key={index}>{pathPart} / </span>
                                    )
                                }
                            </div>
                        }
                        <button
                            className="absolute right-6 top-6"
                            onClick={close}
                        >
                            <Close width="24px" height="24px" />
                        </button>
                        {content}
                    </div>
                </div>
            }
        </>
    );
};
