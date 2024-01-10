import React from 'react';

import { PlusBold } from '@static/images/common';

export const AddNewOption: React.FC<{ action: () => void; label: string }> = ({
	action,
	label,
}) => (
	<div
		className="flex items-center gap-2 p-2.5 font-semibold transition-all hover:bg-bucket-bucketHoverBackground cursor-pointer"
		onClick={action}
	>
		<PlusBold />
		{label}
	</div>
);
