#include <initguid.h>
#include <ntddk.h>
#include <wdm.h>
#include <wdf.h>
#include <ntstrsafe.h>

#include "common.h"
#include "payload.h"
#include "device.h"
#include "trace.h"

#include "device.tmh"

NTSTATUS
OnPrepareHardware(
    _In_  WDFDEVICE     FxDevice,
    _In_  WDFCMRESLIST  FxResourcesRaw,
    _In_  WDFCMRESLIST  FxResourcesTranslated
    )
{
    PBPB_DEVICE Device = GetDeviceContext(FxDevice);
    BOOLEAN fResourceFound = FALSE;
    NTSTATUS Status = STATUS_SUCCESS;
    ULONG ResourceCount = WdfCmResourceListGetCount(FxResourcesTranslated);
    ULONG Index;
    PCM_PARTIAL_RESOURCE_DESCRIPTOR Descriptor;
    PHYSICAL_ADDRESS Start = { 0 };
    ULONG Length = 0;

    UNREFERENCED_PARAMETER(FxResourcesRaw);

    FuncEntry(TRACE_FLAG_WDFLOADING);

    for (Index = 0; Index < ResourceCount; Index++)
    {
        Descriptor = WdfCmResourceListGetDescriptor(
            FxResourcesTranslated, Index);

        switch (Descriptor->Type)
        {
        case CmResourceTypeMemory:

            Start = Descriptor->u.Memory.Start;
            Length = Descriptor->u.Memory.Length;

            if (!fResourceFound)
            {
                Device->MemoryStart = Descriptor->u.Memory.Start;
                Device->MemoryLength = Descriptor->u.Memory.Length;

                fResourceFound = TRUE;

                Trace(
                    TRACE_LEVEL_INFORMATION,
                    TRACE_FLAG_WDFLOADING,
                    "Memory resource found at 0x%llx (%d bytes)",
                    Device->MemoryStart.QuadPart,
                    Device->MemoryLength
                    );
            }
            else
            {
                Trace(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_WDFLOADING,
                    "Duplicate memory resource found at 0x%llx",
                    Device->MemoryStart.QuadPart
                    );
            }

            break;

        default:

            break;
        }
    }

    // Need exactly one memory region

    if (!fResourceFound)
    {
        Status = STATUS_NOT_FOUND;
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "Memory region not found - %!STATUS!",
            Status
            );

        goto Exit;
    }

    if (!CheckMyPage(Start, Length))
    {
        Status = STATUS_NOT_FOUND;
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "Probe failed"
            );

        goto Exit;
    }

Exit:
    FuncExit(TRACE_FLAG_WDFLOADING);

    return Status;
}

NTSTATUS
OnReleaseHardware(
    _In_  WDFDEVICE     FxDevice,
    _In_  WDFCMRESLIST  FxResourcesTranslated
    )
{
    UNREFERENCED_PARAMETER(FxResourcesTranslated);
    UNREFERENCED_PARAMETER(FxDevice);

    FuncEntry(TRACE_FLAG_WDFLOADING);

    FuncExit(TRACE_FLAG_WDFLOADING);

    return STATUS_SUCCESS;
}

NTSTATUS
OnD0Entry(
    _In_  WDFDEVICE               FxDevice,
    _In_  WDF_POWER_DEVICE_STATE  FxPreviousState
    )
{
    PBPB_DEVICE Device;
    NTSTATUS Status;
    WDF_OBJECT_ATTRIBUTES TargetAttributes;
    WDF_OBJECT_ATTRIBUTES RequestAttributes;
    PBPB_REQUEST Request;

    UNREFERENCED_PARAMETER(FxPreviousState);

    FuncEntry(TRACE_FLAG_WDFLOADING);

    Device = GetDeviceContext(FxDevice);

    // Create the IO target
    WDF_OBJECT_ATTRIBUTES_INIT(&TargetAttributes);

    Status = WdfIoTargetCreate(
        Device->FxDevice,
        &TargetAttributes,
        &Device->IoTarget
        );

    if (NT_ERROR(Status))
    {
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "WdfIoTargetCreate failed with status %!STATUS!",
            Status
            );
    }

    // Create request object
    if (NT_SUCCESS(Status))
    {
        WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&RequestAttributes, BPB_REQUEST);

        Status = WdfRequestCreate(
            &RequestAttributes,
            NULL,
            &Device->IoRequest
            );

        if (NT_ERROR(Status))
        {
            Trace(
                TRACE_LEVEL_ERROR,
                TRACE_FLAG_WDFLOADING,
                "WdfRequestCreate failed with status %!STATUS!",
                Status
                );
        }

        if (NT_SUCCESS(Status))
        {
            Request = GetRequestContext(
                Device->IoRequest);

            Request->FxDevice = Device->FxDevice;
        }
    }

    FuncExit(TRACE_FLAG_WDFLOADING);

    return Status;
}

NTSTATUS
OnD0Exit(
    _In_  WDFDEVICE               FxDevice,
    _In_  WDF_POWER_DEVICE_STATE  FxPreviousState
    )
{
    PBPB_DEVICE Device = GetDeviceContext(FxDevice);

    UNREFERENCED_PARAMETER(FxPreviousState);

    FuncEntry(TRACE_FLAG_WDFLOADING);

    if (Device->IoTarget != WDF_NO_HANDLE)
    {
        WdfObjectDelete(Device->IoTarget);
        Device->IoTarget = WDF_NO_HANDLE;
    }

    if (Device->IoRequest != WDF_NO_HANDLE)
    {
        WdfObjectDelete(Device->IoRequest);
        Device->IoRequest = WDF_NO_HANDLE;
    }

    FuncExit(TRACE_FLAG_WDFLOADING);

    return STATUS_SUCCESS;
}
