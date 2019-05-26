#include <stdio.h>

#include <kernel/tty.h>
#include <kernel/gdt.h>

void kernel_main(void) {
	gdt_install();
	terminal_initialize();
	printf("wos v0.1\n");
	printf("Hello, kernel World!\n");
}
