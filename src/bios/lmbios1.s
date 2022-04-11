#
# lmbios1 - Call a Real Mode function (e.g. BIOS function) from Long Mode
#
# lmbios1 provides a method to call a Real Mode function from Long Mode.
# That is, it switches CPU operating mode from Long Mode to Real Mode,
# calls a function in Real Mode, then switches back to Long Mode.
# As the name suggests, its main purpose is to call BIOS functions
# from Long Mode.
#
# lmbios1 assumes that:
#
#   (1) It is used with lmboot0.
#       In addition, it is assumed that configurations made by lmboot0
#       have not been changed.  That is, Paging type is Identity Mapped
#       Paging, Global Descriptor Table (GDT) has the same selectors, etc.
#       It is also assumed that __lmboot0_boot_drive_id is accessible.
#
#   (2) Interrupt vector table (IVT) has not been changed.
#       BIOS is called via software interrupts, which depend on IVT.
#       If IVT is changed, unexpected function may be called.
#       IVT resides from 0x0000 to 0x03FF (Size: 1KB).
#
#   (3) It is not called simultaneously.
#       Mutual exclusion is not implemented.
#
#   (4) Performance is not a big issue.
#       Typical use cases might be to build an experimental environment
#       for learning purposes, a mockup program, a prototype program,
#       or a short-lived program such as a loader of a small kernel or
#       a loader of a more sophisticated loader of a large kernel.
#
# lmbios1 requires that:
#
#   (1) Code address and stack address are less than 64KB.
#       In Real Mode, Instruction Pointer (IP) and Stack Pointer (SP)
#       need to be copyable to 16-bit registers, while lmbios1 keeps
#       all segment registers zero when it runs.
#
#       Note: Only during a Real Mode function call, DS and ES are
#       changed as specified by the parameter structure.
#
#   (2) Data addresses passed to lmbios1 are less than 4GB.
#       In Real Mode, data addresses need to be copyable to 32-bit
#       regsiters.
#
#       Note: Thanks to Unreal Mode, lmbios1 can access up to 32-bit
#       address space (4GB) by using 32-bit register indirect addressing.
#
#   (3) Any address passed to Real Mode legacy function is less than 1MB.
#       Real Mode legacy function accesses memory using 16-bit segment
#       register and 16-bit offset (20-bit address space).  Therefore,
#       If memory buffer address should be exchanged with Real Mode
#       legacy function, e.g. BIOS, it should be allocated in 20-bit
#       address space (i.e., less than 1MB).
#
#   (4) Configurations made by lmboot0 must not be changed.
#

	.section .lmbios1, "xa"  # xa = executable, allocatable
	.globl lmbios1_dispatch


#########################################################################
#
# lmbios1_dispatch - Call function
#
# IN
#	RBX	: Pointer to parameter structure
#
# OUT
#	RAX	: Executed function number (0xFFFF for unsupported number)
#

	.p2align 4, 0x90  # 0x90 = NOP (= xchgl %eax, %eax)

lmbios1_dispatch:
	.code64

	# Copy function number to RAX.
	xorq	%rax, %rax
	movw	0x00(%rbx), %ax

	# 0x00 - 0xFF : Software Interrupt (INT n)
	cmp	$0xff, %ax
	jbe	lmbios1_intn

	# Additional dispatching code can be put here.

lmbios1_dispatch_unsupported:
	movw	$0xffff, %ax	# 0xFFFF means unsupported.
	retq


#########################################################################
#
# lmbios1_intn - Call Software Interrupt (INT n)
#
# IN
#	RAX	: Software Interrupt Number (INT n)
#	RBX	: Pointer to parameter structure
#
	.p2align 4, 0x90  # 0x90 = NOP (= xchgl %eax, %eax)

lmbios1_intn:
	.code64
	push	%rax		# Save function number.

	########################################################
	#
	# Construct the following instructions (4 bytes) on the stack.
	#
	#	CD hh	INT hh
	#	C3	RET
	#	90	NOP
	#
	# Because X86 is little-endian machine, these 4 bytes codes
	# should be represented in the reverse order, i.e., 0x90C3hhCD.
	#
	shlw	$8, %ax
	orl	$0x90c300cd, %eax
	pushq	%rax		# Instruction codes = 0xCD 0xhh 0xC3 0x90
	movq	%rsp, %rcx	# RCX = Entry point of "INT n; RET; NOP"

	movq	$lmbios1_exec, %rdx
	call	lmbios1_dive

	add	$8, %rsp	# Discard the constructed instruction.

	popq	%rax		# Restore function number to RAX.
	retq


#########################################################################
#
# lmbios1_dive - Dive into Real Mode and call specified subroutine.
#
# IN
#	RDX	: Entry point of subroutine
#
# Note: All general purpose registers are passed to specified subroutine.
#
	.p2align 4, 0x90  # 0x90 = NOP (= xchgl %eax, %eax)

lmbios1_dive:
	.code64
	pushq	%rax	# Save a working register value.

	.set	SEG_CODE64, 0x08	# Selector 1, GDT, RPL=0
	.set	SEG_CODE16, 0x10	# Selector 2, GDT, RPL=0
	.set	SEG_DATA,   0x18	# Selector 3, GDT, RPL=0

	.set	CR0_PE, (1 << 0)	# PE = Protected Mode Enable
	.set	CR0_PG, (1 << 31)	# PG = Paging

	########################################################
	#
	# Switch the segment selector of CS from Code 64 to Code 16.
	#
	# States: CPU = Long Mode, Code segment = 64-bit mode.
	#
	subq	$16, %rsp		# Prepare room for new CS + new RIP.
	movq	$SEG_CODE16, 8(%rsp)	# 8(%rsp) = SEG_CODE16 (new CS)
	movabsq	$lmbios1_dive_lm16, %rax
	movq	%rax, (%rsp)		# 0(%rsp) = lmbios1_dive_lm16 (new RIP)
	lretq				# CS:RIP = SEG_CODE16:lmbios1_dive_lm16

	# Note: lretq pops RIP and CS simultanously.
	# As a result, %rsp is increased by 16 (instead of 10).
	# That is, %rsp turns back to the original level.

lmbios1_dive_lm16:
	.code16

	# Now, code segment is 16-bit mode!

	########################################################
	#
	# Switch from Long Mode to Real Mode
	#
	# States: CPU = Long Mode, Code segment = 16-bit mode.
	#

	# Unset PE (Bit 0) & PG (Bit 31) in CR0 (Protected Mode Enable, Paging)
	movl	%cr0, %eax
	andl	$(~(CR0_PE | CR0_PG)), %eax
	movl	%eax, %cr0

	# Now, CPU is in Real Mode!

	########################################################
	#
	# Update internal segment descriptor caches
	# by setting segment registers.
	#
	# States: CPU = Real Mode, Code segment = 16-bit mode.
	#

	# Initialize segment registers for Real Mode
	xorw	%ax, %ax		# AX = 0x0000
	movw	%ax, %ds		# DS = 0x0000
	movw	%ax, %es		# ES = 0x0000
	movw	%ax, %fs		# FS = 0x0000
	movw	%ax, %gs		# GS = 0x0000
	movw	%ax, %ss		# SS = 0x0000

	jmp	$0x0000, $lmbios1_dive_rm16	# CS:IP = new CS : new IP
lmbios1_dive_rm16:

	# Now, all segment registers have Real-Mode-style values!
	# 32-bit address space can be accessed thanks to Unreal Mode!

	########################################################
	#
	# Call subroutine in Real Mode (so-called Unreal Mode).
	#
	# States: CPU = Real Mode, Code segment = 16-bit mode.
	#
	movl	(%esp), %eax	# Restore a working register value.

	# Call %dx via a complex method (see Appendix at the tail of this file)
	pushw	$lmbios1_dive_subr_done	# Instruction address next to retw
	pushw	%dx			# Subroutine address
	retw				# Pop %ip
lmbios1_dive_subr_done:

	########################################################
	#
	# Switch from Real Mode to Long Mode
	#
	# States: CPU = Real Mode, Code segment = 16-bit mode.
	#

	# Set PE (Bit 0) and PG (Bit 31) in CR0 (Protected Mode Enable, Paging)
	movl	%cr0, %eax
	orl	$(CR0_PE | CR0_PG), %eax
	movl	%eax, %cr0

	# Now, CPU turns back to Long Mode!
	# Note: It is assumed that other settings for Long Mode are unchanged.

	########################################################
	#
	# Update internal segment descriptor caches
	# by setting new segment selectors.
	#
	# States: CPU = Long Mode, Code segment = 16-bit mode.
	#
	movw	$SEG_DATA, %ax
	movw	%ax, %ds
	movw	%ax, %es
	movw	%ax, %fs
	movw	%ax, %gs
	movw	%ax, %ss

	# Clear the instruction prefetch queue and reset CS.
	jmp	$SEG_CODE64, $lmbios1_dive_lm64
lmbios1_dive_lm64:
	.code64

	# Now, code segment is 64-bit mode!
	# And, all segment registers have Long-Mode-style values!

	########################################################
	#
	# Return to the caller.
	#
	# States: CPU = Long Mode, Code segment = 64-bit mode.
	#
	popq	%rax	# Restore a working register value.
	retq


########################################################################
#
# lmbios1_exec - Call subroutine in Real Mode
#
# IN
#	EBX	: Pointer to parameters
#	ECX	: Entry point of subroutine
#
# Scratched: EAX, ECX, EDX, EBX, ESI, EDI, EBP
#

lmbios1_exec:
	.code16

	########################################################
	#
	# Save EBX, ES and DS.
	#
	pushl	%ebx
	pushw	%es
	pushw	%ds

	# Prepare to call %cx via a complex method.
	#   (for more information, see Appendix at the tail of this file)
	pushw	$lmbios1_exec_subr_done	# Instruction address next to retw
	pushw	%cx			# Subroutine address

	# Figure of the stack top at this moment.
	#
	# Offset    Stack contents
	#       +---------------------+
	# 00-03 | Subroutine address  | = Original CX
	#       +---------------------+
	# 04-07 | Instruction address | = lmbios1_exec_subr_done
	#       +---------------------+
	# 08-09 | Saved DS            |
	#       +---------------------+
	# 0A-0B | Saved ES            |
	#       +---------------------+
	# 0C-0F | Saved EBX           |
	#       +---------------------+

	# Load specified parameters to registers.
	# At first, push DS and ES.
	movl	0x20(%ebx), %eax	# DS and ES
	pushl	%eax

	# Next, load specified parameters to general purpose registers.
	movl	0x04(%ebx), %eax	# EAX
	movl	0x0c(%ebx), %ecx	# ECX
	movl	0x10(%ebx), %edx	# EDX
	movl	0x14(%ebx), %esi	# ESI
	movl	0x18(%ebx), %edi	# EDI
	movl	0x1c(%ebx), %ebp	# EBP
	movl	0x08(%ebx), %ebx	# EBX

	# Finally, pop DS and ES.
	popw	%ds			# DS = 0x20(original %ebx)
	popw	%es			# ES = 0x22(original %ebx)

	# Note: %sp turns back to the same level as the figure above.

	# Call original %cx via a complex method prepared above.
	retw

lmbios1_exec_subr_done:
	# Figure of the stack top at this moment.
	#
	# Offset    Stack contents
	#       +---------------------+
	# 00-01 | Saved DS            |
	#       +---------------------+
	# 02-03 | Saved ES            |
	#       +---------------------+
	# 04-07 | Saved EBX           |
	#       +---------------------+

	# Save some of return values.
	pushl	%eax
	pushl	%ebx
	pushw	%es
	pushw	%ds

	# Figure of the stack top at this moment.
	#
	# Offset    Stack contents
	#       +---------------------+
	# 00-03 | Resulting DS and ES |
	#       +---------------------+
	# 04-07 | Resulting EBX       |
	#       +---------------------+
	# 08-0B | Resulting EAX       |
	#       +---------------------+
	# 0C-0D | Saved DS            |
	#       +---------------------+
	# 0E-0F | Saved ES            |
	#       +---------------------+
	# 10-13 | Saved EBX           |
	#       +---------------------+

	# Restore DS and ES.
	movl	0x0c(%esp), %eax
	pushl	%eax
	popw	%ds
	popw	%es

	# Now, DS and ES can be used!

	# Restore EBX that points to parameter structure.
	movl	0x10(%esp), %ebx

	# Save resulting register values to parameter structure.
	movl	0x08(%esp), %eax	# saved EAX
	movl	%eax, 0x04(%ebx)	# EAX
	movl	0x04(%esp), %eax	# saved EBX
	movl	%eax, 0x08(%ebx)	# EBX
	movl	%ecx, 0x0c(%ebx)	# ECX
	movl	%edx, 0x10(%ebx)	# EDX
	movl	%esi, 0x14(%ebx)	# ESI
	movl	%edi, 0x18(%ebx)	# EDI
	movl	%ebp, 0x1c(%ebx)	# EBP

	# Save resulting DS and ES to parameter structure.
	movl	(%esp), %eax		# saved above.
	movl	%eax, 0x20(%ebx)

	# Save FLAGS to parameter structure.
	# Note: FLAGS are not affected by MOV, PUSH and POP above.
	pushf
	popw	%ax
	mov	%ax, 0x02(%ebp)		# FLAGS

	# Now, every resulting values have been saved to parameter structure.

	# Discard resulting values and saved values on the stack.
	addw	$0x14, %sp

	# Note: %sp turns back to the original level.

	########################################################
	#
	# Return to the caller.
	#
	retw


#########################################################################
#
# Appendix: How to call a subroutine pointed by 16-bit register
#
#   # Because we do not know how to write "call %dx" for 16-bit registers,
#   # we use a complex method described below.
#
# Suppose that DX points to a subroutine entry.  Here is an example code.
#
#	pushw	$lmbios1_dive_subr_done	# Instruction address next to retw
#	pushw	%dx			# Subroutine address
#	retw				# Pop %ip
#   lmbios1_dive_subr_done:
#
# Step 1. Push (1) instruction address next to retw, and
#              (2) subroutine entry address
#         by the following two instructions:
#
#	pushw	$lmbios1_dive_subr_done
#	pushw	%dx
#
#    Here is a figure of the stack top at this moment.
#
#	Offset    Stack contents
#	      +---------------------+
#	00-01 | Subroutine address  | = DX value
#	      +---------------------+
#	02-03 | Instruction address | = lmbios1_dive_subr_done
#	      +---------------------+
#	04-   | Other data ..       |
#	      +---------------------+
#
# Step 2. Pop the subroutine address at the top of the stack into %ip
#         by the following instruction:
#
#	retw
#
#    Here is a figure of the stack top at this moment.
#
#	Offset    Stack contents
#	      +---------------------+
#	00-01 | Instruction address | = lmbios1_dive_subr_done
#	      +---------------------+
#	02-   | Other data ..       |
#	      +---------------------+
#
# Step 3. When the subroutine ends, "retw" would pop the next
#         instruction address at the top of the stack into %ip.
#
#    Here is a figure of the stack top at this moment.
#    Note: Stack Pointer (SP) turns back to the original level.
#
#	Offset    Stack contents
#	      +---------------------+
#	00-   | Other data ..       |
#	      +---------------------+
#


#########################################################################
#
# Supplementary Resources for BIOS interrupt call
#	https://en.wikipedia.org/wiki/BIOS_interrupt_call
#	https://en.wikipedia.org/wiki/INT_(x86_instruction)
#	https://en.wikipedia.org/wiki/INT_10H
#	https://en.wikipedia.org/wiki/INT_13H
#

#
# Supplementary Resources for Interrupt Vector Table (IVT)
#	https://wiki.osdev.org/Interrupt_Vector_Table
#	https://en.wikipedia.org/wiki/Interrupt_vector_table
#

#
# Supplementary Resources for Unreal Mode
#	https://wiki.osdev.org/Unreal_Mode
#	https://en.wikipedia.org/wiki/Unreal_mode
#
