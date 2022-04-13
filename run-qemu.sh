#! /bin/sh

NAME=`grep name Cargo.toml | cut -d= -f2 | sed -e 's/[ "]*//g'`

TARGET="x86_64-unknown-none"
BINARY="target/$TARGET/debug/$NAME.bin"

cargo objcopy -- -O binary $BINARY

qemu-system-x86_64 \
	-drive format=raw,file=$BINARY \
	-m 4G \
	-monitor stdio -d int -no-reboot
