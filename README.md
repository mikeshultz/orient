# Orient

Firmware for a device that orients itself north using a stepper motor and compass readings on the STM32F3DISCOVERY board.  It reads from the board's LSM303AGR compass, then a stepper controller to adjust the stepper motor's position to point the entire device north.

## Development

### Buil

TODO

### Debug Build and Run

To run the code on the device, first start the openocd remote debugger connection:

```bash
openocd
```

Then, using `cargo` build and flash the firmware to the device:

```bash
cargo run
```

This will drop you into a debugger that breaks at the rtic init.

## TODO

- Add a circuit diagram to the README
- Add a video of the device operating
- Disable the stepper motor when not adjusting to allow for the free motion of he device.  This will require a buk circuit since we only have 2 5v capable pins on GPIO.  Should also try and determine if a person is moving it before enabling as well.
- Use the available button to enable/disable functionality
- Implement necessary correction for 3d compas z-axis movement

## Attribution

The scaffolding of this project is based on [`cortex-m-quickstart`](https://github.com/rust-embedded/cortex-m-quickstart) and used under the MIT license.
