import React from 'react';
import { Deal } from '@app/types';

interface DealsTableProps {
	deals: Deal[];
}

export const DealsTable: React.FC<DealsTableProps> = ({ deals }) => {
	return (
		<div className="max-h-[calc(100vh-290px)] overflow-x-auto border-1 border-border-regular rounded-xl">
			<table className="table table-pin-rows w-full text-text-600 rounded-xl ">
				<thead className="border-b-table-cellBackground text-xxs font-normal text-text-600">
					<tr className="border-b-table-cellBackground bg-table-headBackground">
						<th className="p-3 whitespace-break-spaces text-left font-medium">
							{'Deal ID'}
						</th>
						<th className="p-3 text-left font-medium whitespace-pre">
							{'Deal State'}
						</th>
						<th className="p-3 text-left font-medium">{'Deal Size'}</th>
						<th className="p-3 w-32 text-left font-medium">{'Accepted By'}</th>
						<th className="p-3 w-32 text-left font-medium">{'Accepted At'}</th>
					</tr>
				</thead>
				<tbody>
					{deals.map((deal: Deal) => (
						<tr key={deal.id}>
							<td className="px-3 py-6 border-t-1 border-r-1 border-border-regular">
								{deal.id}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{deal.state}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{deal.size}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{deal.accepted_by}
							</td>
							<td className="px-3 py-6 border-t-1 border-l-1 border-border-regular">
								{deal.accepted_at}
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
};
