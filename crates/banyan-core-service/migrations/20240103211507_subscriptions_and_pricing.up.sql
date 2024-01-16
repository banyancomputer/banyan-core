CREATE TABLE subscriptions (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  price_key TEXT NOT NULL,
  title TEXT NOT NULL,

  stripe_product_id TEXT,

  allow_overage BOOLEAN NOT NULL,
  archival_available BOOLEAN NOT NULL,
  visible BOOLEAN NOT NULL,

  base_price INTEGER,
  storage_overage_price INTEGER,
  bandwidth_overage_price INTEGER,

  included_archival INTEGER NOT NULL,
  included_bandwidth INTEGER NOT NULL,
  included_storage INTEGER NOT NULL,

  archival_hard_limit INTEGER,
  bandwidth_hard_limit INTEGER,
  storage_hard_limit INTEGER,

  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- We need to manually populate our starter plan so we can automatically
-- associate existing rows to it.
INSERT INTO subscriptions (
    price_key, title, allow_overage, archival_available, visible, included_archival,
    included_bandwidth, included_storage
  ) VALUES ('starter', 'Starter', false, false, true, 0, 10, 10);

ALTER TABLE users ADD COLUMN subscription_id TEXT
    REFERENCES subscriptions(id);

UPDATE users SET subscription_id = (SELECT id FROM subscriptions WHERE price_key = 'starter' LIMIT 1);

ALTER TABLE users ADD COLUMN stripe_customer_id TEXT;
