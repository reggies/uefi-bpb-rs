#include <initguid.h>
#include <ntddk.h>
#include <wdm.h>
#include <wdf.h>
#include <ntstrsafe.h>

#include "internal.h"
#include "payload.h"
#include "trace.h"

#include "payload.tmh"

BOOLEAN
CheckMyPage (
    PHYSICAL_ADDRESS PhysicalAddress,
    ULONG Length
    )
{
    PVOID SystemAddress;
    ULONG ProbeBytes;
    PHYSICAL_ADDRESS liPhysicalAddress;
    BOOLEAN fReturn = FALSE;

    FuncEntry(TRACE_FLAG_WDFLOADING);

    Trace(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_WDFLOADING,
        "PhysicalAddress is 0x%I64x, Length is %d",
        PhysicalAddress.QuadPart,
        Length
        );

    if (Length != 4096) {
        goto Exit;
    }

    liPhysicalAddress = PhysicalAddress;

    SystemAddress = MmMapIoSpace (
        liPhysicalAddress,
        4096,
        MmNonCached
        );

    if (!SystemAddress)
    {
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "MmMapIoSpace failed with NULL"
            );

        goto Exit;
    }

    RtlCopyMemory(&ProbeBytes, SystemAddress, sizeof(ULONG));

    Trace(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_WDFLOADING,
        "Probe bytes: %08x",
        ProbeBytes
        );

    MmUnmapIoSpace (
        SystemAddress,
        4096
        );

    fReturn = (0xFEADDEAD == ProbeBytes);

Exit:
    FuncExit(TRACE_FLAG_WDFLOADING);

    return fReturn;
}
