use std::{thread, time::Duration};
use bevy_color::Srgba;
pub fn main() {
    watch_constants(|| println!("Change detected"));

    let mut index = 0;
    loop {
        let mv = color_val();
        println!("{index:04}: {}",mv.to_hex());
        index += 1;

        thread::sleep(Duration::from_secs(1));
    }
}

use hot_constant::*;

hot_const!(color_val, Srgba, Srgba::RED, Srgba::to_hex, Srgba::hex);
