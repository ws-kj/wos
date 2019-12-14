

cp kernel.bin isodir/boot/kernel.bin  
grub-mkrescue -o kernel.iso isodir
qemu-system-i386 kernel.iso

