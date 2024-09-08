#!/bin/bash

# Get the first argument and if it does not exist set it to the default.
# This is the name of the binary to compile.
BINARY_NAME=${1:-blinky}

# Echo everything
set -x

# First, compile
cargo build --release

# Then, create the assembly file.
cargo objdump --bin ${BINARY_NAME} --release -- -d --no-show-raw-insn --print-imm-hex > firmware/${BINARY_NAME}.asm

# Then dump the .vector_table section to a file.
cargo objdump --bin ${BINARY_NAME} --release -- -s -j .vector_table > firmware/${BINARY_NAME}.vector_table

# Create the firmware directory if it does not exist.
mkdir -p firmware

# Then create the binary file.
# arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/release/blinky firmware.bin
cargo objcopy --release -- -O binary firmware/${BINARY_NAME}.bin

# Run cargo size to get the size of the binary.
cargo size --bin ${BINARY_NAME} --release
