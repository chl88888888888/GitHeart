//! GitHeart Tauri 应用程序的主库 crate。
//!
//! 负责设置 Tauri 应用程序、注册命令处理器，
//! 并配置主窗口标题。

mod commands;
mod models;

use commands::{git_analyzer::analyze_local_repo, github_analyzer::analyze_github_repo};
use tauri::Manager;

/// Tauri 应用程序的入口点。
///
/// 初始化所有插件、设置调用处理器（包含两个分析命令），
/// 并将主窗口标题设为“Git 心电图”。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            analyze_local_repo, analyze_github_repo
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_title("Git 心电图").unwrap();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}