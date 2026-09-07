//! Environment policy resolution; not an operating-system or universal capability API.
use crate::Theme;

pub(crate) fn environment_theme(output_is_tty: bool) -> Theme {
    Theme::new(
        output_is_tty,
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var("TERM").ok().as_deref(),
    )
}
