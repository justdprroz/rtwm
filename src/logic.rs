//! Main windows manager logic processed as response to events

use std::process::exit;

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
            log!("|=  SETTING INPUT FOCUS");
            let win = ws.screens[screen].workspaces[workspace].clients[index].window_id;
            set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
        }
        update_active_window(app);
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
    grab_button(app.core.display, win, Button1, ModKey);
    grab_button(app.core.display, win, Button3, ModKey);

    // Update focus on window
    log!("SETTING FOCUS ON {}", win);
    set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
    send_atom(app, win, app.atoms.wm_take_focus);

    // Update workspace to new one
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
    let title = get_text_property(app.core.display, win, app.atoms.net_wm_name);

    let (instance, class) = {
        let mut ch = ClassHint::default();
        get_class_hint(app.core.display, win, &mut ch);
        (ch.res_name, ch.res_class)
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
    (default_placement, 0)
}
