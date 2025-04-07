//! Functions ran for events

use crate::actions::*;
use crate::arrange::*;
use crate::helper::*;
use crate::logic::*;
use crate::manage::*;
use crate::mouse::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xlib::*;

use x11::xlib::Button1;
use x11::xlib::Button3;
use x11::xlib::CWBorderWidth;
use x11::xlib::CWHeight;
use x11::xlib::CWWidth;
use x11::xlib::NotifyInferior;
use x11::xlib::NotifyNormal;
use x11::xlib::PropModeReplace;
use x11::xlib::XButtonEvent;
use x11::xlib::XClientMessageEvent;
use x11::xlib::XConfigureEvent;
use x11::xlib::XConfigureRequestEvent;
use x11::xlib::XCrossingEvent;
use x11::xlib::XDestroyWindowEvent;
use x11::xlib::XKeyEvent;
use x11::xlib::XMapRequestEvent;
use x11::xlib::XMotionEvent;
use x11::xlib::XPropertyEvent;
use x11::xlib::XSync;
use x11::xlib::XUnmapEvent;
use x11::xlib::XWindowChanges;
use x11::xlib::CWX;
use x11::xlib::CWY;
use x11::xlib::XA_ATOM;

pub fn key_press(app: &mut Application, key_event: XKeyEvent) {
    // Iterate over key actions matching current key input
    for action in app.config.key_actions.clone() {
        if key_event.keycode == keysym_to_keycode(app.core.display, action.keysym)
            && key_event.state == action.modifier
        {
            // Match action result and run related function
            match &action.result {
                ActionResult::KillClient => {
                    kill_client(app);
                }
                ActionResult::Spawn(cmd) => {
                    spawn(app, &cmd.clone(), None);
                }
                ActionResult::MoveToScreen(d) => {
                    move_to_screen(app, *d);
                }
                ActionResult::FocusOnScreen(d) => {
                    focus_on_screen(app, *d);
                }
                ActionResult::MoveToWorkspace(n) => {
                    move_to_workspace(app, *n);
                }
                ActionResult::FocusOnWorkspace(n) => {
                    focus_on_workspace(app, *n, true);
                }
                ActionResult::Quit => {
                    app.core.running = false;
                }
                ActionResult::UpdateMasterCapacity(i) => {
                    update_master_capacity(app, *i);
                }
                ActionResult::UpdateMasterWidth(w) => {
                    update_master_width(app, *w);
                }
                ActionResult::DumpInfo => {
                    log!("{:#?}", &app.runtime);
                }
                ActionResult::ToggleFloat => {
                    toggle_float(app);
                }
                ActionResult::CycleStack(d) => {
                    cycle_stack(app, *d);
                }
                ActionResult::PopPushStack => {
                    pop_push_stack(app, true);
                }
                ActionResult::RotateStack => {
                    pop_push_stack(app, false);
                }
            }
        }
    }
}

pub fn map_request(app: &mut Application, map_request: XMapRequestEvent) {
    let ew: u64 = map_request.window;
    log!("|- Map Request From Window: {ew}");
    manage_client(app, ew, false);
}

pub fn enter_notify(app: &mut Application, crossing_event: XCrossingEvent) {
    let ew: u64 = crossing_event.window;
    log!("|- Crossed Window `{}` ({})", get_client_name(app, ew), ew);
    if ew != app.core.root_win {
        if crossing_event.mode != NotifyNormal || crossing_event.detail == NotifyInferior {
            return;
        }
        if find_window_indexes(app, ew).is_none() {
            return;
        }
        if let Some(cw) = get_current_client_id(app) {
            unfocus(app, cw);
        }
        log!("   |- Setting focus to window");
        focus(app, ew);
    } else {
        //log!("Hehe, fuck you");
        //if let Some(cw) = get_current_client_id(app) {
        //    unfocus(app, cw);
        //}
        //let ws = &mut app.runtime;
        //ws.current_client = None;
        //ws.screens[ws.current_screen].workspaces[ws.current_workspace].current_client = ws.current_client;
        //update_active_window(app);
    }
}

pub fn destroy_notify(app: &mut Application, destroy_notify_event: XDestroyWindowEvent) {
    let ew: u64 = destroy_notify_event.window;
    log!("|- `{}` ({}) destroyed", get_client_name(app, ew), ew);
    unmanage_window(app, ew);
}

pub fn unmap_notify(app: &mut Application, unmap_event: XUnmapEvent) {
    let ew: u64 = unmap_event.window;
    log!("|- `{}` ({}) unmapped", get_client_name(app, ew), ew);
    if find_window_indexes(app, ew).is_some() {
        if unmap_event.send_event == 1 {
            let data: [i64; 2] = [0, 0];
            change_property(
                app.core.display,
                ew,
                app.atoms.wm_state,
                app.atoms.wm_state,
                32,
                PropModeReplace,
                &data as *const [i64; 2] as *mut u8,
                2,
            );
        } else {
            unmanage_window(app, ew);
        }
    }
}

pub fn motion_notify(
    app: &mut Application,
    _button_event: XButtonEvent,
    motion_event: XMotionEvent,
) {
    log!("|- `Motion` detected");
    if app.runtime.mouse_state.button == Button1 {
        move_mouse(app, motion_event);
    }
    if app.runtime.mouse_state.button == Button3 {
        resize_mouse(app, motion_event);
    }
    if motion_event.window == app.core.root_win {
        screen_mouse(app, motion_event);
    }
}

pub fn property_notify(app: &mut Application, property_event: XPropertyEvent) {
    if property_event.window != app.core.root_win {
        update_client_name(app, property_event.window);
    }
}

pub fn configure_notify(app: &mut Application, configure_event: XConfigureEvent) {
    if configure_event.window == app.core.root_win {
        log!("|- Got `ConfigureNotify` for `root window` -> Changing monitor layout");
        update_screens(app);
        update_desktops(app);
    } else if let Some((s, w, c)) = find_window_indexes(app, configure_event.window) {
        let client = &app.runtime.screens[s].workspaces[w].clients[c];
        log!(
            "|- Got `ConfigureNotify` of {:?} for `{}`",
            configure_event.event,
            client.window_name
        );
    } else {
        log!(
            "|- Got `ConfigureNotify` od {:#?} from {}",
            configure_event,
            configure_event.window
        );
    }
}

pub fn client_message(app: &mut Application, client_event: XClientMessageEvent) {
    log!("|- Got `Client Message`");
    if let Some((client_screen_index, client_workspace_index, client_index)) =
        find_window_indexes(app, client_event.window)
    {
        let client_screen = &mut app.runtime.screens[client_screen_index];
        let cc = &mut client_screen.workspaces[client_workspace_index].clients[client_index];
        log!(
            "   |- Of type `{}` From: `{}`",
            get_atom_name(app.core.display, client_event.message_type),
            cc.window_name
        );
        if client_event.message_type == app.atoms.net_wm_state {
            if client_event.data.get_long(1) as u64 == app.atoms.net_wm_fullscreen
                || client_event.data.get_long(2) as u64 == app.atoms.net_wm_fullscreen
            {
                let sf = client_event.data.get_long(0) == 1
                    || client_event.data.get_long(0) == 2 && cc.fullscreen;
                if sf && !cc.fullscreen {
                    change_property(
                        app.core.display,
                        client_event.window,
                        app.atoms.net_wm_state,
                        XA_ATOM,
                        32,
                        PropModeReplace,
                        &mut app.atoms.net_wm_fullscreen as *mut u64 as *mut u8,
                        1,
                    );
                    cc.ow = cc.w;
                    cc.oh = cc.h;
                    cc.w = client_screen.width as u32;
                    cc.h = client_screen.height as u32;
                    cc.fullscreen = true;
                } else if !sf && cc.fullscreen {
                    change_property(
                        app.core.display,
                        client_event.window,
                        app.atoms.net_wm_state,
                        XA_ATOM,
                        32,
                        PropModeReplace,
                        std::ptr::null_mut::<u8>(),
                        0,
                    );
                    cc.w = cc.ow;
                    cc.h = cc.oh;
                    cc.fullscreen = false;
                }
                arrange_visible(app);
                show_workspace(
                    app,
                    app.runtime.current_screen,
                    app.runtime.current_workspace,
                );
            } else {
                log!("      |- Unsupported `state`");
            }
        } else if client_event.message_type == app.atoms.net_active_window {
            log!("=== GOT URGENCY TRIGGER");
            if client_workspace_index != client_screen.current_workspace && !cc.urgent {
                log!("=== SET URGENCY FLAG");
                let win = cc.window_id;
                set_urgent(app, win, true);
            }
        }
    } else if client_event.message_type == app.atoms.net_current_desktop {
        focus_on_workspace(app, client_event.data.get_long(0) as u64, false);
    } else {
        log!(
            "   |- Of type `{}`",
            get_atom_name(app.core.display, client_event.message_type)
        );
    }
}

pub fn configure_request(app: &mut Application, conf_req_event: XConfigureRequestEvent) {
    log!(
        "|- Got `ConfigureRequest` for `{}` ({}): {:#?}",
        get_client_name(app, conf_req_event.window),
        conf_req_event.window,
        conf_req_event
    );
    if let Some((s, w, c)) = find_window_indexes(app, conf_req_event.window) {
        log!("  |- Window found!");
        let sx = app.runtime.screens[s].x as i32;
        let sy = app.runtime.screens[s].y as i32;

        let sw = app.runtime.screens[s].width as i32;
        let sh = app.runtime.screens[s].height as i32;
        let _ba = app.runtime.screens[s].bar_offsets;
        let client = &mut app.runtime.screens[s].workspaces[w].clients[c];
        // let mut resized = false;

        if (conf_req_event.value_mask & CWBorderWidth as u64) != 0 {
            client.border = conf_req_event.border_width as u32;
        } else if client.floating {
            if (conf_req_event.value_mask & CWWidth as u64) != 0 {
                log!("CWWidth");
                client.ow = client.w;
                client.w = conf_req_event.width as u32;
                // resized = true;
            }
            if (conf_req_event.value_mask & CWHeight as u64) != 0 {
                log!("CWHeight");
                client.oh = client.h;
                client.h = conf_req_event.height as u32;
                // resized = true;
            }
            if (conf_req_event.value_mask & CWX as u64) != 0 {
                log!("CWX");
                client.ox = client.x;
                client.x = sx + conf_req_event.x;
                // resized = true;
            }
            if (conf_req_event.value_mask & CWY as u64) != 0 {
                log!("CWY");
                client.oy = client.y;
                client.y = sy + conf_req_event.y;
                // resized = true;
            }
            log!("Configuring window '{}'", conf_req_event.window);
            log!(
                "cx {}, cy {}, cw {}, ch {}",
                client.x,
                client.y,
                client.w,
                client.h
            );
            log!("sx {sx}, sy {sy}, sw {sw}, sh {sh}");

            if (client.x + client.w as i32) > sx + sw && client.floating {
                // log!("cx {}, cw {}, sx {}, sw {}", client.x, client.y, sx, sw);
                client.x = sx + (sw / 2 - client.w as i32 / 2);
            }
            if (client.y + client.h as i32) > sy + sh && client.floating {
                client.y = sy + (sh / 2 - client.h as i32 / 2);
            }

            // if resized {

            // client.y = (sh - (ba.up as i32) - (client.h as i32)) / 2 + sy;

            if (conf_req_event.value_mask & (CWX | CWY) as u64) != 0
                && ((conf_req_event.value_mask & (CWWidth | CWHeight) as u64) == 0)
            {
                configure(app.core.display, client);
            }

            if client.visible {
                move_resize_window(
                    app.core.display,
                    client.window_id,
                    client.x,
                    client.y,
                    client.w,
                    client.h,
                );
            }
            // }
        } else {
            configure(app.core.display, client);
        }
    } else {
        log!("  |- No windows found");
        let mut wc = XWindowChanges {
            x: conf_req_event.x,
            y: conf_req_event.y,
            width: conf_req_event.width,
            height: conf_req_event.height,
            border_width: conf_req_event.border_width,
            sibling: conf_req_event.above,
            stack_mode: conf_req_event.detail,
        };
        configure_window(
            app.core.display,
            conf_req_event.window,
            conf_req_event.value_mask as u32,
            &mut wc,
        );
    }
    unsafe {
        XSync(app.core.display, 0);
    }
}

pub fn button_press(
    app: &mut Application,
    button_event: XButtonEvent,
    _motion_event: XMotionEvent,
) {
    if let Some((s, w, c)) = find_window_indexes(app, button_event.window) {
        let cc = &app.runtime.screens[s].workspaces[w].clients[c];
        if cc.floating {
            app.runtime.mouse_state = MouseState {
                win: button_event.window,
                button: button_event.button,
                pos: (button_event.x_root as i64, button_event.y_root as i64),
            };
            println!("{:?}", app.runtime.mouse_state.pos);
            if button_event.button == Button3 {
                warp_pointer_win(
                    app.core.display,
                    button_event.window,
                    cc.w as i32,
                    cc.h as i32,
                );
                app.runtime.mouse_state.pos = (
                    (button_event.x_root - button_event.x + cc.w as i32) as i64,
                    (button_event.y_root - button_event.y + cc.h as i32) as i64,
                );
            }
        }
    }
}
pub fn button_release(
    app: &mut Application,
    _button_event: XButtonEvent,
    _motion_event: XMotionEvent,
) {
    app.runtime.mouse_state = MouseState {
        win: 0,
        button: 0,
        pos: (0, 0),
    };
}
