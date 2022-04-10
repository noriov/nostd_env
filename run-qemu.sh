#! /bin/sh

TARGET="x86_64-unknown-none"

DEVSYS=`rustup toolchain list | grep nightly | sed -e 's/^nightly-//' | awk ' { print $1 } '`
NAME=`grep name Cargo.toml | cut -d= -f2 | sed -e 's/[ "]*//g'`

TOOLCHAIN_DIR="$HOME/.rustup/toolchains/nightly-$DEVSYS"
OBJCOPY="$TOOLCHAIN_DIR/lib/rustlib/$DEVSYS/bin/llvm-objcopy"

INPUT="target/$TARGET/debug/$NAME"
OUTPUT="target/$TARGET/debug/$NAME.bin"

$OBJCOPY \
	-I elf64-x86-64 \
	-O binary \
	--binary-architecture=i386:x86-64 \
	$INPUT \
	$OUTPUT

qemu-system-x86_64 \
	-drive format=raw,file=$OUTPUT \
	-m 4G \
	-monitor stdio -d int -no-reboot
