#![no_std]

extern crate alloc;

use awkernel_async_lib::{dag::finish_create_dags, scheduler::SchedulerType, sleep, spawn};

mod complex_dag;
mod devs_model;
mod xdevs_driver;

pub async fn run() {
    log::set_max_level(log::LevelFilter::Trace);

    let complex_dag = complex_dag::complex_dag().await;
    if let Err(errors) = finish_create_dags(&[complex_dag]).await {
        for e in errors {
            log::error!("my_app: {e}");
        }
    }
    spawn(
        "periodic_task".into(),
        periodic_task(),
        SchedulerType::GEDF(1500),
    )
    .await;
    spawn(
        "my_app_devs_model".into(),
        devs_model::run_devs_model(),
        SchedulerType::GEDF(1500),
    )
    .await;
}

pub async fn periodic_task() {
    loop {
        log::info!("my_app: periodic task");
        sleep(core::time::Duration::from_millis(2000)).await;
    }
}
