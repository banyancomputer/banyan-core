import React from 'react';
import { User } from '@app/types';
import { AdminClient } from '@/api/admin';
import { ToastNotifications } from '@app/utils/toastNotifications';

interface UsersTableProps {
	users: User[];
	onReset: () => Promise<void>;
}

export const UsersTable: React.FC<UsersTableProps> = ({ users, onReset }) => {
	const client = new AdminClient();
	const resetUser = async (userId: string): Promise<void> => {
		try {
			await client.resetUser(userId)
			await onReset();
		} catch (error: any) {
			ToastNotifications.error('Could not reset user', "Close", () => {});
		}
	};
	return (
		<div className="overflow-x-auto border-1 border-border-regular rounded-xl">
			<table className="table table-pin-rows w-full text-text-600 rounded-xl ">
				<thead className="border-b-table-cellBackground text-xxs font-normal text-text-600">
				<tr className="border-b-table-cellBackground bg-table-headBackground">
					<th className="p-3 whitespace-break-spaces text-left font-medium">
						{'User ID'}
					</th>
					<th className="p-3 text-left font-medium whitespace-pre">
						{'Email'}
					</th>
					<th className="p-3 text-left font-medium">{'Verified Email'}</th>
					<th className="p-3 w-32 text-left font-medium">{'Accepted TOS'}</th>
					<th className="p-3 w-32 text-left font-medium"/>
				</tr>
				</thead>
				<tbody>
				{users.map((user: User) => (
					<tr key={user.id}>
							<td className="px-3 py-6 border-t-1 border-r-1 border-border-regular">
								{user.id}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{user.email}
							</td>
							<td className="px-3 py-6 border-t-1 border-x-1 border-border-regular">
								{user.verifiedEmail ? 'Yes' : 'No'}
							</td>
							<td className="px-3 py-6 border-t-1 border-l-1 border-border-regular">
								{user.acceptedTosAt}
							</td>
							<td className="px-3 py-6 border-t-1 border-l-1 border-border-regular">
								<button
								className="btn-secondary h-10 w-20 ml-4"
								onClick={() => resetUser(user.id)}
								>
								{'Reset User'}
								</button>
							</td>
						</tr>
					))}
				</tbody>
			</table>
		</div>
	);
};
