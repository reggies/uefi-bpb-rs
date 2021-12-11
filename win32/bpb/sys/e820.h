#pragma once

typedef struct tagX86BIOSALLOC
{
	USHORT Segment;
	USHORT Offset;
} X86BIOSALLOC, *PX86BIOSALLOC;

enum
{
	E820_USABLE = 0x1,
	E820_RESERVED = 0x2,
	E820_ACPI_RM = 0x3,
	E820_ACPI_NVS = 0x4,
	E820_BAD = 0x5
};

#pragma pack (push, 1)

typedef struct _E820_DESCRIPTOR {
	LARGE_INTEGER Base;
	LARGE_INTEGER Size;
	ULONG Type;
	ULONG ExtAttr;
} E820_DESCRIPTOR, *PE820_DESCRIPTOR;

#pragma pack (pop)

NTSTATUS
AllocateX86BiosBuffer(
	OUT PX86BIOSALLOC Alloc
	);

NTSTATUS
GetNextDescriptor(
	IN PX86BIOSALLOC X86BiosBuffer,
	IN OUT PULONG Index,
	OUT PE820_DESCRIPTOR Descriptor
	);

void
FreeX86BiosBuffer(
	IN PX86BIOSALLOC Alloc
	);

NTSTATUS
PrintE820Map(
	void
	);

NTSTATUS
ScanX86BiosMemory(void);