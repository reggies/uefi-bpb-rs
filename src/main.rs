#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate uefi;
#[macro_use]
extern crate alloc;

use uefi::prelude::*;

const PHYS_ADDR: u64 = 0x10000;

const EFI_ACPI_CONFIGURATION_TABLE_GUID: uefi::Guid = uefi::Guid::from_values(
    0x8868e871,
    0xe4f1,
    0x11d3,
    0xbc,
    0x22,
    [0x0, 0x80, 0xc7, 0x3c, 0x88, 0x81]
);

fn main() {
    uefi_services::init(&system_table)
        .expect_success("this is only the beginning");
    info!("main");
    let bt = unsafe { uefi_services::system_table().as_ref().boot_services() };

    let phys_addr = bt.allocate_pages(
        uefi::AllocateType::Address(PHYS_ADDR),
        uefi::MemoryType::BOOT_SERVICES_DATA,
        1)
        .ignore_warning()?;

    info!("phys_addr: {:?}", phys_addr);

    let table = bt.allocate_pool(uefi::MemoryType::RUNTMIE_SERVICE_DATA, 1).ignore_warning()?

    unsafe {
        bt.install_configuration_table(&EFI_ACPI_CONFIGURATION_TABLE_GUID, table)?;
    }

    info!("main -- ok");
    uefi::Status::SUCCESS
}
