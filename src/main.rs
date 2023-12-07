use std::error::Error;
use std::ops::Deref;
use std::path::PathBuf;

use clap::Parser;
use evdev::raw_stream::*;
use evdev::uinput::*;
use evdev::*;

type Kind = InputEventKind;
type Event = InputEvent;

const KEY: EventType = EventType::KEY;
const LED: EventType = EventType::LED;

fn q2d(key: Key, value: i32) -> Event {
    //! QWERTY key to Dvorak event
    let key = match key {
        Key::KEY_MINUS => Key::KEY_LEFTBRACE,
        Key::KEY_EQUAL => Key::KEY_RIGHTBRACE,

        Key::KEY_Q => Key::KEY_APOSTROPHE,
        Key::KEY_W => Key::KEY_COMMA,
        Key::KEY_E => Key::KEY_DOT,
        Key::KEY_R => Key::KEY_P,
        Key::KEY_T => Key::KEY_Y,
        Key::KEY_Y => Key::KEY_F,
        Key::KEY_U => Key::KEY_G,
        Key::KEY_I => Key::KEY_C,
        Key::KEY_O => Key::KEY_R,
        Key::KEY_P => Key::KEY_L,
        Key::KEY_LEFTBRACE => Key::KEY_SLASH,
        Key::KEY_RIGHTBRACE => Key::KEY_EQUAL,

        Key::KEY_S => Key::KEY_O,
        Key::KEY_D => Key::KEY_E,
        Key::KEY_F => Key::KEY_U,
        Key::KEY_G => Key::KEY_I,
        Key::KEY_H => Key::KEY_D,
        Key::KEY_J => Key::KEY_H,
        Key::KEY_K => Key::KEY_T,
        Key::KEY_L => Key::KEY_N,
        Key::KEY_SEMICOLON => Key::KEY_S,
        Key::KEY_APOSTROPHE => Key::KEY_MINUS,

        Key::KEY_Z => Key::KEY_SEMICOLON,
        Key::KEY_X => Key::KEY_Q,
        Key::KEY_C => Key::KEY_J,
        Key::KEY_V => Key::KEY_K,
        Key::KEY_B => Key::KEY_X,
        Key::KEY_N => Key::KEY_B,
        Key::KEY_M => Key::KEY_M,
        Key::KEY_COMMA => Key::KEY_W,
        Key::KEY_DOT => Key::KEY_V,
        Key::KEY_SLASH => Key::KEY_Z,
        k => k,
    };
    Event::new(KEY, key.code(), value)
}

#[inline]
fn control(state: &AttributeSet<Key>) -> bool {
    //! check if control key is pressed
    state.contains(Key::KEY_LEFTCTRL) || state.contains(Key::KEY_RIGHTCTRL)
}

#[inline]
fn capslock(brightness: i32) -> Event {
    //! capslock LED event
    Event::new(LED, LedType::LED_CAPSL.0, brightness)
}

#[derive(Parser, Debug)]
struct Args {
    /// keyboard device, /dev/input/event* or /dev/input/by-id/*
    device: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let keys = AttributeSet::<Key>::from_iter((0..0x23e).map(Key));
    let mut fo = VirtualDeviceBuilder::new()?.name("DQ").with_keys(&keys)?.build()?;
    let mut fi = RawDevice::open(args.device)?;
    // let mut fi = Device::open(args.device)?;

    fi.grab()?;
    let mut dvorak = false;
    loop {
        let state = fi.get_key_state()?;
        // let state = fi.cached_state();
        // let state = AttributeSet::<Key>::from_iter(state.key_vals().unwrap().iter());
        let mut toggle = None;
        println!("{:?}", state);
        let events = fi
            .fetch_events()?
            .map(|event| (event, event.kind(), event.value()))
            .filter_map(|(event, kind, value)| match kind {
                // sync events are sent by emit automatically
                Kind::Synchronization(_) | Kind::Key(Key::KEY_CAPSLOCK) if value == 0 => None,
                // toggle dvorak on capslock press
                Kind::Key(Key::KEY_CAPSLOCK) => {
                    dvorak = !dvorak;
                    toggle = Some(capslock(if dvorak { i32::MAX } else { 0 }));
                    None
                }
                // map qwerty to dvorak
                Kind::Key(k) if !control(&state) && dvorak => Some(q2d(k, value)),
                _ => Some(event),
            })
            .collect::<Vec<_>>();
        // toggle capslock LED
        if let Some(event) = toggle {
            fi.send_events(&[event])?;
        }
        fo.emit(&events)?;
    }
}
