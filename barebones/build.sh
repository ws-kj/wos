i686-elf-as boot.s -o boot.o
i686-elf-gcc -c kernel.c -o kernel.o -std=gnu99 -ffreestanding -O2 -Wall -Wextra
i686-elf-gcc -T linker.ld -o wos.bin -ffreestanding -O2 -nostdlib boot.o kernel.o -lgcc

mkdir -p isodir/boot/grub
cp wos.bin isodir/boot/wos.bin
cp grub.cfg isodir/boot/grub/grub.cfg
grub-mkrescue -o wos.iso isodir

rm -rf *.bin

qemu-system-i386 wos.iso
