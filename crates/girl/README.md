# `girl`

> It's a **G**amepad **I**nput **R**ust **L**ibrary!

[![Crates.io](https://img.shields.io/crates/v/girl?style=for-the-badge&logo=rust)](https://crates.io/crates/girl)
[![docs.rs](https://img.shields.io/docsrs/girl/latest?style=for-the-badge&logo=docsdotrs)](https://docs.rs/girl)

### Example

```rust
use std::{thread, time::Duration};

use girl::{Button, Girl, Sensor, Stick, Trigger};

fn main() -> Result<(), girl::Error> {
    tracing_subscriber::fmt::init();

    let mut girl = Girl::new()?;

    let gamepads = girl.gamepads_connected().len();
    dbg!(gamepads);

    let Some(mut gamepad) = girl.gamepad(0) else {
        println!("No gamepad connected!");
        return Ok(());
    };
    println!("{} connected", gamepad.name());

    if gamepad.has_sensor(Sensor::Gyroscope) {
        gamepad.enable_sensor(Sensor::Gyroscope)?;
    }
    if gamepad.has_sensor(Sensor::Accelerometer) {
        gamepad.enable_sensor(Sensor::Accelerometer)?;
    }

    loop {
        girl.update();

        if !gamepad.connected()
            && let Some(gp) = girl.gamepad(0)
        {
            gamepad = gp;
        }

        if gamepad.connected() && gamepad.has_led() {
            let left = gamepad.trigger(Trigger::Left);
            let right = gamepad.trigger(Trigger::Right);

            let red = (left * 255.0) as u8;
            let green = (right * 255.0) as u8;

            gamepad.set_led(red, green, 0)?;
        }

        if gamepad.has_rumble() {
            if gamepad.buttons_pressed(Button::A | Button::B) {
                gamepad
                    .set_rumble(u16::MAX, u16::MAX, Duration::from_millis(100))
                    ?;
            } else if gamepad.buttons_pressed(Button::A) {
                gamepad
                    .set_rumble(u16::MAX, 0, Duration::from_millis(100))
                    ?;
            } else if gamepad.buttons_pressed(Button::B) {
                gamepad
                    .set_rumble(0, u16::MAX, Duration::from_millis(100))
                    ?;
            } else {
                gamepad.end_rumble()?;
            }
        }

        println!(
            "{gamepad:10}, {:6.3?} {:6.3?} {:6.3?} {:6.3?} {:6.3?} {:6.3?}",
            gamepad.buttons(Button::all()),
            gamepad.stick(Stick::Right),
            gamepad.trigger(Trigger::Right),
            gamepad.sensor(Sensor::Gyroscope),
            gamepad.sensor(Sensor::Accelerometer),
            gamepad.touchpad(),
            gamepad = gamepad,
        );

        thread::sleep(Duration::from_millis(10));
        # break; // for doctests
    }
    # Ok(()) // for doctests
}
```

## License

Licensed under either of

-   Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
-   MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
