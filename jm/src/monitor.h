#ifndef MONITOR_H
#define MONITOR_H

//#include "common.h"

void monitor_put(char c);

void monitor_clear();

void monitor_write(char *c);

void monitor_write_dec(uint32_t n);

void monitor_write_hex(uint32_t n);

#endif
