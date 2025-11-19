/// UI components for Colony
///
/// This module provides various UI options for interacting with colony:
/// - TUI (terminal user interface) - default, always available
/// - Webview (embedded web dashboard) - optional, feature-gated

#[cfg(feature = "webview")]
pub mod webview;

#[cfg(feature = "webview")]
pub use webview::show_dashboard;
