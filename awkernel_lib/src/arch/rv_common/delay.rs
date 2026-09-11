use super::INTERRUPT_CONTROLLER;
use crate::delay::Delay;

#[cfg(feature = "rv32")]
use crate::arch::rv32::RV32 as ArchImpl;
#[cfg(feature = "rv64")]
use crate::arch::rv64::RV64 as ArchImpl;

impl Delay for ArchImpl {
    fn wait_interrupt() {
        super::wait_interrupt();
    }

    fn wait_microsec(usec: u64) {
        INTERRUPT_CONTROLLER.clint().wait_microsec(usec);
    }

    fn uptime() -> u64 {
        INTERRUPT_CONTROLLER.clint().uptime()
    }

    fn uptime_nano() -> u128 {
        INTERRUPT_CONTROLLER.clint().uptime_nano()
    }

    fn cpu_counter() -> u64 {
        super::cpu_counter()
    }
}
