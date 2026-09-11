#![no_std]

extern crate alloc;

use awkernel_async_lib::dag::finish_create_dags;

mod complex_dag;

pub async fn run() {
    log::set_max_level(log::LevelFilter::Trace);

    let complex_dag = complex_dag::complex_dag().await;
    if let Err(errors) = finish_create_dags(&[complex_dag]).await {
        for e in errors {
            log::error!("my_app: {e}");
        }
    }
}
