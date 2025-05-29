//! Set of functions used by [`crate::logic`]

use std::ffi::CStr;
use std::mem::size_of;
use std::ptr::null_mut;

use crate::config;
use crate::config::FOCUS_IGNORES_GEOMETRY;
use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xlib::*;

use x11::xlib::Atom;
use x11::xlib::CWBorderWidth;
use x11::xlib::CWHeight;
use x11::xlib::CWWidth;
use x11::xlib::ClientMessage;
use x11::xlib::ControlMask;
use x11::xlib::CurrentTime;
use x11::xlib::EnterWindowMask;
use x11::xlib::LockMask;
use x11::xlib::Mod1Mask;
use x11::xlib::Mod2Mask;
use x11::xlib::Mod3Mask;
use x11::xlib::Mod4Mask;
use x11::xlib::Mod5Mask;
use x11::xlib::NoEventMask;
use x11::xlib::PMaxSize;
use x11::xlib::PMinSize;
use x11::xlib::PropModeAppend;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::ShiftMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::Success;
use x11::xlib::XCheckMaskEvent;
use x11::xlib::XEvent;
use x11::xlib::XSync;
use x11::xlib::XWindowChanges;
use x11::xlib::CWX;
use x11::xlib::CWY;
use x11::xlib::XA_ATOM;
use x11::xlib::XA_WINDOW;
use x11::xlib::{PropModeReplace, XA_CARDINAL};

/// Set desktop for specified window
pub fn update_client_desktop(app: &mut Application, win: u64, desk: u64) {
    change_property(
        app.core.display,
        win,
        app.atoms.net_wm_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &desk as *const u64 as *mut u8,
        1,
    );
}

pub fn get_current_client_id(app: &mut Application) -> Option<u64> {
    let client_index = app.runtime.current_client?;

    let screen = app.runtime.screens.get(app.runtime.current_screen)?;

    let workspace = screen.workspaces.get(app.runtime.current_workspace)?;

    let client = workspace.clients.get(client_index)?;

    Some(client.window_id)
}

pub fn update_active_window(app: &mut Application) {
    let ws = &mut app.runtime;
    if let Some(index) = ws.current_client {
        let win =
            ws.screens[ws.current_screen].workspaces[ws.current_workspace].clients[index].window_id;
        change_property(
            app.core.display,
            app.core.root_win,
            app.atoms.net_active_window,
            XA_WINDOW,
            32,
            PropModeReplace,
            &win as *const u64 as *mut u8,
            1,
        );
    } else {
        //if ws.screens[ws.current_screen].workspaces[ws.current_workspace]
        //.clients
        //.is_empty()
        log!("SETTING INPUT FOCUS");
        set_input_focus(
            app.core.display,
            app.core.root_win,
            RevertToPointerRoot,
            CurrentTime,
        );
        delete_property(
            app.core.display,
            app.core.root_win,
            app.atoms.net_active_window,
        );
    }
}

/// Returns window, workspace and client indexies for client with specified id
pub fn find_window_indexes(app: &mut Application, win: u64) -> Option<(usize, usize, usize)> {
    let ws = &mut app.runtime;
    for s in 0..ws.screens.len() {
        for w in 0..ws.screens[s].workspaces.len() {
            for c in 0..ws.screens[s].workspaces[w].clients.len() {
                if ws.screens[s].workspaces[w].clients[c].window_id == win {
                    return Some((s, w, c));
                }
            }
        }
    }
    None
}

// TODO: What is going on here
pub fn get_atom_prop(app: &mut Application, win: u64, prop: Atom) -> Atom {
    let mut dummy_atom: u64 = 0;
    let mut dummy_int: i32 = 0;
    let mut dummy_long1: u64 = 0;
    let mut dummy_long2: u64 = 0;
    let mut property_return: *mut u8 = &mut 0;
    let mut atom: u64 = 0;
    if get_window_property(
        app.core.display,
        win,
        prop,
        0,
        size_of::<Atom>() as i64,
        false,
        XA_ATOM,
        &mut dummy_atom,
        &mut dummy_int,
        &mut dummy_long1,
        &mut dummy_long2,
        &mut property_return,
    ) == Success as i32
        && property_return as usize != 0
    {
        unsafe {
            atom = *(property_return as *mut Atom);
            x11::xlib::XFree(property_return as *mut libc::c_void)
        };
    }
    atom
}

/// Updates client list property of WM
/// 1. Delete present list
/// 2. For every client on every workspace on every screen add client to list
pub fn update_client_list(app: &mut Application) {
    // 1. Delete
    delete_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_client_list,
    );

    // 2. Update
    for screen in &app.runtime.screens {
        for workspace in &screen.workspaces {
            for client in &workspace.clients {
                change_property(
                    app.core.display,
                    app.core.root_win,
                    app.atoms.net_client_list,
                    XA_WINDOW,
                    32,
                    PropModeAppend,
                    &client.window_id as *const u64 as *mut u8,
                    1,
                );
            }
        }
    }
}

/// Safely sends atom to X server
pub fn send_atom(app: &mut Application, win: u64, e: x11::xlib::Atom) -> bool {
    if let Some(ps) = get_wm_protocols(app.core.display, win) {
        // If protocol not supported
        if ps
            .into_iter()
            .filter(|p| *p == e)
            .collect::<Vec<_>>()
            .is_empty()
        {
            return false;
        }
    } else {
        // If failed obtaining protocols
        return false;
    }

    // proceed to send event to window
    let ev = EEvent::ClientMessage {
        client_message_event: x11::xlib::XClientMessageEvent {
            type_: ClientMessage,
            serial: 0,
            send_event: 0,
            display: null_mut(),
            window: win,
            message_type: app.atoms.wm_protocols,
            format: 32,
            data: {
                let mut d = x11::xlib::ClientMessageData::new();
                d.set_long(0, e as i64);
                d.set_long(1, CurrentTime as i64);
                d
            },
        },
    };
    send_event(app.core.display, win, false, NoEventMask, ev)
}

pub fn update_normal_hints(app: &mut Application, c: &mut Client) {
    if let Some((sh, _)) = get_wm_normal_hints(app.core.display, c.window_id) {
        if (sh.flags & PMaxSize) != 0 {
            c.maxw = sh.max_width;
            c.maxh = sh.max_height;
        }
        if (sh.flags & PMinSize) != 0 {
            c.minw = sh.min_width;
            c.minh = sh.min_height;
        }
    }

    if c.minw != 0 && c.w < c.minw as u32 {
        c.w = c.minw as u32;
    }
    if c.minh != 0 && c.h < c.minh as u32 {
        c.h = c.minh as u32;
    }

    if c.maxw != 0 && c.maxh != 0 && c.maxw == c.minw && c.maxh == c.minh {
        c.fixed = true;
    }
}

/// Shows all windows on current workspace
pub fn show_workspace(app: &mut Application, screen: usize, workspace: usize) {
    let screen = &mut app.runtime.screens[screen];
    let workspace = &mut screen.workspaces.get_mut(workspace).unwrap();
    // Iterate over all clients
    for client in &mut workspace.clients {
        // 10. Fullscreen window if needed
        if client.fullscreen {
            move_resize_window(
                app.core.display,
                client.window_id,
                screen.x as i32,
                screen.y as i32,
                screen.width as u32,
                screen.height as u32,
            );
            set_window_border_width(app.core.display, client.window_id, 0);
            raise_window(app.core.display, client.window_id);
        } else {
            // 11. Update borders
            set_window_border_width(app.core.display, client.window_id, client.border);
            // 12. Position windows
            //move_resize_window(
            //    app.core.display,
            //    client.window_id,
            //    client.x,
            //    client.y,
            //    client.w,
            //    client.h,
            //);
            resize_client(app.core.display, client);
            if client.floating {
                raise_window(app.core.display, client.window_id);
            }
        };
        client.visible = true;
    }
}

/// Hides all windows on current workspace
pub fn hide_workspace(app: &mut Application, screen: usize, workspace: usize) {
    let window_decoration_offset = app.config.gap_width + app.config.border_size;
    let screen = &mut app.runtime.screens[screen];
    let workspace = &mut screen.workspaces.get_mut(workspace).unwrap();
    // Iterate over all clients
    for client in &mut workspace.clients {
        move_resize_window(
            app.core.display,
            client.window_id,
            -(2 * client.w as i32 + window_decoration_offset as i32),
            0,
            client.w,
            client.h,
        );
        // flip visibility state
        client.visible = false;
    }
}

/// Spawn new program by forking
///
/// 1. Fork get child PID for rules
/// 2. For child close connections from Parent
/// 3. Spawn program using sh
pub fn spawn<S: AsRef<CStr>>(app: &mut Application, args: &[S], rule: Option<(usize, usize)>) {
    unsafe {
        match nix::unistd::fork() {
            Ok(nix::unistd::ForkResult::Parent { child }) => {
                // 1. Add child to rules if specified
                if let Some((s, w)) = rule {
                    app.runtime.autostart_rules.push(AutostartRulePID {
                        pid: child.into(),
                        screen: s,
                        workspace: w,
                    })
                }
            }
            Ok(nix::unistd::ForkResult::Child) => {
                // 2. Close
                if app.core.display as *mut x11::xlib::Display as usize != 0 {
                    match nix::unistd::close(x11::xlib::XConnectionNumber(app.core.display)) {
                        Ok(_) | Err(_) => {}
                    };
                }
                // 3. Run
                let _ = nix::unistd::execvp(args[0].as_ref(), args);
            }
            Err(_) => {}
        }
    }
}

pub fn get_client_pid(app: &mut Application, win: u64) -> Option<i32> {
    let mut actual_type: Atom = 0;
    let mut actual_format: i32 = 0;
    let mut nitems: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut prop: *mut u8 = &mut 0;
    get_window_property(
        app.core.display,
        win,
        app.atoms.net_wm_pid,
        0,
        size_of::<Atom>() as i64,
        false,
        XA_CARDINAL,
        &mut actual_type,
        &mut actual_format,
        &mut nitems,
        &mut bytes_after,
        &mut prop,
    );
    if actual_type != 0 {
        unsafe { Some(*prop as i32 + *(prop.wrapping_add(1)) as i32 * 256) }
    } else {
        None
    }
}

pub fn get_client_workspace(app: &mut Application, win: u64) -> Option<(usize, usize)> {
    let client_desktop = {
        let mut actual_type: Atom = 0;
        let mut actual_format: i32 = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut prop: *mut u8 = &mut 0;
        get_window_property(
            app.core.display,
            win,
            app.atoms.net_wm_desktop,
            0,
            size_of::<Atom>() as i64,
            false,
            XA_CARDINAL,
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut prop,
        );
        if actual_type != 0 {
            unsafe { Some(*prop as u64) }
        } else {
            None
        }
    };

    match client_desktop {
        Some(d) => {
            let s = d as usize / config::NUMBER_OF_DESKTOPS;
            let w = d as usize % config::NUMBER_OF_DESKTOPS;
            if s < app.runtime.screens.len() && w < app.runtime.screens[s].workspaces.len() {
                Some((s, w))
            } else {
                None
            }
        }
        None => None,
    }
}

/// Update EWMH desktop properties
pub fn update_desktop_ewmh_info(
    app: &mut Application,
    names: Vec<String>,
    mut viewports: Vec<i64>,
) {
    // Set amount of workspaces
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_number_of_desktops,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &mut names.len() as *mut usize as *mut u8,
        1,
    );

    // Set workspaces names
    let mut bytes = vec_string_to_bytes(names);
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_desktop_names,
        app.atoms.utf8string,
        8,
        PropModeReplace,
        bytes.as_mut_ptr(),
        bytes.len() as i32,
    );

    // Set workspaces viewports
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_desktop_viewport,
        XA_CARDINAL,
        32,
        PropModeReplace,
        viewports.as_mut_ptr() as *mut u8,
        viewports.len() as i32,
    );
}

pub fn configure(dpy: &mut x11::xlib::Display, client: &mut Client) {
    let ce = x11::xlib::XConfigureEvent {
        type_: x11::xlib::ConfigureNotify,
        display: dpy,
        event: client.window_id,
        window: client.window_id,
        x: client.x,
        y: client.y,
        width: client.w as i32,
        height: client.h as i32,
        border_width: client.border as i32,
        above: 0,
        override_redirect: 0,
        serial: 0,
        send_event: 0,
    };
    send_event(
        dpy,
        client.window_id,
        false,
        StructureNotifyMask,
        EEvent::ConfigureNotify { configure: ce },
    );
}

pub fn set_urgent(app: &mut Application, win: u64, urg: bool) {
    log!("|- Setting urgency to {urg} for {win}");

    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c].urgent = urg;
    }

    unsafe {
        let wmh = x11::xlib::XGetWMHints(app.core.display, win);
        if !wmh.is_null() {
            if urg {
                (*wmh).flags |= x11::xlib::XUrgencyHint;
                set_window_border(
                    app.core.display,
                    win,
                    argb_to_int(app.config.urgent_border_color),
                );
            } else {
                (*wmh).flags &= !x11::xlib::XUrgencyHint;
                set_window_border(
                    app.core.display,
                    win,
                    argb_to_int(app.config.active_border_color),
                );
            }
            x11::xlib::XSetWMHints(app.core.display, win, wmh);
            x11::xlib::XFree(wmh as *mut libc::c_void);
        }
    };
}

pub fn resize_client(dpy: &mut x11::xlib::Display, client: &mut Client) {
    let mut wc: XWindowChanges = XWindowChanges {
        x: client.x,
        y: client.y,
        width: client.w as i32,
        height: client.h as i32,
        border_width: client.border as i32,
        sibling: 0,
        stack_mode: 0,
    };
    configure_window(
        dpy,
        client.window_id,
        (CWX | CWY | CWWidth | CWHeight | CWBorderWidth) as u32,
        &mut wc,
    );
    configure(dpy, client);
}

pub fn suppress_notify_strict(app: &mut Application) {
    unsafe {
        XSync(app.core.display, 0);
        let mut ev = XEvent { type_: 0 };
        while XCheckMaskEvent(app.core.display, EnterWindowMask, &mut ev) == 1 {
            log!("===Ignoring EnterEvent");
        }
    }
}

pub fn suppress_notify(app: &mut Application) {
    if !FOCUS_IGNORES_GEOMETRY {
        return;
    }
    suppress_notify_strict(app);
}

pub fn match_modifier(mod1: u32, mod2: u32) -> bool {
    let masked1 = mod1
        & !LockMask
        & (ShiftMask | ControlMask | Mod1Mask | Mod2Mask | Mod3Mask | Mod4Mask | Mod5Mask);
    let masked2 = mod2
        & !LockMask
        & (ShiftMask | ControlMask | Mod1Mask | Mod2Mask | Mod3Mask | Mod4Mask | Mod5Mask);
    masked1 == masked2
}
