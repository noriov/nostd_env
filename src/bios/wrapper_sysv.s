#
# SysV AMD64 ABI wrapper for lmbios1
#
# Summary of the function calling sequence of SysV AMD64 ABI:
#
# (1) Return registers are RAX and RDX.
#
#   * Return value up to 64 bits is stored in RAX.
#   * Return value up to 128 bits is stored in RAX and RDX.
#
# (2) Arguments are passed in RDI, RSI, RDX, RCX, R8, R9, then stack.
#
#   * 1st argument: RDI
#   * 2nd argument: RSI
#   * 3rd argument: RDX
#   * 4th argument: RCX
#   * 5th argument: R8
#   * 6th argument: R9
#   * 7th argument and more: on the stack
#
#   Note: These registers are caller-saved.
#
# (3) Callee-saved registers are RBX, RSP, RBP, R12 - 15, and X87 control word.
#
#   Note: If the lower bits of 64-bit registers are changed in Real Mode,
#   their higher bits seem to be cleared, while there is no way to save
#   and restore their higher bits in Real Mode.  Hence, if there is
#   a possibility that the lower bits of a 64-bit register is changed,
#   and if it is a callee-saved register, its whole value must be saved
#   and restored in Long Mode.  Therefore,
#
#   * RBX and RBP must be saved and restored in this wrapper.
#
#   * RSP need not be explicitly saved and restored
#     because its value at the end of a subroutine must be the
#     same as its value at the beginning of the subroutine.
#
#   * R12 - R15 need not be saved
#     because their values cannot be changed in Real Mode.
#
#   * X87 control word is intentionally not saved in this wrapper.
#     Hence, the internal subroutine must take care of it.
#
# (4) RSP + 8 or 8(%rsp) must be aligned on a 16-byte boundary.
#
# For more information, see the references listed at the tail of this file.
#

	.section .text.lmbios, "xa" # xa = executable, allocatable
	.globl lmbios_call
	.globl lmbios_get_boot_drive_id
	.code64


#########################################################################
#
# lmbios_call - Call BIOS function from Long Mode (SysV AMD64 ABI wrapper)
#
# IN
#	RDI	: Address of struct LmbiosRegs (1st argument)
#
# OUT
#	RAX	: Executed function number (0xFFFF if unsupported)
#

	.p2align 4, 0x90  # 0x90 = NOP (= xchgl %eax, %eax)

lmbios_call:				# Wrapper subroutine
	# Save RBX and RBP values.
	pushq	%rbx
	pushq	%rbp

	# Call BIOS function.
	movq	%rdi, %rbx		# Address of struct LmbiosRegs
	call	lmbios1_call		# Main subroutine

	# Restore RBX and RBP values.
	popq	%rbp
	popq	%rbx

	retq


########################################################################
#
# lmbios_get_boot_drive_id - Return boot drive ID save by lmboot0.
#
# OUT
#	RAX	: Boot Drive ID
#

	.p2align 4, 0x90  # 0x90 = NOP (= xchgl %eax, %eax)

lmbios_get_boot_drive_id:
	xorq	%rax, %rax
	movb	(__lmboot0_boot_drive_id), %al
	retq


#########################################################################
#
# Reference:
#	Section 3.2 "Function Calling Sequence" in
#	System V Application Binary Interface
#	AMD64 Architecture Processor Supplement
#	(With LP64 and ILP32 Programming Models)
#	Version 1.0, May 8, 2020
#	https://gitlab.com/x86-psABIs/x86-64-ABI
#
# Supplementary Resources:
#	https://en.wikipedia.org/wiki/X86_calling_conventions
#

#
# Related resource on generic ABI:
#	System V Application Binary Interface Edition 4.1
#	http://www.sco.com/developers/devspecs/
#
