# nostd_env

`nostd_env` is an experimental and simple environment to run a Rust
`no_std` program.  Because this project has just begun, everything is
subject to change.

Current status:

* A Rust `no_std` program runs on QEMU in X86 Long Mode.
* BIOS functions can be called from a Rust `no_std` program.
* Rust alloc library is working with a first-fit memory allocator.
* `nostd_env` runs on QEMU 6.2 macOS 12 Monterey.

# Components

## lmboot0

lmboot0 is a small boot loader to run a Rust `no_std` program in X86
Long Mode.  Because its size <= 0x1BE, it fits in a classical Master
Boot Record (MBR).

## lmbios1

lmbios1 provides a method to call Real Mode functions from Long Mode.
That is, it switches CPU operating mode from Long Mode to Real Mode,
calls a function in Real Mode, then switches back to Long Mode.  As
the name suggests, its main purpose is to call BIOS functions from
Long Mode.

## Micro (mu) Library

A small library containing:

* MuAlloc - An implementation of alloc::GlobalAlloc and alloc::Allocator
* MuHeap - A First-Fit Memory Allocator using Doubly Linked List
* MuMutex - A Mutual Exclusion Primitive using Spin Lock

# Required Software

(1) Nightly Rust (cf. <https://www.rust-lang.org>)

```sh
% rustup toolchain install nightly
```

(2) cargo-binutils (cf. <https://github.com/rust-embedded/cargo-binutils>)

```sh
% cargo install cargo-binutils
% rustup component add llvm-tools-preview
```

(3) qemu-system-x86_64 (cf. <https://www.qemu.org>)

```sh
% brew install qemu
```

# How to build and run

On macOS: At the top directory, enter the following commands.
Then, edit `src/main.rs` and other files as you like.

```sh
% cargo build
% ./run-qemu.sh
```

On other systems: (To be described..)

 * * *

Fore more information, please see the description at the head of each
assembly source file and the technical references at the tail of each
assembly source file.  These assembly source files can be found under
`src/bios`.
