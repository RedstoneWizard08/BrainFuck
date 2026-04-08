#[macro_export]
macro_rules! any_needs_64 {
    ($($args: ident),*) => {
        $($args.needs_64() || $args.bit_width() == 64)||*
    }
}
