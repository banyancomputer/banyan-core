CREATE TABLE subscriptions (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  service_key TEXT NOT NULL,
  tax_class TEXT NOT NULL,
  title TEXT NOT NULL,
  visible BOOLEAN NOT NULL,

  plan_base_price INTEGER,
  plan_price_stripe_id TEXT,

  archival_available BOOLEAN NOT NULL DEFAULT false,
  archival_price INTEGER,
  archival_stripe_price_id TEXT,
  archival_hard_limit INTEGER,

  hot_storage_price INTEGER,
  hot_storage_stripe_price_id TEXT,
  hot_storage_hard_limit INTEGER,

  bandwidth_price INTEGER,
  bandwidth_stripe_price_id TEXT,
  bandwidth_hard_limit INTEGER,

  included_hot_replica_count INTEGER NOT NULL DEFAULT 2,
  included_hot_storage INTEGER NOT NULL,
  included_bandwidth INTEGER NOT NULL,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- We need to manually populate our starter plan so we can automatically
-- associate existing rows to it.
INSERT INTO subscriptions (
    service_key, tax_class, title, visible, hot_storage_hard_limit, bandwidth_hard_limit,
    included_hot_storage, included_bandwidth
  ) VALUES ('starter', 'not_applicable', 'Starter Plan', false, 20, 10, 20, 10);

ALTER TABLE users ADD COLUMN stripe_customer_id TEXT;
ALTER TABLE users ADD COLUMN subscription_id TEXT REFERENCES subscriptions(id);

UPDATE users SET subscription_id = (
    SELECT id FROM subscriptions WHERE service_key = 'starter' LIMIT 1
  );

CREATE TABLE stripe_products (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  product_key TEXT NOT NULL,
  title TEXT NOT NULL,

  stripe_product_id TEXT
);

INSERT INTO stripe_products (product_key, title)
  VALUES
    ('lite-plan', 'Lite Plan'),
    ('business-plan', 'Business Plan'),
    ('bandwidth', 'Bandwidth Usage'),
    ('storage', 'Hot Storage'),
    ('archival', 'Archival Storage');

