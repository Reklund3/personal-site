use site::configuration::get_configuration;
use site::issue_delivery_worker::run_worker_until_stopped;
use site::startup::Application;
use site::telemetry::{get_subscriber, init_subscriber};
use std::fmt::{Debug, Display};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("site".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let worker_enabled = configuration.application.worker_enabled;
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    if worker_enabled {
        let worker_task = tokio::spawn(run_worker_until_stopped(configuration));
        tokio::select! {
            o = application_task => report_exit("API", o),
            o = worker_task => report_exit("Background worker", o),
        };
    } else {
        tracing::info!("Background worker disabled via configuration; running in API-only mode");
        report_exit("API", application_task.await);
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
