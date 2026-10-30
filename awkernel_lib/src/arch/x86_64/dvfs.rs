use crate::{
    delay::wait_millisec,
    dvfs::{Dvfs, Error, Result},
};

use super::{
    cpuid::{freq_info, tpm_info},
    hwp,
    msr::{
        ia32_hwp_capabilities, ia32_hwp_request, ia32_misc_enable, ia32_perf_aperf, ia32_perf_ctl,
        ia32_perf_mperf, platform_info,
    },
    X86,
};

/// Tries to enable HWP for the package. Enabling HWP cannot be undone until reset.
///
/// # Safety
///
/// Call once on the primary CPU before any CPU calls [`init_non_primary`].
pub unsafe fn init_primary() -> Result<()> {
    // Safety: This is called on primary CPU during boot
    if unsafe { !hwp::enable() } {
        log::warn!("[DVFS] Failed to enable HWP. Using non-HWP mode.");
    }
    init_non_primary()
}

/// If HWP is not enabled, this will enable Enhanced SpeedStep.
/// Then, it sets the CPU to its minimum frequency.
pub fn init_non_primary() -> Result<()> {
    if !hwp::is_enabled() {
        let mut misc_enable = ia32_misc_enable::try_read().ok_or(Error::NotSupported)?;
        misc_enable.set_enhanced_speedstep_enable(true);
        // Safety: MSR IA32_MISC_ENABLE exists
        unsafe { ia32_misc_enable::write_unchecked(misc_enable) };
    }
    X86::set_min_cpu_freq()
}

/// Round `freq` up to a multiple of the bus clock, so the CPU is never below the request.
#[inline]
fn ceil_to_bus_ratio(freq: u64) -> u64 {
    let bus_hz = freq_info::bus_freq_hz();
    freq.div_ceil(bus_hz) * bus_hz
}

/// Round `freq` down to a multiple of the bus clock.
#[inline]
fn floor_to_bus_ratio(freq: u64) -> u64 {
    let bus_hz = freq_info::bus_freq_hz();
    freq / bus_hz * bus_hz
}

impl Dvfs for X86 {
    /// Fix the frequency of the current CPU.
    fn set_cpu_freq(freq: u64) -> Result<()> {
        // First we check the frequency is within the CPU's range
        let min = Self::get_min_cpu_freq()?;
        let max = Self::get_max_cpu_freq()?;
        if freq < min || freq > max {
            return Err(Error::FrequencyOutOfRange {
                requested: freq,
                min,
                max,
            });
        }
        // Round the frequency to a bus ratio, so the CPU is never below the request
        let freq = ceil_to_bus_ratio(freq).min(max);

        // If HWP is enabled, we use IA32_HWP_REQUEST to set the frequency
        if hwp::is_enabled() {
            // Rounds up, so the CPU is never below the target.
            let ratio = ia32_hwp_capabilities::hz_to_ratio(freq);

            // Minimum[7:0] = Maximum[15:8] = perf; Desired[23:16] = 0 and EPP[31:24] = 0.
            let mut request = ia32_hwp_request::Ia32HwpRequest::new(0);
            request.set_min_perf_ratio(ratio);
            request.set_max_perf_ratio(ratio);

            // Safety: HWP is enabled, so IA32_HWP_REQUEST exists.
            unsafe { ia32_hwp_request::write_unchecked(request) };

            return Ok(());
        }
        // When HWP is not enabled, we try to use IA32_PERF_CTL
        if let Some(mut perf_ctl) = ia32_perf_ctl::try_read() {
            let ratio = ia32_perf_ctl::hz_to_ratio(freq);
            perf_ctl.set_ida_engage(true);
            perf_ctl.set_performance_state_freq(ratio);
            // Safety: IA32_PERF_CTL exists if Enhanced SpeedStep is supported.
            unsafe { ia32_perf_ctl::write_unchecked(perf_ctl) };
            return Ok(());
        }
        Err(Error::NotSupported)
    }

    /// Get the maximum (non-turbo) frequency of the current CPU.
    fn get_max_cpu_freq() -> Result<u64> {
        // If HWP is enabled, we use IA32_HWP_CAPABILITIES to get the guaranteed performance
        if hwp::is_enabled() {
            // Safety: HWP is enabled, so IA32_HWP_CAPABILITIES exists.
            let caps = unsafe { ia32_hwp_capabilities::read_unchecked() };
            return caps
                .guaranteed_perf_freq_hz()
                .map(floor_to_bus_ratio)
                .ok_or(Error::NotSupported);
        }
        // If HWP is not enabled, we try with the CPUID.16H
        if let Some(freq_hz) = freq_info::base_freq_hz() {
            return Ok(freq_hz);
        }
        // If CPUID.16H not supported, we try with the MSR_PLATFORM_INFO
        platform_info::try_read()
            .and_then(|info| info.max_nonturbo_freq_hz())
            .ok_or(Error::NotSupported)
    }

    /// Get the minimum frequency of the current CPU.
    fn get_min_cpu_freq() -> Result<u64> {
        // We only rely on the platform info MSR to get the minimum frequency.
        platform_info::try_read()
            .and_then(|info| {
                // First, try to get the minimum operating frequency.
                info.min_operating_freq_hz()
                    // If not available, fall back to the maximum efficiency frequency.
                    .or_else(|| info.max_efficiency_freq_hz())
            })
            .map(|freq| ceil_to_bus_ratio(freq as u64))
            .ok_or(Error::NotSupported)
    }

    /// Get the current frequency of the current CPU.
    fn get_curr_cpu_freq() -> Result<u64> {
        // First we check that MPERF and APERF are supported
        if !tpm_info::hw_coord_feedback_supported() {
            return Err(Error::NotSupported);
        }
        // MPERF ticks at the package base frequency, not at the per-core maximum.
        let base_freq_hz = freq_info::base_freq_hz().ok_or(Error::NotSupported)?;

        // Safety: xPERF MSRs exist on CPUs that support HWP and HW feedback.
        unsafe {
            ia32_perf_mperf::write_unchecked(0);
            ia32_perf_aperf::write_unchecked(0);
        }
        wait_millisec(10);
        // Safety: xPERF MSRs exist on CPUs that support HWP and HW feedback.
        let (mperf_delta, aperf_delta) = unsafe {
            (
                ia32_perf_mperf::read_unchecked(),
                ia32_perf_aperf::read_unchecked(),
            )
        };
        if mperf_delta == 0 {
            return Err(Error::InternalError);
        }

        let freq = (aperf_delta as u128 * base_freq_hz as u128) / mperf_delta as u128;
        Ok(freq as u64)
    }
}
