-- Table for managing user notifications
CREATE TABLE notifications ( 
	id TEXT NOT NULL PRIMARY KEY DEFAULT (
		lower(hex(randomblob(4))) || '-' ||
		lower(hex(randomblob(2))) || '-4' ||
		substr(lower(hex(randomblob(2))), 2) || '-a' ||
		substr(lower(hex(randomblob(2))), 2) || '-6' ||
		substr(lower(hex(randomblob(6))), 2)),

	-- Reference the user that is being delivered the notification 
	user_id TEXT NOT NULL REFERENCES users(id),

	-- Whether the user can mark the notification as shown, or whether it 
	-- is a persistent notification that will stick around until addressed.
	dismissable BOOLEAN NOT NULL DEFAULT true,

	-- Message to display to the user.
	message TEXT NOT NULL,

	-- Unique snake cased handle on the notification type
	message_key TEXT NOT NULL,

	-- The severity of the notification
	-- 'warning' means something non-critical needs to be communicated to the user
	-- 'error' means something breaking needs to be communicated to the user
	severity TEXT NOT NULL,

	-- Notification issuance time
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
