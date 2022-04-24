#
# Debug helper
#
# Subroutine debug_clear_stack_area searches the stack area and returns
# the lowest address (8-byte aligned) whose value is not zero in RAX.
# Additionally, it clears the stack area to make it possible to find
# the possibly lowest address where the stack has grown to next time.
#

	.section .text.debug_helper, "xa" # xa = executable, allocatable
	.globl debug_clear_stack_area

	.set	STACK_START, __lmb_stack_start

debug_clear_stack_area:
	# Save working register values.
	push	%rcx
	push	%rdi
	push	%rsi
	pushf

	# Initialize the direction flag.
	cld				# DF = 0 (Direction flag)

	# Find non-zero value.
	#   RSI = $STACK_START
	#   RCX = (RSP - $STACK_START) / 8
	movq	$STACK_START, %rsi	# RSI = $STACK_START
	movq	%rsp, %rcx		# RCX = RSP
	subq	%rsi, %rcx		# RCX = (RSP - $STACK_START)
	shrq	$3, %rcx		# RCX = (RSP - $STACK_START) / 8
debug_clear_stack_area_loop:
	lodsq				# RAX = [RSI++]
	test	%rax, %rax
	je	debug_clear_stack_area_loop

	# RSI = The lowest address whose value is not zero (= RSI - 8).
	subq	$8, %rsi

	# Clear stack area.
	#   RDI = RSI
	#   RCX = (RSP - RSI) / 8
	#   RAX = 0
	movq	%rsi, %rdi		# RDI = RSI
	movq	%rsp, %rcx		# RCX = RSP
	subq	%rdi, %rcx		# RCX = (RSP - RSI)
	shrq	$3, %rcx		# RCX = (RSP - RSI) / 8
	xorq	%rax, %rax		# RAX = 0
	rep stosq

	# RAX = The lowest address whose value is not zero (computed above).
	movq	%rsi, %rax

	# Restore working register values.
	popf
	pop	%rsi
	pop	%rdi
	pop	%rcx

	retq
