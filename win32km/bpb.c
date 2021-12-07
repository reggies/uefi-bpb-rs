#include <ntddk.h>
#include <initguid.h>

#include "bpb.h"

#ifdef ALLOC_PRAGMA
#pragma alloc_text (INIT, DriverEntry)
#pragma alloc_text (PAGE, MyUnload)
#pragma alloc_text (PAGE, MyDispatchPnp)
#pragma alloc_text (PAGE, MyAddDevice)
#endif

NTSTATUS
MyDispatchPnp (
    IN PDEVICE_OBJECT DeviceObject,
    IN PIRP Irp
    )
{
	UNREFERENCED_PARAMETER(Irp);
	UNREFERENCED_PARAMETER(DeviceObject);

    DbgPrint("MyDispatchPnp" );

    return STATUS_SUCCESS;
}

NTSTATUS
MyAddDevice (
    IN PDRIVER_OBJECT DriverObject,
    IN PDEVICE_OBJECT BusPhysicalDeviceObject
    )
{
    NTSTATUS Status;
    PDEVICE_OBJECT DeviceObject;
    PDEVICE_OBJECT TopOfTheStack;

    DbgPrint("MyAddDevice");

    Status = IoCreateDevice(
        DriverObject,
        0, // sizeof(FDO_DEVICE_DATA)
        NULL,
        FILE_DEVICE_BUS_EXTENDER,
        0,
        TRUE,
        &DeviceObject);
    if (NT_ERROR(Status))
    {
        return Status;
    }

    TopOfTheStack = IoAttachDeviceToDeviceStack(DeviceObject, BusPhysicalDeviceObject);
    if (!TopOfTheStack)
    {
        IoDeleteDevice(DeviceObject);
        return STATUS_UNSUCCESSFUL;
    }

    if (TopOfTheStack->Flags & DO_BUFFERED_IO) {
        DeviceObject->Flags |= DO_BUFFERED_IO;
    } else if (TopOfTheStack->Flags & DO_DIRECT_IO) {
        DeviceObject->Flags |= DO_DIRECT_IO;
    }

    DeviceObject->Flags |= DO_POWER_PAGABLE;
    DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;

    return STATUS_SUCCESS;
}

NTSTATUS
MyDispatchPassThrough (
    IN PDEVICE_OBJECT DeviceObject,
    IN PIRP Irp
    )
{
    DbgPrint("MyDispatchPassThrough");

    IoSkipCurrentIrpStackLocation (Irp);
    return IoCallDriver (DeviceObject, Irp);
}

VOID
MyUnload (
    IN PDRIVER_OBJECT DriverObject
    )
{
	UNREFERENCED_PARAMETER(DriverObject);

    DbgPrint("MyUnload");
}

BOOLEAN
CheckMyPage (
    ULONGLONG PhysicalAddress
    )
{
    PVOID SystemAddress;
    ULONG ProbeBytes;
	PHYSICAL_ADDRESS liPhysicalAddress;

    DbgPrint("CheckMyPage: %p", (void *)PhysicalAddress);

	liPhysicalAddress.QuadPart = PhysicalAddress;

    SystemAddress = MmMapIoSpace (
        liPhysicalAddress,
        4096,
        MmNonCached
        );

    if (!SystemAddress)
    {
        DbgPrint("MmMapIoSpace returned NULL");
        return FALSE;
    }

    RtlCopyMemory(&ProbeBytes, SystemAddress, sizeof(ULONG));

    DbgPrint("Probe bytes: %08x", ProbeBytes);

    MmUnmapIoSpace (
        SystemAddress,
        4096
        );

    return 0xFEADDEAD == ProbeBytes;
}

BOOLEAN
CheckRuntimeVarsPage (
    VOID
    )
{
    NTSTATUS Status;
    UNICODE_STRING VariableName;
    UCHAR Data[8];
    ULONG DataSize = ARRAYSIZE(Data);
    ULONGLONG PhysicalAddress;
	GUID MyVendorGuid = GUID_MY_VENDOR;

    DbgPrint("CheckRuntimeVarsPage");

    RtlInitUnicodeString(&VariableName, L"BpbAddress");

    Status = ExGetFirmwareEnvironmentVariable(
        &VariableName,
        &MyVendorGuid,
        Data,
        &DataSize,
        NULL
        );
    if (NT_ERROR(Status))
    {
        DbgPrint("ExGetFirmwareEnvironmentVariable returned %08x", Status);
        return FALSE;
    }

    RtlCopyMemory(&PhysicalAddress, Data, sizeof(ULONGLONG));

    return CheckMyPage(PhysicalAddress);
}

NTSTATUS
SetupRootDevice (
    IN PDRIVER_OBJECT DriverObject
    )
{
    // NTSTATUS Status;
    // PDEVICE_OBJECT DeviceObject;

	UNREFERENCED_PARAMETER(DriverObject);

    // Create root device to initiate memory region detection
    // Status = IoCreateDevice(
    //     DriverObject,
    //     0,
    //     NULL,
    //     FILE_DEVICE_BUS_EXTENDER,
    //     0,
    //     TRUE,
    //     &DeviceObject);
    // if (NT_ERROR(Status))
    // {
    //     return Status;
    // }

    // DeviceObject->Flags |= DO_POWER_PAGABLE;
    // DeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;

    // Status = IoReportRootDevice(DeviceObject);
    // if (NT_ERROR(Status))
    // {
    //     IoDeleteDevice(DeviceObject);
    //     return Status;
    // }

    // TBD: handle PNP IRP_MN_QUERY_RESOURCE_REQUIREMENTS for child devices
    // TBD: or, handle IRP_MN_QUERY_RESOURCES for child devices

    return STATUS_SUCCESS;
}

NTSTATUS
TryLegacyDeviceDetection (
    IN PDRIVER_OBJECT DriverObject
    )
{
    NTSTATUS Status;
    BOOLEAN fConflictDetected;
    CM_RESOURCE_LIST ResourceList;
    CM_PARTIAL_RESOURCE_DESCRIPTOR ResourceDescriptor;
    BOOLEAN fPageDetected;
    PDEVICE_OBJECT DeviceObject;

    // For IoReportResourceForDetection we must report
    // untranslated resource list
    RtlZeroMemory(&ResourceDescriptor, sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR));
    ResourceDescriptor.Type = CmResourceTypeMemory;
    ResourceDescriptor.ShareDisposition = CmResourceShareDeviceExclusive;
    ResourceDescriptor.Flags = CM_RESOURCE_MEMORY_READ_WRITE;
    ResourceDescriptor.u.Memory.Start.QuadPart = MY_PHYS_ADDRESS;
    ResourceDescriptor.u.Memory.Length = 4096;

    RtlZeroMemory(&ResourceList, sizeof(CM_RESOURCE_LIST));
    ResourceList.Count = 1;
    ResourceList.List[0].PartialResourceList.Version = 1;
    ResourceList.List[0].PartialResourceList.Revision = 1;
    ResourceList.List[0].PartialResourceList.Count = 1;
    ResourceList.List[0].PartialResourceList.PartialDescriptors[0] = ResourceDescriptor;

    // For majority of fixed physical addresses we got a
    // confict.
    Status = IoReportResourceForDetection(
        DriverObject,
        &ResourceList,
        sizeof(CM_RESOURCE_LIST),
        NULL,
		NULL,
        0,
        &fConflictDetected
        );
    if (NT_ERROR(Status))
    {
        if (STATUS_CONFLICTING_ADDRESSES == Status)
        {
            DbgPrint("ConflictDetected: %d", fConflictDetected);
        }
        DbgPrint("IoReportResourceForDetection returned %08x", Status);
        return Status;
    }

    fPageDetected = CheckMyPage(MY_PHYS_ADDRESS);

    DeviceObject = NULL;

    Status = IoReportDetectedDevice(
        DriverObject,
        InterfaceTypeUndefined,
        (ULONG) -1,                                      // BusNumber
        (ULONG) -1,                                      // SlotNumber
        &ResourceList,
        NULL,
        fPageDetected,
        &DeviceObject
        );
    if (NT_ERROR(Status))
    {
        DbgPrint("IoReportDetectedDevice returned %08x", Status);
        return Status;
    }

    DbgPrint("Detected DeviceObject is %p", DeviceObject);

    IoDeleteDevice(DeviceObject);

    return STATUS_SUCCESS;
}

NTSTATUS
DriverEntry (
    IN PDRIVER_OBJECT DriverObject,
    IN PUNICODE_STRING RegistryPath
    )
{
    ULONG Index;

    UNREFERENCED_PARAMETER(RegistryPath);

    DbgPrint("DriverEntry");

    for (Index = 0; Index < IRP_MJ_MAXIMUM_FUNCTION; Index++)
    {
        DriverObject->MajorFunction[Index] = MyDispatchPassThrough;
    }

    // DriverObject->MajorFunction [IRP_MJ_PNP] = MyDispatchPnp;
    DriverObject->DriverUnload = MyUnload;
    // DriverObject->DriverExtension->AddDevice = MyAddDevice;

    CheckMyPage(MY_PHYS_ADDRESS);

    CheckRuntimeVarsPage();

    TryLegacyDeviceDetection(DriverObject);

    return STATUS_SUCCESS;
}
