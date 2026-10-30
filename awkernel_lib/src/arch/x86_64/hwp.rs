//! Pin the calling CPU's frequency with Intel HWP (Speed Shift).
//!
//! The pinned level is the core's own *Guaranteed Performance* from
//! `IA32_HWP_CAPABILITIES`: the highest level the core can sustain within the
//! package's power/thermal budget, i.e. its base frequency with turbo excluded.
//! Each core reports its own value, so on hybrid processors P-cores and E-cores
//! are each pinned to their own base clock.

use super::cpuid::tpm_info;
use super::msr::{ia32_hwp_capabilities, ia32_hwp_request, ia32_perf_aperf, ia32_pm_enable};

pub fn is_supported() -> bool {
    tpm_info::try_read().map_or(false, |tpm| tpm.hwp())
}

pub fn is_enabled() -> bool {
    is_supported() && ia32_pm_enable::hwp_enabled()
}

pub unsafe fn enable() -> bool {
    is_supported() && ia32_pm_enable::enable_hwp()
}

/// `IA32_HWP_CAPABILITIES` of one logical CPU, in HWP performance units.
#[derive(Debug, Clone, Copy)]
pub struct HwpLevels {
    pub highest: u8,
    pub guaranteed: u8,
    pub lowest: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwpError {
    /// CPUID.06H:EAX[7] is clear (e.g. under QEMU).
    NotSupported,
    MsrFault,
}

/// Enable HWP on the calling CPU and request min = max = guaranteed performance.
///
/// Enabling HWP cannot be undone until reset, which is fine for a kernel that
/// owns the machine for the whole boot.
pub fn pin_to_guaranteed() -> Result<HwpLevels, HwpError> {
    if !is_supported() {
        return Err(HwpError::NotSupported);
    }

    let _int_guard = crate::interrupt::InterruptGuard::new();

    // Safety: HWP support was verified, and enabling it is the requested operation.
    if !unsafe { enable() } {
        return Err(HwpError::MsrFault);
    }

    let caps = ia32_hwp_capabilities::try_read().ok_or(HwpError::MsrFault)?;
    let highest = caps.highest_perf_ratio().ok_or(HwpError::MsrFault)?;
    let guaranteed = caps.guaranteed_perf_ratio().ok_or(HwpError::MsrFault)?;
    let lowest = caps.lowest_perf_ratio().ok_or(HwpError::MsrFault)?;

    // Minimum[7:0] = Maximum[15:8] = guaranteed; Desired[23:16] = 0 (autonomous
    // within [min, max], which is a single point); EPP[31:24] = 0; activity
    // window and package control left 0.
    let mut request = ia32_hwp_request::Ia32HwpRequest::new(0);
    request.set_min_perf_ratio(guaranteed);
    request.set_max_perf_ratio(guaranteed);
    if !ia32_hwp_request::try_write(request) {
        return Err(HwpError::MsrFault);
    }

    Ok(HwpLevels {
        highest,
        guaranteed,
        lowest,
    })
}

/// Measure the calling CPU's actual clock over `window_us` of busy spinning.
///
/// APERF counts actual core cycles while the core is in C0, which a spin keeps
/// it in, so cycles / elapsed microseconds is the running frequency in MHz.
pub fn measure_freq_mhz(window_us: u64) -> Option<u64> {
    let _int_guard = crate::interrupt::InterruptGuard::new();

    let start_us = crate::delay::uptime();
    let start_cycles = ia32_perf_aperf::try_read()?;
    crate::delay::wait_microsec(window_us);
    let end_cycles = ia32_perf_aperf::try_read()?;
    let elapsed_us = crate::delay::uptime().saturating_sub(start_us);

    Some(end_cycles.wrapping_sub(start_cycles) / elapsed_us.max(1))
}
