cd #!/usr/bin/env bash

set -e

DEV=/dev/loop2
WHOAMI=$(whoami)
WORKDIR=/home/reggies/uefi-bpb-rs

dd if=/dev/zero of=$WORKDIR/tmp.img count=1 bs=$((512*1024*1024))
fdisk $WORKDIR/tmp.img <<EOF
g
n
1
2048
1048542
t
1
w
EOF
sudo losetup -P $DEV $WORKDIR/tmp.img
sudo mkfs.fat -F 32 ${DEV}p1
mkdir -p $WORKDIR/mnt
sudo mount -o umask=0022,gid="$WHOAMI",uid="$WHOAMI" ${DEV}p1 $WORKDIR/mnt
cp -vr $WORKDIR/hda/* $WORKDIR/mnt/
sudo umount $WORKDIR/mnt
sudo losetup -d $DEV
qemu-img convert -O vdi $WORKDIR/tmp.img $WORKDIR/tmp.vdi

VBoxManage storagectl win10-boot-param-block --controller PIIX4 --name PIIX4 --remove
VBoxManage closemedium disk $WORKDIR/disk.vdi
mv $WORKDIR/tmp.vdi $WORKDIR/disk.vdi
VBoxManage storagectl win10-boot-param-block --add ide --controller PIIX4 --name PIIX4
VBoxManage storageattach win10-boot-param-block --storagectl PIIX4 --port 0 --device 0 --type hdd --medium $WORKDIR/disk.vdi
