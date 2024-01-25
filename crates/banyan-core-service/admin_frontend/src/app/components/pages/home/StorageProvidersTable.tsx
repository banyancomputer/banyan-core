import React from 'react';
import { StorageHost } from '@app/types';

interface StorageProvidersTableProps {
	storageProviders: StorageHost[];
}

export const StorageProvidersTable: React.FC<StorageProvidersTableProps> = ({
	storageProviders,
}) => {
	return (
		<div className="overflow-x-auto border-1 border-border-regular rounded-xl">
			<table className="table table-pin-rows w-full text-text-600 rounded-xl ">
				<thead className="border-b-table-cellBackground text-xxs font-normal text-text-600">
					<tr className="border-b-table-cellBackground bg-table-headBackground">
						<th className="p-3 whitespace-break-spaces text-left font-medium">
							{'Name'}
						</th>
						<th className="p-3 text-left font-medium whitespace-pre">
							{'URL'}
						</th>
						<th className="p-3 text-left font-medium">{'Used Storage'}</th>
						<th className="p-3 w-32 text-left font-medium">
							{'Available Storage'}
						</th>
					</tr>
				</thead>
				<tbody>
					{storageProviders.map((storage: StorageHost) => (
						<tr key={storage.id}>
							<td className="px-3 py-6 border-t-1 border-r-1 border-border-regular">
								{storage.name}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{storage.url}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{storage.used_storage}
							</td>
							<td className="px-3 py-6 border-t-1 border-l-1 border-border-regular">
								{storage.available_storage}
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
};
