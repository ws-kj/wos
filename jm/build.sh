rm -rf *.o *.bin *.iso

nasm -felf32 boot.s

i686-elf-gcc -c common.c -o common.o -ffreestanding -std=gnu99 -O2 -Wall -Wextra
i686-elf-gcc -c monitor.c -o monitor.o -ffreestanding -std=gnu99 -O2 -Wall -Wextra
i686-elf-gcc -c main.c -o main.o -ffreestanding -std=gnu99 -O2 -Wall -Wextra
i686-elf-gcc -T link.ld -o kernel.bin -ffreestanding -O2 -nostdlib boot.o main.o common.o monitor.o -lgcc

