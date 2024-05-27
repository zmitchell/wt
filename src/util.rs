use std::path::Path;

/// Returns a `tracing`-compatible form of a [Path]
pub fn traceable_path(p: impl AsRef<Path>) -> impl tracing::Value {
    let path = p.as_ref();
    path.display().to_string()
}
