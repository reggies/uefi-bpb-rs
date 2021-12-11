DefinitionBlock (
    "ssdt_bpb.aml",                                       // Output filename
    "SSDT",                                               // Signature
    2,                                                    // DSDT Compliance Revision
    "OEM",                                                // OEM ID
    "BPBT0",                                              // OEM Table ID
    0x4                                                   // OEM Revision
    )
{
    Scope (\_SB)
    {
        Device (BPB0)
        {
            Name (_ADR, 0)                                // _ADR: Required but not used
            Name (_UID, 1)
            Name (_HID, "BPB0001")                        // _HID: Vendor-defined device
            Name (_CRS, ResourceTemplate ()               // _CRS: Current Resource Settings
            {
                Memory32Fixed (
                    ReadWrite,
                    0x11223344,
                    0x00001000
                )
            })
        }
    }
}
