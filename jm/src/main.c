#include "common.h"
#include "monitor.h"
#include "descriptor_tables.h"

int kernel_main(struct multiboot *mboot_ptr) {
	init_descriptor_tables();
	monitor_clear();
	monitor_write("Hello, world!");
}
