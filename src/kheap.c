// kheap.c -- Kernel heap functions, also provides
//            a placement malloc() for use before the heap is 
//            initialised.
//            Written for JamesM's kernel development tutorials.

#include "kheap.h"
#include "paging.h"
#include "stddef.h"

// end is defined in the linker script.
extern uint32_t end;
uint32_t placement_address = (uint32_t)&end;
extern page_directory_t *kernel_directory;

uint32_t kmalloc_int(uint32_t sz, int align, uint32_t *phys)
{
	if (align == 1 && (placement_address & 0xFFFFF000) ){
    		placement_address = (placement_address + 0xFFF) & ~((size_t)0xFFF);
	}
	if (phys) {
    		*phys = placement_address;
	}
	uint32_t tmp = placement_address;
	placement_address += sz;
	return tmp;
}

uint32_t kmalloc_a(uint32_t sz)
{
    return kmalloc_int(sz, 1, 0);
}

uint32_t kmalloc_p(uint32_t sz, uint32_t *phys)
{
    return kmalloc_int(sz, 0, phys);
}

uint32_t kmalloc_ap(uint32_t sz, uint32_t *phys)
{
    return kmalloc_int(sz, 1, phys);
}

uint32_t kmalloc(uint32_t sz)
{
    return kmalloc_int(sz, 0, 0);
}
