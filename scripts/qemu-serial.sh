#!/usr/bin/env bash

# use all threads and quarter of available system memory
threads="$(nproc)"
mem="$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 4096))"
image_path="/tmp/funny.iso"

qemu="qemu-system-x86_64"
flags="-boot d -cdrom $image_path -m $mem -smp $threads -nographic"
[ -e /dev/kvm ] && flags+=" -accel kvm" # enable kvm if available

extract > "$image_path"

clear
$qemu $flags
rm "$image_path"
