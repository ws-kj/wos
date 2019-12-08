#include <stdio.h>

#include <kernel/tty.h>
#include <kernel/gdt.h>
#include <kernel/idt.h>
#include <kernel/isrs.h>
#include <kernel/irq.h>

void kernel_main(void) {
	gdt_install();
	idt_install();
	isrs_install();

	irq_install();
	__asm__ __volatile__("sti");

	terminal_initialize();

	printf("wos v0.01\n");
	printf("WARNING: This system is not stable.\n");
}
