#![allow(clippy::needless_return)]

use models::DbInitFlags;
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() {
    let agent = PyroscopeAgent::builder("http://pyroscope:4040", "server")
        .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
        .build()
        .expect("Failed to create Pyroscope agent.");
    let agent_running = agent.start().expect("Failed to start Pyroscope agent.");

    let app = server::app(DbInitFlags::NONE).await;

    let listener = TcpListener::bind("0.0.0.0:80")
        .await
        .expect("Failed to bind to port 80.");

    info!("Server running on {}.", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Failed to run server.");

    let agent_ready = agent_running.stop().expect("Failed to Pyroscope stop agent.");
    agent_ready.shutdown();
}
