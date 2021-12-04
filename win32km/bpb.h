#ifndef BPB_H_
#define BPB_H_

DRIVER_INITIALIZE DriverEntry;

DRIVER_UNLOAD MyUnload;

_Dispatch_type_(IRP_MJ_OTHER)
DRIVER_DISPATCH MyDispatchPassThrough;

_Dispatch_type_(IRP_MJ_PNP)
DRIVER_DISPATCH MyDispatchPnp;

DRIVER_ADD_DEVICE MyAddDevice;

#endif
