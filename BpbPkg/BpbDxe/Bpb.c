#include <Uefi.h>

#include <Library/BaseLib.h>
#include <Library/BaseMemoryLib.h>
#include <Library/UefiDriverEntryPoint.h>
#include <Library/UefiBootServicesTableLib.h>
#include <Library/UefiRuntimeServicesTableLib.h>
#include <Library/DebugLib.h>
#include <Library/DevicePathLib.h>
#include <Library/UefiRuntimeLib.h>

#include <Guid/EventGroup.h>

static EFI_GET_VARIABLE OriginalGetVariable = NULL;

static EFI_SET_VARIABLE OriginalSetVariable = NULL;

static EFI_GUID MyVendorGuid = {
  0xf08ae394,
  0x4e98,
  0x46e6,
  { 0xb3, 0xb0, 0x1b, 0xb9, 0x40, 0xac, 0x66, 0x3d }
};

static UINT8 BpbBuffer[4096];
static BOOLEAN BpbBufferCleared = FALSE;

EFI_STATUS
EFIAPI
GetVariable (
  IN  CHAR16                       *VariableName,
  IN  EFI_GUID                     *VendorGuid,
  OUT UINT32                       *Attributes, OPTIONAL
  OUT UINTN                        *DataSize,
  OUT VOID                         *Data OPTIONAL
  )
{
  EFI_STATUS Status;
  if (VendorGuid != NULL && VariableName != NULL) {
    if (CompareGuid(VendorGuid, &MyVendorGuid)) {
      if (StrCmp(VariableName, L"MyInternalBpb") == 0) {
        if (BpbBufferCleared) {
          return EFI_NOT_FOUND;
        }
        if (DataSize == NULL) {
          return EFI_INVALID_PARAMETER;
        }
        if (Attributes) {
          *Attributes = 0;
        }
        if (*DataSize < sizeof(BpbBuffer)) {
          *DataSize = sizeof(BpbBuffer);
          return EFI_BUFFER_TOO_SMALL;
        }
        // TBD: speculation barrier
        CopyMem(Data, BpbBuffer, sizeof(BpbBuffer));
        return EFI_SUCCESS;
      }
    }
  }
  if (OriginalGetVariable) {
    Status = OriginalGetVariable(
      VariableName,
      VendorGuid,
      Attributes,
      DataSize,
      Data
      );
    return Status;
  }
  return EFI_UNSUPPORTED;
}

EFI_STATUS
EFIAPI
SetVariable (
  IN  CHAR16                       *VariableName,
  IN  EFI_GUID                     *VendorGuid,
  IN  UINT32                       Attributes,
  IN  UINTN                        DataSize,
  IN  VOID                         *Data
  )
{
  EFI_STATUS Status;
  if (VendorGuid != NULL && VariableName != NULL) {
    if (CompareGuid(VendorGuid, &MyVendorGuid)) {
      if (StrCmp(VariableName, L"MyInternalBpb") == 0) {
        ZeroMem(BpbBuffer, sizeof(BpbBuffer));
        BpbBufferCleared = TRUE;
        return EFI_SUCCESS;
      }
    }
  }
  if (OriginalSetVariable) {
    Status = OriginalSetVariable(
      VariableName,
      VendorGuid,
      Attributes,
      DataSize,
      Data
      );
    return Status;
  }
  return EFI_UNSUPPORTED;
}

EFI_STATUS
HookRuntimeVariableServices (VOID)
{
  EFI_TPL OldTpl;

  OldTpl = gBS->RaiseTPL(TPL_HIGH_LEVEL);
  OriginalSetVariable = gRT->SetVariable;
  OriginalGetVariable = gRT->GetVariable;
  gRT->SetVariable = SetVariable;
  gRT->GetVariable = GetVariable;
  gRT->Hdr.CRC32 = 0;
  gBS->CalculateCrc32(&gRT->Hdr, gRT->Hdr.HeaderSize, &gRT->Hdr.CRC32);
  gBS->RestoreTPL(OldTpl);
  return EFI_SUCCESS;
}

VOID
EFIAPI
OnSetVirtualAddressMapEvent (
  IN  EFI_EVENT     Event,
  IN  VOID          *Context
  )
{
  gRT->ConvertPointer(0, (VOID **) &OriginalSetVariable);
  gRT->ConvertPointer(0, (VOID **) &OriginalGetVariable);
  gRT->ConvertPointer(0, (VOID **) &gRT);
}

EFI_STATUS
EFIAPI
UefiMain(
    IN EFI_HANDLE ImageHandle,
    IN EFI_SYSTEM_TABLE *SystemTable
    )
{
  EFI_STATUS Status;
  EFI_EVENT SetVirtualAddressMapEvent;
  UINT32 BpbBufferHeader = 0xfeeddead;

  HookRuntimeVariableServices();

  CopyMem(BpbBuffer, &BpbBufferHeader, sizeof(UINT32));

  Status = gBS->CreateEventEx (
         EVT_NOTIFY_SIGNAL,
         TPL_NOTIFY,
         OnSetVirtualAddressMapEvent,
         NULL,
         &gEfiEventVirtualAddressChangeGuid,
         &SetVirtualAddressMapEvent
         );
  ASSERT_EFI_ERROR(Status);

  return EFI_SUCCESS;
}
