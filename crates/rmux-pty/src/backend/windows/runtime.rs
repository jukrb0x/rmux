use std::os::windows::ffi::OsStrExt;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{FreeLibrary, HANDLE};
use windows_sys::Win32::System::Console::{
    ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON,
};
use windows_sys::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
};

pub(super) struct Api {
    pub create: unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> i32,
    pub resize: unsafe extern "system" fn(HPCON, COORD) -> i32,
    pub close: unsafe extern "system" fn(HPCON),
    pub bundled: bool,
}

pub(super) fn api() -> &'static Api {
    static API: OnceLock<Api> = OnceLock::new();
    API.get_or_init(|| {
        load_bundled().unwrap_or(Api {
            create: CreatePseudoConsole,
            resize: ResizePseudoConsole,
            close: ClosePseudoConsole,
            bundled: false,
        })
    })
}

fn load_bundled() -> Option<Api> {
    let executable = std::env::current_exe().ok()?;
    let path = executable.with_file_name("conpty.dll");
    if !path.is_file() || !executable.with_file_name("OpenConsole.exe").is_file() {
        return None;
    }
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: The absolute sibling path is NUL-terminated; DLL dependencies are
    // restricted to its own directory and System32, never CWD or PATH.
    let module = unsafe {
        LoadLibraryExW(
            path.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    };
    if module.is_null() {
        tracing::warn!(
            "could not load bundled ConPTY: {}",
            std::io::Error::last_os_error()
        );
        return None;
    }
    // SAFETY: `module` is live and each export name is NUL-terminated.
    let exports = unsafe {
        (
            GetProcAddress(module, c"ConptyCreatePseudoConsole".as_ptr().cast()),
            GetProcAddress(module, c"ConptyResizePseudoConsole".as_ptr().cast()),
            GetProcAddress(module, c"ConptyClosePseudoConsole".as_ptr().cast()),
        )
    };
    let (Some(create), Some(resize), Some(close)) = exports else {
        // SAFETY: No function pointers escaped from this failed initialization.
        unsafe { FreeLibrary(module) };
        tracing::warn!("bundled ConPTY is missing required exports");
        return None;
    };
    // SAFETY: These signatures match Microsoft's conpty.h. Retain the module's
    // load reference for process lifetime so all HPCON operations use this API.
    Some(unsafe {
        Api {
            create: std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> i32,
            >(create),
            resize: std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(HPCON, COORD) -> i32,
            >(resize),
            close: std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(HPCON),
            >(close),
            bundled: true,
        }
    })
}
