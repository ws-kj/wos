#include "common.h"
#include "monitor.h"

int kernel_main(struct multiboot *mboot_ptr) {
	monitor_clear();
	monitor_write("Hello, world!");
	return 0xDEADBABA;
}
