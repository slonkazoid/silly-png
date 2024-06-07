#!/usr/bin/env bash

# use all threads and half of available system memory
threads="$(nproc)"
mem="$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 2048))"
image_path="/tmp/funny.iso"

qemu="qemu-system-x86_64"
flags="-boot d -cdrom $image_path -m $mem -smp $threads -vga qxl"
[ -e /dev/kvm ] && flags+=" -accel kvm" # enable kvm if available

extract > "$image_path"

$qemu $flags; rm "$image_path" &
