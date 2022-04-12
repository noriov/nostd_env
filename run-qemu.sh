#! /bin/sh

TARGET="x86_64-unknown-none"
NAME=`grep name Cargo.toml | cut -d= -f2 | sed -e 's/[ "]*//g'`
OUTPUT="target/$TARGET/debug/$NAME.bin"

cargo objcopy --bin nostd_env -- -O binary $OUTPUT

qemu-system-x86_64 \
	-drive format=raw,file=$OUTPUT \
	-m 4G \
	-monitor stdio -d int -no-reboot
