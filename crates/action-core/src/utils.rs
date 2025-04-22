use std::ffi::OsStr;

/// `toPosixPath` converts the given path to the posix form.
///
/// On Windows, \\ will be replaced with /.
pub fn to_posix_path(path: impl AsRef<str>) -> String {
    path.as_ref().replace('\\', "/")
}

/// `toWin32Path` converts the given path to the win32 form.
///
/// On Linux, / will be replaced with \\.
pub fn to_win32_path(path: impl AsRef<str>) -> String {
    path.as_ref().replace('/', "\\")
}

/// `toPlatformPath` converts the given path to a platform-specific path.
///
/// It does this by replacing instances of / and \ with
/// the platform-specific path separator.
pub fn to_platform_path(path: impl AsRef<str>) -> String {
    path.as_ref()
        .replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR)
}

pub fn escape_data(data: impl AsRef<str>) -> String {
    data.as_ref()
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

pub fn escape_property(prop: impl AsRef<str>) -> String {
    prop.as_ref()
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

/// Filters empty values.
///
/// # Errors
/// If the value is empty.
pub fn not_empty<T>(value: T) -> Option<T>
where
    T: AsRef<OsStr>,
{
    if value.as_ref().is_empty() {
        None
    } else {
        Some(value)
    }
}
