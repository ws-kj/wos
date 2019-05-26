# Declare constants for the multiboot header.
.set ALIGN,    1<<0             # align loaded modules on page boundaries
.set MEMINFO,  1<<1             # provide memory map
.set FLAGS,    ALIGN | MEMINFO  # this is the Multiboot 'flag' field
.set MAGIC,    0x1BADB002       # 'magic number' lets bootloader find the header
.set CHECKSUM, -(MAGIC + FLAGS) # checksum of above, to prove we are multiboot

# Declare a header as in the Multiboot Standard.
.section .multiboot
.align 4
.long MAGIC
.long FLAGS
.long CHECKSUM

# Reserve a stack for the initial thread.
.section .bss
.align 16
stack_bottom:
.skip 16384 # 16 KiB
stack_top:

.section .text

#kernel entry point
.global _start
.type _start, @function
_start:
	movl $stack_top, %esp

	# Call the global constructors.
	call _init

	# Transfer control to the main kernel.
	call kernel_main

	# Hang if kernel_main unexpectedly returns.
	cli
1:	hlt
	jmp 1b
.size _start, . - _start


#setup gdt
.global gdt_flush     # Allows the C code to link to this
.type gdt_flush, @function
.global gp
gdt_flush:
 	lgdt (gp)        # Load the GDT with our '_gp' which is a special pointer
        mov $0x10, %ax       # 0x10 is the offset in the GDT to our data segment
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %fs
	mov %ax, %gs
    	mov %ax, %ss
	ljmp $0x8,$flush2   # 0x08 is the offset to our code segment: Far jump!
flush2:
	ret   


#load idt 
.global idt_load
.type idt_load, @function
.global idtp
idt_load:
	lidt (idtp)
	ret

#ISRs
.global isr0
.global isr1
.global isr2
.global isr3
.global isr4
.global isr5
.global isr6
.global isr7
.global isr8
.global isr9
.global isr10
.global isr11
.global isr12
.global isr13
.global isr14
.global isr15
.global isr16
.global isr17
.global isr18
.global isr19
.global isr20
.global isr21
.global isr22
.global isr23
.global isr24
.global isr25
.global isr26
.global isr27
.global isr28
.global isr29
.global isr30
.global isr31

isr0:
	cli
	push 	$0, byte
	push	$0, byte
	jmp 	isr_common_stub
isr1:
	cli
	push 	$0, byte
	push	$1, byte
	jmp 	isr_common_stub
isr2:
	cli
	push 	$0, byte
	push	$2, byte
	jmp 	isr_common_stub
isr4:
	cli
	push 	$0, byte
	push	$3, byte
	jmp 	isr_common_stub
isr5:
	cli
	push 	$0, byte
	push	$4, byte
	jmp 	isr_common_stub
isr6:
	cli
	push 	$0, byte
	push	$5, byte
	jmp 	isr_common_stub
isr7:
	cli
	push 	$0, byte
	push	$6, byte
	jmp 	isr_common_stub
isr8:
	cli
	push 	$0, byte
	push	$7, byte
	jmp 	isr_common_stub
isr9:
	cli
	push 	$0, byte
	push	$8, byte
	jmp 	isr_common_stub
isr10:
	cli
	push 	$0, byte
	push	$9, byte
	jmp 	isr_common_stub
isr11:
	cli
	push 	$0, byte
	push	$10, byte
	jmp 	isr_common_stub
isr12:
	cli
	push 	$0, byte
	push	$11, byte
	jmp 	isr_common_stub
isr13:
	cli
	push 	$0, byte
	push	$12, byte
	jmp 	isr_common_stub
isr14:
	cli
	push 	$0, byte
	push	$13, byte
	jmp 	isr_common_stub
isr15:
	cli
	push 	$0, byte
	push	$14, byte
	jmp 	isr_common_stub
isr16:
	cli
	push 	$0, byte
	push	$15, byte
	jmp 	isr_common_stub
isr17:
	cli
	push 	$0, byte
	push	$16, byte
	jmp 	isr_common_stub
isr18:
	cli
	push 	$0, byte
	push	$17, byte
	jmp 	isr_common_stub
isr18:
	cli
	push 	$0, byte
	push	$18, byte
	jmp 	isr_common_stub
isr19:
	cli
	push 	$0, byte
	push	$19, byte
	jmp 	isr_common_stub
isr20:
	cli
	push 	$0, byte
	push	$20, byte
	jmp 	isr_common_stub
isr21:
	cli
	push 	$0, byte
	push	$21, byte
	jmp 	isr_common_stub
isr22:
	cli
	push 	$0, byte
	push	$22, byte
	jmp 	isr_common_stub
isr23:
	cli
	push 	$0, byte
	push	$23, byte
	jmp 	isr_common_stub
isr24:
	cli
	push 	$0, byte
	push	$24, byte
	jmp 	isr_common_stub
isr25:
	cli
	push 	$0, byte
	push	$25, byte
	jmp 	isr_common_stub
isr26:
	cli
	push 	$0, byte
	push	$26, byte
	jmp 	isr_common_stub
isr27:
	cli
	push 	$0, byte
	push	$27, byte
	jmp 	isr_common_stub
isr28:
	cli
	push 	$0, byte
	push	$28, byte
	jmp 	isr_common_stub
isr29:
	cli
	push 	$0, byte
	push	$29, byte
	jmp 	isr_common_stub
isr30:
	cli
	push 	$0, byte
	push	$30, byte
	jmp 	isr_common_stub
isr31:
	cli
	push	$0, byte
	push	$31, byte
	jmp 	isr_common_stub
