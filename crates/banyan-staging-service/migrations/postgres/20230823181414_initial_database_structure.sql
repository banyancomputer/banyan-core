CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE clients (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  current_storage_grant_id UUID REFERENCES storage_grants(id),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);

CREATE TABLE storage_grants (
  id UUID DEFAULT uuid_generate_v4() NOT NULL PRIMARY KEY,

  remote_id UUID NOT NULL,
  client_id UUID NOT NULL REFERENCES clients(id),

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);
