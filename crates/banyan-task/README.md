# banyan-task
Background task orchestrator for core services

# note
You can implement different types of Task Stores if you want.
For now this crate relies heavily on SqliteTaskStore.

If you would like to use that implementation you must run the example migration at `./migrations/create_background_tasks.up.sql` against a SqliteDB accessible from your service

