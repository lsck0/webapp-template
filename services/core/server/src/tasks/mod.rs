mod hello_world_task;

use hello_world_task::add_hello_world_task;
use tokio::sync::OnceCell;
use tokio_cron_scheduler::JobScheduler;

static CRON_INITIALIZED: OnceCell<bool> = OnceCell::const_new();

pub(crate) async fn initialize_cron_tasks() {
    match CRON_INITIALIZED.get() {
        Some(_) => panic!("Cron tasks already initialized."),
        None => CRON_INITIALIZED.set(true).expect("unreachable"),
    }

    let mut scheduler = JobScheduler::new().await.expect("Failed to create scheduler.");

    add_hello_world_task(&mut scheduler).await;

    tokio::spawn(async move {
        scheduler.start().await.expect("Failed to start scheduler.");
    });
}
