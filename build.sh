#!/usr/bin/env bash

set -e

# rustup install nightly
# rustup component add build-std
# rustup default nightly
cargo build -Z patch-in-config -Z build-std --target x86_64-unknown-uefi

FILE=$(readlink -f $0)
FILEPATH=`dirname $FILE`

export WORKSPACE=~/edk2
export EDK_TOOLS_PATH=$WORKSPACE/BaseTools
export PACKAGES_PATH=$FILEPATH:~/edk2-libc:$WORKSPACE

source $WORKSPACE/edksetup.sh

build \
    -t GCC5 \
    -b DEBUG \
    -p BpbPkg/BpbPkg.dsc \
    -m BpbPkg/BpbDxe/BpbDxe.inf \
    -a X64 \
    -n 8
