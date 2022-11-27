#![no_std]
#![no_main]

use panic_semihosting as _;

use rtic::app;

#[app(device = stm32f3xx_hal::pac, peripherals = true, dispatchers = [SPI1, SPI2, SPI3])]
mod app {
    use cortex_m_semihosting::hprintln;
    use num_traits::float::Float;
    use systick_monotonic::fugit::ExtU64;
    use systick_monotonic::Systick;

    use orient::stepper::CircularDirection;
    use orient::{display_bearing, ConfiguredDevice};

    #[shared]
    struct Shared {
        board: ConfiguredDevice,
        bearing_north: f32,
        stepper_enabled: bool,
    }

    #[local]
    struct Local {}

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1_000>;

    /// The degrees of orientation we want to allow for drift to prevent
    /// orientating for fuzzy readings or jerky movements.
    const ACCURACY_THRESHOLD: f32 = 30.0;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        hprintln!("Configuring device").ok();

        let mut board = ConfiguredDevice::new(cx.device);
        let tick = Systick::new(cx.core.SYST, board.clocks.sysclk().0);
        let mono = init::Monotonics(tick);

        hprintln!("Device initialized").ok();

        // Start update cycle for bearing
        update_bearing::spawn_after(1u64.secs()).unwrap();

        // Start orienting the device after a bit of an arbitrary delay
        orientate::spawn_after(3u64.secs()).unwrap();

        (
            Shared {
                bearing_north: 0.0,
                board,
                stepper_enabled: false,
            },
            Local {},
            mono,
        )
    }

    #[idle]
    fn idle(cx: idle::Context) -> ! {
        hprintln!("sleeping...").ok();
        loop {
            // Now Wait For Interrupt is used instead of a busy-wait loop
            // to allow MCU to sleep between interrupts
            // https://developer.arm.com/documentation/ddi0406/c/Application-Level-Architecture/Instruction-Details/Alphabetical-list-of-instructions/WFI
            rtic::export::wfi()
        }
    }

    ///
    /// Tasks
    ///

    /// An interupt loop to orient the deivce by rotating the stepper
    #[task(priority = 1, shared = [bearing_north, board])]
    fn orientate(mut cx: orientate::Context) {
        let mut enable = false;
        let mut direction = CircularDirection::CW;

        cx.shared.bearing_north.lock(|bearing| {
            // If we're of more than the threshold
            if bearing.abs() > ACCURACY_THRESHOLD {
                // enable the stepper
                enable = true;

                // If we're on the "right" side of the bearing
                if *bearing > 0.0 {
                    // Move left (?) TODO: update when chassis is built
                    direction = CircularDirection::CCW;
                }
            }
        });

        if enable {
            enable_stepper::spawn(direction).unwrap();
        } else {
            disable_stepper::spawn().unwrap();
        }

        // For responsiveness, keep this somewhat short without being an
        // interrupt hog and blocking other tasks
        orientate::spawn_after(250u64.millis()).unwrap();
    }

    /// Enable the stepper in the given direction
    #[task(priority = 1, shared = [board, stepper_enabled])]
    fn enable_stepper(mut cx: enable_stepper::Context, direction: CircularDirection) {
        let mut board = cx.shared.board;
        let mut enabled = cx.shared.stepper_enabled;

        board.lock(|b| {
            // TODO: what happens if we change direction and stepper already
            // enabled?
            b.stepper_set_direction(direction);
            b.stepper_enable();
            enabled.lock(|e| *e = true);
        });
    }

    /// Disable the stepper
    #[task(priority = 1, shared = [board, stepper_enabled])]
    fn disable_stepper(mut cx: disable_stepper::Context) {
        let mut board = cx.shared.board;
        let mut enabled = cx.shared.stepper_enabled;

        board.lock(|b| {
            b.stepper_disable();
            enabled.lock(|e| *e = false);
        });
    }

    /// Update the bearing toward north from the compass
    #[task(priority = 2, shared = [bearing_north, board])]
    fn update_bearing(mut cx: update_bearing::Context) {
        let mut board = cx.shared.board;
        let mut bearing_north = cx.shared.bearing_north;

        board.lock(|b| {
            let new_bearing = b.bearing_north();
            bearing_north.lock(|bearing| *bearing = new_bearing);

            // Update the LED directionals
            update_display::spawn(new_bearing).unwrap();
        });

        // Generally, we probably want this to update at least as often as the
        // stepper.  Pointless to update the stepper without a new bearing.
        update_bearing::spawn_after(100u64.millis()).unwrap();
    }

    /// Update the LED display to show the given bearing
    #[task(priority = 2, shared = [bearing_north, board])]
    fn update_display(mut cx: update_display::Context, bearing: f32) {
        cx.shared.board.lock(|board| {
            display_bearing(board, bearing).unwrap();
        });
    }
}
