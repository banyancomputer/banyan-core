import React, { ReactElement } from 'react';

import { Loader } from '../Loader';

export const Fallback: React.FC<{
	children: React.ReactNode;
	shouldRender: boolean;
	fallbackComponent?: ReactElement;
}> = ({
	children,
	shouldRender,
	fallbackComponent = <Loader spinnerSize="50px" containerHeight="200px" />,
}) => {
	if (!shouldRender) {
		return fallbackComponent;
	}

	return <>{children}</>;
};
