#include <kernel/timer.h>
#include <kernel/irq.h>
#include <kernel/tty.h>

volatile int timer_ticks = 0;

void timer_handler(struct regs *r) {
	timer_ticks ++;

	if (timer_ticks % 18 == 0) {
		printf(1/0);
	}
}

void wait(int ticks) {
	volatile unsigned int eticks;

	eticks = timer_ticks + ticks;
	while(timer_ticks < eticks) {
		__asm__ __volatile__ ("sti//hlt//cli");
	}
}

void timer_install() {
	irq_install_handler(0, timer_handler);
}
