#pragma once

#define BPB_POOL_TAG ((ULONG)'BPBP')

#define IDLE_TIMEOUT_MONITOR  2000

typedef struct tagDEVICE
{
    WDFDEVICE FxDevice;                                  // Handle to the WDF device.
    PHYSICAL_ADDRESS MemoryStart;                        // Resources obtained by Pnp
    ULONG MemoryLength;                                  // Resources obtained by Pnp
    WDFIOTARGET IoTarget;                                // IO controller target
    WDFREQUEST IoRequest;                                // Request object
    WDFMEMORY InputMemory;                               // Input memory for request. Valid while request in progress.
} BPB_DEVICE, *PBPB_DEVICE;

typedef struct tagREQUEST
{
    WDFDEVICE FxDevice;                                  // Associated framework device object
} BPB_REQUEST, *PBPB_REQUEST;

WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(BPB_DEVICE,  GetDeviceContext);
WDF_DECLARE_CONTEXT_TYPE_WITH_NAME(BPB_REQUEST, GetRequestContext);
