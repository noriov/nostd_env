/*!

`nostd_env` is an experimental environment to run a Rust `no_std`
program.

Current status:

* A Rust `no_std` program runs on QEMU in X86 Long Mode.
* BIOS functions can be called from a Rust `no_std` program.
* Rust `alloc` library (`Vec`, `Box`, etc.) is working
  on a first-fit memory allocator.

Everything is a work in progress, everything is subject to change.

# Components

* `lmboot0` - is a small boot loader to run a Rust `no_std` program in
  X86 Long Mode.  Because its size <= 0x171 bytes (369 bytes), it fits
  in a Master Boot Record (MBR).

* `lmbios1` - provides a method to call a Real Mode function from Long
  Mode.  That is, it switches CPU operating mode from Long Mode to
  Real Mode, calls a function in Real Mode, then switches back to Long
  Mode.  As the name suggests, its main purpose is to call BIOS
  functions from Long Mode.

* Micro (mu) Library
  - MuAlloc - An implementation of alloc::GlobalAlloc and alloc::Allocator
  - MuHeap - A First-Fit Memory Allocator using Doubly Linked List
  - MuMutex - A Mutual Exclusion Primitive using Spin Lock

# Documents

To see the documents, run the following command.

```sh
% cargo doc --open
```

# Required Software

1. Nightly Rust (cf. <https://www.rust-lang.org>)

```sh
% rustup toolchain install nightly
```

2. cargo-binutils (cf. <https://github.com/rust-embedded/cargo-binutils>)

```sh
% cargo install cargo-binutils
% rustup component add llvm-tools-preview
```

One of the following commands may be required when building `nostd_env`.

```sh
macos% rustup component add rust-src --toolchain nightly-x86_64-apple-darwin
win11> rustup component add rust-src --toolchain nightly-x86_64-pc-windows-msvc
linux$ rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
```

3. qemu-system-x86_64 (cf. <https://www.qemu.org>)

# How to Build and Run

On Linux and macOS: At the top directory, enter the following commands.

```sh
% cargo build
% ./run-qemu.sh
```

On Windows 11: At the top directory, enter the following commands.

```sh
> cargo build
> .\run-qemu.ps1
```

Then, make a branch and edit files as you like.

 */

#![no_std]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

extern crate alloc;

pub mod bios;
pub mod man_heap;
pub mod man_video;
pub mod mu;
pub mod test_alloc;
pub mod test_diskio;
pub mod text_writer;
pub mod x86;
