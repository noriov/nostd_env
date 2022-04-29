fn main() {
    println!("cargo:rerun-if-changed=src/bios/asm/debug_helper.s");
    println!("cargo:rerun-if-changed=src/bios/asm/lmboot0.s");
    println!("cargo:rerun-if-changed=src/bios/asm/lmbios1.s");
    println!("cargo:rerun-if-changed=src/bios/asm/wrapper_sysv.s");
    println!("cargo:rerun-if-changed=config/x86_64-unknown-none.json");
    println!("cargo:rerun-if-changed=config/x86_64-unknown-none.ld");
}
