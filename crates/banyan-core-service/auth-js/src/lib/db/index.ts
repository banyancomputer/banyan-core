import { join } from 'path';
import { DataSource } from 'typeorm';
import * as entities from './entities';

console.log('DB_PATH', process.env.DB_PATH || join(__dirname, 'db.sqlite'));

export const connection: any = {
	type: 'sqlite',
	database: process.env.DB_PATH || join(__dirname, 'db.sqlite'),
	synchronize: process.env.NODE_ENV !== 'production',
	entities: Object.values(entities),
};

// Get the application data source

let _applicationDataSource: DataSource | null = null;

export const getApplicationDataSource = async () => {
	if (!_applicationDataSource) {
		_applicationDataSource = new DataSource(connection);
	}

	const manager = _applicationDataSource.manager;

	if (!manager.connection.isInitialized) {
		await manager.connection.initialize();
	}

	return manager;
};

export * from './allowedEmail';
export * from './escrowedDevicePrivateKey';
export * from './devicePublicKey';