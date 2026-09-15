use awkernel_async_lib::sleep_until;
use awkernel_lib::time::Time;
use core::{marker::PhantomData, time::Duration};
use xdevs::{Config, bag::Bag, simulation::AsyncInput};

/// A simple asynchronous input handler that sleeps until the next state transition of the model.
#[derive(Default)]
pub struct SleepAsync<T: Bag> {
    /// The last recorded real time instant.
    last_rt: Option<Time>,
    /// Phantom data to associate with the input bag type.
    input: PhantomData<T>,
}

impl<T: Bag> SleepAsync<T> {
    /// Creates a new `SleepAsync` instance.
    pub fn new() -> Self {
        Self {
            last_rt: None,
            input: PhantomData,
        }
    }
}

impl<T: Bag> AsyncInput for SleepAsync<T> {
    type Input = T;

    async fn handle(
        &mut self,
        config: &Config,
        t_from: f64,
        t_until: f64,
        _input: &mut Self::Input,
    ) -> f64 {
        let last_rt = self.last_rt.unwrap_or_else(Time::now);
        let duration = Duration::from_secs_f64((t_until - t_from) * config.time_scale);
        let next_rt = last_rt + duration;
        sleep_until(next_rt).await;
        self.last_rt = Some(next_rt);
        t_until
    }
}
