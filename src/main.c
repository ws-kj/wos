#include "common.h"
#include "monitor.h"
#include "descriptor_tables.h"
#include "timer.h"
#include "keyboard.h"
#include "paging.h"
#include "kheap.h"

int kernel_main(struct multiboot *mboot_ptr) {
	init_descriptor_tables();
	monitor_clear();
	uint32_t a = kmalloc(8);
	init_paging();

	init_timer(50);
	init_keyboard();

	monitor_write("wos v0.01\n");

	uint32_t b = kmalloc(8);
	uint32_t c = kmalloc( 8);
	monitor_write("a: ");
	monitor_write_hex(a);
	monitor_write("b: ");
	monitor_write_hex(b);
	monitor_write("\nc: ");
	monitor_write_hex(c);
	kfree(c);
	kfree(b);
	uint32_t d = kmalloc(12);
	monitor_write(", d: ");
	monitor_write_hex(d); 

	for(;;) {
    		__asm__ __volatile__("hlt");
 	}

}
