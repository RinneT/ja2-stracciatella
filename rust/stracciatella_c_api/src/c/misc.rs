//! This module contains miscellaneous code for C.
//!
//! The code in this module is not associated with any module in particular.

use std::ffi::CString;
use std::path::PathBuf;
use std::ptr;

use stracciatella::config::find_stracciatella_home;
use stracciatella::fs::{canonicalize, resolve_existing_components};
use stracciatella::get_assets_dir;
use stracciatella::guess::guess_vanilla_version;

use crate::c::common::*;

/// Converts the launcher executable path to the game executable path.
/// The executable is assumed to be in the same directory as the launcher.
/// The caller is responsible for the returned memory.
#[no_mangle]
pub extern "C" fn findJa2Executable(launcher_path_ptr: *const c_char) -> *mut c_char {
    let launcher_path = str_from_c_str_or_panic(unsafe_c_str(launcher_path_ptr));
    let is_exe = launcher_path.to_lowercase().ends_with(".exe");
    let end_of_executable_slice = launcher_path.len() - if is_exe { 13 } else { 9 };
    let mut executable_path = String::from(&launcher_path[0..end_of_executable_slice]);

    if is_exe {
        executable_path.push_str(if is_exe { ".exe" } else { "" });
    }

    let c_string = c_string_from_str(&executable_path);
    c_string.into_raw()
}

/// Deletes a CString.
/// The caller is no longer responsible for the memory.
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn CString_destroy(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe { CString::from_raw(s) };
}

/// Guesses the resource version from the contents of the game directory.
/// Returns a VanillaVersion value if it was sucessful, -1 otherwise.
#[no_mangle]
pub extern "C" fn guessResourceVersion(gamedir: *const c_char) -> c_int {
    let path = str_from_c_str_or_panic(unsafe_c_str(gamedir));
    let logged = guess_vanilla_version(&path);
    let mut result = -1;
    if let Some(version) = logged.vanilla_version {
        result = version as c_int;
    }
    result
}

/// Finds a path relative to the assets directory.
/// If path is null, it returns the assets directory.
/// If test_exists is true and the path does not exist, it returns null.
/// The caller is responsible for the returned memory.
#[no_mangle]
pub extern "C" fn findPathFromAssetsDir(
    path: *const c_char,
    test_exists: bool,
    caseless: bool,
) -> *mut c_char {
    let assets_dir = get_assets_dir();
    let path = if !path.is_null() {
        let path = path_from_c_str_or_panic(unsafe_c_str(path));
        resolve_existing_components(path, Some(&assets_dir), caseless)
    } else {
        assets_dir
    };
    if test_exists && !path.exists() {
        ptr::null_mut()
    } else {
        let c_string = c_string_from_path_or_panic(&path);
        c_string.into_raw()
    }
}

/// Finds a path relative to the stracciatella home directory.
/// If path is null, it finds the stracciatella home directory.
/// If test_exists is true, it makes sure the path exists.
/// The caller is responsible for the returned memory.
#[no_mangle]
pub extern "C" fn findPathFromStracciatellaHome(
    path: *const c_char,
    test_exists: bool,
    caseless: bool,
) -> *mut c_char {
    if let Ok(mut path_buf) = find_stracciatella_home() {
        if !path.is_null() {
            let path = path_from_c_str_or_panic(unsafe_c_str(path));
            let path = resolve_existing_components(path, Some(&path_buf), caseless);
            path_buf = path;
        }
        if test_exists && !path_buf.exists() {
            ptr::null_mut() // path not found
        } else {
            if let Ok(p) = canonicalize(&path_buf) {
                path_buf = p;
            }
            let s: String = path_buf.to_string_lossy().into();
            CString::new(s).unwrap().into_raw() // path found
        }
    } else {
        ptr::null_mut() // no home
    }
}

/// Returns true if it was able to find path relative to base.
/// Makes caseless searches one component at a time.
#[no_mangle]
pub extern "C" fn checkIfRelativePathExists(
    base: *const c_char,
    path: *const c_char,
    caseless: bool,
) -> bool {
    let base: PathBuf = path_from_c_str_or_panic(unsafe_c_str(base)).to_owned();
    let path: PathBuf = path_from_c_str_or_panic(unsafe_c_str(path)).to_owned();
    let path = resolve_existing_components(&path, Some(&base), caseless);
    path.exists()
}

/// Returns a list of available mods.
/// The caller is responsible for the returned memory.
#[no_mangle]
pub extern "C" fn findAvailableMods() -> *mut VecCString {
    let mut path = get_assets_dir();
    path.push("mods");
    if let Ok(entries) = path.read_dir() {
        let mods: Vec<_> = entries
            .filter_map(|x| x.ok()) // DirEntry
            .filter_map(|x| {
                if let Ok(metadata) = x.metadata() {
                    if metadata.is_dir() {
                        return x.file_name().into_string().ok();
                    }
                }
                None
            }) // String
            .filter_map(|x| CString::new(x.as_bytes().to_owned()).ok()) // CString
            .collect();
        into_ptr(VecCString::from_vec(mods))
    } else {
        into_ptr(VecCString::new())
    }
}

/// A wrapper around `Vec<CString>` for C.
#[derive(Default)]
pub struct VecCString {
    inner: Vec<CString>,
}

impl VecCString {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn from_vec(vec: Vec<CString>) -> Self {
        Self { inner: vec }
    }
}

/// Deletes the vector.
#[no_mangle]
pub extern "C" fn VecCString_destroy(vec: *mut VecCString) {
    let _drop_me = from_ptr(vec);
}

/// Returns the vector length.
#[no_mangle]
pub extern "C" fn VecCString_length(vec: *mut VecCString) -> size_t {
    let vec = unsafe_ref(vec);
    vec.inner.len()
}

/// Returns the string at the target index.
/// The caller is responsible for the returned memory.
#[no_mangle]
pub extern "C" fn VecCString_get(vec: *mut VecCString, index: size_t) -> *mut c_char {
    let vec = unsafe_ref(vec);
    vec.inner[index].clone().into_raw()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::complexity)]

    use std::ffi::CString;
    use std::fs;

    use tempfile::TempDir;

    use crate::c::common::*;
    use crate::c::misc::CString_destroy;

    #[test]
    fn find_ja2_executable_should_determine_game_path_from_launcher_path() {
        macro_rules! t {
            ($path:expr, $expected:expr) => {
                let path = c_string_from_str($path);
                let got = super::findJa2Executable(path.as_ptr());
                assert_eq!(str_from_c_str_or_panic(unsafe_c_str(got)), $expected);
                CString_destroy(got);
            };
        }
        t!("/home/test/ja2-launcher", "/home/test/ja2");
        t!(
            "C:\\\\home\\\\test\\\\ja2-launcher.exe",
            "C:\\\\home\\\\test\\\\ja2.exe"
        );
        t!("ja2-launcher", "ja2");
        t!("ja2-launcher.exe", "ja2.exe");
        t!("JA2-LAUNCHER.EXE", "JA2.exe");
    }

    #[test]
    fn check_if_relative_path_exists() {
        // The case sensitivity of the filesystem is always unknown.
        // Even parts of the path can have different case sensitivity.
        // Make sure the expected result never depends on the case sensitivity of the filesystem!
        //
        // Different representations of the umlaut ö in utf-8:
        // "utf8-gedöns" can be "utf8-ged\u{00F6}ns" or "utf8-gedo\u{0308}ns"
        // "utf8-GEDÖNS" can be "utf8-GED\u{00D6}NS" or "utf8-GEDO\u{0308}NS"

        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("foo/bar")).unwrap();
        fs::create_dir_all(temp_dir.path().join("with space/inner")).unwrap();
        fs::create_dir_all(temp_dir.path().join("utf8-ged\u{00F6}ns/inner")).unwrap();

        macro_rules! t {
            ($base: expr, $path:expr, $caseless:expr, $expected:expr) => {
                let base = CString::new($base.to_str().unwrap()).unwrap();
                let path = CString::new($path).unwrap();
                assert_eq!(
                    super::checkIfRelativePathExists(base.as_ptr(), path.as_ptr(), $caseless),
                    $expected
                );
            };
        }

        t!(temp_dir.path(), "baz", false, false);
        t!(temp_dir.path(), "baz", true, false);

        t!(temp_dir.path(), "foo", false, true);
        t!(temp_dir.path(), "foo", true, true);
        t!(temp_dir.path(), "FOO", true, true);

        t!(temp_dir.path(), "foo/bar", false, true);
        t!(temp_dir.path(), "foo/bar", true, true);
        t!(temp_dir.path(), "foo/BAR", true, true);
        t!(temp_dir.path(), "FOO/BAR", true, true);
        t!(temp_dir.path(), "FOO/bar", true, true);

        t!(temp_dir.path(), "withspace", false, false);
        t!(temp_dir.path(), "withspace", true, false);
        t!(temp_dir.path(), "with space", false, true);
        t!(temp_dir.path(), "with space", true, true);
        t!(temp_dir.path(), "with SPACE", true, true);

        t!(temp_dir.path(), "with space/inner", false, true);
        t!(temp_dir.path(), "with SPACE/inner", true, true);
        t!(temp_dir.path(), "with SPACE/INNER", true, true);
        t!(temp_dir.path(), "with space/INNER", true, true);

        t!(temp_dir.path(), "utf8-ged\u{00F6}ns/inner", false, true);
        t!(temp_dir.path(), "utf8-ged\u{00F6}ns/inner", true, true);
        t!(temp_dir.path(), "utf8-ged\u{00F6}ns/other", false, false);
        t!(temp_dir.path(), "utf8-gedo\u{0308}ns/inner", true, true);
        t!(temp_dir.path(), "utf8-GED\u{00D6}NS/inner", true, true);
        t!(temp_dir.path(), "utf8-GEDO\u{0308}NS/inner", true, true);
    }
}
