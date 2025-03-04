//! Main windows manager logic processed as response to events

use std::process::exit;

use crate::config;
use crate::config::NUMBER_OF_DESKTOPS;
use crate::helper::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xinerama::xinerama_query_screens;
use crate::wrapper::xlib::*;

use x11::xinerama::XineramaScreenInfo;
use x11::xlib::AnyButton;
use x11::xlib::AnyModifier;
use x11::xlib::Button1;
use x11::xlib::Button3;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyAll;
use x11::xlib::Mod4Mask as ModKey;
use x11::xlib::PropModeReplace;
use x11::xlib::RevertToPointerRoot;
use x11::xlib::XA_CARDINAL;
/// Shifts current client tracker after destroying clients
pub fn shift_current_client(
    app: &mut Application,
    screen: Option<usize>,
    workspace: Option<usize>,
) {
    let screen = match screen {
        Some(index) => index,
        None => app.runtime.current_screen,
    };

    let workspace = match workspace {
        Some(index) => index,
        None => app.runtime.current_workspace,
    };

    let ws = &mut app.runtime;
    // Find next client
    ws.screens[screen].workspaces[workspace].current_client = {
        // Get reference to windows stack
        let clients = &ws.screens[screen].workspaces[workspace].clients;
        if clients.is_empty() {
            // None if no windows
            None
        } else {
            // Get old client index
            if let Some(cc) = ws.screens[screen].workspaces[workspace].current_client {
                // If selected client was not last do nothing
                if cc < clients.len() {
                    Some(cc)
                } else {
                    // Else set it to being last
                    Some(clients.len() - 1)
                }
            } else {
                None
            }
        }
    };
    // Only do global changes if current_workspace is equal to workspace we shifting!
    if workspace == ws.current_workspace {
        // update secondary tracker
        ws.current_client = ws.screens[screen].workspaces[workspace].current_client;
        if let Some(index) = ws.current_client {
            let win = ws.screens[screen].workspaces[workspace].clients[index].window_id;
            set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
        }
        update_active_window(app);
    }
}

/// Kill active window
/// 1. Check if there is focused client
/// 2. Ask client to close
/// 3. Forcefully close client
pub fn kill_client(app: &mut Application) {
    // 1. Check
    if let Some(index) = app.runtime.current_client {
        let id = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        // 2. Ask
        if !send_atom(app, id, app.atoms.wm_delete) {
            // 3. Close
            grab_server(app.core.display);
            set_close_down_mode(app.core.display, DestroyAll);
            x_kill_client(app.core.display, id);
            ungrab_server(app.core.display);
        };
    };
}

pub fn move_to_screen(app: &mut Application, d: ScreenSwitching) {
    // Check if window is selected
    if let Some(index) = app.runtime.current_client {
        // Update index depending on supplied direction
        let new_screen_index = match d {
            ScreenSwitching::Next => (app.runtime.current_screen + 1) % app.runtime.screens.len(),
            ScreenSwitching::Previous => {
                (app.runtime.current_screen + app.runtime.screens.len() - 1)
                    % app.runtime.screens.len()
            }
        };

        // Pop client
        let mut client = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients
            .remove(index);
        set_window_border(
            app.core.display,
            client.window_id,
            argb_to_int(app.config.normal_border_color),
        );

        // Update workspace
        let new_workspace: usize = app.runtime.screens[new_screen_index].current_workspace
            + new_screen_index * config::NUMBER_OF_DESKTOPS;
        update_client_desktop(app, client.window_id, new_workspace as u64);

        // For floating windows change positions
        if client.floating {
            let cur_screen = &app.runtime.screens[app.runtime.current_screen];
            let rel_x = client.x - cur_screen.x as i32;
            let rel_y = client.y - cur_screen.y as i32;

            let new_screen = &app.runtime.screens[new_screen_index];
            client.x = new_screen.x as i32 + rel_x;
            client.y = new_screen.y as i32 + rel_y;
        }

        // Update client tracker on current screen
        shift_current_client(app, None, None);
        // Get workspace tracker(borrow checker is really mad at me)
        let nw = app.runtime.screens[new_screen_index].current_workspace;
        // Add window to stack of another display
        app.runtime.screens[new_screen_index].workspaces[nw]
            .clients
            .push(client);

        // Arrange all monitors
        arrange_current(app);
        show_workspace(
            app,
            new_screen_index,
            app.runtime.screens[new_screen_index].current_workspace,
        );
        show_workspace(
            app,
            app.runtime.current_screen,
            app.runtime.screens[app.runtime.current_screen].current_workspace,
        );
    }
}

pub fn focus_on_screen_index(app: &mut Application, n: usize) {
    log!("Focusing on screen");
    if let Some(cw) = get_current_client_id(app) {
        log!("unfocusing {}", cw);
        set_window_border(
            app.core.display,
            cw,
            argb_to_int(app.config.normal_border_color),
        );
        unfocus(app, cw);
    }
    // Change trackers
    app.runtime.current_screen = n;
    app.runtime.current_workspace =
        app.runtime.screens[app.runtime.current_screen].current_workspace;
    app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace]
        .current_client;
    if let Some(index) = app.runtime.current_client {
        let win = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
    }
    update_active_window(app);
    if let Some(cw) = get_current_client_id(app) {
        set_window_border(
            app.core.display,
            cw,
            argb_to_int(app.config.active_border_color),
        );
    }
    let w: u64 = n as u64 * config::NUMBER_OF_DESKTOPS as u64
        + app.runtime.screens[n].current_workspace as u64;
    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_current_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &w as *const u64 as *mut u64 as *mut u8,
        1,
    );
}

pub fn focus_on_screen(app: &mut Application, d: ScreenSwitching) {
    // Get current screen
    let mut cs = app.runtime.current_screen;
    // Update it
    cs = match d {
        ScreenSwitching::Next => (cs + 1) % app.runtime.screens.len(),
        ScreenSwitching::Previous => {
            (cs + app.runtime.screens.len() - 1) % app.runtime.screens.len()
        }
    };
    focus_on_screen_index(app, cs);
}

pub fn move_to_workspace(app: &mut Application, n: u64) {
    log!("   |- Got `MoveToWorkspace` Action ");
    // Check if moving to another workspace
    if let Some(index) = app.runtime.current_client {
        if n as usize != app.runtime.current_workspace {
            // Check if any client is selected
            // Pop current client
            let mut cc = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients
                .remove(index);
            set_window_border(
                app.core.display,
                cc.window_id,
                argb_to_int(app.config.normal_border_color),
            );
            let cur_workspace: usize =
                n as usize + app.runtime.current_screen * config::NUMBER_OF_DESKTOPS;

            update_client_desktop(app, cc.window_id, cur_workspace as u64);

            // Update current workspace layout
            arrange_current(app);
            show_workspace(
                app,
                app.runtime.current_screen,
                app.runtime.current_workspace,
            );
            // Move window out of view
            move_resize_window(
                app.core.display,
                cc.window_id,
                -((cc.w + cc.border * 2) as i32),
                -((cc.h + cc.border * 2) as i32),
                cc.w,
                cc.h,
            );
            cc.visible = !cc.visible;
            // Update tracker
            shift_current_client(app, None, None);
            // Add client to choosen workspace (will be arranged later)
            app.runtime.screens[app.runtime.current_screen].workspaces[n as usize]
                .clients
                .push(cc);
            arrange_workspace(app, app.runtime.current_screen, n as usize);
        } else {
            let cc = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients
                .remove(index);
            app.runtime.screens[app.runtime.current_screen].workspaces[n as usize]
                .clients
                .push(cc);
            arrange_current(app);
            show_workspace(
                app,
                app.runtime.current_screen,
                app.runtime.current_workspace,
            );
            app.runtime.screens[app.runtime.current_screen].workspaces[n as usize].current_client =
                Some(0);
            app.runtime.current_client = Some(0);
        }
    }
}

pub fn focus_on_workspace(app: &mut Application, n: u64, r: bool) {
    let n = if !r {
        focus_on_screen_index(app, n as usize / config::NUMBER_OF_DESKTOPS);
        n % config::NUMBER_OF_DESKTOPS as u64
    } else {
        n
    };
    log!("   |- Got `FocusOnWorkspace` Action");
    // Check is focusing on another workspace
    if n as usize != app.runtime.current_workspace {
        let pw = app.runtime.current_workspace;
        // unfocus current win
        if let Some(cw) = get_current_client_id(app) {
            set_window_border(
                app.core.display,
                cw,
                argb_to_int(app.config.normal_border_color),
            );
            unfocus(app, cw);
        }
        // Update workspace index
        app.runtime.current_workspace = n as usize;
        app.runtime.screens[app.runtime.current_screen].current_workspace = n as usize;

        let w = n + app.runtime.current_screen as u64 * config::NUMBER_OF_DESKTOPS as u64;

        change_property(
            app.core.display,
            app.core.root_win,
            app.atoms.net_current_desktop,
            XA_CARDINAL,
            32,
            PropModeReplace,
            &w as *const u64 as *mut u64 as *mut u8,
            1,
        );

        // Update current client
        app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .current_client;
        if let Some(cw) = get_current_client_id(app) {
            focus(app, cw);
        }
        // Show current client
        show_workspace(
            app,
            app.runtime.current_screen,
            app.runtime.current_workspace,
        );
        // Hide current workspace
        hide_workspace(app, app.runtime.current_screen, pw);
    }
}

pub fn update_master_width(app: &mut Application, w: f64) {
    // Update master width
    let mw = &mut app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace]
        .master_width;
    if f64::abs(w) < *mw + w && *mw + w < 1.0 {
        *mw += w;
    }
    // Rearrange windows
    arrange_current(app);
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
}

pub fn update_master_capacity(app: &mut Application, i: i64) {
    // Change master size
    app.runtime.screens[app.runtime.current_screen].workspaces[app.runtime.current_workspace]
        .master_capacity += i;
    // Rearrange windows
    arrange_current(app);
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
}

pub fn toggle_float(app: &mut Application) {
    if let Some(c) = app.runtime.current_client {
        let client = &mut app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[c];
        client.floating = !client.floating;

        client.border = if client.floating {
            app.config.border_size as u32
        } else {
            0
        };

        arrange_current(app);
        show_workspace(
            app,
            app.runtime.current_screen,
            app.runtime.current_workspace,
        );
    }
}

/// Get name from x server for specified window and undate it in struct
/// 1. Get name property
/// 2. Set window name if window is managed
pub fn update_client_name(app: &mut Application, win: u64) {
    // 1. Get
    let name = match get_text_property(app.core.display, win, app.atoms.net_wm_name) {
        Some(name) => name,
        None => "_".to_string(),
    };

    // 2. Set
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c].window_name = name;
    }
}

/// Returns name of specified client
///
/// 1. If client is managed return its name
/// 2. If not managed return "Unmanaged Window"
pub fn get_client_name(app: &mut Application, win: u64) -> String {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        app.runtime.screens[s].workspaces[w].clients[c]
            .window_name
            .clone()
    } else {
        "Unmanaged Window".to_string()
    }
}

pub fn focus(app: &mut Application, win: u64) {
    set_urgent(app, win, false);
    set_window_border(
        app.core.display,
        win,
        argb_to_int(app.config.active_border_color),
    );
    update_trackers(app, win);
    update_active_window(app);
    set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
    grab_button(app.core.display, win, Button1, ModKey);
    grab_button(app.core.display, win, Button3, ModKey);

    let w = app.runtime.current_workspace + app.runtime.current_screen * NUMBER_OF_DESKTOPS;

    change_property(
        app.core.display,
        app.core.root_win,
        app.atoms.net_current_desktop,
        XA_CARDINAL,
        32,
        PropModeReplace,
        &w as *const usize as *mut usize as *mut u8,
        1,
    );
}

pub fn unfocus(app: &mut Application, win: u64) {
    set_window_border(
        app.core.display,
        win,
        argb_to_int(app.config.normal_border_color),
    );
    ungrab_button(app.core.display, AnyButton as u32, AnyModifier, win);
}

pub fn update_trackers(app: &mut Application, win: u64) {
    if let Some((s, w, c)) = find_window_indexes(app, win) {
        let ws = &mut app.runtime;
        ws.current_screen = s;
        ws.current_workspace = w;
        ws.screens[s].current_workspace = w;
        ws.current_client = Some(c);
        ws.screens[s].workspaces[w].current_client = Some(c);
    };
}

/// Arrange windows of current workspace in specified layout
/// 1. Iterate over all screens
/// 2. Arrange current workspace
pub fn arrange_current(app: &mut Application) {
    log!("   |- Arranging...");
    // 1. Iterate over all screens
    let screens_amount = app.runtime.screens.len();
    for index in 0..screens_amount {
        let current_workspace = app.runtime.screens[index].current_workspace;
        arrange_workspace(app, index, current_workspace);
    }
}

/// Arrange all clients
/// 1. Iterate over all screens
/// 2. Iterate over all workspaces
/// 3. Arrange it
pub fn arrange_all(app: &mut Application) {
    let screens_amount = app.runtime.screens.len();
    for screen in 0..screens_amount {
        let workspaces_amount = app.runtime.screens[screen].workspaces.len();
        for workspace in 0..workspaces_amount {
            arrange_workspace(app, screen, workspace);
        }
    }
}

/// Update screens
///
/// 1. Get screens from xinerama
/// 2. Add more screens if amount of new screens is larger than amount of existing screens
/// 3. Init newly created screens
/// 4. Move everything from exceeding screens and delete them
pub fn update_screens(app: &mut Application) {
    // 1. Get screens
    let n = app.runtime.screens.len();
    let screens = match xinerama_query_screens(app.core.display) {
        Some(s) => s,
        None => {
            eprintln!("Running without xinerama is not supported");
            exit(1);
        }
    };

    let screens = {
        let mut tmp: Vec<XineramaScreenInfo> = vec![];
        for screen in screens {
            if tmp
                .iter()
                .filter(|ts| ts.x_org == screen.x_org && ts.y_org == screen.y_org)
                .count()
                == 0
            {
                tmp.push(screen);
            }
        }
        tmp
    };

    let screens_amount = screens.len();

    // 2. Add new screens
    for _ in n..screens_amount {
        app.runtime.screens.push(Screen {
            number: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            workspaces: vec![],
            current_workspace: 0,
            bar_offsets: BarOffsets::default(),
        })
    }

    // 3. Init screens
    for (index, screen) in screens.iter().enumerate() {
        app.runtime.screens[index].number = screen.screen_number as i64;
        app.runtime.screens[index].x = screen.x_org as i64;
        app.runtime.screens[index].y = screen.y_org as i64;
        app.runtime.screens[index].width = screen.width as i64;
        app.runtime.screens[index].height = screen.height as i64;
    }

    // 4. Move & delete removed screens
    for _ in screens_amount..n {
        if let Some(removed_screen) = app.runtime.screens.pop() {
            let removed_workspaces = removed_screen.workspaces;
            for (index, workspace) in removed_workspaces.into_iter().enumerate() {
                for client in workspace.clients {
                    update_client_desktop(app, client.window_id, index as u64);
                    app.runtime.screens[0].workspaces[index]
                        .clients
                        .push(client);
                }
            }
        }
    }
}

/// Create and set up workspaces
///
/// 1. Iterate over all screens
/// 2. If no workspaces create new
/// 3. Get names and geometry for workspaces
/// 4. Setup EWMH info of desktops
///
pub fn update_desktops(app: &mut Application) {
    let mut desktop_names_ewmh: Vec<String> = vec![];
    let mut viewports: Vec<i64> = vec![];

    // 1. Iterate over all screens
    for (index, screen) in app.runtime.screens.iter_mut().enumerate() {
        // 2. Create workspaces if needed
        if screen.workspaces.is_empty() {
            for i in 0..NUMBER_OF_DESKTOPS {
                let mw = if index < app.config.desktops.splits.len() {
                    app.config.desktops.splits[index][i]
                } else {
                    0.5
                };
                screen.workspaces.push(Workspace {
                    number: i as u64,
                    clients: Vec::new(),
                    current_client: None,
                    master_capacity: 1,
                    master_width: mw,
                });
            }
        }

        // 3. Get names & geometry
        for i in 0..screen.workspaces.len() {
            if index < app.config.desktops.names.len() {
                desktop_names_ewmh.push(app.config.desktops.names[index][i].to_string());
            } else {
                desktop_names_ewmh.push(format!("{}", i + 1));
            }
            viewports.push(screen.x);
            viewports.push(screen.y);
        }
    }
    // 4. SEt info
    update_desktop_ewmh_info(app, desktop_names_ewmh, viewports);
}

pub fn get_window_placement(app: &mut Application, win: u64, scan: bool) -> ((usize, usize), u64) {
    let default_placement = (app.runtime.current_screen, app.runtime.current_workspace);

    let mut trans = 0;

    // Try to inherit parents' position
    if get_transient_for_hint(app.core.display, win, &mut trans) == 1
        && find_window_indexes(app, trans).is_some()
    {
        log!("==== Inherit parents position");
        return (
            if let Some((s, w, _c)) = find_window_indexes(app, trans) {
                (s, w)
            } else {
                default_placement
            },
            trans,
        );
    }

    // Try to use previous position on startup
    if scan {
        if let Some(sw) = get_client_workspace(app, win) {
            log!("==== Fetched startup position");
            return (sw, 0);
        };
    }

    // Try loading from autostart rules
    if let Some(pid) = get_client_pid(app, win) {
        log!("==== PID for {win} is {pid}");
        if let Some(ri) = app
            .runtime
            .autostart_rules
            .iter()
            .position(|r| r.pid == pid)
        {
            let rule = &app.runtime.autostart_rules[ri];
            log!("==== Fetched autostart position");
            if rule.screen < app.runtime.screens.len()
                && rule.workspace < app.runtime.screens[rule.screen].workspaces.len()
            {
                return ((rule.screen, rule.workspace), 0);
            }
        };
    }

    // Try permanent rules
    let title = match get_text_property(app.core.display, win, app.atoms.net_wm_name) {
        Some(name) => Some(name),
        None => None,
    };

    let (instance, class) = {
        let mut ch: x11::xlib::XClassHint = x11::xlib::XClassHint {
            res_name: std::ptr::null_mut(),
            res_class: std::ptr::null_mut(),
        };
        get_class_hint(app.core.display, win, &mut ch);

        let instance = cstr_to_string(ch.res_name as *const i8);
        let class = cstr_to_string(ch.res_class as *const i8);

        (instance, class)
    };

    for rule in &app.config.placements {
        let instance_flag = {
            if let (Some(rule_instance), Some(client_instance)) = (&rule.instance, &instance) {
                *rule_instance == *client_instance
            } else {
                rule.instance.is_none()
            }
        };
        let class_flag = {
            if let (Some(rule_class), Some(client_class)) = (&rule.class, &class) {
                *rule_class == *client_class
            } else {
                rule.class.is_none()
            }
        };
        let title_flag = {
            if let (Some(rule_title), Some(client_title)) = (&rule.title, &title) {
                *rule_title == *client_title
            } else {
                rule.title.is_none()
            }
        };
        if instance_flag && class_flag && title_flag {
            let s = if let Some(s) = rule.rule_screen {
                s
            } else {
                app.runtime.current_screen
            };
            let w = if let Some(w) = rule.rule_workspace {
                w
            } else {
                app.runtime.current_workspace
            };
            return ((s, w), 0);
        }
    }

    // Use current placement if nothing found;
    return (default_placement, 0);
}
