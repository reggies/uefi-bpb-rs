#!/usr/bin/env bash

set -e

WHOAMI=$(whoami)

sudo modprobe nbd
sudo qemu-nbd -c /dev/nbd1 disk.vdi
mkdir -p mnt
sudo mount -o umask=0022,gid="$WHOAMI",uid="$WHOAMI" /dev/nbd1p1 mnt
cp -vr hda/* ./mnt/
sudo umount mnt
sudo qemu-nbd -d /dev/nbd1
