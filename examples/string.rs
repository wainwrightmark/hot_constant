use std::{thread, time::Duration};

pub fn main() {
    watch_constants(|| println!("Change detected"));

    let mut index = 0;
    loop {
        let mv = my_str();
        println!("{index:04}: {mv}",);
        index += 1;

        thread::sleep(Duration::from_secs(1));
    }
}

use hot_constant::*;

hot_const_str!(my_str,  "hello"); //todo escape this string or whatever
