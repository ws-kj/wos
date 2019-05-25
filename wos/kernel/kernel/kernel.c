#include <stdio.h>

#include <kernel/tty.h>

void kernel_main(void) {
	terminal_initialize();
	printf("wos v0.1\n");
	printf("Hello, kernel World!\n");
}
