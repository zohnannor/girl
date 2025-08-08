#![allow(missing_docs, unsafe_code)]

use std::{ffi::CStr, time::Instant};

use sdl2_sys::{
    SDL_Event, SDL_EventType, SDL_GameControllerClose,
    SDL_GameControllerFromInstanceID, SDL_GameControllerGetSensorData,
    SDL_GameControllerGetSensorDataRate, SDL_GameControllerName,
    SDL_GameControllerOpen, SDL_GameControllerSetSensorEnabled, SDL_GetError,
    SDL_INIT_GAMECONTROLLER, SDL_Init, SDL_NumJoysticks, SDL_Quit,
    SDL_SensorType, SDL_WaitEvent, SDL_bool, Uint32,
};

macro_rules! assert_ok {
    ($condition:expr, $context:literal $(,)?) => {
        if !$condition {
            let err = unsafe { SDL_GetError() };
            let error = unsafe { CStr::from_ptr(err) };
            panic!(
                "[{}:{}:{}] {}: {}",
                file!(),
                line!(),
                column!(),
                $context,
                error.to_str().unwrap()
            );
        }
    };
}

#[allow(clippy::too_many_lines)]
fn main() {
    {
        let res = unsafe { SDL_Init(SDL_INIT_GAMECONTROLLER) };
        assert_ok!(res >= 0, "SDL_Init failed");
    }

    {
        let num_joysticks = unsafe { SDL_NumJoysticks() };
        dbg!(num_joysticks);
        assert_ok!(num_joysticks > 0, "No controllers detected");
    }

    let controller = unsafe { SDL_GameControllerOpen(0) };
    assert_ok!(!controller.is_null(), "Couldn't open controller");

    {
        let res = unsafe { SDL_GameControllerName(controller) };
        assert_ok!(!res.is_null(), "Couldn't get controller name");
        let name = unsafe { CStr::from_ptr(res) };
        println!("Controller opened: {}", name.to_string_lossy());
    }

    {
        let res = unsafe {
            SDL_GameControllerSetSensorEnabled(
                controller,
                SDL_SensorType::SDL_SENSOR_GYRO,
                SDL_bool::SDL_TRUE,
            )
        };
        assert_ok!(res == 0, "Couldn't enable gyroscope");
        println!("Gyroscope enabled");
    }

    let rate = unsafe {
        SDL_GameControllerGetSensorDataRate(
            controller,
            SDL_SensorType::SDL_SENSOR_GYRO,
        )
    };
    assert_ok!(rate != 0.0, "Couldn't get sensor data rate");
    println!("Gyroscope data rate: {rate} Hz");

    let mut event = SDL_Event { type_: 0 };

    println!("Waiting for gyro events... (move your controller)");

    let mut event_count = 0;
    let time = Instant::now();

    while event_count < 200 {
        const SDL_QUIT: Uint32 = SDL_EventType::SDL_QUIT as _;
        const SDL_CONTROLLERSENSORUPDATE: Uint32 =
            SDL_EventType::SDL_CONTROLLERSENSORUPDATE as _;

        let good = unsafe { SDL_WaitEvent(&raw mut event) };
        assert_ok!(good > 0, "SDL_WaitEvent failed");
        if good > 0 {
            match unsafe { event.type_ } {
                SDL_QUIT => break,
                SDL_CONTROLLERSENSORUPDATE => {
                    let controller_event = unsafe { event.csensor };

                    let event_controller = unsafe {
                        SDL_GameControllerFromInstanceID(controller_event.which)
                    };
                    assert_ok!(
                        !event_controller.is_null(),
                        "Couldn't get controller"
                    );

                    if event_controller == controller {
                        let mut data = [0.0f32; 3];
                        let res = unsafe {
                            SDL_GameControllerGetSensorData(
                                controller,
                                SDL_SensorType::SDL_SENSOR_GYRO,
                                data.as_mut_ptr(),
                                3,
                            )
                        };
                        assert_ok!(res == 0, "Couldn't read gyroscope data");

                        event_count += 1;

                        println!(
                            "[{:+8.8?}] Event \
                             {}: x={:>+8.8}, y={:>+8.8}, z={:>+8.8}",
                            time.elapsed(),
                            event_count,
                            data[0],
                            data[1],
                            data[2]
                        );
                    }
                }
                _ => {}
            }
        }
    }

    unsafe { SDL_GameControllerClose(controller) };
    unsafe { SDL_Quit() };
    println!("Clean exit");
}
