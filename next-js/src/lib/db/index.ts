import { join } from 'path';
import { DataSource } from 'typeorm';
import * as entities from './entities';

// PG config

export const pgConnection: any = {
	type: 'postgres',
	host: process.env.DB_HOST,
	port: parseInt(process.env.DB_PORT ? process.env.DB_PORT : '5432'),
	username: process.env.DB_USERNAME,
	password: process.env.DB_PASSWORD,
	database: process.env.DB_NAME,
};

// Get the application data source

let _applicationDataSource: DataSource | null = null;

export const getApplicationDataSource = async () => {
	if (!_applicationDataSource) {
		_applicationDataSource = new DataSource({
			entities: Object.values(entities),
			synchronize: process.env.NODE_ENV !== 'production',
			...pgConnection,
		});
	}

	const manager = _applicationDataSource.manager;

	if (!manager.connection.isInitialized) {
		await manager.connection.initialize();
	}

	return manager;
};

export * from './allowed';
export * from './escrowedKey';
export * from './publicKey';
