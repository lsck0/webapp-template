mod hello_world_task;

use hello_world_task::hello_world_task;
use tokio::sync::OnceCell;
use tokio_cron_scheduler::JobScheduler;

static CRON_INITIALIZED: OnceCell<bool> = OnceCell::const_new();

pub async fn initialize_cron_tasks() {
    match CRON_INITIALIZED.get() {
        Some(_) => return,
        None => CRON_INITIALIZED.set(true).expect("unreachable"),
    }

    let mut scheduler = JobScheduler::new().await.expect("Failed to create scheduler.");

    hello_world_task(&mut scheduler).await;

    tokio::spawn(async move {
        scheduler.start().await.expect("Failed to start scheduler.");
    });
}
