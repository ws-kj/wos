#ifndef _KERNEL_GDT_H
#define _KERNEL_GDT_H

extern void gdt_set_gate(int num, unsigned long base, unsigned long limit, unsigned char access, unsigned char gran);
extern void gdt_install();

#endif
