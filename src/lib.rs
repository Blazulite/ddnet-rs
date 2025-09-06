#![allow(clippy::too_many_arguments)]
#![allow(clippy::module_inception)]

pub mod client;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use base::system::System;
use client::client::ddnet_main;
pub use client::*;
use game_base::local_server_info::LocalServerInfo;
use native::native::app::NativeApp;

#[cfg(all(feature = "alloc_track", feature = "alloc_stats"))]
std::compile_error!(
    "Only one of the features alloc_track & alloc_stats can be activated at a time"
);

#[cfg(feature = "alloc_track")]
#[global_allocator]
static GLOBAL_ALLOC: alloc_track::AllocTrack<std::alloc::System> =
    alloc_track::AllocTrack::new(std::alloc::System, alloc_track::BacktraceMode::Short);

#[cfg(feature = "alloc_stats")]
#[global_allocator]
static GLOBAL: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

#[cfg(not(target_os = "android"))]
fn show_message_box(title: &str, message: &str) {
    use native_dialog::{MessageDialogBuilder, MessageLevel};
    let _ = MessageDialogBuilder::default()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_text(message)
        .alert()
        .show();
}

#[cfg(target_os = "android")]
fn show_message_box(title: &str, message: &str) {
    log::info!("[UNSUPPORTED] msg box: {title} {message}");
}

fn main_impl(app: NativeApp) {
    let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);
    let sys = System::new();

    let shared_info: Arc<LocalServerInfo> = Arc::new(LocalServerInfo::new(true));

    let thread_id = std::thread::current().id();
    std::panic::set_hook(Box::new(move |info| {
        // Try to extract the panic message
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic message"
        };

        let loc = if let Some(loc) = info.location() {
            format!("In: {loc}")
        } else {
            "".to_string()
        };

        let err_msg = format!("Fatal error:\n{message}\n{loc}");
        println!("{err_msg}");

        // Try to generate and print backtrace
        let backtrace = std::backtrace::Backtrace::force_capture();
        println!("Backtrace:\n{backtrace}");

        if thread_id != std::thread::current().id() {
            return;
        }

        show_message_box("The game crashed", &err_msg);
    }));

    let mut args: Vec<_> = std::env::args().collect();
    // TODO: don't rely on first arg being executable
    if !args.is_empty() {
        args.remove(0);
    }
    if let Err(err) = ddnet_main(args, sys, shared_info, app) {
        panic!("exited client with an error: {} - {}", err, err.backtrace()); // TODO: panic or graceful closing?
    }
}

#[allow(dead_code)]
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info,symphonia=warn,df::tract=error") };
    }
    env_logger::init();
    #[cfg(not(target_os = "android"))]
    main_impl(Default::default())
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: NativeApp) {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info,symphonia=warn,df::tract=error") };
    }
    if std::env::var("RUST_BACKTRACE").is_err() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "full") };
    }

    // Get the application's internal storage directory
    let app_dir = app
        .external_data_path()
        .ok_or("Failed to get the external data path")
        .unwrap()
        .to_path_buf();

    // Set the current directory to the app's directory
    std::env::set_current_dir(&app_dir).unwrap();

    use log::LevelFilter;

    android_logger::init_once(android_logger::Config::default().with_max_level(LevelFilter::Trace));
    dbg!(app_dir);
    main_impl(app)
}
