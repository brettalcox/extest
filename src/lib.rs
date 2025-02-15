mod steam_keys;
use steam_keys::KEYS;

mod wayland;
use wayland::get_axes_range;

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent, Key, RelativeAxisType,
    UinputAbsSetup,
};
use once_cell::sync::Lazy;
use std::ffi::{c_int, c_uint, c_ulong};
use std::sync::Mutex;

// Opaque type
#[repr(C)]
pub struct Display {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

static DEVICE: Lazy<Mutex<VirtualDevice>> = Lazy::new(|| {
    let size = get_axes_range();
    Mutex::new(
        VirtualDeviceBuilder::new()
            .unwrap()
            .name("extest fake device")
            .with_keys(&AttributeSet::from_iter(
                [Key::BTN_LEFT, Key::BTN_RIGHT, Key::BTN_MIDDLE]
                    .into_iter()
                    .chain(KEYS.iter().copied()),
            ))
            .unwrap()
            .with_relative_axes(&AttributeSet::from_iter(
                [
                    RelativeAxisType::REL_X,
                    RelativeAxisType::REL_Y,
                    RelativeAxisType::REL_WHEEL,
                ]
                .into_iter(),
            ))
            .unwrap()
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisType::ABS_X,
                AbsInfo::new(0, 0, size.width, 0, 0, 1),
            ))
            .unwrap()
            .with_absolute_axis(&UinputAbsSetup::new(
                AbsoluteAxisType::ABS_Y,
                AbsInfo::new(0, 0, size.height, 0, 0, 1),
            ))
            .unwrap()
            .build()
            .unwrap(),
    )
});

#[no_mangle]
pub extern "C" fn XTestFakeKeyEvent(
    _: *mut Display,
    keycode: c_uint,
    is_press: bool,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();

    // Seems that X11 keycodes are just 8 + linux keycode - https://wiki.archlinux.org/title/Keyboard_input#Identifying_keycodes
    let key = match keycode {
        156 => Key::KEY_TAB, // I have no idea where this comes from
        keycode => Key::new((keycode - 8) as u16)
    };
    dev.emit(&[InputEvent::new_now(EventType::KEY, key.0, is_press as i32)])
        .unwrap();
    1
}

#[no_mangle]
pub extern "C" fn XTestFakeButtonEvent(
    _: *mut Display,
    button: c_uint,
    is_press: bool,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();
    // values determined via xev
    let key = match button {
        1 => Key::BTN_LEFT,
        2 => Key::BTN_MIDDLE,
        3 => Key::BTN_RIGHT,
        scrolldir @ (4 | 5) => {
            // 4 = scrollup, 5 = scrolldown
            let value = match scrolldir {
                4 => 1,
                5 => -1,
                _ => unreachable!(),
            };
            dev.emit(&[InputEvent::new_now(
                EventType::RELATIVE,
                RelativeAxisType::REL_WHEEL.0,
                value,
            )])
            .unwrap();
            return 1;
        }
        other => {
            println!("WARNING: received unknown keycode {other}");
            return 1;
        }
    };

    dev.emit(&[InputEvent::new_now(EventType::KEY, key.0, is_press as i32)])
        .unwrap();
    1
}

#[no_mangle]
pub extern "C" fn XTestFakeRelativeMotionEvent(
    _: *mut Display,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();
    let events = [
        InputEvent::new_now(EventType::RELATIVE, RelativeAxisType::REL_X.0, x),
        InputEvent::new_now(EventType::RELATIVE, RelativeAxisType::REL_Y.0, y),
    ];
    dev.emit(&events).unwrap();
    1
}

#[no_mangle]
pub extern "C" fn XTestFakeMotionEvent(
    _: *mut Display,
    _: c_int,
    x: c_int,
    y: c_int,
    _: c_ulong,
) -> c_int {
    let mut dev = DEVICE.lock().unwrap();
    let events = [
        InputEvent::new_now(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, x),
        InputEvent::new_now(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, y),
    ];
    dev.emit(&events).unwrap();
    1
}
