#include <ntddk.h>

#include "bpb.h"
#include "e820.h"

#pragma pack(push, 1)

typedef struct tagX86REGS
{
	ULONG Eax;
	ULONG Ecx;
	ULONG Edx;
	ULONG Ebx;
	ULONG Ebp;
	ULONG Esi;
	ULONG Edi;
	USHORT Ds;
	USHORT Es;
} X86REGS, *PX86REGS;

#pragma pack(pop)

NTHALAPI BOOLEAN x86BiosCall(ULONG Interrupt, PX86REGS Regs);

NTHALAPI NTSTATUS x86BiosAllocateBuffer(PULONG cbAllocate, PUSHORT Segment, PUSHORT Offset);
NTHALAPI NTSTATUS x86BiosFreeBuffer(USHORT Segment, USHORT Offset);

NTHALAPI NTSTATUS x86BiosReadMemory(USHORT Segment, USHORT Offset, PVOID Pointer, ULONG cbRead);
NTHALAPI NTSTATUS x86BiosWriteMemory(USHORT Segment, USHORT Offset, PVOID Pointer, ULONG cbWrite);

NTSTATUS
AllocateX86BiosBuffer (
	OUT PX86BIOSALLOC Alloc
)
{
	ULONG cbSize = sizeof(E820_DESCRIPTOR);
	NTSTATUS Status;
	USHORT Segment = 0;
	USHORT Offset = 0;

	Status = x86BiosAllocateBuffer(&cbSize, &Segment, &Offset);
	if (NT_ERROR(Status))
	{
		MyDbgPrint("x86BiosAllocateBuffer returned %08x", Status);
		return Status;
	}

	Alloc->Offset = Offset;
	Alloc->Segment = Segment;

	return STATUS_SUCCESS;
}

NTSTATUS
GetNextDescriptor(
	IN PX86BIOSALLOC X86BiosBuffer,
	IN OUT PULONG Index,
	OUT PE820_DESCRIPTOR Descriptor
)
{
	X86REGS Regs;
	NTSTATUS Status;
	E820_DESCRIPTOR Temp;
	ULONG cbSize;

	RtlZeroMemory(&Temp, sizeof(E820_DESCRIPTOR));

	RtlZeroMemory(&Regs, sizeof(X86REGS));
	Regs.Es = X86BiosBuffer->Segment;
	Regs.Edi = X86BiosBuffer->Offset;
	Regs.Edx = 'SMAP';
	Regs.Ecx = sizeof(E820_DESCRIPTOR);
	Regs.Ebx = *Index;
	Regs.Eax = 0xe820;

	if (!x86BiosCall(0x15, &Regs))
	{
		MyDbgPrint("x86BiosCall failed");
		return STATUS_UNSUCCESSFUL;
	}

	if (Regs.Eax != 'SMAP')
	{
		MyDbgPrint("x86BiosCall: not an e820 entry");
		return STATUS_UNSUCCESSFUL;
	}

	cbSize = min(Regs.Ecx, sizeof(E820_DESCRIPTOR));
	Status = x86BiosReadMemory(X86BiosBuffer->Segment, X86BiosBuffer->Offset, &Temp, cbSize);
	if (NT_ERROR(Status))
	{
		MyDbgPrint("x86BiosReadMemory returned %08x", Status);
		return Status;
	}

	*Descriptor = Temp;
	*Index = Regs.Ebx;

	return STATUS_SUCCESS;
}

void
FreeX86BiosBuffer(
	IN PX86BIOSALLOC Alloc
)
{
	x86BiosFreeBuffer(Alloc->Segment, Alloc->Offset);
}

NTSTATUS
PrintE820Map(
	void
)
{
	X86BIOSALLOC Buffer;
	NTSTATUS Status;
	E820_DESCRIPTOR Descriptor;
	ULONG Index = 0;

	Status = AllocateX86BiosBuffer(&Buffer);
	if (NT_ERROR(Status))
	{
		MyDbgPrint("AllocateX86BiosBuffer returned %08x", Status);
		return Status;
	}

	do
	{
		Status = GetNextDescriptor(&Buffer, &Index, &Descriptor);
		if (NT_ERROR(Status))
		{
			MyDbgPrint("GetNextDescriptor returned %08x", Status);
			break;
		}

		MyDbgPrint("E820: %016I64x:%016I64x %02x", Descriptor.Base.QuadPart, Descriptor.Size.QuadPart, Descriptor.Type);

	} while (Index != 0);

	FreeX86BiosBuffer(&Buffer);

	return Status;
}

NTSTATUS
ScanX86BiosMemory(void)
{
	X86BIOSALLOC Buffer;
	NTSTATUS Status;
	E820_DESCRIPTOR Descriptor;
	ULONG Index = 0;
	USHORT Offset = 0;
	UCHAR Page[0x1000];
	ULONG ProbeBytes;
	USHORT Segment;

	Status = AllocateX86BiosBuffer(&Buffer);
	if (NT_ERROR(Status))
	{
		MyDbgPrint("AllocateX86BiosBuffer returned %08x", Status);
		return Status;
	}

	do
	{
		Status = GetNextDescriptor(&Buffer, &Index, &Descriptor);
		if (NT_ERROR(Status))
		{
			MyDbgPrint("GetNextDescriptor returned %08x", Status);
			break;
		}

		if (Descriptor.Type != E820_USABLE)
			continue;

		for (Offset = 0; (ULONG)(Offset + 0x1000) <= Descriptor.Size.LowPart; Offset += 0x1000)
		{
			Segment = (USHORT) Descriptor.Base.LowPart;
			Status = x86BiosReadMemory(Segment, Offset, Page, 0x1000);
			if (NT_ERROR(Status))
			{
				MyDbgPrint("Reading at %04x:%04x failed with %08x", Segment, Offset, Status);
				continue;
			}

			RtlCopyMemory(&ProbeBytes, Page, sizeof(ULONG));

			MyDbgPrint("Probe bytes: %08x", ProbeBytes);
		}

	} while (Index != 0);

	FreeX86BiosBuffer(&Buffer);

	return STATUS_SUCCESS;
}