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

	for(;;) {
    		__asm__ __volatile__("hlt");
 	}

}
