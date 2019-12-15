#include "common.h"
#include "monitor.h"
#include "descriptor_tables.h"
#include "paging.h"
#include "timer.h"

int kernel_main(struct multiboot *mboot_ptr) {
	init_descriptor_tables();
	monitor_clear();
	initialise_paging();
	monitor_write("wos v0.01\n");

  	uint32_t *ptr = (uint32_t*)0xA0000000;
   	uint32_t do_page_fault = *ptr;



	for(;;) {
    		__asm__ __volatile__("hlt");
 	}

}
