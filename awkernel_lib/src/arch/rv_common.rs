use crate::{cpu::num_cpu, interrupt::InterruptController};
use riscv::{
    asm::wfi,
    register::{cycle, mie},
};

#[cfg(feature = "rv32")]
use super::rv32::RV32 as ArchImpl;
#[cfg(feature = "rv64")]
use super::rv64::RV64 as ArchImpl;

#[cfg(feature = "rv32")]
use super::rv32::plic_get_context_id;
#[cfg(feature = "rv64")]
use super::rv64::plic_get_context_id;

pub mod clint;
pub mod plic;

pub(super) mod delay;
pub(super) mod interrupt;

// TODO: get info from device tree
pub(super) static INTERRUPT_CONTROLLER: RiscvInterruptController = RiscvInterruptController {
    clint: clint::Clint {
        base_addr: 0x0200_0000,
        mtime_freq: 10_000_000,
    },
    plic: plic::Plic {
        base_addr: 0x0c00_0000,
        max_priority: 7,
        num_sources: 53,
        num_contexts: 16,
    },
};

/// Assembly instruction to wait for interrupt.
#[inline]
pub fn wait_interrupt() {
    wfi();
}

/// Get the CPU counter value.
#[inline]
pub fn cpu_counter() -> u64 {
    cycle::read64()
}

/// Enable software interrupts for the current hart.
///
/// # Safety
///
/// Call only once during kernel initialization.
#[inline]
pub unsafe fn enable_software_interrutps() {
    unsafe { mie::set_msoft() };
}
pub struct RiscvInterruptController {
    clint: clint::Clint,
    plic: plic::Plic,
}

impl RiscvInterruptController {
    #[inline]
    const fn clint(&self) -> &clint::Clint {
        &self.clint
    }

    #[inline]
    const fn plic(&self) -> &plic::Plic {
        &self.plic
    }
}

impl InterruptController for RiscvInterruptController {
    #[inline]
    fn enable_irq(&mut self, irq: u16) {
        let context = plic_get_context_id();
        self.plic.enable_interrupt(context, irq);
    }

    #[inline]
    fn disable_irq(&mut self, irq: u16) {
        let context = plic_get_context_id();
        self.plic.disable_interrupt(context, irq);
    }

    #[inline]
    fn pending_irqs(&self) -> alloc::boxed::Box<dyn Iterator<Item = u16>> {
        todo!()
    }

    #[inline]
    fn send_ipi(&mut self, _irq: u16, cpu_id: u32) {
        if cpu_id <= num_cpu() as u32 {
            unsafe { self.clint.soft_interrupt(cpu_id as _) };
        }
    }

    #[inline]
    fn send_ipi_broadcast(&mut self, _irq: u16) {
        self.clint.soft_interrupt_broadcast();
    }

    #[inline]
    fn send_ipi_broadcast_without_self(&mut self, _irq: u16) {
        self.clint.soft_interrupt_broadcast_without_self();
    }

    #[inline]
    fn irq_range(&self) -> (u16, u16) {
        (1, self.plic.num_sources + 1)
    }

    #[inline]
    fn irq_range_for_pnp(&self) -> (u16, u16) {
        todo!()
    }
}
