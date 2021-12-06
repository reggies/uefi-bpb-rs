pub const ACPI_1_FADT_SIGNATURE: u32 = 0x50434146;
pub const ACPI_1_FADT_REVISION: u8 = 0x01;

pub const ACPI_3_FADT_SIGNATURE: u32 = 0x50434146;
pub const ACPI_3_FADT_REVISION: u8 = 0x04;

/// RSD_PTR Revision
pub const ACPI_1_RSDP_REVISION: u8 = 0x01;
pub const ACPI_2_RSDP_REVISION: u8 = 0x02;

/// The common ACPI description table header. This
/// structure prefaces most ACPI tables.
#[repr(C, packed)]
pub struct DescriptionHeader {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: u64,
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32
}

/// Root System Description Pointer Structure
#[repr(C, packed)]
pub struct RootSystemDescriptionPointer3 {
    pub signature: u64,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3]
}

/// ACPI 3.0 Generic Address Space Address IDs
pub const ACPI_3_SYSTEM_MEMORY: u8 = 0x0;
pub const ACPI_3_SYSTEM_IO: u8 = 0x1;
pub const ACPI_3_PCI_CONFIGURATION_SPACE: u8 = 0x2;
pub const ACPI_3_EMBEDDED_CONTROLLER: u8 = 0x3;
pub const ACPI_3_SMBUS: u8 = 0x4;
pub const ACPI_3_FUNCTIONAL_FIXED_HARDWARE: u8 = 0x7f;

/// ACPI 3.0 Generic Address Space definition
#[repr(C, packed)]
pub struct GenericAddressSpace {
    pub address_space_id: u8,
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub access_size: u8,
    pub address: u64,
}

impl GenericAddressSpace {
    pub fn new() -> GenericAddressSpace {
        GenericAddressSpace {
            address_space_id: 0,
            register_bit_width: 0,
            register_bit_offset: 0,
            access_size: 0,
            address: 0,
        }
    }
}

/// Fixed ACPI Description Table Structure (FADT).
#[repr(C, packed)]
pub struct FixedDescriptionTable3 {
    pub header: DescriptionHeader,
    // Physical memory address of the FACS, where OSPM and
    // Firmware exchange control information. See Section
    // 5.2.6, “Root System Description Table,” for a
    // description of the FACS. If the X_FIRMWARE_CTRL field
    // contains a non zero value then this field must be
    // zero. A zero value indicates that no FACS is
    // specified by this field
    pub firmware_ctrl: u32,
    // Physical memory address (0-4 GB) of the DSDT.
    pub dsdt: u32,
    // ACPI 1.0 defined this offset as a field named
    // INT_MODEL, which was eliminated in ACPI
    // 2.0. Platforms should set this field to zero but
    // field values of one are also allowed to maintain
    // compatibility with ACPI 1.0.
    pub reserved0: u8,
    // This field is set by the OEM to convey the preferred
    // power management profile to OSPM. OSPM can use this
    // field to set default power management policy
    // parameters during OS installation.
    pub preferred_pm_profile: u8,
    // System vector the SCI interrupt is wired to in 8259
    // mode. On systems that do not contain the 8259, this
    // field contains the Global System interrupt number of
    // the SCI interrupt. OSPM is required to treat the ACPI
    // SCI interrupt as a sharable, level, active low
    // interrupt.
    pub sci_int: u16,
    // System port address of the SMI Command Port. During
    // ACPI OS initialization, OSPM can determine that the
    // ACPI hardware registers are owned by SMI (by way of
    // the SCI_EN bit), in which case the ACPI OS issues the
    // ACPI_ENABLE command to the SMI_CMD port. The SCI_EN
    // bit effectively tracks the ownership of the ACPI
    // hardware registers. OSPM issues commands to the
    // SMI_CMD port synchronously from the boot
    // processor. This field is reserved and must be zero on
    // system that does not support System Management mode
    pub smi_cmd: u32,
    // The value to write to SMI_CMD to disable SMI
    // ownership of the ACPI hardware registers. The last
    // action SMI does to relinquish ownership is to set the
    // SCI_EN bit. During the OS initialization process,
    // OSPM will synchronously wait for the transfer of SMI
    // ownership to complete, so the ACPI system releases
    // SMI ownership as quickly as possible. This field is
    // reserved and must be zero on systems that do not
    // support Legacy Mode.
    pub acpi_enable: u8,
    // The value to write to SMI_CMD to re-enable SMI
    // ownership of the ACPI hardware registers. This can
    // only be done when ownership was originally acquired
    // from SMI by OSPM using ACPI_ENABLE. An OS can hand
    // ownership back to SMI by relinquishing use to the
    // ACPI hardware registers, masking off all SCI
    // interrupts, clearing the SCI_EN bit and then writing
    // ACPI_DISABLE to the SMI_CMD port from the boot
    // processor. This field is reserved and must be zero on
    // systems that do not support Legacy Mode
    pub acpi_disable: u8,
    // The value to write to SMI_CMD to enter the S4BIOS
    // state.  The S4BIOS state provides an alternate way to
    // enter the S4 state where the firmware saves and
    // restores the memory context. A value of zero in
    // S4BIOS_F indicates S4BIOS_REQ is not supported. (See
    // Table 5-38)
    pub s4_bios_req: u8,
    // If non-zero, this field contains the value OSPM
    // writes to the SMI_CMD register to assume processor
    // performance state control responsibility.
    pub pstate_cnt: u8,
    // System port address of the PM1a Event Register
    // Block. See Section 4.8.3.1, “PM1 Event Grouping,” for
    // a hardware description layout of this register
    // block. This is a required field. This field is
    // superseded by the X_PM1a_EVT_BLK field.
    pub pm1a_evt_blk: u32,
    // System port address of the PM1b Event Register
    // Block. See Section 4.8.3.1, “PM1 Event Grouping,” for
    // a hardware description layout of this register
    // block. This field is optional; if this register block
    // is not supported, this field contains zero.  This
    // field is superseded by the X_PM1b_EVT_BLK field.
    pub pm1b_evt_blk: u32,
    // System port address of the PM1a Control Register
    // Block. See Section 4.8.3.2, “PM1 Control Grouping,”
    // for a hardware description layout of this register
    // block. This is a required field. This field is
    // superseded by the X_PM1a_CNT_BLK field.
    pub pm1a_cnt_blk: u32,
    // System port address of the PM1b Control Register
    // Block. See Section 4.8.3.2, “PM1 Control Grouping,”
    // for a hardware description layout of this register
    // block. This field is optional; if this register block
    // is not supported, this field contains zero.  This
    // field is superseded by the X_PM1b_CNT_BLK field.
    pub pm1b_cnt_blk: u32,
    // System port address of the PM2 Control Register
    // Block. See Section 4.8.3.4, “PM2 Control (PM2_CNT),”
    // for a hardware description layout of this register
    // block. This field is optional; if this register block
    // is not supported, this field contains zero.  This
    // field is superseded by the X_PM2_CNT_BLK field.
    pub pm2_cnt_blk: u32,
    // System port address of the Power Management Timer
    // Control Register Block. See Section 4.8.3.3, “Power
    // Management Timer (PM_TMR),” for a hardware
    // description layout of this register block. This is an
    // optional field; if this register block is not
    // supported, this field contains zero. This field is
    // superseded by the X_PM_TMR_BLK field.
    pub pm_tmr_blk: u32,
    // System port address of General-Purpose Event 0
    // Register Block. See Section 4.8.4.1, “General-Purpose
    // Event Register Blocks,” for a hardware description of
    // this register block. This is an optional field; if
    // this register block is not supported, this field
    // contains zero. This field is superseded by the
    // X_GPE0_BLK field.
    pub gpe0_blk: u32,
    // System port address of General-Purpose Event 1
    // Register Block. See Section 4.8.4.1, “General-Purpose
    // Event Register Blocks,” for a hardware description of
    // this register block. This is an optional field; if
    // this register block is not supported, this field
    // contains zero. This field is superseded by the
    // X_GPE1_BLK field.
    pub gpe1_blk: u32,
    // Number of bytes decoded by PM1a_EVT_BLK and, if
    // supported, PM1b_EVT_BLK. This value is >= 4.
    pub pm1_evt_len: u8,
    // Number of bytes decoded by PM1a_CNT_BLK and, if
    // supported, PM1b_CNT_BLK. This value is >= 2.
    pub pm1_cnt_len: u8,
    // Number of bytes decoded by PM2_CNT_BLK. Support for
    // the PM2 register block is optional. If supported,
    // this value is >= 1. If not supported, this field
    // contains zero.
    pub pm2_cnt_len: u8,
    // Number of bytes decoded by PM_TMR_BLK. If the PM
    // Timer is supported, this field’s value must be 4. If
    // not supported, this field contains zero.
    pub pm_tm_len: u8,
    // Number of bytes decoded by GPE0_BLK. The value is a
    // non- negative multiple of 2.
    pub gpe0_blk_len: u8,
    // Number of bytes decoded by GPE1_BLK. The value is a
    // non- negative multiple of 2.
    pub gpe1_blk_len: u8,
    // Offset within the ACPI general-purpose event model
    // where GPE1 based events start.
    pub gpe1_base: u8,
    // If non-zero, this field contains the value OSPM
    // writes to the SMI_CMD register to indicate OS support
    // for the _CST object and C States Changed
    // notification.
    pub cst_cnt: u8,
    // The worst-case hardware latency, in microseconds, to
    // enter and exit a C2 state. A value > 100 indicates
    // the system does not support a C2 state.
    pub p_lvl2_lat: u16,
    // The worst-case hardware latency, in microseconds, to
    // enter and exit a C3 state. A value > 1000 indicates
    // the system does not support a C3 state.
    pub p_lvl3_lat: u16,
    // If WBINVD=0, the value of this field is the number of
    // flush strides that need to be read (using cacheable
    // addresses) to completely flush dirty lines from any
    // processor’s memory caches. Notice that the value in
    // FLUSH_STRIDE is typically the smallest cache line
    // width on any of the processor’s caches (for more
    // information, see the FLUSH_STRIDE field
    // definition). If the system does not support a method
    // for flushing the processor’s caches, then FLUSH_SIZE
    // and WBINVD are set to zero. Notice that this method
    // of flushing the processor caches has limitations, and
    // WBINVD=1 is the preferred way to flush the processors
    // caches. This value is typically at least 2 times the
    // cache size. The maximum allowed value for FLUSH_SIZE
    // multiplied by FLUSH_STRIDE is 2 MB for a typical
    // maximum supported cache size of 1 MB. Larger cache
    // sizes are supported using WBINVD=1.  This value is
    // ignored if WBINVD=1.  This field is maintained for
    // ACPI 1.0 processor compatibility on existing
    // systems. Processors in new ACPI-compatible systems
    // are required to support the WBINVD function and
    // indicate this to OSPM by setting the WBINVD field =
    // 1.
    pub flush_size: u16,
    // If WBINVD=0, the value of this field is the cache
    // line width, in bytes, of the processor’s memory
    // caches. This value is typically the smallest cache
    // line width on any of the processor’s caches. For more
    // information, see the description of the FLUSH_SIZE
    // field.  This value is ignored if WBINVD=1.  This
    // field is maintained for ACPI 1.0 processor
    // compatibility on existing systems. Processors in new
    // ACPI-compatible systems are required to support the
    // WBINVD function and indicate this to OSPM by setting
    // the WBINVD field = 1.
    pub flush_stride: u16,
    // The zero-based index of where the processor’s duty
    // cycle setting is within the processor’s P_CNT
    // register.
    pub duty_offset: u8,
    // The bit width of the processor’s duty cycle setting
    // value in the P_CNT register. Each processor’s duty
    // cycle setting allows the software to select a nominal
    // processor frequency below its absolute frequency as
    // defined by: THTL_EN = 1 BF * DC/(2 DUTY_WIDTH )
    // Where: BF–Base frequency DC–Duty cycle setting When
    // THTL_EN is 0, the processor runs at its absolute
    // BF. A DUTY_WIDTH value of 0 indicates that processor
    // duty cycle is not supported and the processor
    // continuously runs at its base frequency.
    pub duty_width: u8,
    // The RTC CMOS RAM index to the day-of-month alarm
    // value.  If this field contains a zero, then the RTC
    // day of the month alarm feature is not supported. If
    // this field has a non-zero value, then this field
    // contains an index into RTC RAM space that OSPM can
    // use to program the day of the month alarm.  See
    // Section 4.8.2.4 “Real Time Clock Alarm,” for a
    // description of how the hardware works.
    pub day_alrm: u8,
    // The RTC CMOS RAM index to the month of year alarm
    // value.  If this field contains a zero, then the RTC
    // month of the year alarm feature is not supported. If
    // this field has a non-zero value, then this field
    // contains an index into RTC RAM space that OSPM can
    // use to program the month of the year alarm. If this
    // feature is supported, then the DAY_ALRM feature must
    // be supported also.
    pub mon_alrm: u8,
    // The RTC CMOS RAM index to the century of data value
    // (hundred and thousand year decimals). If this field
    // contains a zero, then the RTC centenary feature is
    // not supported. If this field has a non-zero value,
    // then this field contains an index into RTC RAM space
    // that OSPM can use to program the centenary field.
    pub century: u8,
    // IA-PC Boot Architecture Flags. See Table 5-36 for a
    // description of this field.
    pub iapc_boot_arch: u16,
    // Must be 0.
    pub reserved1: u8,
    // Fixed feature flags. See Table 5-35 for a description
    // of this field.
    pub flags: u32,
    pub reset_reg: GenericAddressSpace,
    pub reset_value: u8,
    pub reserved2: u8,
    pub reserved3: u8,
    pub reserved4: u8,
    pub x_firmware_ctrl: u64,
    pub x_dsdt: u64,
    pub x_pm1a_evt_blk: GenericAddressSpace,
    pub x_pm1b_evt_blk: GenericAddressSpace,
    pub x_pm1a_cnt_blk: GenericAddressSpace,
    pub x_pm1b_cnt_blk: GenericAddressSpace,
    pub x_pm2_cnt_blk: GenericAddressSpace,
    pub x_pm_tmr_blk: GenericAddressSpace,
    pub x_gpe0_blk: GenericAddressSpace,
    pub x_gpe1_blk: GenericAddressSpace,
}

/// Fixed ACPI Description Table Structure (FADT).
#[repr(C, packed)]
pub struct FixedDescriptionTable1 {
    pub header: DescriptionHeader,
    // Physical memory address of the FACS, where OSPM and
    // Firmware exchange control information. See Section
    // 5.2.6, “Root System Description Table,” for a
    // description of the FACS. If the X_FIRMWARE_CTRL field
    // contains a non zero value then this field must be
    // zero. A zero value indicates that no FACS is
    // specified by this field
    pub firmware_ctrl: u32,
    // Physical memory address (0-4 GB) of the DSDT.
    pub dsdt: u32,
    // ACPI 1.0 defined this offset as a field named
    // INT_MODEL, which was eliminated in ACPI
    // 2.0. Platforms should set this field to zero but
    // field values of one are also allowed to maintain
    // compatibility with ACPI 1.0.
    pub int_model: u8,
    // Must be zero.
    pub reserved0: u8,
    // System vector the SCI interrupt is wired to in 8259
    // mode. On systems that do not contain the 8259, this
    // field contains the Global System interrupt number of
    // the SCI interrupt. OSPM is required to treat the ACPI
    // SCI interrupt as a sharable, level, active low
    // interrupt.
    pub sci_int: u16,
    // System port address of the SMI Command Port. During
    // ACPI OS initialization, OSPM can determine that the
    // ACPI hardware registers are owned by SMI (by way of
    // the SCI_EN bit), in which case the ACPI OS issues the
    // ACPI_ENABLE command to the SMI_CMD port. The SCI_EN
    // bit effectively tracks the ownership of the ACPI
    // hardware registers. OSPM issues commands to the
    // SMI_CMD port synchronously from the boot
    // processor. This field is reserved and must be zero on
    // system that does not support System Management mode
    pub smi_cmd: u32,
    // The value to write to SMI_CMD to disable SMI
    // ownership of the ACPI hardware registers. The last
    // action SMI does to relinquish ownership is to set the
    // SCI_EN bit. During the OS initialization process,
    // OSPM will synchronously wait for the transfer of SMI
    // ownership to complete, so the ACPI system releases
    // SMI ownership as quickly as possible. This field is
    // reserved and must be zero on systems that do not
    // support Legacy Mode.
    pub acpi_enable: u8,
    // The value to write to SMI_CMD to re-enable SMI
    // ownership of the ACPI hardware registers. This can
    // only be done when ownership was originally acquired
    // from SMI by OSPM using ACPI_ENABLE. An OS can hand
    // ownership back to SMI by relinquishing use to the
    // ACPI hardware registers, masking off all SCI
    // interrupts, clearing the SCI_EN bit and then writing
    // ACPI_DISABLE to the SMI_CMD port from the boot
    // processor. This field is reserved and must be zero on
    // systems that do not support Legacy Mode
    pub acpi_disable: u8,
    // The value to write to SMI_CMD to enter the S4BIOS
    // state.  The S4BIOS state provides an alternate way to
    // enter the S4 state where the firmware saves and
    // restores the memory context. A value of zero in
    // S4BIOS_F indicates S4BIOS_REQ is not supported. (See
    // Table 5-38)
    pub s4_bios_req: u8,
    // Must be zero.
    pub reserved1: u8,
    // System port address of the PM1a Event Register
    // Block. See Section 4.8.3.1, “PM1 Event Grouping,” for
    // a hardware description layout of this register
    // block. This is a required field. This field is
    // superseded by the X_PM1a_EVT_BLK field.
    pub pm1a_evt_blk: u32,
    // System port address of the PM1b Event Register
    // Block. See Section 4.8.3.1, “PM1 Event Grouping,” for
    // a hardware description layout of this register
    // block. This field is optional; if this register block
    // is not supported, this field contains zero.  This
    // field is superseded by the X_PM1b_EVT_BLK field.
    pub pm1b_evt_blk: u32,
    // System port address of the PM1a Control Register
    // Block. See Section 4.8.3.2, “PM1 Control Grouping,”
    // for a hardware description layout of this register
    // block. This is a required field. This field is
    // superseded by the X_PM1a_CNT_BLK field.
    pub pm1a_cnt_blk: u32,
    // System port address of the PM1b Control Register
    // Block. See Section 4.8.3.2, “PM1 Control Grouping,”
    // for a hardware description layout of this register
    // block. This field is optional; if this register block
    // is not supported, this field contains zero.  This
    // field is superseded by the X_PM1b_CNT_BLK field.
    pub pm1b_cnt_blk: u32,
    // System port address of the PM2 Control Register
    // Block. See Section 4.8.3.4, “PM2 Control (PM2_CNT),”
    // for a hardware description layout of this register
    // block. This field is optional; if this register block
    // is not supported, this field contains zero.  This
    // field is superseded by the X_PM2_CNT_BLK field.
    pub pm2_cnt_blk: u32,
    // System port address of the Power Management Timer
    // Control Register Block. See Section 4.8.3.3, “Power
    // Management Timer (PM_TMR),” for a hardware
    // description layout of this register block. This is an
    // optional field; if this register block is not
    // supported, this field contains zero. This field is
    // superseded by the X_PM_TMR_BLK field.
    pub pm_tmr_blk: u32,
    // System port address of General-Purpose Event 0
    // Register Block. See Section 4.8.4.1, “General-Purpose
    // Event Register Blocks,” for a hardware description of
    // this register block. This is an optional field; if
    // this register block is not supported, this field
    // contains zero. This field is superseded by the
    // X_GPE0_BLK field.
    pub gpe0_blk: u32,
    // System port address of General-Purpose Event 1
    // Register Block. See Section 4.8.4.1, “General-Purpose
    // Event Register Blocks,” for a hardware description of
    // this register block. This is an optional field; if
    // this register block is not supported, this field
    // contains zero. This field is superseded by the
    // X_GPE1_BLK field.
    pub gpe1_blk: u32,
    // Number of bytes decoded by PM1a_EVT_BLK and, if
    // supported, PM1b_EVT_BLK. This value is >= 4.
    pub pm1_evt_len: u8,
    // Number of bytes decoded by PM1a_CNT_BLK and, if
    // supported, PM1b_CNT_BLK. This value is >= 2.
    pub pm1_cnt_len: u8,
    // Number of bytes decoded by PM2_CNT_BLK. Support for
    // the PM2 register block is optional. If supported,
    // this value is >= 1. If not supported, this field
    // contains zero.
    pub pm2_cnt_len: u8,
    // Number of bytes decoded by PM_TMR_BLK. If the PM
    // Timer is supported, this field’s value must be 4. If
    // not supported, this field contains zero.
    pub pm_tm_len: u8,
    // Number of bytes decoded by GPE0_BLK. The value is a
    // non- negative multiple of 2.
    pub gpe0_blk_len: u8,
    // Number of bytes decoded by GPE1_BLK. The value is a
    // non- negative multiple of 2.
    pub gpe1_blk_len: u8,
    // Offset within the ACPI general-purpose event model
    // where GPE1 based events start.
    pub gpe1_base: u8,
    // Must be 0.
    pub reserved2: u8,
    // The worst-case hardware latency, in microseconds, to
    // enter and exit a C2 state. A value > 100 indicates
    // the system does not support a C2 state.
    pub p_lvl2_lat: u16,
    // The worst-case hardware latency, in microseconds, to
    // enter and exit a C3 state. A value > 1000 indicates
    // the system does not support a C3 state.
    pub p_lvl3_lat: u16,
    // If WBINVD=0, the value of this field is the number of
    // flush strides that need to be read (using cacheable
    // addresses) to completely flush dirty lines from any
    // processor’s memory caches. Notice that the value in
    // FLUSH_STRIDE is typically the smallest cache line
    // width on any of the processor’s caches (for more
    // information, see the FLUSH_STRIDE field
    // definition). If the system does not support a method
    // for flushing the processor’s caches, then FLUSH_SIZE
    // and WBINVD are set to zero. Notice that this method
    // of flushing the processor caches has limitations, and
    // WBINVD=1 is the preferred way to flush the processors
    // caches. This value is typically at least 2 times the
    // cache size. The maximum allowed value for FLUSH_SIZE
    // multiplied by FLUSH_STRIDE is 2 MB for a typical
    // maximum supported cache size of 1 MB. Larger cache
    // sizes are supported using WBINVD=1.  This value is
    // ignored if WBINVD=1.  This field is maintained for
    // ACPI 1.0 processor compatibility on existing
    // systems. Processors in new ACPI-compatible systems
    // are required to support the WBINVD function and
    // indicate this to OSPM by setting the WBINVD field =
    // 1.
    pub flush_size: u16,
    // If WBINVD=0, the value of this field is the cache
    // line width, in bytes, of the processor’s memory
    // caches. This value is typically the smallest cache
    // line width on any of the processor’s caches. For more
    // information, see the description of the FLUSH_SIZE
    // field.  This value is ignored if WBINVD=1.  This
    // field is maintained for ACPI 1.0 processor
    // compatibility on existing systems. Processors in new
    // ACPI-compatible systems are required to support the
    // WBINVD function and indicate this to OSPM by setting
    // the WBINVD field = 1.
    pub flush_stride: u16,
    // The zero-based index of where the processor’s duty
    // cycle setting is within the processor’s P_CNT
    // register.
    pub duty_offset: u8,
    // The bit width of the processor’s duty cycle setting
    // value in the P_CNT register. Each processor’s duty
    // cycle setting allows the software to select a nominal
    // processor frequency below its absolute frequency as
    // defined by: THTL_EN = 1 BF * DC/(2 DUTY_WIDTH )
    // Where: BF–Base frequency DC–Duty cycle setting When
    // THTL_EN is 0, the processor runs at its absolute
    // BF. A DUTY_WIDTH value of 0 indicates that processor
    // duty cycle is not supported and the processor
    // continuously runs at its base frequency.
    pub duty_width: u8,
    // The RTC CMOS RAM index to the day-of-month alarm
    // value.  If this field contains a zero, then the RTC
    // day of the month alarm feature is not supported. If
    // this field has a non-zero value, then this field
    // contains an index into RTC RAM space that OSPM can
    // use to program the day of the month alarm.  See
    // Section 4.8.2.4 “Real Time Clock Alarm,” for a
    // description of how the hardware works.
    pub day_alrm: u8,
    // The RTC CMOS RAM index to the month of year alarm
    // value.  If this field contains a zero, then the RTC
    // month of the year alarm feature is not supported. If
    // this field has a non-zero value, then this field
    // contains an index into RTC RAM space that OSPM can
    // use to program the month of the year alarm. If this
    // feature is supported, then the DAY_ALRM feature must
    // be supported also.
    pub mon_alrm: u8,
    // The RTC CMOS RAM index to the century of data value
    // (hundred and thousand year decimals). If this field
    // contains a zero, then the RTC centenary feature is
    // not supported. If this field has a non-zero value,
    // then this field contains an index into RTC RAM space
    // that OSPM can use to program the centenary field.
    pub century: u8,
    // Must be 0.
    pub reserved3: u8,
    // Must be 0.
    pub reserved4: u8,
    // Must be 0.
    pub reserved5: u8,
    // Fixed feature flags. See Table 5-35 for a description
    // of this field.
    pub flags: u32,
}
