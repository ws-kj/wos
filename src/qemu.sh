

cp wos.bin isodir/boot/wos.bin  
grub-mkrescue -o wos.iso isodir
qemu-system-i386 -monitor stdio wos.iso

