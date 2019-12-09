#ifndef KERNEL_TIMER_H
#define KERNEL_TIMER_H

extern void timer_handler(struct regs *r);
extern void timer_install();
extern void timer_wait(int ticks);

#endif
