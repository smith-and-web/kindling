//! Debug-only, macOS webview snapshots. Reached via the authenticated QA socket's
//! execute_js -> Tauri invoke bridge, without modifying the upstream MCP plugin.
use serde_json::{json, Value};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use tokio::sync::oneshot;

type Reply = Result<Value, String>;
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
static PENDING: LazyLock<Mutex<HashMap<u64, oneshot::Sender<Reply>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

extern "C" {
    fn kindling_qa_supported() -> bool;
    fn kindling_qa_request(
        webview: *mut c_void,
        request: u64,
        operation: *const c_char,
        width: u32,
        height: u32,
        scale: u32,
        callback: extern "C" fn(u64, *const c_char),
    );
}

pub fn enabled() -> bool {
    if std::env::var("KINDLING_QA_BACKGROUND").as_deref() != Ok("1") {
        return false;
    }
    assert!(
        std::env::var("KINDLING_DATA_DIR").is_ok_and(|v| !v.trim().is_empty()),
        "Background QA requires an explicit KINDLING_DATA_DIR; refusing to open the normal app"
    );
    assert!(
        unsafe { kindling_qa_supported() },
        "Background QA requires macOS 14+"
    );
    true
}

fn validate(width: u32, height: u32, scale: u32) -> Result<(), String> {
    if !(320..=2400).contains(&width)
        || !(240..=1800).contains(&height)
        || !(1..=2).contains(&scale)
        || u64::from(width) * u64::from(height) * u64::from(scale).pow(2) > 16_000_000
    {
        return Err("Invalid QA viewport/scale (maximum 16 million pixels)".into());
    }
    Ok(())
}

extern "C" fn receive(request: u64, raw: *const c_char) {
    // The native callback's string is borrowed only for this synchronous call.
    let result = if raw.is_null() {
        Err("Empty native snapshot response".into())
    } else {
        let text = unsafe { CStr::from_ptr(raw) }.to_string_lossy();
        serde_json::from_str::<Value>(&text)
            .map_err(|e| e.to_string())
            .and_then(|value| match value.get("error").and_then(Value::as_str) {
                Some(error) => Err(error.to_string()),
                None => Ok(value),
            })
    };
    if let Ok(mut pending) = PENDING.lock() {
        if let Some(sender) = pending.remove(&request) {
            let _ = sender.send(result);
        }
    }
}

// Cancellation/timeout removes the sender. Late native completions are ignored,
// cannot fulfill a subsequent request, and do not write files after cancellation.
struct RequestGuard(u64);
impl Drop for RequestGuard {
    fn drop(&mut self) {
        if let Ok(mut pending) = PENDING.lock() {
            pending.remove(&self.0);
        }
    }
}

async fn request(
    window: tauri::WebviewWindow,
    operation: &'static CStr,
    width: u32,
    height: u32,
    scale: u32,
) -> Reply {
    if !enabled() {
        return Err("Start the isolated debug app with npm run tauri:qa".into());
    }
    validate(width, height, scale)?;
    let id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel();
    PENDING.lock().map_err(|e| e.to_string())?.insert(id, tx);
    let _guard = RequestGuard(id);
    window
        .with_webview(move |webview| unsafe {
            kindling_qa_request(
                webview.inner(),
                id,
                operation.as_ptr(),
                width,
                height,
                scale,
                receive,
            );
        })
        .map_err(|e| e.to_string())?;
    tokio::time::timeout(std::time::Duration::from_secs(10), rx)
        .await
        .map_err(|_| "WebKit snapshot timed out; no desktop capture fallback".to_string())?
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn qa_capabilities() -> Reply {
    if !enabled() {
        return Err("Start the isolated debug app with npm run tauri:qa".into());
    }
    Ok(
        json!({"version":1,"profile":"wkwebview-snapshot-2x-png-v1", "background":true,
        "engine":"WKWebView.takeSnapshot","format":"png","scales":[2],
        "checkout":std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent()}),
    )
}

#[tauri::command]
pub async fn qa_set_viewport(window: tauri::WebviewWindow, width: u32, height: u32) -> Reply {
    request(window, c"resize", width, height, 1).await
}

#[tauri::command]
pub async fn qa_snapshot(
    window: tauri::WebviewWindow,
    width: u32,
    height: u32,
    scale: u32,
) -> Reply {
    if scale != 2 {
        return Err("This snapshot profile requires scale 2".into());
    }
    request(window, c"snapshot", width, height, scale).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_limits_prevent_unbounded_allocation() {
        assert!(validate(1600, 968, 2).is_ok());
        assert!(validate(1100, 668, 2).is_ok());
        for args in [
            (0, 968, 2),
            (1600, 0, 2),
            (1600, 968, 0),
            (1600, 968, 3),
            (u32::MAX, 968, 2),
            (2400, 1800, 2),
        ] {
            assert!(validate(args.0, args.1, args.2).is_err());
        }
    }

    #[tokio::test]
    async fn cancelled_callback_cannot_complete_another_request() {
        let id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        PENDING.lock().unwrap().insert(id, tx);
        drop(RequestGuard(id));
        receive(id, c"{\"late\":true}".as_ptr());
        assert!(rx.await.is_err());
        let next = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        PENDING.lock().unwrap().insert(next, tx);
        receive(id, c"{\"late\":true}".as_ptr());
        receive(next, c"{\"fresh\":true}".as_ptr());
        assert_eq!(rx.await.unwrap().unwrap(), json!({"fresh":true}));
    }
}
