fn main() {
    println!("cargo:rerun-if-changed=src/bios/debug_helper.s");
    println!("cargo:rerun-if-changed=src/bios/lmboot0.s");
    println!("cargo:rerun-if-changed=src/bios/lmbios1.s");
    println!("cargo:rerun-if-changed=src/bios/wrapper_sysv.s");
    println!("cargo:rerun-if-changed=config/x86_64-unknown-none.json");
    println!("cargo:rerun-if-changed=config/x86_64-unknown-none.ld");
}
