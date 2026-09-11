use crate::cpu::num_cpu;
use riscv::register::mhartid;

pub struct Clint {
    pub(super) base_addr: usize,
    pub(super) mtime_freq: usize,
}

impl Clint {
    const MSWI_BASE: usize = 0x0000;
    const MTIMECMP_BASE: usize = 0x4000;
    const MTIME_OFFSET: usize = 0xbff8;

    #[inline]
    const fn msip(&self, hart_id: usize) -> *mut u32 {
        (self.base_addr + Self::MSWI_BASE + hart_id * 4) as *mut u32
    }

    #[inline]
    const fn mtime(&self) -> *const u64 {
        (self.base_addr + Self::MTIME_OFFSET) as *const u64
    }

    #[inline]
    pub fn wait_microsec(&self, usec: u64) {
        let mtime = self.mtime();
        let end = unsafe { *mtime + ((self.mtime_freq as u64 / 1000) * usec) / 1000 };
        while unsafe { core::ptr::read_volatile(mtime) } < end {}
    }

    #[inline]
    pub fn uptime(&self) -> u64 {
        // as microsec
        let mtime = self.mtime();
        unsafe { *mtime * 1_000_000 / self.mtime_freq as u64 }
    }

    #[inline]
    pub fn uptime_nano(&self) -> u128 {
        // as nano sec
        let mtime = self.mtime() as *const u128;
        unsafe { *mtime * 1_000_000_000 / self.mtime_freq as u128 }
    }

    /// Trigger a software interrupt to the specified hart.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the hart_id is valid.
    #[inline]
    pub unsafe fn soft_interrupt(&self, hart_id: usize) {
        unsafe { core::ptr::write_volatile(self.msip(hart_id), 1) };
    }

    pub fn soft_interrupt_broadcast(&self) {
        for hart_id in 0..=num_cpu() {
            unsafe { self.soft_interrupt(hart_id) };
        }
    }

    pub fn soft_interrupt_broadcast_without_self(&self) {
        let self_hart_id = mhartid::read();
        for hart_id in 0..=num_cpu() {
            if hart_id != self_hart_id {
                unsafe { self.soft_interrupt(hart_id) };
            }
        }
    }
}
