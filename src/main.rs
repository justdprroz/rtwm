//! A window manager written in Rust with nearly same functionality of [dwm](https://dwm.suckless.org/)
//!
//! List of features supported by rwm:
//! - Multi monitor setup
//! - Workspaces aka tags
//! - Stack layout
//! - Shortcuts

pub mod actions;
pub mod arrange;
pub mod config;
pub mod events;
pub mod helper;
pub mod logic;
pub mod manage;
pub mod mouse;
pub mod setup;
pub mod structs;
pub mod utils;
pub mod wrapper;

use std::path::Path;

use events::*;
use helper::spawn;
use libc::LC_CTYPE;
use setup::setup;
use structs::Application;
use wrapper::sys::no_zombies;
use wrapper::sys::set_locale;
use wrapper::xlib::next_event;
use wrapper::xlib::EEvent;

fn run(app: &mut Application) {
    log!("|===== run =====");
    while app.core.running {
        let event = next_event(app.core.display);
        match event {
            EEvent::KeyPress { key } => key_press(app, key),
            EEvent::KeyRelease { key: _ } => {}
            EEvent::MapRequest { map_request_event } => map_request(app, map_request_event),
            EEvent::EnterNotify { crossing } => enter_notify(app, crossing),
            EEvent::LeaveNotify { crossing: _ } => {}
            EEvent::DestroyNotify { destroy_window } => destroy_notify(app, destroy_window),
            EEvent::UnmapNotify { unmap } => unmap_notify(app, unmap),
            EEvent::MotionNotify { button, motion } => motion_notify(app, button, motion),
            EEvent::ButtonPress { button, motion } => button_press(app, button, motion),
            EEvent::ButtonRelease { button, motion } => button_release(app, button, motion),
            EEvent::PropertyNotify { property } => property_notify(app, property),
            EEvent::ConfigureNotify { configure } => configure_notify(app, configure),
            EEvent::ClientMessage {
                client_message_event,
            } => client_message(app, client_message_event),
            EEvent::ConfigureRequest {
                configure_request_event,
            } => configure_request(app, configure_request_event),
            EEvent::Unmanaged { type_: _, name } => {
                log!("|- Event `{}` is not currently managed", name);
            }
        };
    }
}

fn main() {
    set_locale(LC_CTYPE, "");
    no_zombies();
    let mut app: Application = setup();
    if !Path::new("/tmp/rtwmrunning").exists() {
        for rule in app.config.autostart.clone() {
            spawn(&mut app, &rule.cmd, rule.rule);
        }
    }
    setup::scan(&mut app);
    run(&mut app);
}
