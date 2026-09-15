use alloc::vec::Vec;
use xdevs::prelude::*;

use crate::xdevs_driver;

pub struct Generator {
    sigma: f64,
    period: f64,
    job_count: u64,
}

impl Generator {
    pub const fn new(period: f64) -> Self {
        Self {
            period,
            sigma: period,
            job_count: 0,
        }
    }
}

impl Component for Generator {
    type Kind = AtomicKind;
    type Input = ();
    type Output = Option<u64>;
}

impl Atomic for Generator {
    fn delta_int(&mut self) {
        self.sigma = self.period;
        self.job_count += 1;
    }

    fn delta_ext(&mut self, elapsed: f64, _input: &Self::Input) {
        self.sigma -= elapsed;
    }

    fn lambda(&self, output: &mut Self::Output) {
        log::info!("    devs_model: Generator emitted job {}", self.job_count);
        *output = Some(self.job_count);
    }

    fn ta(&self) -> f64 {
        self.sigma
    }
}

pub struct Processor {
    sigma: f64,
    proc_time: f64,
    current_job: Option<u64>,
}

impl Processor {
    pub const fn new(proc_time: f64) -> Self {
        Self {
            sigma: f64::INFINITY,
            proc_time,
            current_job: None,
        }
    }
}

impl Component for Processor {
    type Kind = AtomicKind;
    type Input = Vec<u64>;
    type Output = Option<u64>;
}

impl Atomic for Processor {
    fn delta_int(&mut self) {
        self.sigma = f64::INFINITY;
        self.current_job = None;
    }

    fn delta_ext(&mut self, elapsed: f64, input: &Self::Input) {
        self.sigma -= elapsed;
        // Only accept the first job in the input vector if the processor is idle
        if self.current_job.is_none()
            && let Some(job) = input.first()
        {
            let job = *job;
            self.current_job = Some(job);
            self.sigma = self.proc_time;
            log::info!("    devs_model: Processor is processing job {}", job);
        }
    }

    fn lambda(&self, output: &mut Self::Output) {
        *output = self.current_job;
        if let Some(job) = *output {
            log::info!("    devs_model: Processor finished job {}", job);
        }
    }

    fn ta(&self) -> f64 {
        self.sigma
    }
}

#[xdevs::coupled]
pub struct TopModel {
    generator: Generator,
    processor: Processor,
}

impl TopModel {
    pub fn new(gen_period: f64, proc_time: f64) -> Self {
        Self::build(Generator::new(gen_period), Processor::new(proc_time))
    }
}

impl Component for TopModel {
    type Kind = CoupledKind;
    type Input = ();
    type Output = Option<u64>;
}

impl Coupled for TopModel {
    fn ic(from: &ComponentsOutput<Self>, to: &mut ComponentsInput<Self>) {
        let _ = couple(&from.generator, &mut to.processor);
    }

    fn eoc(from: &ComponentsOutput<Self>, to: &mut Self::Output) {
        let _ = couple(&from.processor, to);
    }
}

pub async fn run_devs_model() {
    let top_model = TopModel::new(2.0, 1.0);
    let mut sim = top_model.to_simulator();

    let config = Config::default();
    let input_handler = xdevs_driver::SleepAsync::new();

    sim.simulate_rt_async(&config, input_handler, |_| {}).await;
}
