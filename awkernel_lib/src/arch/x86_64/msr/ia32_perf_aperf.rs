use super::{rdmsr_safe, wrmsr_safe};
use x86_64::registers::model_specific::Msr;

const MSR_NUMBER: u32 = 0xE8;

#[inline]
pub fn try_read() -> Option<u64> {
    unsafe { rdmsr_safe(&Msr::new(MSR_NUMBER)) }
}

#[inline]
pub fn try_write(value: u64) -> bool {
    unsafe { wrmsr_safe(&mut Msr::new(MSR_NUMBER), value) }
}

#[inline]
pub unsafe fn read_unchecked() -> u64 {
    Msr::new(MSR_NUMBER).read()
}

#[inline]
pub unsafe fn write_unchecked(value: u64) {
    Msr::new(MSR_NUMBER).write(value);
}
