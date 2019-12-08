#ifndef KERNEL_IRQ_H
#define	KERNEL_IRQ_H

extern void irq_install();
//extern void irq_install_handler(int irq, void (*handler)(struct regs *r));
extern void irq_uninstall_handler(int irq);

#endif
