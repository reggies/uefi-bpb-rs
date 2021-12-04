#!/usr/bin/env bash

set -e

./build.sh
cp ./target/x86_64-unknown-uefi/debug/bpb-test.efi qemu-hda

qemu-system-x86_64 \
    -machine q35 \
    -m 1024 \
    -vga std \
    -hda fat:rw:qemu-hda \
    -bios ovmf/OVMF.fd \
    -debugcon file:debug.log \
    -global isa-debugcon.iobase=0x402 \
    -s \
    -serial file:serial.txt \
    -serial stdio
