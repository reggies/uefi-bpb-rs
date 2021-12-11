#include <initguid.h>
#include <ntddk.h>
#include <wdm.h>
#include <wdf.h>
#include <ntstrsafe.h>

#include "common.h"
#include "driver.h"
#include "device.h"
#include "trace.h"

#include "driver.tmh"

NTSTATUS
DriverEntry(
    _In_ PDRIVER_OBJECT  DriverObject,
    _In_ PUNICODE_STRING RegistryPath
    )
{
    WDF_DRIVER_CONFIG DriverConfig;
    WDF_OBJECT_ATTRIBUTES DriverAttributes;
    WDFDRIVER FxDriver;
    NTSTATUS Status;

    WPP_INIT_TRACING(DriverObject, RegistryPath);

    FuncEntry(TRACE_FLAG_WDFLOADING);

    WDF_DRIVER_CONFIG_INIT(&DriverConfig, OnDeviceAdd);
    DriverConfig.DriverPoolTag = BPB_POOL_TAG;

    WDF_OBJECT_ATTRIBUTES_INIT(&DriverAttributes);
    DriverAttributes.EvtCleanupCallback = OnDriverCleanup;

    Status = WdfDriverCreate(
        DriverObject,
        RegistryPath,
        &DriverAttributes,
        &DriverConfig,
        &FxDriver);

    if (NT_ERROR(Status))
    {
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "WdfDriverCreate failed with status %!STATUS!",
            Status
            );

        goto Exit;
    }

    Trace(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_WDFLOADING,
        "Created WDFDRIVER %p",
        FxDriver);

Exit:
    FuncExit(TRACE_FLAG_WDFLOADING);

    return Status;
}

VOID
OnDriverCleanup(
    _In_ WDFOBJECT Object
    )
{
    FuncEntry(TRACE_FLAG_WDFLOADING);

    UNREFERENCED_PARAMETER(Object);

    FuncExit(TRACE_FLAG_WDFLOADING);
    WPP_CLEANUP(NULL);
}

static
void
InitializePnpDispatch(
    _Inout_ PWDFDEVICE_INIT FxDeviceInit
    )
{
    WDF_PNPPOWER_EVENT_CALLBACKS PnpCallbacks;

    WDF_PNPPOWER_EVENT_CALLBACKS_INIT(&PnpCallbacks);

    PnpCallbacks.EvtDevicePrepareHardware = OnPrepareHardware;
    PnpCallbacks.EvtDeviceReleaseHardware = OnReleaseHardware;
    PnpCallbacks.EvtDeviceD0Entry = OnD0Entry;
    PnpCallbacks.EvtDeviceD0Exit = OnD0Exit;

    WdfDeviceInitSetPnpPowerEventCallbacks(FxDeviceInit, &PnpCallbacks);
}

NTSTATUS
OnDeviceAdd(
    _In_    WDFDRIVER       FxDriver,
    _Inout_ PWDFDEVICE_INIT FxDeviceInit
    )
{
    PBPB_DEVICE Device;
    NTSTATUS Status;
    WDF_OBJECT_ATTRIBUTES DeviceAttributes;
    WDFDEVICE FxDevice;
    WDF_DEVICE_STATE DeviceState;
    WDF_DEVICE_POWER_POLICY_IDLE_SETTINGS IdleSettings;

    UNREFERENCED_PARAMETER(FxDriver);

    FuncEntry(TRACE_FLAG_WDFLOADING);

    InitializePnpDispatch(FxDeviceInit);

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(
        &DeviceAttributes,
        BPB_DEVICE
        );

    Status = WdfDeviceCreate(
        &FxDeviceInit,
        &DeviceAttributes,
        &FxDevice
        );

    if (NT_ERROR(Status))
    {
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "WdfDeviceCreate failed with status %!STATUS!",
            Status
            );

        goto Exit;
    }

    Device = GetDeviceContext(FxDevice);
    NT_ASSERT(Device != NULL);

    Device->FxDevice = FxDevice;

    WDF_DEVICE_STATE_INIT(&DeviceState);
    DeviceState.NotDisableable = WdfFalse;
    WdfDeviceSetDeviceState(
        Device->FxDevice,
        &DeviceState
        );

    WDF_DEVICE_POWER_POLICY_IDLE_SETTINGS_INIT(
        &IdleSettings,
        IdleCannotWakeFromS0
        );

    // Set system managed idle timeout

    IdleSettings.IdleTimeoutType = SystemManagedIdleTimeoutWithHint;
    IdleSettings.IdleTimeout = IDLE_TIMEOUT_MONITOR;

    Status = WdfDeviceAssignS0IdleSettings(
        Device->FxDevice,
        &IdleSettings
        );

    if (NT_ERROR(Status))
    {
        Trace(
            TRACE_LEVEL_ERROR,
            TRACE_FLAG_WDFLOADING,
            "WdfDeviceAssignS0IdleSettings failed with status %!STATUS!",
            Status
            );

        goto Exit;
    }

    Trace(
        TRACE_LEVEL_VERBOSE,
        TRACE_FLAG_WDFLOADING,
        "Created WDFDEVICE %p",
        FxDevice
        );

Exit:
    FuncExit(TRACE_FLAG_WDFLOADING);

    return Status;
}
