#![no_std]

extern crate alloc;

use alloc::{borrow::Cow, vec};
use awkernel_async_lib::{
    dag::{create_dag, finish_create_dags},
    scheduler::SchedulerType,
};
use awkernel_lib::delay::wait_millisec;
use core::time::Duration;

fn source_reactor() -> (i32,) {
    log::info!("Hello from source!");
    wait_millisec(100);
    (1,)
}

fn intermediate_reactor((x,): (i32,)) -> (i32, i32) {
    log::info!("Hello from intermediate!");
    wait_millisec(500);
    (x + 1, x + 2)
}

fn sink_reactor((a, b): (i32, i32)) {
    log::info!("Hello from sink!");
    wait_millisec(200);
    log::info!("a: {a}, b: {b}");
}

pub async fn run() {
    log::set_max_level(log::LevelFilter::Trace);

    let dag = create_dag();
    // Source: released every 100 ms, publishes one i32 on "my_app/raw".
    dag.register_periodic_reactor::<_, (i32,)>(
        "my_app_source".into(),
        source_reactor,
        vec![Cow::from("my_app/raw")],
        SchedulerType::GEDF(0),
        Duration::from_millis(1000),
    )
    .await;
    // Intermediate: one input, two outputs.
    dag.register_reactor::<_, (i32,), (i32, i32)>(
        "my_app_filter".into(),
        intermediate_reactor,
        vec![Cow::from("my_app/raw")],
        vec![Cow::from("my_app/a"), Cow::from("my_app/b")],
        SchedulerType::GEDF(0),
    )
    .await;
    // Sink: waits for both inputs, carries the end-to-end relative deadline.
    dag.register_sink_reactor::<_, (i32, i32)>(
        "my_app_sink".into(),
        sink_reactor,
        vec![Cow::from("my_app/a"), Cow::from("my_app/b")],
        SchedulerType::GEDF(0),
        Duration::from_millis(50),
    )
    .await;
    if let Err(errors) = finish_create_dags(&[dag]).await {
        for e in errors {
            log::error!("my_app: {e}");
        }
    }
}
