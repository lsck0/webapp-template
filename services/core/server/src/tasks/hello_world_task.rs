use std::time::Duration;

use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

/// Example. Says "Hello, World!" in the logs every 5 minutes.
pub async fn add_hello_world_task(scheduler: &mut JobScheduler) {
    let task = Job::new_repeated(Duration::from_secs(300), |_uuid, _l| {
        info!("Hello, World!");
    })
    .expect("Failed to create task.");

    scheduler.add(task).await.expect("Failed to add task to scheduler.");
}
