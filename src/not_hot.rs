#[macro_export]
macro_rules! hot_const {
    ($id: ident, $ty: ty, $value: expr) => {
        pub fn $id() -> $ty {
            $value
        }
    };

    ($id: ident, $ty: ty, $value: expr, $to_str:expr, $from_str:expr) => {
        pub fn $id() -> $ty {
            $value
        }
    };
}

pub fn watch_constants(on_changed: impl Fn() + Sync + Send + 'static ) {}