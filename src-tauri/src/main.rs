// 在 Windows 上发布时禁用额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 二进制程序入口点。
///
/// 仅委托给库的 `run` 函数。
fn main() {
    githeart_lib::run()
}