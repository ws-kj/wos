#include <stdio.h>

#include <kernel/tty.h>
#include <kernel/gdt.h>
#include <kernel/idt.h>
#include <kernel/isrs.h>

void kernel_main(void) {
	gdt_install();
	idt_install();
	isrs_install();
	terminal_initialize();

	printf("wos v0.1\n");
	printf("Hello, kernel World!\n");
	//test exception
	printf("%d", (1/0));
}
