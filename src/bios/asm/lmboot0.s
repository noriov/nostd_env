#
# lmboot0 - Boot a program in Long Mode (for X86_64 / PC Compatible machines)
#
# lmboot0 is a small boot loader to run a Rust no_std program in X86
# Long Mode.  Because its size <= 0x171 bytes (369 bytes), it fits in
# a Master Boot Record (MBR).
#
# lmboot0 assumes that:
#   (1) CPU supports Long Mode,
#   (2) CPU's 4-Level Paging supports 1GB-page,
#   (3) The A20 line can be enabled via I/O Port 92h Bit 1, and
#   (4) BIOS supports INT 13h AH=42h (Extended Read Sectors From Drive).
#
# lmboot0 also assumes that:
#   (5) All configuration will be re-done by loaded program, and
#   (6) All messages should be printed by loaded program.
#
# And loaded program is assumed that:
#   (7) It is contiguously stored from LBA_START in the boot drive,
#   (8) Its memory area is from MAIN1_START to MAIN1_END - 1, and
#   (9) Its entry point is __bare_start.
#
# Hence, lmboot0 simply loads a program using BIOS and executes it in
# Long Mode without printing any messages except fatal error messages.
#
# Configurations made by lmboot0 are:
#   (1) CPU runs in Long Mode,
#   (2) Paging is Identity Mapped Paging,
#       - Virtual address range is 0 to 4GB - 1 regardless of memory size,
#       - One PML4 (4KB) with 1 entry at PML4_START,
#       - One PDPT (4KB) with 4 entries of 1GB-Pages at PDPT_START,
#   (3) Global Descriptor Table (GDT) has three selectors,
#       - Selector 1: Code (64-bit mode) for CS
#       - Selector 2: Code (16-bit mode) for CS (used in lmbios1)
#       - Selector 3: Data for DS, ES, FS, GS and SS
#       (GDT is stored at 0x7D98 - 0x7DB7)
#   (4) Initial RSP = STACK_END, and
#   (5) Interrupts are disabled.
#
# Because the size of lmboot0 <= 0x171, lmboot0 fits in an MBR.
# It works on QEMU Ver 6.2.0 with SeaBIOS Rel 1.15.0 as of Apr 2022.
#
# Note1: The values of symbols above are imported from the linker script.
#        They can be changed by editing the linker script or
#        by editing .set directives below.
#
# Note2: lmboot0 is written in att_sytax to write the following lines:
#	ljmp	$0x0000, $lmboot0_rm16	# CS = 0x0000, IP = $lmboot0_rm16
#	ljmp	$0x10, $lmboot0_lm64	# CS = 0x0010, IP = $lmboot0_lm64
#
#   NASM allows to write them as following, but intel_sytax seems not..
#	jmp	0x0000:lmboot0_rm16	# CS = 0x0000, IP = lmboot0_rm16
#	jmp	0x10:lmboot0_lm64	# CS = 0x0008, IP = lmboot0_lm64
#
#   # We thought "ljmp $0, $addr" saves a couple of bytes compared to
#   # "pushw $0; pushw $addr; lretw".  But generated codes are:
#   #   ea 10 7c 00 00     (5 bytes) for "ljmp $0, $addr"
#   #   6a 00 68 11 7c cb  (6 bytes) for "pushw $0; pushw $addr; lretw"
#
# Note3: 32-bit registers work in Real Mode without any special settings.
#        Therefore, 20-bit address space (1MB) can be accessed using
#        register indirect addressing mode in addition to SEGMENT:OFFSET.
#
# Technical references and summaries are listed at the tail of this file.
#

	.section .lmboot0, "axw"  # axw = allocatable, executable, writable
	.globl __lmboot0_entry
	.globl __lmboot0_boot_drive_id
	.code16

	# The values of these symbols are imported from the linker script.
	.set	PML4_START, __lmb_pml4_start
	.set	PDPT_START, __lmb_pdpt_start
	.set	PGTBL_SIZE, 0x1000 * 2
	.set	STACK_END, __lmb_stack_end
	.set	MAIN1_START, __lmb_main1_start
	.set	MAIN1_END, __lmb_main1_end
	.set	LBA_START, 1


#########################################################################
#
# __lmboot0_entry - Load and execute a program in Long Mode
#
# IN
#	DL	: Boot Drive ID
#

__lmboot0_entry:
	jmp	lmboot0_start
#	nop

	# Room for disk parameters.
	#   e.g. BIOS Parameter Block of FAT12 or FAT16 (0x3E bytes)

.org 0x047

lmboot0_start:

	########################################################
	#
	# Initialize segment registers and FLAGS.
	#
	# States: CPU = Real Mode, Code segment = 16-bit mode.
	#
	xorw	%ax, %ax		# AX = 0x0000
	movw	%ax, %ds		# DS = 0x0000
	movw	%ax, %es		# ES = 0x0000
	movw	%ax, %ss		# SS = 0x0000

	movw	$STACK_END, %sp		# SP = $STACK_END
	ljmp	$0x0000, $lmboot0_rm16	# CS = 0, IP = $lmboot0_rm16
lmboot0_rm16:

	# Initialize the direction flag.
	cld				# DF = 0 (Direction flag)

	########################################################
	#
	# Save boot drive ID.
	#
	movb	%dl, __lmboot0_boot_drive_id

	########################################################
	#
	# Load main program from the boot drive.
	#

	# Memory address: EDI = $MAIN1_START
	movl	$MAIN1_START, %edi	# EDI = Start address of main1

	# Number of bytes: ECX = $MAIN1_END - $MAIN1_START
	movl	$MAIN1_END, %ecx	# ECX = End address of main1 (for now)
	subl	%edi, %ecx		# ECX = Size in bytes of main1

	# Number of blocks: ECX = (ECX + 511) / 512
	addl	$511, %ecx		# block size - 1 is added to round up
	shrl	$9, %ecx		# CX = Size in blocks of main1

	# Segment of memory address (higher 16-bit of 20-bit address)
	shrl	$4, %edi		# DI = Segment of main1

	# Logical Block Address (LBA): EBX = $LBA_START
	movl	$LBA_START, %ebx	# EBX = start LBA of main1

	# Load blocks from the boot drive.
	# Input:
	#   EBX : Logical Block Address (LBA)
	#   CX  : Number of blocks
	#   DL  : Drive ID
	#   DI  : Segment of memory address
	# Output:
	#   CF  : 0 if successful, 1 if failed.
	#
	# Note: %dl already has the boot drive ID.
	#
	call	lmboot0_load_blocks
	jc	lmboot0_io_error

	########################################################
	#
	# Enable the A20 line via I/O Port 92h Bit 1.
	#
	# Note: It is assumed that I/O Port 92h Bit 1 is supported.
	#
	in	$0x92, %al
	or	$0x02, %al	# Bit 1 = ON
	out	%al, $0x92

	########################################################
	#
	# Disable interrupts.
	#
	# Note: All interrupts must be disabled until the Interrupt
	#       Descriptor Table (IDT) for Long Mode is ready.
	#

	# Clear the Interrupt Flag (IF)
	cli

	########################################################
	#
	# Construct page tables for 4-Level Paging of Long Mode.
	#
	# Identity Paging from 0 to 4GB - 1 (32-bit address space)
	# regardless of available memory size.
	#
	# Two page tables are constructed.
	#   (1) One PML4 (Size: 4KB) with 1 entry pointing to PDPT
	#   (2) One PDPT (Size: 4KB) with 4 entries pointing to 1GB-Pages
	#

	# Set PML4 start address (page table start address) to ESI.
	movl	$PML4_START, %esi	# ESI = PML4 start address

	# Clear page table area. # Alreay DF = 0 (Direction flag)
	movw	%si, %di		# DI = start of page tables
	movw	$(PGTBL_SIZE / 2), %cx	# CX = page table size / 2
	xorw	%ax, %ax		# AX = 0
	rep stosl

	# Set PDPT start address to DI.
	movw	$PDPT_START, %di	# DI = PDPT start address

	# Construct PML4 table (with 1 entry) that is the root of page tables.
	movzx	%di, %eax		# EAX = PDPT start address
	orb	$0x03, %al		# Bit 0: Present, Bit 1: R/W
	movl	%eax, 0x00(%si)		# 0th entry of PML4 table

	# Construct the 0th PDPT (with 4 entries for four 1GB-Pages).
	xorl	%eax, %eax		# EAX = 0GB (the 1st 1GB page)
	orb	$0x83, %al # Bit 0: Present, Bit 1: R/W, Bit 7: 1GB-Page
	movl	$(1 << 30), %ecx # ECX = 1GB
	movl	%eax, 0x00(%di)		# 0th entry of PDPT (0GB - 1GB)
	addl	%ecx, %eax # EAX += 1GB
	movl	%eax, 0x08(%di)		# 1st entry of PDPT (1GB - 2GB)
	addl	%ecx, %eax # EAX += 1GB
	movl	%eax, 0x10(%di)		# 2nd entry of PDPT (2GB - 3GB)
	addl	%ecx, %eax # EAX += 1GB
	movl	%eax, 0x18(%di)		# 3rd entry of PDPT (3GB - 4GB)

	# Set the root of the page tables (PML4 table address) to CR3.
	movl	%esi, %cr3		# CR3 = PML4 table address

	########################################################
	#
	# Set Global Descriptor Table (GDT) for Long Mode.
	#
	lgdt	(lmboot0_gdt_location)

	# List of segment selectors.
	.set	SEG_CODE64, (1 << 3)	# Selector 1, GDT, RPL=0
	.set	SEG_CODE16, (2 << 3)	# Selector 2, GDT, RPL=0
	.set	SEG_DATA,   (3 << 3)	# Selector 3, GDT, RPL=0

	########################################################
	#
	# Enter Long Mode.
	#

	# Define control register flags related to Long Mode settings.
	.set	CR0_PE, (1 << 0)	# PE = Protected Mode Enable
	.set	CR0_PG, (1 << 31)	# PG = Paging
	.set	CR4_PAE, (1 << 5)	# PAE = Physical Address Extension
	.set	EFER_LME, (1 << 8)	# LME = Long Mode Enable
	.set	EFER_LMA, (1 << 10)	# LMA = Long Mode Active
	.set	MSR_EFER, 0xc0000080	# EFER:Extended Feature Enable Register

	# Set PAE (Bit 5) in CR4  (PAE = Physical Address Extension).
	movl	%cr4, %eax
	orl	$CR4_PAE, %eax
	movl	%eax, %cr4

	# Set LME (Bit 8) in EFER  (LME = Long Mode Enable).
	movl	$MSR_EFER, %ecx
	rdmsr
	orl	$EFER_LME, %eax
	wrmsr

	# Set PE (Bit 0) and PG (Bit 31) in CR0 (Protected Mode Enable, Paging)
	movl	%cr0, %eax
	orl	$(CR0_PE | CR0_PG), %eax
	movl	%eax, %cr0

	# Check LMA (Bit 10) in EFER  (LMA = Long Mode Active).
	# Note: ECX = $MSR_EFER
	rdmsr
	testl	$EFER_LMA, %eax
	jz	lmboot0_no_long_mode

	########################################################
	#
	# Update internal segment descriptor caches
	# by setting new segment selectors.
	#
	# States: CPU = Long Mode, Code segment = 16-bit mode.
	#
	movw	$SEG_DATA, %ax		# Segment Selector for Data
	movw	%ax, %ds		# DS = SEG_DATA
	movw	%ax, %es		# ES = SEG_DATA
	movw	%ax, %fs		# FS = SEG_DATA
	movw	%ax, %gs		# GS = SEG_DATA
	movw	%ax, %ss		# SS = SEG_DATA

	# Clear the instruction prefetch queue and reset CS.
	ljmp	$SEG_CODE64, $lmboot0_lm64	# CS = SEG_CODE64
lmboot0_lm64:

	########################################################
	#
	# Start loaded program.
	#
	# Note: Because the address of __bare_start is up to 20 bits,
	#       16-bit addressing "jmp" cannot be used.  Instead,
	#       32-bit relative addressing "jmp" must be used.
	#
	# States: CPU = Long Mode, Code segment = 64-bit mode.
	#
	.code64
	jmp	__bare_start
	.code16

	# Now, mission completed!


########################################################################
#
# Print fatal error messages.
#

lmboot0_io_error_msg:
	.asciz	"I/O Error\r\n"
lmboot0_no_long_mode_msg:
	.asciz	"No Long Mode\r\n"

lmboot0_io_error:
	movw	$lmboot0_io_error_msg, %si
	jmp	lmboot0_print_asciz

lmboot0_no_long_mode:
	movw	$lmboot0_no_long_mode_msg, %si
#	goes through into lmboot0_print_asciz


#
# lmboot0_print_asciz - Print Null-terminated string.
#
# IN
#	SI	: Null-terminated string
#
# Scratched: AX, BX
#
lmboot0_print_asciz:
	# BH (Page number) = 0, BL (Color) = 15 (White)
	movw	$0x000f, %bx

lmboot0_print_asciz_loop:
	# Load the next character into AL.
	#   Note: Already DF = 0 (Direction flag)
	lodsb		# AL = DS:[SI++]
	testb	%al, %al
	jz	lmboot0_halt_repeatedly

	# INT 10h AH=0Eh (Teletype Output)
	# AL = Character, BH = Page number, BL = Foreground color
	movb	$0x0e, %ah
	int	$0x10

	jmp	lmboot0_print_asciz_loop

#
# lmboot0_halt_repeatedly - Halt forever
#
lmboot0_halt_repeatedly:
	hlt
	jmp	lmboot0_halt_repeatedly


########################################################################
#
# lmboot0_load_blocks - Load contiguous logical blocks from drive.
#
# Note1: It is assumed that BIOS supports INT 13h AH=42h.
#
# Note2: Because 20-bit address space (1MiB) = 2048 * 512 bytes,
#        the theoretically maximum number of blocks is 2048.
#
# Note3: The maximum number of blocks that can be loaded by one
#        "INT 13h AH=42h" call seems to vary depending on BIOSes.
#        The possibly lowest number of the maximum number of blocks
#        is said to be 127.  (Note: 127 * 512 = 65024 < 65536)
#
# IN
#	EBX	: Logical Block Address (LBA)
#	CX	: Number of blocks
#	DL	: Drive ID
#	DI	: Segment of memory address
#
# OUT
#	FLAGS	: CF = 0 if successful, CF = 1 if failed.
#
# Scratched: EAX, EBX, CX, SI, DI, BP
#

	.set	BLK_SIZE, 512	# Logical Block Size
	.set	MAX_NBLK, 127	# Maximum Number of Blocks (see Note3 above)

lmboot0_load_blocks:
	# If the number of blocks to be loaded is below or equal to
	# the maximum number (127), quit this loop.
	movw	$MAX_NBLK, %ax			# AX = Maximum number of blocks
	cmpw	%ax, %cx
	jbe	lmboot0_load_blocks_final	# not call but jmp.

	call	lmboot0_load_blocks_amap
	jc	lmboot0_load_blocks_done	# I/O error is detected.

	# Update parameters for next loading.
	# Note: Now, EAX holds the maximum nuber of blocks because
	#       higher 16-bit of EAX is zeroed in lmboot0_load_blocks_amap.
	addl	%eax, %ebx			# LBA += MAX_NBLK
	subw	%ax, %cx			# NB  -= MAX_NBLK
	addw	$((BLK_SIZE * MAX_NBLK) >> 4), %di # SEG += NSEG
	jmp	lmboot0_load_blocks

lmboot0_load_blocks_final:
	movw	%cx, %ax			# AX = Final number of blocks

#
# Load contiguous logical blocks as many as possible using INT 13h AH=42h.
#
# IN
#	AX	: Number of blocks (Maximum number = MAX_NBLK)
#	EBX	: Logical Block Address (LBA)
#	DL	: Drive ID
#	DI	: Segment of memory address
#
# OUT
#	FLAGS	: CF = 0 if successful, CF = 1 if failed.
#
# Scratched: SI, BP, EAX (higher 16-bits are zeroed)
#

lmboot0_load_blocks_amap:
	# Save working register values.
	pushw	%ax

	# Allocate memory for the Disk Address Packet (DAP) on the stack.
	movw	%sp, %bp		# Save %sp to %bp
	subw	$0x10, %sp		# The size of DAP = 0x10

	# Construct the Disk Address Packet (DAP).
	movw	%sp, %si		#offset:Disk Address Packet description
	movw	$0x0010, (%si)		# 00   : Size of DAP = 0x10
					# 01   : (reserved)  = 0x00
	movw	%ax, 0x02(%si)		# 02-03: Number of blocks to be loaded
	xorl	%eax, %eax  # EAX = 0
	movw	%ax, 0x04(%si)		# 04-05: Offset to memory buffer = 0
	movw	%di, 0x06(%si)		# 06-07: Segment of memory buffer
	movl	%ebx, 0x08(%si)		# 08-0B: Start block (lower 32 bits)
	movl	%eax, 0x0c(%si)		# 0C-0F: Start block (higher 32 bits)=0

	# INT 13h AH=42h (Extended Read Sectors From Drive)
	# DL = Drive ID, DS:SI = Address of Disk Address Packet (DAP)
	movb	$0x42, %ah
	int	$0x13

	# Deallocate memory for the Disk Address Packet (without changing CF).
	movw	%bp, %sp		# Restore %sp from %bp

	# Restore saved register values.
	popw	%ax

lmboot0_load_blocks_done:
	retw


########################################################################
#
# Storage area for future use
#

# Boot Drive ID
__lmboot0_boot_drive_id:
	.byte	0


########################################################################
#
# Global Descriptor Table (GDT) for Long Mode
#

.org 0x192

lmboot0_gdt_location:
	.word	lmboot0_gdt_end - lmboot0_gdt_start - 1	# Limit
	.long	lmboot0_gdt_start			# Base

.org 0x198

lmboot0_gdt_start:
	.quad	0			# 0: NULL
	.quad	0x00209a0000000000	# 1: Code 64-bit mode
	.quad	0x00009a0000000000	# 2: Code 16-bit mode
	.quad	0x0000920000000000	# 3: Data
lmboot0_gdt_end:


########################################################################
#
# Trailer of Master Boot Record (MBR)
#

# Disk signature (4 bytes + 2 bytes)
.org 0x1b8
	.long	0	# 32-bit disk signature
	.word	0	# 0x0000 (0x5A5A if copy-protected)

# Partition table (16 bytes * 4 entries)
.org 0x1be
	.quad	0, 0	# Partition entry 1
	.quad	0, 0	# Partition entry 2
	.quad	0, 0	# Partition entry 3
	.quad	0, 0	# Partition entry 4

# Boot signature (2 bytes)
.org 0x1fe
	.byte 0x55, 0xaa


#########################################################################
#
# Reference:
#	[iSDM2018]
#	Intel(R) 64 and IA-32 Architectures
#	Software Developer's Manual (Nov, 2018)
#	https://software.intel.com/en-us/articles/intel-sdm
#

#
# The location information of GDT and IDT
#   in LSB first (i.e., lower address first / lower bit first)
#
#	 0 1 2 3 4 5 6 7 8 9 A B C D E F 0 1 2 3 4 5 6 7 8 9 A B C D E F
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	|        Limit (Size - 1)       |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	|                    Base (Linear address)                      |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#
# Reference:
#	[iSDM2018] Vol.2A: Section 3.2 LGDT / LIDT
#

#
# GDT is an array of segment descriptors.
#
# Segment descriptor format for code segment
#   in LSB first (i.e., lower address first / lower bit first)
#
#	 0 1 2 3 4 5 6 7 8 9 A B C D E F 0 1 2 3 4 5 6 7 8 9 A B C D E F
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	|  Segment Limit (Bit 0 - 15)   |   Base Address (Bit 0 - 15)   |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	| Base(16 - 23) |A|R|C|1|S|DPL|P|L 16-19|0|L|D|G| Base(24 - 31) |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#
# Segment descriptor format for data segment
#   in LSB first (i.e., lower address first / lower bit first)
#
#	 0 1 2 3 4 5 6 7 8 9 A B C D E F 0 1 2 3 4 5 6 7 8 9 A B C D E F
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	|  Segment Limit (Bit 0 - 15)   |   Base Address (Bit 0 - 15)   |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	| Base(16 - 23) |A|W|E|0|S|DPL|P|L 16-19|0|L|B|G| Base(24 - 31) |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#
# Note: Below are the current configuration made by lmboot0.
#
# Selector 0 (NULL)
#
# Selector 1 (Code segment of 64-bit instruction mode)
#	Base = 0x0000_0000, Limit = 0x0_0000, AR=0x209A
#	G=0 (Granularity=1B), D=0 (must be zero), L=1 (x86-64)
#	P=1 (Present), DPL=0 (Descriptor Privilege Level=0), S=1 (Code/Data)
#	C=0 (Not conforming), R=1 (Readable), A=0 (Accessed)
#
# Selector 2 (Code segment of 16-bit instruction mode)
#	Base = 0x0000_0000, Limit = 0x0_0000, AR=0x009A
#	G=0 (Granularity=1B), D=0 (16-bit), L=0 (i386)
#	P=1 (Present), DPL=0 (Descriptor Privilege Level=0), S=1 (Code/Data)
#	C=0 (Not conforming), R=1 (Readable), A=0 (Accessed)
#
# Selector 3 (Data segment)
#	Base = 0x0000_0000h, Limit = 0x0_0000, AR=0x0092
#	G=0 (Granularity=1B), B=0 (Segment upper bound=64KB), L=0 (should be 0)
#	P=1 (Present), DPL=0 (Descriptor Privilege Level=0), S=1 (Code/Data)
#	E=0 (Segment expands up), W=1 (Writable), A=0 (Accessed)
#
# Reference:
#	[iSDM2018] Vol.3A: Section 3.4.5
#
# Supplementary Resources:
#	https://en.wikipedia.org/wiki/Segment_descriptor
#	https://en.wikipedia.org/wiki/Global_Descriptor_Table
#	https://wiki.osdev.org/Global_Descriptor_Table
#

#
# Segment Selector (16bit)
#   in MSB first (i.e., higher address first / higher bit first)
#
#	 F E D C B A 9 8 7 6 5 4 3 2 1 0
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	|           Index         |T|RPL|
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#
#	Index (13bit) ... Index to Global/Local Descriptor Table.
#	Table Indicator (1bit) ... 0 = GDT, 1 = LDT.
#	Requested Privilege Level (2bit) ... Privilege Level (0 - 3)
#
# Reference:
#	[iSDM2018] Vol.3A: Section 3.4.2
#

#
# 4-Level Paging
#
# Note: For simplicity, lmboot0 uses 1GB-Pages only.
#
# Linear Address (1GB Page):
#   in MSB first (i.e., higher address first / higher bit first)
#
#	 F E D C B A 9 8 7 6 5 4 3 2 1 0 F E D C B A 9 8 7 6 5 4 3 2 1 0
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	|           (Ignored)           |      PML4       |     PDPT    /
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#	/   |                          Offset                           |
#	+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#
# CR3:
#	Bit  3: PWT (Page-Level Write-Through)
#	Bit  4: PCD (Page-Level Cache Disable)
#	Bit 12-51: Physical Address of PML4 Table.
#
# Page Map Level 4 (PML4) Entry
#	Bit  0: P (Present)
#	Bit  1: R/W (Read/write); 0 = Read, 1 = Read/Write.
#	Bit  2: U/S (User/Supervisor); 0 = Supervisor, 1 = User/Supervisor.
#	Bit  3: PWT (Page-Level Write-Through)
#	Bit  4: PCD (Page-Level Cache Disable)
#	Bit  5: A (Accessed)
#	Bit 12-51: Physical Address of PDPT.
#	Bit 63: XD (eXecute-Disable)
#
# Page Directory Pointer Table (PDPT) Entry for 1GB Page (PS=1)
#	Bit  0: P (Present)
#	Bit  1: R/W (Read/write); 0 = Read, 1 = Read/Write.
#	Bit  2: U/S (User/Supervisor); 0 = Supervisor, 1 = User/Supervisor.
#	Bit  3: PWT (Page-Level Write-Through)
#	Bit  4: PCD (Page-Level Cache Disable)
#	Bit  5: A (Accessed)
#	Bit  6: D (Dirty)
#	Bit  7: PS (Page Size); 0 = Page Directory, 1 = 1GB Page.
#	Bit  8: G (Global)
#	Bit 12: PAT (Page Attribute Table)
#	Bit 30-51: Physical Address of 1GB Page.
#	Bit 59-62: Protection Key
#	Bit 63: XD (eXecute-Disable)
#
# Reference:
#	[iSDM2018] Vol.3A: Section 4.5
#
# Supplementary Resource:
#	https://wiki.osdev.org/Paging
#

#
# Reference for Control Regisetrs:
#	[iSDM2018] Vol.3A: Section 2.5
#
# Supplementary Resource:
#	https://en.wikipedia.org/wiki/Control_register
#

#
# Supplementary Resources for the A20 line:
#	https://wiki.osdev.org/A20_Line
#	https://en.wikipedia.org/wiki/A20_line
#	https://www.win.tue.nl/~aeb/linux/kbd/A20.html
#

#
# Supplementary Resource for INT 13h AH=42h (Extended Read Sectors From Drive)
#	https://en.wikipedia.org/wiki/INT_13H#INT_13h_AH=42h:_Extended_Read_Sectors_From_Drive
#

#
# Supplementary Resources for Master Boot Record (MBR)
#	https://en.wikipedia.org/wiki/Master_boot_record
#	https://wiki.osdev.org/MBR_(x86)
#
