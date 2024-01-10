import React from 'react';

import { Spinner } from '@static/images/common';

export const Loader: React.FC<{
	spinnerSize: string;
	containerHeight?: string;
	className?: string;
}> = ({ spinnerSize, containerHeight = 'unset', className = '' }) => (
	<div
		className="w-full h-full flex justify-center items-center"
		style={{ height: containerHeight }}
	>
		<div
			className={`animate-spin ${className}`}
			style={{ height: spinnerSize, width: spinnerSize }}
		>
			<Spinner width={spinnerSize} height={spinnerSize} />
		</div>
	</div>
);
