#ifndef BPB_H_
#define BPB_H_

#define MY_PHYS_ADDRESS 0x1000000

DRIVER_INITIALIZE DriverEntry;

DRIVER_UNLOAD MyUnload;

_Dispatch_type_(IRP_MJ_OTHER)
DRIVER_DISPATCH MyDispatchPassThrough;

_Dispatch_type_(IRP_MJ_PNP)
DRIVER_DISPATCH MyDispatchPnp;

DRIVER_ADD_DEVICE MyAddDevice;

BOOLEAN
CheckMyPage (
    ULONGLONG PhysicalAddress
    );

// f08ae394-4e98-46e6-b0b3-1bb940ac663d
DEFINE_GUID(GUID_MY_VENDOR, 0xf08ae394, 0x4e98, 0x46e6, 0xb0, 0xb3, 0x1b, 0xb9, 0x40, 0xac, 0x66, 0x3d);

#endif
