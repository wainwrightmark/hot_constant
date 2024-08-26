#[macro_export]
macro_rules! hot_const {
    ($id: ident, $ty: ty, $value: literal) => {
        pub fn $id() -> $ty {
            $value
        }
    };
}

pub fn watch_constants(on_changed: impl Fn() + Sync + Send + 'static ) {}