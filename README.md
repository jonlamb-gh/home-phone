# Home Phone

## Building

```rust
cargo xbuild
```

Copy elf to binary if needed (no bootelf):

```bash
cargo objcopy -- -O binary target/$(TARGET)/release/home-path /tmp/home-phone.bin
```

## Simulating

```bash
# For output on UART1
qemu-system-aarch64 -M raspi4 -nographic -serial null -serial mon:stdio -kernel /path/to/binORelf
```

Or using the custom runner:

```bash
cargo xrun
```

## Testing

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
