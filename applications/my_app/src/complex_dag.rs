use alloc::{borrow::Cow, sync::Arc, vec};
use awkernel_async_lib::{
    dag::{Dag, create_dag},
    scheduler::SchedulerType,
};
use awkernel_lib::delay::wait_millisec;
use core::time::Duration;

fn n0_reactor() -> (i32, i32) {
    log::info!("complex_dag: N0 started");
    wait_millisec(100);
    log::info!("complex_dag: N0 finished");
    (1, 2)
}

fn n1_reactor((x,): (i32,)) -> (i32, i32) {
    log::info!("complex_dag: N1 started. Input: {x}");
    wait_millisec(100);
    log::info!("complex_dag: N1 finished");
    (x + 1, x + 2)
}

fn n2_reactor((x,): (i32,)) -> (i32,) {
    log::info!("complex_dag: N2 started. Input: {x}");
    wait_millisec(200);
    log::info!("complex_dag: N2 finished");
    (x * 2,)
}

fn n3_reactor((x,): (i32,)) -> (i32,) {
    log::info!("complex_dag: N3 started. Input: {x}");
    wait_millisec(100);
    log::info!("complex_dag: N3 finished");
    (x + 1,)
}

fn n4_reactor((x,): (i32,)) -> (i32,) {
    log::info!("complex_dag: N4 started. Input: {x}");
    wait_millisec(100);
    log::info!("complex_dag: N4 finished");
    (x + 3,)
}

fn n5_reactor((x,): (i32,)) -> (i32,) {
    log::info!("complex_dag: N5 started. Input: {x}");
    wait_millisec(100);
    log::info!("complex_dag: N5 finished");
    (x + 3,)
}

fn n6_reactor((x, y): (i32, i32)) -> (i32,) {
    log::info!("complex_dag: N6 started. Inputs: x: {x}, y: {y}");
    wait_millisec(100);
    log::info!("complex_dag: N6 finished");
    (x + y,)
}

fn n7_reactor((a, b): (i32, i32)) {
    log::info!("complex_dag: N7 started. Inputs: a: {a}, b: {b}");
    wait_millisec(100);
    log::info!("complex_dag: N7 finished. Values: a: {a}, b: {b}");
    log::info!("");
}

pub async fn complex_dag() -> Arc<Dag> {
    let dag = create_dag();

    dag.register_periodic_reactor::<_, (i32, i32)>(
        "my_app_complex_n0".into(),
        n0_reactor,
        vec![
            Cow::from("my_app/complex/n0/1"),
            Cow::from("my_app/complex/n0/2"),
        ],
        SchedulerType::GEDF(0),
        Duration::from_millis(1000),
    )
    .await;

    dag.register_reactor::<_, (i32,), (i32, i32)>(
        "my_app_complex_n1".into(),
        n1_reactor,
        vec![Cow::from("my_app/complex/n0/1")],
        vec![
            Cow::from("my_app/complex/n1/1"),
            Cow::from("my_app/complex/n1/2"),
        ],
        SchedulerType::GEDF(0),
    )
    .await;

    dag.register_reactor::<_, (i32,), (i32,)>(
        "my_app_complex_n2".into(),
        n2_reactor,
        vec![Cow::from("my_app/complex/n0/2")],
        vec![Cow::from("my_app/complex/n2")],
        SchedulerType::GEDF(0),
    )
    .await;

    dag.register_reactor::<_, (i32,), (i32,)>(
        "my_app_complex_n3".into(),
        n3_reactor,
        vec![Cow::from("my_app/complex/n1/1")],
        vec![Cow::from("my_app/complex/n3")],
        SchedulerType::GEDF(0),
    )
    .await;

    dag.register_reactor::<_, (i32,), (i32,)>(
        "my_app_complex_n4".into(),
        n4_reactor,
        vec![Cow::from("my_app/complex/n1/2")],
        vec![Cow::from("my_app/complex/n4")],
        SchedulerType::GEDF(0),
    )
    .await;

    dag.register_reactor::<_, (i32,), (i32,)>(
        "my_app_complex_n5".into(),
        n5_reactor,
        vec![Cow::from("my_app/complex/n2")],
        vec![Cow::from("my_app/complex/n5")],
        SchedulerType::GEDF(0),
    )
    .await;

    dag.register_reactor::<_, (i32, i32), (i32,)>(
        "my_app_complex_n6".into(),
        n6_reactor,
        vec![
            Cow::from("my_app/complex/n4"),
            Cow::from("my_app/complex/n5"),
        ],
        vec![Cow::from("my_app/complex/n6")],
        SchedulerType::GEDF(0),
    )
    .await;

    // Sink: waits for both inputs, carries the end-to-end relative deadline.
    dag.register_sink_reactor::<_, (i32, i32)>(
        "my_app_complex_n7".into(),
        n7_reactor,
        vec![
            Cow::from("my_app/complex/n3"),
            Cow::from("my_app/complex/n6"),
        ],
        SchedulerType::GEDF(0),
        Duration::from_millis(50),
    )
    .await;
    dag
}
