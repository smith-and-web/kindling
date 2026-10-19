//! Routes a system-initiated quit through the frontend's save-before-quit flow.
//!
//! On macOS, Dock > Quit, logout/shutdown and AppleScript `quit` call
//! `-[NSApplication terminate:]`. tao 0.34 implements no
//! `applicationShouldTerminate:`, so AppKit terminates at once: no
//! `CloseRequested`, no `RunEvent::ExitRequested`, and the debounced prose and
//! synopsis saves never run. We add that delegate method, answer
//! `NSTerminateLater`, and emit [`QUIT_REQUESTED_EVENT`]. The frontend then runs
//! the same flush as the menu Quit and either calls `exit(0)` (which arrives
//! here as `ExitRequested` and approves the pending terminate) or invokes
//! [`cancel_app_quit`] (save failed, or an update is installing).
//!
//! AppKit must get its reply outside tao's event callback, because approving
//! runs `applicationWillTerminate:`, which re-enters tao's handler. Replies are
//! therefore queued with `performSelectorOnMainThread:`.

use std::sync::Mutex;
use std::time::Duration;

/// Event the frontend listens for to run its quit flow.
pub const QUIT_REQUESTED_EVENT: &str = "app-quit-requested";

/// How long a system quit waits for the frontend before going ahead anyway, so
/// a hung or unloaded webview can never block Dock Quit or logout forever.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const FRONTEND_TIMEOUT: Duration = Duration::from_secs(10);

/// Tracks the single outstanding `NSTerminateLater` reply.
struct QuitGate {
    state: Mutex<GateState>,
}

struct GateState {
    pending: Option<u64>,
    next: u64,
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
impl QuitGate {
    const fn new() -> Self {
        Self {
            state: Mutex::new(GateState {
                pending: None,
                next: 0,
            }),
        }
    }

    /// Records a system quit request. Returns its id, or `None` when one is
    /// already waiting on the frontend.
    fn request(&self) -> Option<u64> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.pending.is_some() {
            return None;
        }
        state.next += 1;
        state.pending = Some(state.next);
        state.pending
    }

    /// Clears the outstanding request. True when there was one to answer.
    fn resolve(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.pending.take().is_some()
    }

    /// Clears the request only if it is still `id`, so a timeout from an
    /// earlier request never answers a later one.
    fn expire(&self, id: u64) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.pending == Some(id) {
            state.pending = None;
            true
        } else {
            false
        }
    }
}

static GATE: QuitGate = QuitGate::new();

/// Called from the run loop on `RunEvent::ExitRequested`. Returns true when the
/// exit answered a pending system quit: the caller must then prevent this exit
/// and let AppKit finish terminating, which it has been asked to do.
pub fn on_exit_requested(code: Option<i32>) -> bool {
    // `None` means the last window closed, which already ran the frontend flush
    // through `onCloseRequested`; only our own `exit()` answers a system quit.
    if code.is_none() || !GATE.resolve() {
        return false;
    }
    platform::reply(true);
    true
}

/// Frontend reply when the save-before-quit flow did not exit.
#[tauri::command]
pub fn cancel_app_quit() {
    if GATE.resolve() {
        platform::reply(false);
    }
}

/// Installs the macOS delegate hook. No-op elsewhere.
pub fn install<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    platform::install(app);
}

#[cfg(not(target_os = "macos"))]
mod platform {
    pub fn install<R: tauri::Runtime>(_app: &tauri::AppHandle<R>) {}
    pub fn reply(_terminate: bool) {}
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{FRONTEND_TIMEOUT, GATE, QUIT_REQUESTED_EVENT};
    use std::ffi::{c_char, c_void, CStr};
    use std::sync::OnceLock;
    use tauri::{Emitter, Manager};

    type Id = *mut c_void;
    type Sel = *const c_void;

    const NS_TERMINATE_LATER: usize = 2;

    #[link(name = "objc")]
    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn object_getClass(object: Id) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn class_addMethod(class: Id, sel: Sel, imp: *const c_void, types: *const c_char) -> bool;
        fn objc_msgSend();
    }

    type Emit = Box<dyn Fn() -> bool + Send + Sync>;
    static EMIT: OnceLock<Emit> = OnceLock::new();

    fn sel(name: &CStr) -> Sel {
        unsafe { sel_registerName(name.as_ptr()) }
    }

    fn shared_app() -> Id {
        let send: unsafe extern "C" fn(Id, Sel) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe {
            send(
                objc_getClass(c"NSApplication".as_ptr()),
                sel(c"sharedApplication"),
            )
        }
    }

    fn delegate() -> Id {
        let send: unsafe extern "C" fn(Id, Sel) -> Id =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe { send(shared_app(), sel(c"delegate")) }
    }

    pub fn install<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
        let handle = app.clone();
        let _ = EMIT.set(Box::new(move || {
            handle.get_webview_window("main").is_some_and(|window| {
                window
                    .emit(QUIT_REQUESTED_EVENT, ())
                    .inspect_err(|e| eprintln!("Failed to forward system quit: {e}"))
                    .is_ok()
            })
        }));

        let delegate = delegate();
        if delegate.is_null() {
            eprintln!("No app delegate; system quit will skip the final save");
            return;
        }
        let class = unsafe { object_getClass(delegate) };
        let methods: [(&CStr, *const c_void, &CStr); 3] = [
            (
                c"applicationShouldTerminate:",
                should_terminate as *const c_void,
                c"Q@:@",
            ),
            (
                c"kindlingApproveTerminate:",
                approve as *const c_void,
                c"v@:@",
            ),
            (
                c"kindlingCancelTerminate:",
                cancel as *const c_void,
                c"v@:@",
            ),
        ];
        for (name, imp, types) in methods {
            // Fails if tao ever implements the method itself; keep its version.
            if !unsafe { class_addMethod(class, sel(name), imp, types.as_ptr()) } {
                eprintln!("Could not add {name:?} to the app delegate");
            }
        }
    }

    extern "C" fn should_terminate(_this: Id, _cmd: Sel, _sender: Id) -> usize {
        let Some(id) = GATE.request() else {
            return NS_TERMINATE_LATER;
        };
        // Emitting from AppKit's terminate: call could re-enter tao, so hand
        // the event to another thread; it reaches the webview via the proxy.
        std::thread::spawn(move || {
            let forwarded = EMIT.get().is_some_and(|emit| emit());
            if !forwarded {
                if GATE.expire(id) {
                    reply(true);
                }
                return;
            }
            std::thread::sleep(FRONTEND_TIMEOUT);
            if GATE.expire(id) {
                eprintln!("Frontend did not answer the system quit; quitting anyway");
                reply(true);
            }
        });
        NS_TERMINATE_LATER
    }

    fn reply_now(terminate: bool) {
        let send: unsafe extern "C" fn(Id, Sel, bool) =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe {
            send(
                shared_app(),
                sel(c"replyToApplicationShouldTerminate:"),
                terminate,
            )
        };
    }

    extern "C" fn approve(_this: Id, _cmd: Sel, _arg: Id) {
        reply_now(true);
    }

    extern "C" fn cancel(_this: Id, _cmd: Sel, _arg: Id) {
        reply_now(false);
    }

    /// Queues the AppKit reply on the main run loop in the common modes, which
    /// include the modal mode AppKit runs while a terminate is pending.
    pub fn reply(terminate: bool) {
        let name = if terminate {
            c"kindlingApproveTerminate:"
        } else {
            c"kindlingCancelTerminate:"
        };
        let send: unsafe extern "C" fn(Id, Sel, Sel, Id, bool) =
            unsafe { std::mem::transmute(objc_msgSend as unsafe extern "C" fn()) };
        unsafe {
            send(
                delegate(),
                sel(c"performSelectorOnMainThread:withObject:waitUntilDone:"),
                sel(name),
                std::ptr::null_mut(),
                false,
            )
        };
    }
}

#[cfg(test)]
mod tests {
    use super::QuitGate;

    #[test]
    fn request_is_single_flight() {
        let gate = QuitGate::new();
        assert_eq!(gate.request(), Some(1));
        assert_eq!(gate.request(), None);
        assert!(gate.resolve());
        assert_eq!(gate.request(), Some(2));
    }

    #[test]
    fn resolve_answers_only_a_pending_request() {
        let gate = QuitGate::new();
        assert!(
            !gate.resolve(),
            "our own Quit with nothing pending must just exit"
        );
        gate.request();
        assert!(gate.resolve());
        assert!(
            !gate.resolve(),
            "a second exit after the reply must go through"
        );
    }

    #[test]
    fn stale_timeout_does_not_answer_a_newer_request() {
        let gate = QuitGate::new();
        let first = gate.request().unwrap();
        assert!(gate.resolve());
        let second = gate.request().unwrap();
        assert!(!gate.expire(first));
        assert!(gate.expire(second));
        assert!(!gate.resolve());
    }
}
