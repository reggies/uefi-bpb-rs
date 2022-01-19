#include <linux/module.h>
#include <linux/efi.h>

#define MY_VARIABLE_NAME \
    L"RuntimeListHead"

#define MY_VENDOR_GUID \
    EFI_GUID(0xf08ae394, 0x4e98, 0x46e6, 0xb3, 0xb0, 0x1b, 0xb9, 0x40, 0xac, 0x66, 0x3d)

static int get_list_address(u64 *addr)
{
    efi_char16_t name[] = MY_VARIABLE_NAME;
    efi_guid_t guid = MY_VENDOR_GUID;
    u32 attr;
    unsigned long data_size = 0;
    u8 *data = NULL;
    efi_status_t status;

    /* get the size of the variable */
    status = efi.get_variable(name, &guid, &attr, &data_size, data);
    if (status == EFI_BUFFER_TOO_SMALL) {
        /* allocate temporary buffer of data_size bytes */
        data = (u8*)vmalloc(data_size);
        if (!data) {
            goto error;
        }

        /* get variable contents into buffer */
        status = efi.get_variable(name, &guid, &attr, &data_size, data);
        if (status != EFI_SUCCESS) {
            if (data_size != sizeof(u64)) {
                goto error;
            }
            memcpy(addr, data, sizeof(u64));
        }
        else {
            goto error;
        }
    }
    else {
        goto error;
    }

    vfree(data);

    return 0;

error:
    if (data) {
        vfree(data);
    }
    return -1;
}

static int myinit(void)
{
    u64 addr;
    if (get_list_address(&addr) == 0) {
        printk("addr: %016Lx\n", addr);
    } else {
        printk("failed to get addr\n");
    }
	return 0;
}

static void myexit(void)
{
}

module_init(myinit);
module_exit(myexit);
MODULE_LICENSE("GPL");
