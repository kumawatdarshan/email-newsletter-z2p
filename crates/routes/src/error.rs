use std::error::Error;

pub(crate) fn write_error_chain(
    f: &mut std::fmt::Formatter,
    error: &impl Error,
) -> std::fmt::Result {
    writeln!(f, "{error}")?;
    let mut cause = error.source();
    let mut depth = 1;

    while let Some(err) = cause {
        writeln!(f, "{:>width$}+ {err}", "", width = depth * 2)?;
        cause = err.source();
        depth += 1;
    }
    Ok(())
}
