#!/bin/sh
set -e
. ./build.sh

mkdir -p isodir
mkdir -p isodir/boot
mkdir -p isodir/boot/grub

cp sysroot/boot/wos.kernel isodir/boot/wos.kernel
cat > isodir/boot/grub/grub.cfg << EOF
menuentry "wos" {
	multiboot /boot/wos.kernel
}
EOF
grub-mkrescue -o wos.iso isodir
