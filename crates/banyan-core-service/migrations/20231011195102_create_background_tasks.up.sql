CREATE TABLE background_tasks (
  id TEXT NOT NULL PRIMARY KEY DEFAULT (
    lower(hex(randomblob(4))) || '-' ||
    lower(hex(randomblob(2))) || '-4' ||
    substr(lower(hex(randomblob(2))), 2) || '-a' ||
    substr(lower(hex(randomblob(2))), 2) || '-6' ||
    substr(lower(hex(randomblob(6))), 2)
  ),

  next_id TEXT NULL
    REFERENCES background_tasks(id)
    ON DELETE SET NULL,
  previous_id TEXT NULL
    REFERENCES background_tasks(id)
    ON DELETE SET NULL,

  name TEXT NOT NULL,
  task_queue TEXT NOT NULL DEFAULT 'default',

  unique_key TEXT,
  state TEXT NOT NULL
    CHECK (state IN ('new', 'in_progress', 'panicked', 'retry', 'cancelled', 'error', 'complete', 'timed_out', 'dead'))
    DEFAULT 'new',

  current_attempt INTEGER NOT NULL DEFAULT 0,
  maximum_attempts INTEGER NOT NULL,

  -- actually a JSON blob might want to split more of this out
  payload BLOB NOT NULL,

  -- should probably be structured, for now will be JSON blob
  error BLOB,

  scheduled_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  scheduled_to_run_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

  started_at TIMESTAMP,
  finished_at TIMESTAMP
);

CREATE INDEX idx_background_tasks_on_scheduled_at ON background_tasks(scheduled_at);
CREATE INDEX idx_background_tasks_on_scheduled_to_run_at ON background_tasks(scheduled_to_run_at);
CREATE INDEX idx_background_tasks_on_state ON background_tasks(state);
CREATE INDEX idx_background_tasks_on_name ON background_tasks(name);
CREATE INDEX idx_background_tasks_on_task_queue ON background_tasks(task_queue);
CREATE INDEX idx_background_tasks_on_unique_key ON background_tasks(unique_key) WHERE unique_key != NULL;
