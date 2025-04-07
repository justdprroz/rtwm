//! Functions related to adding/removing windows to/from WM runtime

use x11::xlib::CWBorderWidth;
use x11::xlib::EnterWindowMask;
use x11::xlib::FocusChangeMask;
use x11::xlib::PropModeAppend;
use x11::xlib::PropModeReplace;
use x11::xlib::PropertyChangeMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::XWindowAttributes;
use x11::xlib::XA_WINDOW;

use crate::arrange::*;
use crate::config;
use crate::helper::*;
use crate::logic::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xlib::*;

/// Add client to runtime
///
/// 1. Get window attributes
///     * Exit if no proper attributes or if `override_redirect` is set
/// 2. Check if already managed
///     * Return
/// 3. Check if window is dock and setup
///     * Call [`attach_dock`]
///     * Map window
///     * Return
/// 4. Create client and setup essential fields
/// 5. Get properties
/// 6. Update hints by running [`update_normal_hints`]
/// 7. Set flags
/// 8. Set input mask for events
/// 9. set previously active client border to normal
/// 10. Get desktop info left from previous wm session
/// 11. Find where to place window
/// 12. Add to stack
/// 13. Update client list & desktops
/// 14. Configure window
/// 15. Arrange clients
/// 16. Map window
pub fn manage_client(app: &mut Application, win: u64, scan: bool) {
    // 1. Get attributes
    let wa;
    if let Some(a) = get_window_attributes(app.core.display, win) {
        if a.override_redirect == 0 {
            wa = a;
        } else {
            return;
        };
    } else {
        return;
    }

    // 2. Check managed
    if find_window_indexes(app, win).is_some() {
        return;
    }

    // 3. Check if dock
    if get_atom_prop(app, win, app.atoms.net_wm_window_type) == app.atoms.net_wm_window_type_dock {
        attach_dock(app, &wa, win);
        map_window(app.core.display, win);
        select_input(
            app.core.display,
            win,
            StructureNotifyMask | SubstructureNotifyMask,
        );
        return;
    }

    // 4. Create client
    let mut c: Client = Client::default();
    c.window_id = win;
    c.w = wa.width as u32;
    c.h = wa.height as u32;
    c.ow = c.w;
    c.oh = c.h;
    c.x = wa.x
        + app.runtime.screens[app.runtime.current_screen]
            .bar_offsets
            .left as i32;
    c.y = wa.y
        + app.runtime.screens[app.runtime.current_screen]
            .bar_offsets
            .up as i32;
    c.visible = true;

    println!("{:#?}", c);

    // 5. Properties
    let state = get_atom_prop(app, win, app.atoms.net_wm_state);
    let wtype = get_atom_prop(app, win, app.atoms.net_wm_window_type);

    let (instance, class) = {
        let mut ch = ClassHint::default();
        get_class_hint(app.core.display, win, &mut ch);
        (ch.res_name, ch.res_class)
    };

    log!("Found: {:?}, {:?}", instance, class);

    // 10. Get window workspace
    let ((client_screen, client_workspace), trans) = get_window_placement(app, win, scan);

    // 6. Update hints
    update_normal_hints(app, &mut c);

    // 7. Set flags
    if state == app.atoms.net_wm_fullscreen {
        c.floating = true;
        c.fullscreen = true;
    }
    if wtype == app.atoms.net_wm_window_type_dialog {
        c.floating = true;
    }

    if !c.floating {
        c.floating = c.fixed || trans != 0;
    }

    c.border = if c.floating {
        app.config.border_size as u32
    } else {
        0
    };

    // 8. Set input mask for events
    select_input(
        app.core.display,
        win,
        EnterWindowMask | FocusChangeMask | PropertyChangeMask | StructureNotifyMask,
    );

    // 9. Unfocus current windows
    if let Some(cw) = get_current_client_id(app) {
        unfocus(app, cw);
    }

    let screen = &app.runtime.screens[client_screen];
    if c.floating {
        if c.x > screen.width as i32 {
            c.x %= screen.x as i32;
        }
        if c.y > screen.height as i32 {
            c.y %= screen.y as i32;
        }
    }

    if c.x < screen.x as i32 {
        c.x += screen.x as i32;
    }

    if c.y < screen.y as i32 {
        c.y += screen.y as i32;
    }

    let workspace = &mut app.runtime.screens[client_screen].workspaces[client_workspace];

    // 12. Add window to stack
    workspace.current_client = Some(workspace.clients.len());
    app.runtime.current_client = workspace.current_client;
    workspace.clients.push(c);

    // 13. Update client list & window desktop
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_client_list,
        XA_WINDOW,
        32,
        PropModeAppend,
        &win as *const u64 as *mut u8,
        1,
    );
    let cur_workspace: usize = client_workspace + client_screen * config::NUMBER_OF_DESKTOPS;
    update_client_desktop(app, win, cur_workspace as u64);

    // 14. Configure window
    let mut wc = x11::xlib::XWindowChanges {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        border_width: app.config.border_size as i32,
        sibling: 0,
        stack_mode: 0,
    };
    configure_window(app.core.display, win, CWBorderWidth as u32, &mut wc);

    let data: [i64; 2] = [1, 0];
    change_property(
        app.core.display,
        win,
        app.atoms.wm_state,
        app.atoms.wm_state,
        32,
        PropModeReplace,
        &data as *const [i64; 2] as *mut u8,
        2,
    );

    // 15. Arrange current workspace
    arrange_workspace(app, client_screen, client_workspace);
    if client_workspace == app.runtime.screens[client_screen].current_workspace {
        show_workspace(app, client_screen, client_workspace);
    } else {
        hide_workspace(app, client_screen, client_workspace);
        set_urgent(app, win, true);
    }

    // 16. Tag window as mapped
    map_window(app.core.display, win);

    if client_screen == app.runtime.current_screen
        && client_workspace == app.runtime.current_workspace
    {
        focus(app, win);
    }
}

pub fn update_docks(app: &mut Application) {
    for screen in &mut app.runtime.screens {
        screen.bar_offsets = BarOffsets::default();
    }

    for bar in &app.runtime.bars {
        let dx = bar.x;
        let dy = bar.y;
        let dw = bar.w;
        let dh = bar.h;
        for screen in &mut app.runtime.screens {
            if dx >= screen.x
                && dx < (screen.x + screen.width)
                && dy >= screen.y
                && dy < (screen.y + screen.height)
            {
                let mut ba = screen.bar_offsets;
                // Found corresponding screen
                if dw > dh {
                    // dock is horizontal
                    if dy == screen.y {
                        // dock is on the top
                        ba.up = dh;
                    } else {
                        // dock is on the bottom
                        ba.down = dh;
                    }
                } else {
                    // dock is vertical
                    if dx == screen.x {
                        // dock is on the left
                        ba.left = dw;
                    } else {
                        // dock is on the right
                        ba.right = dw;
                    }
                }
                screen.bar_offsets = ba;
                break;
            }
        }
    }
    arrange_all(app);
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
}

pub fn attach_dock(app: &mut Application, wa: &XWindowAttributes, win: u64) {
    let dx = wa.x as i64;
    let dy = wa.y as i64;
    let dw = wa.width as usize;
    let dh = wa.height as usize;
    app.runtime.bars.push(Bar {
        window_id: win,
        x: dx,
        y: dy,
        w: dw,
        h: dh,
    });
    update_docks(app);
}

pub fn detach_dock(app: &mut Application, win: u64) {
    app.runtime.bars.retain(|b| b.window_id != win);
    update_docks(app);
}

/// Removes window from runtime
pub fn unmanage_window(app: &mut Application, win: u64) {
    // Find trackers for window
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        // Remove unmapped client
        log!("   |- Found window {} at indexes {}, {}, {}", win, s, w, c);
        // delete_property(app.core.display, win, app.atoms.net_wm_desktop);
        app.runtime.screens[s].workspaces[w].clients.remove(c);
        shift_current_client(app, Some(s), Some(w));

        grab_server(app.core.display);
        select_input(app.core.display, win, x11::xlib::NoEventMask);
        ungrab_button(
            app.core.display,
            x11::xlib::AnyButton as u32,
            x11::xlib::AnyModifier,
            win,
        );
        println!("===== Set state withdrawn");
        let data: [i64; 2] = [0, 0];
        change_property(
            app.core.display,
            win,
            app.atoms.wm_state,
            app.atoms.wm_state,
            32,
            PropModeReplace,
            &data as *const [i64; 2] as *mut u8,
            2,
        );
        println!("===== Ungrab server");
        ungrab_server(app.core.display);

        // Update layout
        arrange_workspace(app, s, w);
        if w == app.runtime.screens[s].current_workspace {
            show_workspace(app, s, w);
        }
        update_client_list(app);
    } else if app.runtime.bars.iter().any(|b| b.window_id == win) {
        detach_dock(app, win);
    }
}
