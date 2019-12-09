#include <stdio.h>

#include <kernel/tty.h>
#include <kernel/gdt.h>
#include <kernel/idt.h>
#include <kernel/isrs.h>
#include <kernel/irq.h>
#include <kernel/timer.h>

void kernel_main(void) {
	gdt_install();
	idt_install();
	isrs_install();
	timer_install();
	irq_install();
	__asm__ __volatile__("sti");
	terminal_initialize();

	printf("wos v0.01\n");
	timer_wait(18);
	printf("WARNING: This system is not stable.\n");
	for(;;);
}
