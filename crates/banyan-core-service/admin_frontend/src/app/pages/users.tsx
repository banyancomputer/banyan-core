import React, { useEffect, useState } from 'react';

import { AdminClient } from '@/api/admin';
import { NotFoundError } from '@/api/http';
import { HttpClient } from '@/api/http/client';
import { UsersTable } from '@components/pages/users/UsersTable';
import { User } from '@app/types';

const client = new AdminClient();

const Users = () => {
	const [users, setUsers] = useState<User[]>([]);

	const loadUsers = async () => {
		try {
			const users = await client.getUsers();
			setUsers(users);
		} catch (error: any) {
			if (error instanceof NotFoundError) {
				const api = new HttpClient();
				await api.get('/auth/logout');
				window.location.href = '/login';
			}
		}
	};

	useEffect(() => {
		loadUsers();
	}, []);

	return (
		<section className="py-9 pt-14 px-4" id="users">
			<UsersTable users={users} onReset={loadUsers}/>
		</section>
	);
};

export default Users;
