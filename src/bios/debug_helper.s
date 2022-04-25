#
# Debug helper
#

	.section .text.debug_helper, "xa" # xa = executable, allocatable
	.globl debug_clear_stack_area

	.set	STACK_START, __lmb_stack_start


#########################################################################
#
# debug_clear_stack_area:
#
#   (1) It searches the stack area for the lowest address (8-byte aligned)
#       whose value is not zero and returns the address in RAX.
#
#   (2) It clears the stack area to make it possible to find the possibly
#       lowest address where the stack has grown for the next time.
#
# OUT
#	RAX	: The lowest address (8-byte aligned) whose value is not zero
#

debug_clear_stack_area:
	# Save working register values.
	push	%rcx
	push	%rdi
	push	%rsi
	pushf

	# Initialize the direction flag.
	cld				# DF = 0 (Direction flag)

	# Step 1: Search the stack area for a non-zero value.
	#   RSI = $STACK_START
	#   RCX = (RSP - $STACK_START) / 8
	movq	$STACK_START, %rsi	# RSI = $STACK_START
	movq	%rsp, %rcx		# RCX = RSP
	subq	%rsi, %rcx		# RCX = (RSP - $STACK_START)
	shrq	$3, %rcx		# RCX = (RSP - $STACK_START) / 8
debug_clear_stack_area_search_loop:
	lodsq				# RAX = [RSI++]
	test	%rax, %rax
	jne	debug_clear_stack_area_search_done
	subq	$1, %rcx
	ja	debug_clear_stack_area_search_loop

debug_clear_stack_area_search_done:
	# RSI = The lowest address whose value is not zero
	subq	$8, %rsi		# RSI = RSI - 8

	# Step 2: Clear the stack area.
	#   RDI = RSI
	#   RCX = (RSP - RDI) / 8
	#   RAX = 0
	movq	%rsi, %rdi		# RDI = RSI
	movq	%rsp, %rcx		# RCX = RSP
	subq	%rdi, %rcx		# RCX = (RSP - RDI)
	shrq	$3, %rcx		# RCX = (RSP - RDI) / 8
	xorq	%rax, %rax		# RAX = 0
	rep stosq			# [RDI++] = RAX for RCX times

	# RAX = The lowest address whose value is not zero.
	movq	%rsi, %rax		# RAX = RSI (computed in step 1)

	# Restore working register values.
	popf
	pop	%rsi
	pop	%rdi
	pop	%rcx

	retq
