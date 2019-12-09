#include <kernel/timer.h>
#include <kernel/irq.h>
#include <kernel/tty.h>

volatile unsigned int timer_ticks = 0;

void timer_handler(struct regs *r) {
	timer_ticks++;
	if(timer_ticks % 18 ==0) {
		printf("1 second\n");
	}
}

void timer_install() {
	irq_install_handler(0, timer_handler);
}

void timer_wait(int ticks)
{
	unsigned int eticks;
	     
        eticks = timer_ticks + ticks;
	while(timer_ticks < eticks) 
        {
		__asm__ __volatile__ ("sti//hlt//cli");
	}
}
