# nostd_env

`nostd_env` is an experimental and simple environment to run a Rust
`no_std` program.  Because this project has just begun, everything is
subject to change.

Current status:

* A Rust `no_std` program runs on QEMU in X86 Long Mode.
* BIOS functions can be called from a Rust `no_std` program.

# lmboot0

lmboot0 is a small boot loader to run a Rust `no_std` program in X86
Long Mode.  Because its size <= 0x1BE, it fits in a classical Master
Boot Record (MBR).

# lmbios1

lmbios1 provides a method to call a Real Mode function from Long Mode.
That is, it switches CPU operating mode from Long Mode to Real Mode,
calls a function in Real Mode, then switches back to Long Mode.  As
the name suggests, its main purpose is to call BIOS functions from
Long Mode.

It is written in assembly language.  Currently, only SysV AMD64 ABI is
supported.

# Required Hardware and Software

(1) x86-64 CPU is required.

(2) qemu-system-x86_64 is required. (cf. <https://www.qemu.org>)

(3) The following tools seem to be required.

```sh
% rustup toolchain install nightly
% cargo install cargo-binutils
% rustup component add llvm-tools-preview
```

# How to build and run

On macOS: At the top directory, enter the following commands.
Then, edit `src/main.rs` and other files as you like.

```sh
% cargo build
% ./run-qemu.sh
```

On SysV AMD64 ABI OSes, the same commands above may work, or may not work..

On other ABI OSes, at least a wrapper for lmbios1 needs to be written..

 * * *

Fore more information, please see the description at the head of each
assembly source file and the technical references at the tail of each
assembly source file.  These assembly source files can be found under
`src/bios`.
