ENTRY(__lmboot0_entry)

                      /* Comments below briefly shows memory map. */
SECTIONS {
                      /* 0x0000-0x03ff: Interrupt Vector Table (IVT) */
                      /* 0x0400-0x04ff: BIOS Data Area (BDA) */
                      /* 0x0500-0x0fff: free */
    . = 0x1000;
    __lmb_page_tables_start = .;
    __lmb_pml4_start = .;
    . += 0x1000;      /* 0x1000-0x1fff: Page Map Level 4 (PML4) Table */
    __lmb_pdpt_start = .;
    . += 0x1000;      /* 0x2000-0x2fff: Page Directory Pointer Table (PDPT) */
    __lmb_page_tables_end = .;
                      /* 0x3000-0x????: free */
                      /* 0x????-0x7c00: stack */
    . = 0x7c00;
    __lmb_stack_end = .;

    .boot0 : {
        *(.lmboot0)   /* 0x7c00-0x7dff: Copy of Master Boot Record (MBR) */
    }

    .main1 : {
        __lmb_main1_start = .;
        *(.lmbios1)   /* 0x7e00-: Its highest addr must be less than 64KB. */

        /* A Rust program follows lmbios1. */
        *(.text*)
        *(.rodata*)
        *(.data*)
        *(.bss*)
        *(.tdata*)
        *(.tbss*)

        /* Not sure .eh_frame* and .rel.eh_frame* should be here
         * or can be discarded. */
        *(.eh_frame*)
        *(.rel.eh_frame*)

        . = ALIGN(512);
        __lmb_main1_end = .;
    }

    .data2 : {
        /* Additional data (e.g. file system, program) can be appended here.
         * Because data in .data2 will not be loaded by lmboot0,
         * loaded program in .main1 should read / write them via BIOS, etc. */
    }

    /DISCARD/ : {
        *(.debug*)   /* Discard to simplify linker map */
    }
}


/*
 * Reference:
 *	Linker Scripts (GNU binutils ld)
 *	https://sourceware.org/binutils/docs/ld/Scripts.html
 *
 * Supplementary Resource:
 *	https://wiki.osdev.org/Linker_Scripts
 */

/*
 * Supplementary Resource for PC memory map:
 *	https://wiki.osdev.org/Memory_Map_(x86)
 */
