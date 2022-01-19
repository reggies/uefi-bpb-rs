#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <uchar.h>

typedef uint64_t UINT64;
typedef uint32_t UINT32;
typedef uint16_t UINT16;
typedef uint8_t  UINT8;
typedef char16_t CHAR16;
typedef struct {
  UINT32  Data1;
  UINT16  Data2;
  UINT16  Data3;
  UINT8   Data4[8];
} GUID;
#include "pe.h"

#define PAGE_ALIGN(n, d) \
    ((((n) + (d) - 1) / (d)) * (d))

#define SIGNATURE_16(A, B)        ((A) | (B << 8))
#define SIGNATURE_32(A, B, C, D)  (SIGNATURE_16 (A, B) | (SIGNATURE_16 (C, D) << 16))
#define SIGNATURE_64(A, B, C, D, E, F, G, H) \
    (SIGNATURE_32 (A, B, C, D) | ((UINT64) (SIGNATURE_32 (E, F, G, H)) << 32))

#define RUNTIME_MAX_DP 256

#define RUNTIME_MAX_MOD 256

#define MY_VAR_PATH                                                     \
    "/sys/firmware/efi/efivars/RuntimeListHead-f08ae394-4e98-46e6-b3b0-1bb940ac663d"

#pragma pack(push, 1)

struct myefivar {
    uint32_t attr;
    uint64_t value;
};

#pragma pack(pop)

#pragma pack(push, 1)

struct runtime_module {
    struct runtime_module *next;
    void *base;
    uint64_t size;
    uint32_t segtext;
    uint32_t segdata;
    char16_t dp[RUNTIME_MAX_DP];
    char16_t module[RUNTIME_MAX_MOD];
};

#pragma pack(pop)

void die(const char *what)
{
    if (errno != 0)
        fprintf(stderr, "%s: %s\n", what, strerror(errno));
    else
        fprintf(stderr, "%s\n", what);
    exit(-1);
}

int get_list_pa(uint64_t *addr)
{
    char path[] = MY_VAR_PATH;
    int fd;
    struct myefivar buf = { 0 };

    fd = open(path, O_RDONLY | O_SYNC);
    if (fd == -1) {
        return -1;
    }

    if (read(fd, &buf, sizeof(struct myefivar)) != sizeof(struct myefivar)) {
        close(fd);
        return -1;
    }

    memcpy(addr, &buf.value, sizeof(uint64_t));
    close(fd);
    return 0;
}

void ucs2tomb(char *out, size_t outc, char16_t *in, size_t inc)
{
    size_t n;
    size_t rc;
    mbstate_t state;
    memset(out, 0, outc);
    for(n = 0; n < outc; ++n) {
        rc = c16rtomb(out, in[n], &state);
        if(rc == (size_t)-1)
            break;
        out += rc;
    }
}

#ifdef __amd64
typedef EFI_IMAGE_NT_HEADERS64 EFI_IMAGE_NT_HEADERS;
#else
typedef EFI_IMAGE_NT_HEADERS32 EFI_IMAGE_NT_HEADERS;
#endif

void inspect(void *base)
{
    EFI_IMAGE_DOS_HEADER *dos = base;
    EFI_IMAGE_NT_HEADERS *nt = (void *)((char *)base + dos->e_lfanew);
    EFI_TE_IMAGE_HEADER *te = (void *)((char *)base + dos->e_lfanew);
    EFI_IMAGE_SECTION_HEADER *sections = (void *)((char *)nt + sizeof(EFI_IMAGE_NT_HEADERS));
    int n;

    fprintf(stdout, "DOS header:\n");
    fprintf(stdout, " + e_magic: %04x\n", dos->e_magic);
    if (dos->e_magic == EFI_IMAGE_DOS_SIGNATURE) {
        fprintf(stdout, " + e_lfanew: %d\n", dos->e_lfanew);
        fprintf(stdout, "NT header:\n");
        if (nt->Signature == EFI_IMAGE_NT_SIGNATURE) {
            fprintf(stdout, " + signature: %04x\n", nt->Signature);
            fprintf(stdout, " + magic: %04x\n", nt->OptionalHeader.Magic);
            if (nt->OptionalHeader.Magic == EFI_IMAGE_NT_OPTIONAL_HDR64_MAGIC ||
                nt->OptionalHeader.Magic == EFI_IMAGE_NT_OPTIONAL_HDR32_MAGIC) {
                fprintf(stdout, " + subsystem: %02x\n", nt->OptionalHeader.Subsystem);
                fprintf(stdout, " + entry: %04x\n", nt->OptionalHeader.AddressOfEntryPoint);
                fprintf(stdout, " + base: %p\n", (void *)nt->OptionalHeader.ImageBase);
            }
        }
    }
}

int main(int argc, char *ragv[])
{
    void *map_base;
    void *virt_addr;
    uint64_t read_val;
    off_t target;
    unsigned int page_size;
    unsigned int mapped_size;
    unsigned int offset_in_page;
    int fd;
    uint64_t addr;
    struct runtime_module mod;
    char dp[RUNTIME_MAX_DP * MB_CUR_MAX];
    char mod_name[RUNTIME_MAX_MOD * MB_CUR_MAX];

    if (get_list_pa(&addr) < 0) {
        die("get_list_pa");
    }

    if (addr == 0)
        die("null address");

    fd = open("/dev/mem", O_RDONLY | O_SYNC);
    if (fd == -1)
        die("open");

    for (target = addr; target != 0; target = (off_t)mod.next) {
        page_size = getpagesize();
        mapped_size = PAGE_ALIGN(sizeof(struct runtime_module), page_size);
        offset_in_page = (unsigned)target & (page_size - 1);
        map_base = mmap(NULL,
                        mapped_size,
                        PROT_READ,
                        MAP_SHARED,
                        fd,
                        target & ~(off_t)(page_size - 1));
        if (map_base == MAP_FAILED)
            die("mmap");

        virt_addr = (char*)map_base + offset_in_page;
        memcpy(&mod, (struct runtime_module *)virt_addr, sizeof(struct runtime_module));

        if (munmap(map_base, mapped_size) == -1)
            die("munmap");

        ucs2tomb(dp, RUNTIME_MAX_DP * MB_CUR_MAX, mod.dp, RUNTIME_MAX_DP);
        ucs2tomb(mod_name, RUNTIME_MAX_MOD * MB_CUR_MAX, mod.module, RUNTIME_MAX_MOD);
        fprintf(stdout, "%p: %s\n", mod.base, mod_name);
        fprintf(stdout, " %s\n", dp);

        /* runtime services code */
        if (mod.segtext == 0x5) {
            page_size = getpagesize();
            mapped_size = PAGE_ALIGN(mod.size, page_size);
            offset_in_page = (unsigned)(off_t)mod.base & (page_size - 1);
            map_base = mmap(NULL,
                            mapped_size,
                            PROT_READ,
                            MAP_SHARED,
                            fd,
                            (off_t)mod.base & ~(off_t)(page_size - 1));
            if (map_base == MAP_FAILED)
                die("mmap");

            void *va = (char*)map_base + offset_in_page;
            inspect(va);

            if (munmap(map_base, mapped_size) == -1)
                die("munmap");
        }
    }

    close(fd);

    return 0;
}
