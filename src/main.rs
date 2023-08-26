use std::error::Error;
use std::path::PathBuf;
// use std::time::{Duration, Instant};

use clap::Parser;
use evdev::raw_stream::*;
use evdev::uinput::*;
use evdev::*;

fn q2d(key: Key) -> Key {
    //! QWERTY to Dvorak
    match key {
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
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// keyboard device, /dev/input/event* or /dev/input/by-id/*
    device: PathBuf,
}

type Kind = InputEventKind;
type Event = InputEvent;
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut keys = AttributeSet::<Key>::new();
    for i in 0..0x23e {
        keys.insert(Key(i));
    }
    let mut fo = VirtualDeviceBuilder::new()?.name("DQ").with_keys(&keys)?.build()?;
    let mut fi = RawDevice::open(args.device)?;

    fi.grab()?;
    let map = |event: Event, state: &AttributeSet<Key>| match event.kind() {
        _ if state.contains(Key::KEY_LEFTCTRL) || state.contains(Key::KEY_RIGHTCTRL) => event,
        Kind::Key(k) => Event::new(EventType(0x01), q2d(k).code(), event.value()),
        _ => event,
    };
    // let start = Instant::now();
    // let duration = Duration::from_secs(5);
    // while Instant::now() - start < duration {
    loop {
        let state = fi.get_key_state()?;
        let events = fi
            .fetch_events()?
            .filter(|event| !matches!(event.kind(), Kind::Synchronization(_)))
            .map(|event| map(event, &state))
            .collect::<Vec<_>>();
        fo.emit(&events)?;
    }
    // fi.ungrab()?;
    // Ok(())
}
