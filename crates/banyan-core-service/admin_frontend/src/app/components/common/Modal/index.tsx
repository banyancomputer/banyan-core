import React, { useRef } from 'react';

import { useModal } from '@/app/contexts/modals';

import { ArrowDown, Close } from '@static/images/common';

export const Modal = () => {
	const modalRef = useRef<HTMLDivElement | null>(null);
	const {
		modalState: { content, onBack, mandatory, className = 'p-6 rounded-md' },
		closeModal,
	} = useModal();

	const close = (event: React.MouseEvent<HTMLDivElement>) => {
		if (mandatory) {
			return;
		}
		if (!modalRef.current!.contains(event.target as Node)) {
			closeModal();
		}
	};

	return (
		<>
			{content && (
				<div
					className="absolute w-screen h-screen bg flex items-center justify-center z-20 bg-slate-800 bg-opacity-80 backdrop-blur-sm text-text-900"
					onClick={close}
				>
					<div
						className={`relative bg-modalBackground ${className}`}
						ref={modalRef}
					>
						{onBack && (
							<button onClick={onBack} className="rotate-90">
								<ArrowDown width="24px" height="24px" />
							</button>
						)}
						{!mandatory && (
							<button className="absolute right-6 top-6" onClick={closeModal}>
								<Close width="24px" height="24px" />
							</button>
						)}
						{content}
					</div>
				</div>
			)}
		</>
	);
};
