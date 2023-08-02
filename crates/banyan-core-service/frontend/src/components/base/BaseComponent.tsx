import React from 'react';
export interface IBaseComponent {}
const BaseComponent: React.FC<IBaseComponent> = ({}) => {
	return (
		<>
			<p> BaseComponent </p>
		</>
	);
};

export default BaseComponent;
