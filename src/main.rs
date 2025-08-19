use std::{thread, time::Duration};

#[cfg(feature = "sensors")]
use girl::Sensor;
use girl::{Button, Girl, Stick, Trigger};

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

    #[cfg(feature = "sensors")]
    if gamepad.has_sensor(Sensor::Gyroscope) {
        gamepad.enable_sensor(Sensor::Gyroscope)?;
    }
    #[cfg(feature = "sensors")]
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
                gamepad.set_rumble(
                    u16::MAX,
                    u16::MAX,
                    Duration::from_millis(100),
                )?;
            } else if gamepad.buttons_pressed(Button::A) {
                gamepad.set_rumble(u16::MAX, 0, Duration::from_millis(100))?;
            } else if gamepad.buttons_pressed(Button::B) {
                gamepad.set_rumble(0, u16::MAX, Duration::from_millis(100))?;
            } else {
                gamepad.end_rumble()?;
            }
        }

        println!(
            "{gamepad:10}, {:6.3?} {:6.3?} {:6.3?} {:6.3?} {:6.3?} {:6.3?}",
            gamepad.buttons(Button::all()),
            gamepad.stick(Stick::Right),
            gamepad.trigger(Trigger::Right),
            {
                #[cfg(feature = "sensors")]
                gamepad.sensor(Sensor::Gyroscope)
            },
            {
                #[cfg(feature = "sensors")]
                gamepad.sensor(Sensor::Accelerometer)
            },
            gamepad.touchpad(),
            gamepad = gamepad,
        );

        thread::sleep(Duration::from_millis(10));
    }
}
