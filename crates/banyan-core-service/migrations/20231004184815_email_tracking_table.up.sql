-- Add up migration script here

CREATE TABLE emails (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),
  
  -- The account_id of the user who should receive this email
  account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,

  -- When the email was sent
  sent_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  -- The type of the email. This corresponds to the EmailMessage::template_name
  type TEXT NOT NULL,

  -- The state of the email.
    -- 'sent' means the email was successfully sent
    -- 'delivered' means the email was successfully delivered
    -- 'opened' means the email was successfully opened
    -- 'marked_as_spam' means the email was marked as spam
    -- 'unsubscribed' means the email was unsubscribed
    -- 'delivery_failed' means the email failed to be delivered
  state VARCHAR(32) NOT NULL CHECK (state IN ('sent', 'delivered', 'opened', 'marked_as_spam', 'unsubscribed', 'delivery_failed'))
    DEFAULT 'sent'
);