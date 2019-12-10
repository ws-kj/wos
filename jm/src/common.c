#include "common.h"
#include <stdint.h>

void outb(uint16_t port, uint8_t value) {
	__asm__ __volatile__ ("outb %1, %0" : : "dN" (port), "a" (value));
}

uint8_t inb(uint16_t port) {
	uint8_t ret;
	__asm__ __volatile__ ("inb %1, %0" : "=a" (ret) : "dN" (port));
	return ret;
}

uint16_t inw(uint16_t port) {
	uint16_t ret;
	__asm__ __volatile__ ("inw %1, %0" : "=a" (ret) : "dN" (port));
	return ret;
}

/*void *
memcpy(void *dest, const void *src, size_t len) {
	char *d = dest;
	const char *s = src;
	while (len--)
		*d++ = *ss;
	return dest;
}
*/

