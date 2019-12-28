// kheap.h -- Interface for kernel heap functions, also provides
//            a placement malloc() for use before the heap is 
//            initialised.
//            Written for JamesM's kernel development tutorials.

#ifndef KHEAP_H
#define KHEAP_H

#include "common.h"


uint32_t kmalloc_int(uint32_t sz, int align, uint32_t *phys);

/**
   Allocate a chunk of memory, sz in size. The chunk must be
   page aligned.
**/
uint32_t kmalloc_a(uint32_t sz);

/**
   Allocate a chunk of memory, sz in size. The physical address
   is returned in phys. Phys MUST be a valid pointer to uint32_t!
**/
uint32_t kmalloc_p(uint32_t sz, uint32_t *phys);

/**
   Allocate a chunk of memory, sz in size. The physical address 
   is returned in phys. It must be page-aligned.
**/
uint32_t kmalloc_ap(uint32_t sz, uint32_t *phys);

/**
   General allocation function.
**/
uint32_t kmalloc(uint32_t sz);

/**
   General deallocation function.
**/
void kfree(void *p);

#endif // KHEAP_H
