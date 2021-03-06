# Home Phone

Firmware for my VoIP home phone.

![cherry-enclosure](https://jonlamb-gh.github.io/home-phone/home_phone.jpg)

## Building

```rust
cargo xbuild
```

Copy elf to binary if needed (no bootelf):

```bash
cargo objcopy -- -O binary target/$(TARGET)/release/home-path /tmp/home-phone.bin
```

## Simulating

TODO

https://github.com/xpack-dev-tools/qemu-arm-xpack/releases

```bash
# For output on UART1
qemu-system-aarch64 -M raspi4 -nographic -serial null -serial mon:stdio -kernel /path/to/binORelf
```

Or using the custom runner:

```bash
cargo xrun
```

## Testing

### QEMU, no hardware

Using the custom runner with QEMU:

```bash
# Binary unit tests and integration tests
cargo xtest

# Libary unit tests
cargo xtext -p lib
```

### Hardware tests

TODO

## U-boot

Using 64 bit U-boot:

TODO

Environment:
TODO

## SD card

TODO

## HAL and device crate

- [bcm2711-hal](https://github.com/jonlamb-gh/rpi4-rust-workspace/tree/master/bcm2711-hal)
- [bcm2711](https://github.com/jonlamb-gh/rpi4-rust-workspace/tree/master/bcm2711)
