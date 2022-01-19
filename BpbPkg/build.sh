#!/usr/bin/env bash

set -e

FILE=$(readlink -f $0)
FILEPATH=`dirname $FILE`

export WORKSPACE=~/edk2
export EDK_TOOLS_PATH=$WORKSPACE/BaseTools
export PACKAGES_PATH=$FILEPATH:~/edk2-libc:$WORKSPACE

source $WORKSPACE/edksetup.sh

build \
    -t GCC5 \
    -b DEBUG \
    -p BpbPkg.dsc \
    -m RuntimeSpyDxe/RuntimeSpyDxe.inf \
    -a X64 \
    -n 8

build \
    -t GCC5 \
    -b DEBUG \
    -p BpbPkg.dsc \
    -m BpbDxe/BpbDxe.inf \
    -a X64 \
    -n 8
