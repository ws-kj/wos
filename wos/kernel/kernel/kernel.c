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

	printf("wos v0.01\n");
	printf("WARNING: This system is not stable.\n");
	printf("Hello, World!\n\n");
	//test exception
}
