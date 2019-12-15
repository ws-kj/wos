#include "common.h"
#include "monitor.h"
#include "descriptor_tables.h"
#include "timer.h"

int kernel_main(struct multiboot *mboot_ptr) {
	init_descriptor_tables();
	monitor_clear();
	monitor_write("wos v0.01\n");

	for(;;) {
    		__asm__ __volatile__("hlt");
 	}

}
