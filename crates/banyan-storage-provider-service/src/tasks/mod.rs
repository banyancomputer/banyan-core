mod prune_blocks;
mod report_bandwidth_metrics;
mod report_health;
mod report_redistribution;
mod report_upload;

use std::sync::{Arc, Mutex, RwLock};

use banyan_task::{QueueConfig, SqliteTaskStore, TaskLike, TaskLikeExt, TaskStore, WorkerPool};
pub use prune_blocks::PruneBlocksTask;
pub use report_health::ReportHealthTask;
pub use report_redistribution::ReportRedistributionTask;
pub use report_upload::ReportUploadTask;
use sqlx::SqliteConnection;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::app::AppState;
use crate::database::{correct, DatabaseConnection};
use crate::tasks::report_bandwidth_metrics::ReportBandwidthMetricsTask;

pub async fn start_background_workers(
    state: AppState,
    mut shutdown_rx: watch::Receiver<()>,
) -> Result<JoinHandle<()>, ()> {
    let database = state.database();
    let task_store = SqliteTaskStore::new(state.database());

    let x = state.clone();

    let mut conn = state.database().acquire().await.unwrap();
    //let conn = correct(&mut conn);
    //let x: &mut SqliteConnection = correct(&mut conn);
    //let locked_connection = Arc::new(Mutex::new(x));

    /*
    enqueue_task_if_none_in_progress::<ReportBandwidthMetricsTask>(&task_store, &mut conn).await;
    enqueue_task_if_none_in_progress::<ReportHealthTask>(&task_store, &mut conn).await;
    */

    /*
    async fn run_thingy() {
    }
    */

    WorkerPool::new(task_store.clone(), move || state.clone())
        .configure_queue(QueueConfig::new("default").with_worker_count(5))
        .register_task_type::<ReportUploadTask>()
        .register_task_type::<PruneBlocksTask>()
        .register_recurring_task_type::<ReportHealthTask>()
        .register_recurring_task_type::<ReportBandwidthMetricsTask>()
        .register_task_type::<ReportRedistributionTask>()
        .start(
            //move || correct(&mut conn),
            //&mut *x,
            //locked_connection,
            /*
            move |func| {
                let mut conn = locked_connection.write().unwrap();
                func(&mut *conn)
            },
            */
            async move {
                let _ = shutdown_rx.changed().await;
            },
        )
        .await
        .map_err(|_| ())
}

async fn enqueue_task_if_none_in_progress<T: TaskLikeExt + TaskLike + Default>(
    task_store: &SqliteTaskStore,
    conn: &mut DatabaseConnection,
) {
    if task_store
        .get_living_task(T::TASK_NAME)
        .await
        .expect("get task")
        .is_some()
    {
        return;
    }

    T::default()
        .enqueue::<SqliteTaskStore>(conn)
        .await
        .expect("enqueue task");
}
