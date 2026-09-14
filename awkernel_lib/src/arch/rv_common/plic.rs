pub struct Plic {
    pub(crate) base_addr: usize,
    pub(crate) max_priority: u32,
    pub(crate) num_sources: u16,
    pub(crate) num_contexts: u16,
}

impl Plic {
    const PRIORITIES_BASE: usize = 0x0000;

    const PENDING_BASE: usize = 0x1000;

    const ENABLES_OFFSET: usize = 0x2000;
    const ENABLES_SEPARATION: usize = 0x80;

    const THRESHOLDS_OFFSET: usize = 0x20_0000;
    const THRESHOLDS_SEPARATION: usize = 0x1000;

    const CLAIMS_OFFSET: usize = 0x20_0004;
    const CLAIMS_SEPARATION: usize = 0x1000;

    /// Create a new PLIC instance.
    ///
    /// # Safety
    ///
    /// The caller must ensure that all the parameters are valid.
    pub const unsafe fn new(
        base_addr: usize,
        max_priority: u32,
        num_sources: u16,
        num_contexts: u16,
    ) -> Self {
        Self {
            base_addr,
            max_priority,
            num_sources,
            num_contexts,
        }
    }

    const fn priority_in_range(&self, priority: u32) -> bool {
        priority <= self.max_priority
    }

    const fn source_in_range(&self, source: u16) -> bool {
        source > 0 && source <= self.num_sources
    }

    const fn context_in_range(&self, context_id: u16) -> bool {
        context_id < self.num_contexts
    }

    const fn priority_reg(&self, source: u16) -> *mut u32 {
        (self.base_addr + Self::PRIORITIES_BASE + source as usize * 4) as _
    }

    const fn enable_reg(&self, context_id: u16, source: u16) -> *mut u32 {
        let context_id = context_id as usize;
        let reg_index = source as usize / 32;
        let base = self.base_addr + Self::ENABLES_OFFSET + context_id * Self::ENABLES_SEPARATION;
        (base + reg_index * 4) as _
    }

    const fn threshold_reg(&self, context_id: u16) -> *mut u32 {
        let context_id = context_id as usize;
        (self.base_addr + Self::THRESHOLDS_OFFSET + context_id * Self::THRESHOLDS_SEPARATION) as _
    }

    const fn claim_reg(&self, context_id: u16) -> *mut u32 {
        let context_id = context_id as usize;
        (self.base_addr + Self::CLAIMS_OFFSET + context_id * Self::CLAIMS_SEPARATION) as _
    }

    /// Set the priority of a given interrupt source.
    ///
    /// # Note
    ///
    /// If source and/or priority are out of range, this function will do nothing.
    pub fn set_priority(&self, source: u16, priority: u32) {
        if self.source_in_range(source) && self.priority_in_range(priority) {
            let reg = self.priority_reg(source);
            // Safety: source and priority are in range
            unsafe { core::ptr::write_volatile(reg, priority) };
        }
    }

    /// Enable an interrupt for a given context.
    ///
    /// # Note
    ///
    /// If context_id or source is out of range, this function will do nothing.
    pub fn enable_interrupt(&self, context_id: u16, source: u16) {
        if self.context_in_range(context_id) && self.source_in_range(source) {
            let enables_reg = self.enable_reg(context_id, source);
            let mask = 1 << (source % 32);
            // Safety: context and source are in range
            unsafe {
                let current = core::ptr::read_volatile(enables_reg);
                core::ptr::write_volatile(enables_reg, current | mask);
            }
        }
    }

    /// Disable an interrupt for a given context.
    ///
    /// # Note
    ///
    /// If context_id or source is out of range, this function will do nothing.
    pub fn disable_interrupt(&self, context_id: u16, source: u16) {
        if self.context_in_range(context_id) && self.source_in_range(source) {
            let enables_reg = self.enable_reg(context_id, source);
            let mask = !(1 << (source % 32));
            unsafe {
                let current = core::ptr::read_volatile(enables_reg);
                core::ptr::write_volatile(enables_reg, current & mask);
            }
        }
    }
}
