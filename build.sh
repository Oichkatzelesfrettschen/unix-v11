#!/bin/zsh

set -e

cargo build -r --target x86_64-unknown-uefi
mkdir -p dist/efi/boot
cp target/x86_64-unknown-uefi/release/unixv11.efi dist/efi/boot/bootx64.efi

dd if=/dev/zero of=uefi.img bs=1m count=64
diskno=$(hdiutil attach -imagekey diskimage-class=CRawDiskImage -nomount uefi.img | head -n 1 | awk '{print $1}')
diskutil eraseDisk FAT32 UNIXV11 MBRFormat $diskno
cp -R dist/* /Volumes/UNIXV11/
hdiutil detach $diskno

qemu-system-x86_64 -cpu qemu64 -bios ovmf_amd64.fd -drive file=uefi.img,if=ide -m 512M