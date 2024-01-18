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
    ('hot-storage', 'Hot Storage'),
    ('archival', 'Archival Storage');

CREATE TABLE account_subscriptions (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  service_key TEXT NOT NULL,
  tax_type TEXT NOT NULL,

  title TEXT NOT NULL,
  visible BOOLEAN NOT NULL,

  plan_base_price INTEGER,
  plan_price_stripe_id TEXT,

  archival_available BOOLEAN NOT NULL DEFAULT false,
  archival_price INTEGER,
  archival_stripe_price_id TEXT,
  archival_total_hard_limit INTEGER,

  hot_replica_price INTEGER,
  hot_replica_stripe_price_id TEXT,
  hot_replica_total_hard_limit INTEGER,

  bandwidth_price INTEGER,
  bandwidth_stripe_price_id TEXT,
  bandwidth_total_hard_limit INTEGER,

  included_hot_replica_count INTEGER DEFAULT 2,
  included_hot_replica_storage INTEGER,
  included_bandwidth_amount INTEGER,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX idx_unique_service_key_tax_type_where_visible ON
  account_subscriptions(service_key, tax_type)
  WHERE visible = true;

-- We need to manually populate our starter plan so we can automatically
-- associate existing rows to it.
INSERT INTO account_subscriptions (
    service_key, tax_type, title, visible, hot_replica_total_hard_limit, bandwidth_total_hard_limit,
    included_hot_replica_storage, included_bandwidth_amount
  ) VALUES ('starter', 'personal', 'Starter Plan', false, 10, 10, 10, 10),
           ('starter', 'business', 'Starter Plan', false, 10, 10, 10, 10);

ALTER TABLE users ADD COLUMN account_subscription_id TEXT
    REFERENCES account_subscriptions(id);

UPDATE users SET account_subscription_id = (
  SELECT id FROM account_subscriptions
    WHERE service_key = 'starter' AND tax_type = 'personal'
    LIMIT 1
  );

ALTER TABLE users ADD COLUMN stripe_customer_id TEXT;
