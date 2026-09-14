use super::{ArchImpl, INTERRUPT_CONTROLLER};
use crate::delay::Delay;

impl Delay for ArchImpl {
    #[inline]
    fn wait_interrupt() {
        super::wait_interrupt();
    }

    #[inline]
    fn wait_microsec(usec: u64) {
        INTERRUPT_CONTROLLER.clint().wait_microsec(usec);
    }

    #[inline]
    fn uptime() -> u64 {
        INTERRUPT_CONTROLLER.clint().uptime()
    }

    #[inline]
    fn uptime_nano() -> u128 {
        INTERRUPT_CONTROLLER.clint().uptime_nano()
    }

    #[inline]
    fn cpu_counter() -> u64 {
        super::cpu_counter()
    }
}
