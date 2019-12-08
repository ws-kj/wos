#include <kernel/timer.h>
#include <kernel/irq.h>
#include <kernel/tty.h>

int timer_ticks = 0;

void timer_handler(struct regs *r) {
	timer_ticks ++;

	if (timer_ticks % 18 == 0) {
		terminal_writestring("One second has passed.\n");
	}
}

void timer_install() {
	irq_install_handler(0, timer_handler);
}
