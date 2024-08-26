use std::{thread, time::Duration};

pub fn main() {
    watch_constants(|| println!("Change detected"));

    let mut index = 0;
    loop {
        let mv = my_val();
        println!("{index:04}: {mv}",);
        index += 1;

        thread::sleep(Duration::from_secs(1));
    }
}

use hot_constant::*;

hot_const!(my_val, f32, 10.0);
