rm -rf *.o
nasm -o start.o start.asm -f elf32
gcc -Wall -O -fstrength-reduce -fomit-frame-pointer -finline-functions -nostdinc -fno-builtin -I./include -c -o kernel.o kernel.c
i686-elf-gcc -T link.ld -o kernel.bin -ffreestanding -O2 -nostdlib start.o -lgcc
