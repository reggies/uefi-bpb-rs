#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(dead_code)]
#[macro_use]
extern crate log;
extern crate uefi;
extern crate uefi_services;
#[macro_use]
extern crate alloc;

use uefi::prelude::*;
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi::table::cfg::{ConfigTableEntry, ACPI_GUID, ACPI2_GUID};
use alloc::vec::*;
use core::mem;
use core::fmt;
use core::ffi::c_void;
use uefi::proto::acpi_table::AcpiTable;

use uefi::{CStr16};
use uefi::table::runtime::VariableAttributes;

mod acpi;
use acpi::*;

const PHYS_ADDR: usize = 0x1000000;
const PAGE_COUNT: usize = 1;

const MY_CONFIGURATION_TABLE_GUID: uefi::Guid = uefi::Guid::from_values(
    0x8868e871, 0xe4f1, 0x11d3, 0x22bc, [0x0, 0x80, 0xc7, 0x3c, 0x88, 0x81]
);

const SSDT_SIGNATURE: u32 = 0x5444_5353;                 // "SSDT"
const SSDT_REVISION: u8 = 0x2;

const OEM_VENDOR_ID: [u8; 6] = [0x4f, 0x45, 0x4d, 0x0, 0x0, 0x0]; // "OEM"
const OEM_REVISION: u32 = 0x3;
const OEM_TABLE_ID: u64 = 0x00000030_54425042;         // "BPBT0"

const MY_TABLE_SIGNATURE: u32 = 0x5442_5042;              // "BPBT"
const MY_TABLE_REVISION: u8 = 2;

const MY_VENDOR_GUID: uefi::Guid = uefi::Guid::from_values(
    0xf08ae394,
    0x4e98,
    0x46e6,
    0xb0b3,
    [0x1b, 0xb9, 0x40, 0xac, 0x66, 0x3d]
);

// ssdt_bpb.aml
const MY_AML_CODE_SIZE: usize = 99 - mem::size_of::<DescriptionHeader>();
const MY_AML_CODE: &[u8; MY_AML_CODE_SIZE] = &[
    // 0x53, 0x53, 0x44, 0x54, 0x63, 0x00, 0x00, 0x00, 0x02, 0xe5, 0x4f, 0x45,
    // 0x4d, 0x00, 0x00, 0x00, 0x42, 0x50, 0x42, 0x54, 0x30, 0x00, 0x00, 0x00,
    // 0x04, 0x00, 0x00, 0x00, 0x49, 0x4e, 0x54, 0x4c, 0x05, 0x01, 0x18, 0x20,
    0x10, 0x3e, 0x5c, 0x5f, 0x53, 0x42, 0x5f, 0x5b, 0x82, 0x36, 0x42, 0x50,
    0x42, 0x30, 0x08, 0x5f, 0x41, 0x44, 0x52, 0x00, 0x08, 0x5f, 0x55, 0x49,
    0x44, 0x01, 0x08, 0x5f, 0x48, 0x49, 0x44, 0x0d, 0x42, 0x50, 0x42, 0x30,
    0x30, 0x30, 0x31, 0x00, 0x08, 0x5f, 0x43, 0x52, 0x53, 0x11, 0x11, 0x0a,
    0x0e, 0x86, 0x09, 0x00, 0x01, 0x44, 0x33, 0x22, 0x11, 0x00, 0x10, 0x00,
    0x00, 0x79, 0x00
];

#[repr(C, packed)]
struct MySsdtTable {
    header: DescriptionHeader,
    aml_code: [u8; MY_AML_CODE_SIZE]
}

#[repr(C, packed)]
struct MyBpbtTable {
    header: DescriptionHeader,
    payload: MyPayload
}

#[repr(C)]
struct MyConfTable {
    guid: uefi::Guid,
    payload: *const MyPayload,
}

#[repr(C)]
struct MyPayload {
    magic: u64,
    physical_address: u64,
    length_bytes: u64
}

fn inspect<'a, E: fmt::Debug + 'a>(name: &'a str) -> impl FnOnce(E) -> E + 'a {
    move |errdata| {
        error!("{} returned {:?}", name, errdata);
        errdata
    }
}

fn dump_mmap() -> uefi::Result {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };
    let mmap_size = bs.memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_buffer = vec![0; mmap_size].into_boxed_slice();
    let (_key, mmap_iter) = bs.memory_map(&mut *mmap_buffer)
        .ignore_warning()
        .map_err(inspect("memory_map"))?;
    let mmap_list = mmap_iter
        .copied()
        .collect::<Vec<_>>();
    info!("memory map: {:#?}", mmap_list);
    Ok(().into())
}

fn find_region(addr: u64) -> uefi::Result<MemoryDescriptor> {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };
    let mmap_size = bs.memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_buffer = vec![0; mmap_size].into_boxed_slice();
    let (_key, mmap_iter) = bs.memory_map(&mut *mmap_buffer)
        .ignore_warning()
        .map_err(inspect("memory_map"))?;
    let descriptor = mmap_iter
        .copied()
        .find(|region| {
            addr >= region.phys_start &&
                addr < region.phys_start + 4096 * region.page_count
        });
    match descriptor {
        Some(descriptor) => Ok(descriptor.into()),
        None => Err(uefi::Status::NOT_FOUND.into()),
    }
}

fn enum_acpi_table_protocols() -> uefi::Result {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };
    let acpi_handles = bs.find_handles::<AcpiTable>()
        .ignore_warning()?;
    for acpi_handle in acpi_handles.into_iter() {
        let acpi = bs
            .handle_protocol::<AcpiTable>(acpi_handle)
            .ignore_warning()?;
        let acpi = unsafe { &mut *acpi.get() };
        info!("Got {:?} acpi protocol", &*acpi as *const AcpiTable);
    }
    Ok(().into())
}

fn find_configuration_table(guid: &uefi::Guid) -> uefi::Result<&ConfigTableEntry> {
    let st = unsafe {
        uefi_services::system_table()
            .as_ref()
    };
    let conf_table = st.config_table()
        .iter()
        .find(|entry| entry.guid == *guid);
    match conf_table {
        Some(conf_table) => Ok(conf_table.into()),
        None => Err(uefi::Status::NOT_FOUND.into()),
    }
}

fn install_configuration_table(phys_addr: u64) -> uefi::Result {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    let cfg_table_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let cfg_table_size = mem::size_of::<MyConfTable>();
    let cfg_table = bs.allocate_pool(cfg_table_pool, cfg_table_size)
        .map_err(inspect("allocate_pool (cfg_table)"))
        .ignore_warning()?;
    info!("cfg_table: {:?}", cfg_table);

    let payload_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let payload_size = mem::size_of::<MyPayload>();
    let payload = bs.allocate_pool(payload_pool, payload_size)
        .map_err(inspect("allocate_pool (payload)"))
        .ignore_warning()?;
    info!("payload: {:?}", payload);

    let cfg_table_data = MyConfTable {
        guid: ACPI2_GUID,
        payload: payload.cast()
    };

    let payload_data = MyPayload {
        magic: 0xfeeddead,
        physical_address: phys_addr,
        length_bytes: PAGE_COUNT as u64 * 4096,
    };

    // SAFETY: not safe at all
    unsafe {
        bs.memmove(
            cfg_table,
            &cfg_table_data as *const MyConfTable as *const u8,
            cfg_table_size
        );
    }

    // SAFETY: damn this is hot
    unsafe {
        bs.memmove(
            payload,
            &payload_data as *const MyPayload as *const u8,
            payload_size
        );
    }

    // WTF: is it my responsibility to convert pointer to configuration table?
    // WTF: is it my responsibility to parse conf tables and append my guid
    // WTF: is it my responsibility to convert pointer to payload table if guid is my own?
    // WTF: is it my responsibility to convert pointer to payload table if guid is by a spec?
    // WTF: is memory preserved at fixed physical address when allocated as RUNTIME_SERVICES_DATA?

    // SAFETY: oof
    unsafe {
        let guid = &ACPI2_GUID;
        bs.install_configuration_table(guid, cfg_table as *mut c_void)
            .map_err(inspect("InstallConfigurationTable"))?;
    }

    Ok(().into())
}

unsafe fn checksum8<T: Sized>(value: &T) -> u8 {
    use core::slice;
    use core::num::Wrapping;
    let buffer = slice::from_raw_parts(
        value as *const T as *const Wrapping<u8>,
        mem::size_of::<T>(),
    );
    ((0x100 - u32::from(buffer.iter().copied().sum::<Wrapping<u8>>().0)) & 0xff) as u8
}

fn install_fadt3(phys_addr: u64) -> uefi::Result<usize> {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    let acpi = bs
        .locate_protocol::<AcpiTable>()
        .ignore_warning()?;
    let acpi = unsafe { &mut *acpi.get() };

    let acpi_table_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let acpi_table_size = mem::size_of::<FixedDescriptionTable3>();
    let acpi_table = bs.allocate_pool(acpi_table_pool, acpi_table_size)
        .map_err(inspect("allocate_pool (acpi_table)"))
        .ignore_warning()?;
    info!("acpi_table: {:?}", acpi_table);

    let rsdp_ptr = find_configuration_table(&ACPI2_GUID)
        .map_err(inspect("find_configuration_table(ACPI2)"))
        .or_else(|_| find_configuration_table(&ACPI_GUID))
        .map_err(inspect("find_configuration_table(ACPI1)"))
        .ignore_warning()?;

    let rsdp = unsafe {
        (rsdp_ptr.address as *const RootSystemDescriptionPointer3)
            .read_unaligned()
    };

    let sdt_ptr = {
        if rsdp.revision >= ACPI_2_RSDP_REVISION && rsdp.xsdt_address != 0 {
            rsdp.xsdt_address as usize
        } else {
            rsdp.rsdt_address as usize
        }
    };
    if sdt_ptr == 0 {
        error!("Could not find RSD pointer table!");
        return Err(uefi::Status::UNSUPPORTED.into());
    }

    let sdt = unsafe {
        (sdt_ptr as *const DescriptionHeader)
            .read_unaligned()
    };

    let mut acpi_table_data = FixedDescriptionTable3 {
        header: DescriptionHeader {
            signature: ACPI_3_FADT_SIGNATURE,
            length: acpi_table_size as u32,
            revision: ACPI_3_FADT_REVISION,
            checksum: 0,
            oem_id: sdt.oem_id,
            oem_table_id: sdt.oem_table_id,
            oem_revision: sdt.oem_revision,
            creator_id: 0,
            creator_revision: 0
        },
        firmware_ctrl: 0,
        dsdt: 0,
        reserved0: 0,
        preferred_pm_profile: 0,
        sci_int: 0,
        smi_cmd: 0,
        acpi_enable: 0,
        acpi_disable: 0,
        s4_bios_req: 0,
        pstate_cnt: 0,
        pm1a_evt_blk: 0,
        pm1b_evt_blk: 0,
        pm1a_cnt_blk: 0,
        pm1b_cnt_blk: 0,
        pm2_cnt_blk: 0,
        pm_tmr_blk: 0,
        gpe0_blk: 0,
        gpe1_blk: 0,
        pm1_evt_len: 0,
        pm1_cnt_len: 0,
        pm2_cnt_len: 0,
        pm_tm_len: 0,
        gpe0_blk_len: 0,
        gpe1_blk_len: 0,
        gpe1_base: 0,
        cst_cnt: 0,
        p_lvl2_lat: 0,
        p_lvl3_lat: 0,
        flush_size: 0,
        flush_stride: 0,
        duty_offset: 0,
        duty_width: 0,
        day_alrm: 0,
        mon_alrm: 0,
        century: 0,
        iapc_boot_arch: 0,
        reserved1: 0,
        flags: 0,
        reset_reg: GenericAddressSpace::new(),
        reset_value: 0,
        reserved2: 0,
        reserved3: 0,
        reserved4: 0,
        x_firmware_ctrl: 0,
        x_dsdt: 0,
        x_pm1a_evt_blk: GenericAddressSpace::new(),
        x_pm1b_evt_blk: GenericAddressSpace::new(),
        x_pm1a_cnt_blk: GenericAddressSpace::new(),
        x_pm1b_cnt_blk: GenericAddressSpace::new(),
        x_pm2_cnt_blk: GenericAddressSpace::new(),
        x_pm_tmr_blk: GenericAddressSpace::new(),
        x_gpe0_blk: GenericAddressSpace::new(),
        x_gpe1_blk: GenericAddressSpace::new(),
    };

    // SAFETY: acpi_table_data must be properly initialized e.g. packed
    unsafe {
        acpi_table_data.header.checksum = checksum8(&acpi_table_data);
    }

    unsafe {
        // Entire table must sum to zero
        assert!(checksum8(&acpi_table_data) == 0);
    }

    // SAFETY: not safe at all
    unsafe {
        bs.memmove(
            acpi_table,
            &acpi_table_data as *const FixedDescriptionTable3 as *const u8,
            acpi_table_size
        );
    }

    let table_key = unsafe {
        acpi.install_acpi_table(acpi_table as *const c_void, acpi_table_size)
            .map_err(inspect("install_acpi_table"))
            .ignore_warning()?
    };

    Ok(table_key.into())
}

fn install_fadt1(phys_addr: u64) -> uefi::Result<usize> {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    let acpi = bs
        .locate_protocol::<AcpiTable>()
        .ignore_warning()?;
    let acpi = unsafe { &mut *acpi.get() };

    let acpi_table_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let acpi_table_size = mem::size_of::<FixedDescriptionTable1>();
    let acpi_table = bs.allocate_pool(acpi_table_pool, acpi_table_size)
        .map_err(inspect("allocate_pool (acpi_table)"))
        .ignore_warning()?;
    info!("acpi_table: {:?}", acpi_table);

    let rsdp_ptr = find_configuration_table(&ACPI2_GUID)
        .map_err(inspect("find_configuration_table (ACPI2)"))
        .or_else(|_| find_configuration_table(&ACPI_GUID))
        .map_err(inspect("find_configuration_table (ACPI1)"))
        .ignore_warning()?;

    let rsdp = unsafe {
        (rsdp_ptr.address as *const RootSystemDescriptionPointer3)
            .read_unaligned()
    };

    let sdt_ptr = {
        if rsdp.revision >= ACPI_2_RSDP_REVISION && rsdp.xsdt_address != 0 {
            rsdp.xsdt_address as usize
        } else {
            rsdp.rsdt_address as usize
        }
    };
    if sdt_ptr == 0 {
        error!("Could not find RSD pointer table!");
        return Err(uefi::Status::UNSUPPORTED.into());
    }

    let sdt = unsafe {
        (sdt_ptr as *const DescriptionHeader)
            .read_unaligned()
    };

    let mut acpi_table_data = FixedDescriptionTable1 {
        header: DescriptionHeader {
            signature: ACPI_1_FADT_SIGNATURE,
            length: acpi_table_size as u32,
            revision: ACPI_1_FADT_REVISION,
            checksum: 0,
            oem_id: sdt.oem_id,
            oem_table_id: sdt.oem_table_id,
            oem_revision: sdt.oem_revision,
            creator_id: 0,
            creator_revision: 0
        },
        firmware_ctrl: 0,
        dsdt: 0,
        int_model: 0,
        reserved0: 0,
        sci_int: 0,
        smi_cmd: 0,
        acpi_enable: 0,
        acpi_disable: 0,
        s4_bios_req: 0,
        reserved1: 0,
        pm1a_evt_blk: 0,
        pm1b_evt_blk: 0,
        pm1a_cnt_blk: 0,
        pm1b_cnt_blk: 0,
        pm2_cnt_blk: 0,
        pm_tmr_blk: 0,
        gpe0_blk: 0,
        gpe1_blk: 0,
        pm1_evt_len: 0,
        pm1_cnt_len: 0,
        pm2_cnt_len: 0,
        pm_tm_len: 0,
        gpe0_blk_len: 0,
        gpe1_blk_len: 0,
        gpe1_base: 0,
        reserved2: 0,
        p_lvl2_lat: 0,
        p_lvl3_lat: 0,
        flush_size: 0,
        flush_stride: 0,
        duty_offset: 0,
        duty_width: 0,
        day_alrm: 0,
        mon_alrm: 0,
        century: 0,
        reserved3: 0,
        reserved4: 0,
        reserved5: 0,
        flags: 0,
    };

    // SAFETY: acpi_table_data must be properly initialized e.g. packed
    unsafe {
        acpi_table_data.header.checksum = checksum8(&acpi_table_data);
    }

    unsafe {
        // Entire table must sum to zero
        assert!(checksum8(&acpi_table_data) == 0);
    }

    // SAFETY: not safe at all
    unsafe {
        bs.memmove(
            acpi_table,
            &acpi_table_data as *const FixedDescriptionTable1 as *const u8,
            acpi_table_size
        );
    }

    let table_key = unsafe {
        acpi.install_acpi_table(acpi_table as *const c_void, acpi_table_size)
            .map_err(inspect("install_acpi_table"))
            .ignore_warning()?
    };

    Ok(table_key.into())
}

fn patch_dword(v: &mut [u8], old: u32, new: u32) {
    let mut start = 0;
    let mut end = 4;
    while end <= v.len() {
        let window = &mut v[start..end];
        if *window == u32::to_le_bytes(old) {
            window[0] = new as u8;
            window[1] = (new >> 8) as u8;
            window[2] = (new >> 16) as u8;
            window[3] = (new >> 24) as u8;
        }
        start += 1;
        end += 1;
    }
}

fn install_my_ssdt_table(phys_addr: u64) -> uefi::Result<usize> {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    let acpi = bs
        .locate_protocol::<AcpiTable>()
        .ignore_warning()?;
    let acpi = unsafe { &mut *acpi.get() };

    let acpi_table_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let acpi_table_size = mem::size_of::<MySsdtTable>();
    let acpi_table = bs.allocate_pool(acpi_table_pool, acpi_table_size)
        .map_err(inspect("allocate_pool (acpi_table)"))
        .ignore_warning()?;
    info!("acpi_table: {:?}", acpi_table);

    let rsdp_ptr = find_configuration_table(&ACPI2_GUID)
        .map_err(inspect("find_configuration_table (ACPI2)"))
        .or_else(|_| find_configuration_table(&ACPI_GUID))
        .map_err(inspect("find_configuration_table (ACPI1)"))
        .ignore_warning()?;

    let rsdp = unsafe {
        (rsdp_ptr.address as *const RootSystemDescriptionPointer3)
            .read_unaligned()
    };

    let sdt_ptr = {
        if rsdp.revision >= ACPI_2_RSDP_REVISION && rsdp.xsdt_address != 0 {
            rsdp.xsdt_address as usize
        } else {
            rsdp.rsdt_address as usize
        }
    };
    if sdt_ptr == 0 {
        error!("Could not find RSD pointer table!");
        return Err(uefi::Status::UNSUPPORTED.into());
    }

    let sdt = unsafe {
        (sdt_ptr as *const DescriptionHeader)
            .read_unaligned()
    };

    // Patching AML code is fine because we don't change its length
    let mut aml_code = *MY_AML_CODE;
    patch_dword(&mut aml_code, 0x11223344, phys_addr as u32);

    let mut acpi_table_data = MySsdtTable {
        header: DescriptionHeader {
            signature: SSDT_SIGNATURE,
            length: acpi_table_size as u32,
            revision: SSDT_REVISION,
            checksum: 0,
            // oem_id: sdt.oem_id,
            // oem_table_id: sdt.oem_table_id,
            // oem_revision: sdt.oem_revision,
            oem_id: OEM_VENDOR_ID,
            oem_table_id: OEM_TABLE_ID,
            oem_revision: OEM_REVISION,
            creator_id: 0,
            creator_revision: 0
        },
        aml_code
    };

    // SAFETY: acpi_table_data must be properly initialized e.g. packed
    unsafe {
        acpi_table_data.header.checksum = checksum8(&acpi_table_data);
    }

    unsafe {
        // Entire table must sum to zero
        assert!(checksum8(&acpi_table_data) == 0);
    }

    // SAFETY: not safe at all
    unsafe {
        bs.memmove(
            acpi_table,
            &acpi_table_data as *const MySsdtTable as *const u8,
            acpi_table_size
        );
    }

    let table_key = unsafe {
        acpi.install_acpi_table(acpi_table as *const c_void, acpi_table_size)
            .map_err(inspect("install_acpi_table"))
            .ignore_warning()?
    };

    Ok(0.into())
}

fn install_bpbt_table(phys_addr: u64) -> uefi::Result<usize> {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    let acpi = bs
        .locate_protocol::<AcpiTable>()
        .ignore_warning()?;
    let acpi = unsafe { &mut *acpi.get() };

    let acpi_table_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let acpi_table_size = mem::size_of::<MyBpbtTable>();
    let acpi_table = bs.allocate_pool(acpi_table_pool, acpi_table_size)
        .map_err(inspect("allocate_pool (acpi_table)"))
        .ignore_warning()?;
    info!("acpi_table: {:?}", acpi_table);

    let rsdp_ptr = find_configuration_table(&ACPI2_GUID)
        .map_err(inspect("find_configuration_table (ACPI2)"))
        .or_else(|_| find_configuration_table(&ACPI_GUID))
        .map_err(inspect("find_configuration_table (ACPI1)"))
        .ignore_warning()?;

    let rsdp = unsafe {
        (rsdp_ptr.address as *const RootSystemDescriptionPointer3)
            .read_unaligned()
    };

    let sdt_ptr = {
        if rsdp.revision >= ACPI_2_RSDP_REVISION && rsdp.xsdt_address != 0 {
            rsdp.xsdt_address as usize
        } else {
            rsdp.rsdt_address as usize
        }
    };
    if sdt_ptr == 0 {
        error!("Could not find RSD pointer table!");
        return Err(uefi::Status::UNSUPPORTED.into());
    }

    let sdt = unsafe {
        (sdt_ptr as *const DescriptionHeader)
            .read_unaligned()
    };

    // TBD: test my own ACPI tables
    let mut acpi_table_data = MyBpbtTable {
        header: DescriptionHeader {
            signature: MY_TABLE_SIGNATURE,
            length: acpi_table_size as u32,
            revision: MY_TABLE_REVISION,
            checksum: 0,
            oem_id: sdt.oem_id,
            oem_table_id: sdt.oem_table_id,
            oem_revision: sdt.oem_revision,
            creator_id: 0,
            creator_revision: 0
        },
        payload: MyPayload {
            magic: 0xfeeddead,
            physical_address: phys_addr,
            length_bytes: PAGE_COUNT as u64 * 4096,
        }
    };

    // SAFETY: acpi_table_data must be properly initialized e.g. packed
    unsafe {
        acpi_table_data.header.checksum = checksum8(&acpi_table_data);
    }

    unsafe {
        // Entire table must sum to zero
        assert!(checksum8(&acpi_table_data) == 0);
    }

    // SAFETY: not safe at all
    unsafe {
        bs.memmove(
            acpi_table,
            &acpi_table_data as *const MyBpbtTable as *const u8,
            acpi_table_size
        );
    }

    let table_key = unsafe {
        acpi.install_acpi_table(acpi_table as *const c_void, acpi_table_size)
            .map_err(inspect("install_acpi_table"))
            .ignore_warning()?
    };

    Ok(0.into())
}

fn allocate_mmio_page() -> uefi::Result<u64> {
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    for n in 0..256 {
        let pages_type = AllocateType::Address (4096 * n);
        let pages_pool = MemoryType::MMIO;
        let pages_count = PAGE_COUNT;
        let result = bs.allocate_pages(pages_type, pages_pool, pages_count)
            .map_err(inspect("allocate_pages"))
            .ignore_warning();
        match result {
            Ok(mmio_addr) => {
                info!("allocate_mmio_page {:#x} -> SUCCESS", mmio_addr);
                return Ok(mmio_addr.into())
            },
            Err(error) => {
                info!("allocate_mmio_page {:#x} -> {:?}", 4096*n, error.status());
            },
        }
    }
    Err(uefi::Status::NOT_FOUND.into())
}

#[entry]
fn efi_main(handle: Handle, system_table: SystemTable<Boot>) -> uefi::Status {
    uefi_services::init(&system_table)
        .expect_success("this is only the beginning");
    info!("bpb_main");
    let bs = unsafe {
        uefi_services::system_table()
            .as_ref()
            .boot_services()
    };

    enum_acpi_table_protocols()?;

    let mmio_addr = allocate_mmio_page()
        .ignore_warning()
        .map_err(inspect("allocate_mmio_page"))?;
    info!("mmio_addr: {:#x}", mmio_addr);

    let pages_type = AllocateType::AnyPages;
    let pages_pool = MemoryType::RUNTIME_SERVICES_DATA;
    let pages_count = PAGE_COUNT;
    let phys_addr = bs.allocate_pages(pages_type, pages_pool, pages_count)
        .map_err(inspect("allocate_pages"))
        .ignore_warning()?;
    info!("phys_addr: {:#x}", phys_addr);

    let region = find_region(phys_addr)
        .ignore_warning()?;
    info!("region: {:#?}", region);

    // SAFETY: looks safe to me
    unsafe {
        let probe: u32 = 0xfeaddead;
        bs.memmove(
            phys_addr as *mut u8,
            &probe as *const u32 as *const u8,
            mem::size_of::<u32>()
        );
    }

    // TBD: hang in the gBS call
    // install_configuration_table(phys_addr)
    //     .map_err(inspect("install_configuration_table"))?;

    // TBD: access denied
    // let table_key = install_fadt1(phys_addr)
    //     .map_err(inspect("install_fadt1"))
    //     .ignore_warning()?;
    // info!("table_key: {:?}", table_key);

    // TBD: access denied
    // let table_key = install_fadt3(phys_addr)
    //     .map_err(inspect("install_fadt3"))
    //     .ignore_warning()?;
    // info!("table_key: {:?}", table_key);

    let table_key = install_bpbt_table(phys_addr)
        .map_err(inspect("install_bpbt_table"))
        .ignore_warning()?;
    info!("table_key: {:?}", table_key);

    let table_key1 = install_my_ssdt_table(phys_addr)
        .map_err(inspect("install_my_ssdt_table"))
        .ignore_warning()?;
    info!("table_key1: {:?}", table_key1);

    let rt = unsafe {
        uefi_services::system_table()
            .as_ref()
            .runtime_services()
    };

    let buffer = &mut [0u16; 256];
    rt.set_variable(
        CStr16::from_str_with_buf("BpbAddress", buffer).ok().unwrap(),
        &MY_VENDOR_GUID,
        VariableAttributes::RUNTIME_ACCESS | VariableAttributes::BOOTSERVICE_ACCESS,
        &phys_addr.to_le_bytes())
        .map_err(inspect("set_variable"));

    let system_table_addr = &system_table as *const _ as u64;
    rt.set_variable(
        CStr16::from_str_with_buf("SystemTable", buffer).ok().unwrap(),
        &MY_VENDOR_GUID,
        VariableAttributes::RUNTIME_ACCESS | VariableAttributes::BOOTSERVICE_ACCESS,
        &system_table_addr.to_le_bytes())
        .map_err(inspect("set_variable"));

    info!("bpb_main -- ok");
    uefi::Status::SUCCESS
}
