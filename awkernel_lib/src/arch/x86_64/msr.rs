use x86_64::registers::model_specific::Msr;

use super::fault::get_num_general_protection_fault;

pub mod ia32_energy_perf_bias;
pub mod ia32_hwp_capabilities;
pub mod ia32_hwp_interrupt;
pub mod ia32_hwp_request;
pub mod ia32_hwp_request_pkg;
pub mod ia32_hwp_status;
pub mod ia32_misc_enable;
pub mod ia32_perf_aperf;
pub mod ia32_perf_ctl;
pub mod ia32_perf_mperf;
pub mod ia32_perf_status;
pub mod ia32_pm_enable;
pub mod pkg_energy_status;
pub mod platform_info;
pub mod pp0_energy_status;
pub mod rapl_power_unit;

/// Read 64 bits msr register.
/// If the `msr` is not supported on your environments, this function returns `None`.
///
/// # Safety
///
/// The caller must ensure that this read operation has no unsafe side effects.
pub unsafe fn rdmsr_safe(msr: &Msr) -> Option<u64> {
    let _int_guard = crate::interrupt::InterruptGuard::new();

    let cpuid = crate::cpu::cpu_id();
    let num_fault1 = get_num_general_protection_fault(cpuid);

    let result = unsafe { msr.read() };

    let num_fault2 = get_num_general_protection_fault(cpuid);

    if num_fault1 != num_fault2 {
        None
    } else {
        Some(result)
    }
}

/// Write 64 bits to msr register.
/// If the `msr` is not supported on your environments, this function returns `false`.
///
/// # Safety
///
/// The caller must ensure that this write operation has no unsafe side effects.
pub unsafe fn wrmsr_safe(msr: &mut Msr, value: u64) -> bool {
    let _int_guard = crate::interrupt::InterruptGuard::new();

    let cpuid = crate::cpu::cpu_id();
    let num_fault1 = get_num_general_protection_fault(cpuid);

    unsafe { msr.write(value) };

    let num_fault2 = get_num_general_protection_fault(cpuid);

    num_fault1 == num_fault2
}
