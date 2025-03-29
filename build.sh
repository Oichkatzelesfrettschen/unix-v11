#!/bin/zsh

set -e

cargo build -r --target x86_64-unknown-uefi
mkdir -p dist/efi/boot
cp target/x86_64-unknown-uefi/release/unixv11.efi dist/efi/boot/bootx64.efi 
hdiutil create -fs fat32 -ov -size 64m -volname UNIXV11 -format UDTO -srcfolder dist uefi.cdr
qemu-system-x86_64 -cpu qemu64 -bios ovmf_amd64.fd -drive file=uefi.cdr,if=ide -m 512M