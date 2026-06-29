//! Feedback Payload Model, Builder, and Validation
//!
//! Pure logic for constructing and validating the user feedback payload that
//! is later POSTed (by a separate work unit) to the public feedback endpoint.
//!
//! This module performs **no** network or database I/O and persists nothing.
//! Feedback is fire-and-forget: nothing here is written to the project DB.
//!
//! The payload mirrors the shape the website uses:
//! `{ source, os, appVersion, locale, type, summary?, message?, rating? }`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fixed `source` value identifying feedback that originated in the desktop app.
pub const FEEDBACK_SOURCE: &str = "app";

/// Maximum allowed length (in characters) for the optional `summary` field.
pub const MAX_SUMMARY_LEN: usize = 120;

/// Maximum allowed length (in characters) for the `message` field.
pub const MAX_MESSAGE_LEN: usize = 2000;

/// The kind of feedback being submitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    /// A bug report. Requires a `message`.
    Bug,
    /// A feature request. Requires a `message`.
    Feature,
    /// A star rating. Requires a `rating` between 1 and 5.
    Rating,
}

impl FeedbackType {
    /// The lowercase wire representation of this feedback type.
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedbackType::Bug => "bug",
            FeedbackType::Feature => "feature",
            FeedbackType::Rating => "rating",
        }
    }
}

/// Typed validation errors returned when building an invalid payload.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FeedbackError {
    /// The `summary` exceeded [`MAX_SUMMARY_LEN`] characters.
    #[error("summary must be 120 characters or fewer")]
    SummaryTooLong,
    /// The `message` exceeded [`MAX_MESSAGE_LEN`] characters.
    #[error("message must be 2000 characters or fewer")]
    MessageTooLong,
    /// A `message` is required for the given feedback type but was missing/empty.
    #[error("message is required for {0} feedback")]
    MessageRequired(&'static str),
    /// A `rating` is required for `rating` feedback but was missing.
    #[error("a rating between 1 and 5 is required for rating feedback")]
    RatingRequired,
    /// A `rating` was provided but fell outside the 1-5 range.
    #[error("rating must be between 1 and 5")]
    RatingOutOfRange,
}

/// Map an OS identifier (e.g. [`std::env::consts::OS`]) to one of the four
/// values the feedback endpoint understands: `macos`, `windows`, `linux`, or
/// `unknown`.
pub fn map_os(os: &str) -> &'static str {
    match os {
        "macos" => "macos",
        "windows" => "windows",
        "linux" => "linux",
        _ => "unknown",
    }
}

/// The current platform mapped to a feedback OS value.
pub fn current_os() -> &'static str {
    map_os(std::env::consts::OS)
}

/// A validated feedback payload ready to be serialized and POSTed.
///
/// Field names serialize to the camelCase shape expected by the endpoint, with
/// `feedback_type` emitted as `type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackPayload {
    /// Always [`FEEDBACK_SOURCE`] (`"app"`).
    pub source: String,
    /// One of `macos`, `windows`, `linux`, `unknown`.
    pub os: String,
    /// The application version (`appVersion`).
    pub app_version: String,
    /// The user's locale (e.g. `en-US`).
    pub locale: String,
    /// The kind of feedback.
    #[serde(rename = "type")]
    pub feedback_type: FeedbackType,
    /// Optional short summary (<= [`MAX_SUMMARY_LEN`] chars).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Free-form message (<= [`MAX_MESSAGE_LEN`] chars). Required for bug/feature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Star rating (1-5). Required for rating feedback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<u8>,
}

impl FeedbackPayload {
    /// Start building a payload of the given type.
    pub fn builder(feedback_type: FeedbackType) -> FeedbackBuilder {
        FeedbackBuilder::new(feedback_type)
    }
}

/// Builder for [`FeedbackPayload`].
///
/// `source` is always set to [`FEEDBACK_SOURCE`] and `os` defaults to the
/// current platform (see [`current_os`]); both `app_version` and `locale` are
/// always present (with sensible defaults). Call [`FeedbackBuilder::build`] to
/// validate and produce the payload.
#[derive(Debug, Clone)]
pub struct FeedbackBuilder {
    feedback_type: FeedbackType,
    summary: Option<String>,
    message: Option<String>,
    rating: Option<u8>,
    app_version: String,
    locale: String,
    os: Option<String>,
}

impl FeedbackBuilder {
    /// Create a new builder for the given feedback type.
    pub fn new(feedback_type: FeedbackType) -> Self {
        Self {
            feedback_type,
            summary: None,
            message: None,
            rating: None,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            locale: "en-US".to_string(),
            os: None,
        }
    }

    /// Set the optional summary.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Set the message.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the rating (1-5).
    pub fn rating(mut self, rating: u8) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Override the application version (defaults to the crate version).
    pub fn app_version(mut self, app_version: impl Into<String>) -> Self {
        self.app_version = app_version.into();
        self
    }

    /// Override the locale (defaults to `en-US`).
    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = locale.into();
        self
    }

    /// Override the OS identifier (defaults to the current platform). The value
    /// is mapped through [`map_os`] when the payload is built.
    pub fn os(mut self, os: impl Into<String>) -> Self {
        self.os = Some(os.into());
        self
    }

    /// Validate the inputs and build a [`FeedbackPayload`].
    ///
    /// Returns a [`FeedbackError`] on invalid input.
    pub fn build(self) -> Result<FeedbackPayload, FeedbackError> {
        // Normalize text fields: trim and treat empty strings as absent.
        let summary = normalize(self.summary);
        let message = normalize(self.message);

        // Length checks (character count, not bytes).
        if let Some(s) = &summary {
            if s.chars().count() > MAX_SUMMARY_LEN {
                return Err(FeedbackError::SummaryTooLong);
            }
        }
        if let Some(m) = &message {
            if m.chars().count() > MAX_MESSAGE_LEN {
                return Err(FeedbackError::MessageTooLong);
            }
        }

        // Any provided rating must be within range, regardless of type.
        if let Some(r) = self.rating {
            if !(1..=5).contains(&r) {
                return Err(FeedbackError::RatingOutOfRange);
            }
        }

        // Type-specific requirements.
        match self.feedback_type {
            FeedbackType::Bug | FeedbackType::Feature => {
                if message.is_none() {
                    return Err(FeedbackError::MessageRequired(self.feedback_type.as_str()));
                }
            }
            FeedbackType::Rating => {
                if self.rating.is_none() {
                    return Err(FeedbackError::RatingRequired);
                }
            }
        }

        let os = self
            .os
            .as_deref()
            .map(map_os)
            .unwrap_or_else(current_os)
            .to_string();

        Ok(FeedbackPayload {
            source: FEEDBACK_SOURCE.to_string(),
            os,
            app_version: self.app_version,
            locale: self.locale,
            feedback_type: self.feedback_type,
            summary,
            message,
            rating: self.rating,
        })
    }
}

/// Trim a string and return `None` if it is empty after trimming.
fn normalize(value: Option<String>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ── HTTP submission ───────────────────────────────────────────────────────────

/// The public Lambda Function URL that accepts feedback POSTs.
///
/// No API key or secret is required (public client config, not a secret). The
/// trailing slash is required by the Function URL — do not strip it.
pub const FEEDBACK_ENDPOINT: &str =
    "https://2gcszmyn325n5yaey2poh72qbe0tkmre.lambda-url.ca-central-1.on.aws/";

/// Typed error returned by [`submit_feedback`].
///
/// The enum variants give the frontend enough context to distinguish
/// validation problems (user error) from network/server failures.
#[derive(Debug, Serialize, thiserror::Error)]
pub enum SubmitFeedbackError {
    /// The payload failed builder validation (e.g. missing required field).
    #[error("validation error: {0}")]
    Validation(String),
    /// Transport-level failure — no response was received from the server.
    #[error("network error: {0}")]
    Network(String),
    /// The server responded with a non-2xx HTTP status code.
    #[error("server returned {0}")]
    Server(u16),
}

/// Input accepted from the frontend via `invoke("submit_feedback", { ... })`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitFeedbackInput {
    /// The kind of feedback being submitted.
    pub feedback_type: FeedbackType,
    /// Optional short title (<= [`MAX_SUMMARY_LEN`] chars).
    pub summary: Option<String>,
    /// Free-form body (<= [`MAX_MESSAGE_LEN`] chars). Required for `bug` / `feature`.
    pub message: Option<String>,
    /// Star rating 1-5. Required for `rating`.
    pub rating: Option<u8>,
    /// BCP-47 locale override. Defaults to `en-US` when omitted.
    pub locale: Option<String>,
}

/// Core HTTP submission logic with an injectable poster, enabling unit tests
/// to mock the network layer without a live server.
///
/// `poster` receives the serialised JSON body and must resolve to:
/// - `Ok(status)` — the HTTP status code returned by the server
/// - `Err(msg)` — a transport-level error (DNS, TLS, connection refused, …)
pub(crate) async fn post_feedback_with<F, Fut>(
    payload: &FeedbackPayload,
    poster: F,
) -> Result<(), SubmitFeedbackError>
where
    F: FnOnce(serde_json::Value) -> Fut + Send,
    Fut: std::future::Future<Output = Result<u16, String>> + Send,
{
    let body =
        serde_json::to_value(payload).map_err(|e| SubmitFeedbackError::Network(e.to_string()))?;

    let status = poster(body).await.map_err(SubmitFeedbackError::Network)?;

    if (200u16..300).contains(&status) {
        Ok(())
    } else {
        Err(SubmitFeedbackError::Server(status))
    }
}

/// Tauri IPC command — validate input, build the payload, and POST it to the
/// Lambda endpoint entirely from the Rust backend.
///
/// This is the **only** network call the application makes, and it is strictly
/// user-initiated (triggered by an explicit submit action in the UI). Nothing
/// is sent automatically, and offline use is unaffected when this command
/// is never invoked.
///
/// Feedback is fire-and-forget and is **not** persisted to the project
/// database.
#[tauri::command]
pub async fn submit_feedback(input: SubmitFeedbackInput) -> Result<(), SubmitFeedbackError> {
    let mut builder = FeedbackPayload::builder(input.feedback_type);
    if let Some(s) = input.summary {
        builder = builder.summary(s);
    }
    if let Some(m) = input.message {
        builder = builder.message(m);
    }
    if let Some(r) = input.rating {
        builder = builder.rating(r);
    }
    if let Some(l) = input.locale {
        builder = builder.locale(l);
    }

    let payload = builder
        .build()
        .map_err(|e| SubmitFeedbackError::Validation(e.to_string()))?;

    post_feedback_with(&payload, |body| async move {
        reqwest::Client::new()
            .post(FEEDBACK_ENDPOINT)
            .json(&body)
            .send()
            .await
            .map(|resp| resp.status().as_u16())
            .map_err(|e| e.to_string())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_operating_systems() {
        assert_eq!(map_os("macos"), "macos");
        assert_eq!(map_os("windows"), "windows");
        assert_eq!(map_os("linux"), "linux");
    }

    #[test]
    fn maps_unknown_operating_systems_to_unknown() {
        assert_eq!(map_os("freebsd"), "unknown");
        assert_eq!(map_os("ios"), "unknown");
        assert_eq!(map_os(""), "unknown");
    }

    #[test]
    fn current_os_is_one_of_the_allowed_values() {
        let os = current_os();
        assert!(matches!(os, "macos" | "windows" | "linux" | "unknown"));
    }

    #[test]
    fn builder_sets_source_app() {
        let payload = FeedbackPayload::builder(FeedbackType::Bug)
            .message("It broke")
            .build()
            .expect("valid bug feedback");
        assert_eq!(payload.source, "app");
        assert_eq!(payload.source, FEEDBACK_SOURCE);
    }

    #[test]
    fn builder_maps_os_through_map_os() {
        let payload = FeedbackPayload::builder(FeedbackType::Feature)
            .message("Please add dark mode")
            .os("freebsd")
            .build()
            .expect("valid feature feedback");
        assert_eq!(payload.os, "unknown");

        let payload = FeedbackPayload::builder(FeedbackType::Feature)
            .message("Please add dark mode")
            .os("windows")
            .build()
            .expect("valid feature feedback");
        assert_eq!(payload.os, "windows");
    }

    #[test]
    fn builder_defaults_os_to_current_platform() {
        let payload = FeedbackPayload::builder(FeedbackType::Rating)
            .rating(5)
            .build()
            .expect("valid rating feedback");
        assert!(matches!(
            payload.os.as_str(),
            "macos" | "windows" | "linux" | "unknown"
        ));
    }

    #[test]
    fn builder_includes_app_version_and_locale() {
        let payload = FeedbackPayload::builder(FeedbackType::Bug)
            .message("crash")
            .build()
            .expect("valid bug feedback");
        // Defaults are always populated.
        assert_eq!(payload.app_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(payload.locale, "en-US");

        // Overrides are respected.
        let payload = FeedbackPayload::builder(FeedbackType::Bug)
            .message("crash")
            .app_version("9.9.9")
            .locale("fr-FR")
            .build()
            .expect("valid bug feedback");
        assert_eq!(payload.app_version, "9.9.9");
        assert_eq!(payload.locale, "fr-FR");
    }

    #[test]
    fn summary_at_limit_is_accepted() {
        let summary = "a".repeat(MAX_SUMMARY_LEN);
        let payload = FeedbackPayload::builder(FeedbackType::Bug)
            .summary(summary.clone())
            .message("details")
            .build()
            .expect("summary at limit is valid");
        assert_eq!(payload.summary, Some(summary));
    }

    #[test]
    fn summary_over_limit_is_rejected() {
        let summary = "a".repeat(MAX_SUMMARY_LEN + 1);
        let err = FeedbackPayload::builder(FeedbackType::Bug)
            .summary(summary)
            .message("details")
            .build()
            .expect_err("summary over limit must fail");
        assert_eq!(err, FeedbackError::SummaryTooLong);
    }

    #[test]
    fn message_at_limit_is_accepted() {
        let message = "m".repeat(MAX_MESSAGE_LEN);
        let payload = FeedbackPayload::builder(FeedbackType::Feature)
            .message(message.clone())
            .build()
            .expect("message at limit is valid");
        assert_eq!(payload.message, Some(message));
    }

    #[test]
    fn message_over_limit_is_rejected() {
        let message = "m".repeat(MAX_MESSAGE_LEN + 1);
        let err = FeedbackPayload::builder(FeedbackType::Feature)
            .message(message)
            .build()
            .expect_err("message over limit must fail");
        assert_eq!(err, FeedbackError::MessageTooLong);
    }

    #[test]
    fn bug_requires_message() {
        let err = FeedbackPayload::builder(FeedbackType::Bug)
            .build()
            .expect_err("bug without message must fail");
        assert_eq!(err, FeedbackError::MessageRequired("bug"));
    }

    #[test]
    fn feature_requires_message() {
        let err = FeedbackPayload::builder(FeedbackType::Feature)
            .build()
            .expect_err("feature without message must fail");
        assert_eq!(err, FeedbackError::MessageRequired("feature"));
    }

    #[test]
    fn blank_message_counts_as_missing() {
        let err = FeedbackPayload::builder(FeedbackType::Bug)
            .message("   \n\t  ")
            .build()
            .expect_err("whitespace-only message must fail");
        assert_eq!(err, FeedbackError::MessageRequired("bug"));
    }

    #[test]
    fn rating_requires_a_rating() {
        let err = FeedbackPayload::builder(FeedbackType::Rating)
            .build()
            .expect_err("rating without a rating must fail");
        assert_eq!(err, FeedbackError::RatingRequired);
    }

    #[test]
    fn rating_must_be_within_one_to_five() {
        for r in 1..=5u8 {
            let payload = FeedbackPayload::builder(FeedbackType::Rating)
                .rating(r)
                .build()
                .expect("ratings 1-5 are valid");
            assert_eq!(payload.rating, Some(r));
        }

        let err = FeedbackPayload::builder(FeedbackType::Rating)
            .rating(0)
            .build()
            .expect_err("rating 0 must fail");
        assert_eq!(err, FeedbackError::RatingOutOfRange);

        let err = FeedbackPayload::builder(FeedbackType::Rating)
            .rating(6)
            .build()
            .expect_err("rating 6 must fail");
        assert_eq!(err, FeedbackError::RatingOutOfRange);
    }

    #[test]
    fn out_of_range_rating_rejected_even_for_bug() {
        let err = FeedbackPayload::builder(FeedbackType::Bug)
            .message("details")
            .rating(9)
            .build()
            .expect_err("out-of-range rating must fail regardless of type");
        assert_eq!(err, FeedbackError::RatingOutOfRange);
    }

    #[test]
    fn serializes_to_expected_wire_shape() {
        let payload = FeedbackPayload::builder(FeedbackType::Rating)
            .rating(4)
            .app_version("1.2.0")
            .locale("en-GB")
            .os("macos")
            .build()
            .expect("valid rating feedback");

        let json = serde_json::to_value(&payload).expect("serializes");
        assert_eq!(json["source"], "app");
        assert_eq!(json["os"], "macos");
        assert_eq!(json["appVersion"], "1.2.0");
        assert_eq!(json["locale"], "en-GB");
        assert_eq!(json["type"], "rating");
        assert_eq!(json["rating"], 4);
        // Omitted optional fields are not serialized.
        assert!(json.get("summary").is_none());
        assert!(json.get("message").is_none());
    }
}

#[cfg(test)]
mod submit_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Build a minimal valid bug payload for use in HTTP-layer tests.
    fn bug_payload() -> FeedbackPayload {
        FeedbackPayload::builder(FeedbackType::Bug)
            .message("broken")
            .os("macos")
            .app_version("1.0.0")
            .locale("en-US")
            .build()
            .expect("valid bug payload")
    }

    // ── 2xx → Ok ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_ok_on_200() {
        let result = post_feedback_with(&bug_payload(), |_| async { Ok(200u16) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn returns_ok_on_201_and_204() {
        for status in [201u16, 204] {
            let result =
                post_feedback_with(&bug_payload(), move |_| async move { Ok(status) }).await;
            assert!(result.is_ok(), "expected Ok for HTTP {status}");
        }
    }

    // ── non-2xx → Err(Server) ─────────────────────────────────────────────────

    #[tokio::test]
    async fn returns_server_err_on_4xx() {
        let result = post_feedback_with(&bug_payload(), |_| async { Ok(400u16) }).await;
        assert!(
            matches!(result, Err(SubmitFeedbackError::Server(400))),
            "expected Server(400), got {result:?}"
        );
    }

    #[tokio::test]
    async fn returns_server_err_on_5xx() {
        let result = post_feedback_with(&bug_payload(), |_| async { Ok(500u16) }).await;
        assert!(
            matches!(result, Err(SubmitFeedbackError::Server(500))),
            "expected Server(500), got {result:?}"
        );
    }

    #[tokio::test]
    async fn returns_server_err_preserves_status_code() {
        for status in [400u16, 401, 403, 404, 422, 429, 500, 502, 503] {
            let result =
                post_feedback_with(&bug_payload(), move |_| async move { Ok(status) }).await;
            assert!(
                matches!(result, Err(SubmitFeedbackError::Server(s)) if s == status),
                "expected Server({status}), got {result:?}"
            );
        }
    }

    // ── transport failure → Err(Network) ─────────────────────────────────────

    #[tokio::test]
    async fn returns_network_err_on_transport_failure() {
        let result = post_feedback_with(&bug_payload(), |_| async {
            Err("connection refused".to_string())
        })
        .await;
        assert!(
            matches!(result, Err(SubmitFeedbackError::Network(_))),
            "expected Network error, got {result:?}"
        );
    }

    // ── JSON contract ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn serializes_bug_payload_to_expected_json_contract() {
        let payload = FeedbackPayload::builder(FeedbackType::Bug)
            .message("crash on launch")
            .summary("crash report")
            .app_version("1.2.0")
            .locale("en-GB")
            .os("macos")
            .build()
            .expect("valid payload");

        let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
        let cap = captured.clone();

        let result = post_feedback_with(&payload, move |body| {
            *cap.lock().unwrap() = Some(body);
            async { Ok(200u16) }
        })
        .await;

        assert!(result.is_ok());
        let body = captured.lock().unwrap().take().unwrap();
        assert_eq!(body["source"], "app", "source must be 'app'");
        assert_eq!(body["os"], "macos");
        assert_eq!(body["appVersion"], "1.2.0", "must use camelCase appVersion");
        assert_eq!(body["locale"], "en-GB");
        assert_eq!(body["type"], "bug", "FeedbackType must serialize as 'type'");
        assert_eq!(body["message"], "crash on launch");
        assert_eq!(body["summary"], "crash report");
        assert!(
            body.get("rating").is_none(),
            "rating must be absent for bug payloads"
        );
    }

    #[tokio::test]
    async fn serializes_feature_payload_to_expected_json_contract() {
        let payload = FeedbackPayload::builder(FeedbackType::Feature)
            .message("dark mode please")
            .app_version("1.2.0")
            .locale("en-US")
            .os("windows")
            .build()
            .expect("valid payload");

        let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
        let cap = captured.clone();

        let _ = post_feedback_with(&payload, move |body| {
            *cap.lock().unwrap() = Some(body);
            async { Ok(200u16) }
        })
        .await;

        let body = captured.lock().unwrap().take().unwrap();
        assert_eq!(body["type"], "feature");
        assert_eq!(body["message"], "dark mode please");
        assert!(
            body.get("summary").is_none(),
            "summary must be absent when not provided"
        );
        assert!(
            body.get("rating").is_none(),
            "rating must be absent for feature payloads"
        );
    }

    #[tokio::test]
    async fn serializes_rating_payload_to_expected_json_contract() {
        let payload = FeedbackPayload::builder(FeedbackType::Rating)
            .rating(5)
            .app_version("1.2.0")
            .locale("en-US")
            .os("linux")
            .build()
            .expect("valid payload");

        let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
        let cap = captured.clone();

        let _ = post_feedback_with(&payload, move |body| {
            *cap.lock().unwrap() = Some(body);
            async { Ok(200u16) }
        })
        .await;

        let body = captured.lock().unwrap().take().unwrap();
        assert_eq!(body["type"], "rating");
        assert_eq!(body["rating"], 5);
        assert!(
            body.get("message").is_none(),
            "message must be absent for rating payloads"
        );
        assert!(
            body.get("summary").is_none(),
            "summary must be absent when not provided"
        );
    }
}
