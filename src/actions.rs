use x11::xlib::DestroyAll;
use x11::xlib::PropModeReplace;
use x11::xlib::XA_CARDINAL;

use crate::arrange::*;
use crate::config;
use crate::helper::*;
use crate::logic::*;
use crate::structs::*;
use crate::utils::*;
use crate::wrapper::xlib::*;

pub fn kill_client(app: &mut Application) {
    if let Some(index) = app.runtime.current_client {
        let id = app.runtime.screens[app.runtime.current_screen].workspaces
            [app.runtime.current_workspace]
            .clients[index]
            .window_id;
        if !send_atom(app, id, app.atoms.wm_delete) {
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

        // Update client trackers on NEW workspace

        // Update client tracker on current screen
        shift_current_client(
            app,
            app.runtime.current_screen,
            app.runtime.current_workspace,
        );
        // Get workspace tracker(borrow checker is really mad at me)
        let nw = app.runtime.screens[new_screen_index].current_workspace;
        // Add window to stack of another display
        // app.runtime.screens[new_screen_index].workspaces[nw]
        //     .clients
        //     .push(client);

        let workspace = &mut app.runtime.screens[new_screen_index].workspaces[nw];

        // 12. Add window to stack
        workspace.current_client = Some(workspace.clients.len());
        app.runtime.current_client = workspace.current_client;
        workspace.clients.push(client);

        // Arrange all monitors
        arrange_visible(app);
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

pub fn move_client_to_workspace(app: &mut Application, mut client: Client, n: u64) {
    // 1. Unfocus this client
    unfocus(app, client.window_id);

    // 2. Move window out of view
    move_resize_window(
        app.core.display,
        client.window_id,
        -((client.w + client.border * 2) as i32),
        -((client.h + client.border * 2) as i32),
        client.w,
        client.h,
    );
    client.visible = !client.visible;

    // 3. Override client desktop
    let cur_workspace: usize = n as usize + app.runtime.current_screen * config::NUMBER_OF_DESKTOPS;
    update_client_desktop(app, client.window_id, cur_workspace as u64);

    // 4. Add window to stack
    let workspace = &mut app.runtime.screens[app.runtime.current_screen].workspaces[n as usize];

    workspace.current_client = Some(workspace.clients.len());
    app.runtime.current_client = workspace.current_client;
    workspace.clients.push(client);
    arrange_workspace(app, app.runtime.current_screen, n as usize);
}

pub fn move_to_workspace(app: &mut Application, n: u64) {
    log!("   |- Got `MoveToWorkspace` Action ");
    if let Some(index) = app.runtime.current_client {
        if n as usize != app.runtime.current_workspace {
            // 1.Pop current client
            let popped_client = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .clients
                .remove(index);
            
            // 3. Move popped client
            move_client_to_workspace(app, popped_client, n);

            // 4. Update current workspace
            arrange_visible(app);
            show_workspace(
                app,
                app.runtime.current_screen,
                app.runtime.current_workspace,
            );

            // 5. Update current client index
            shift_current_client(
                app,
                app.runtime.current_screen,
                app.runtime.current_workspace,
            );
        } else {
            // Remap to another action
            pop_push_stack(app, true);
        }
    }
}

pub fn cycle_stack(app: &mut Application, d: i64) {
    let ws = &mut app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace];
    let num_clients = ws.clients.len();

    let cur_index = if num_clients < 2 {
        return;
    } else {
        match ws.current_client {
            Some(i) => i,
            None => return,
        }
    };
    let new_index = (((num_clients + cur_index) as i64 + d) % num_clients as i64) as usize;

    let old_win = ws.clients[cur_index].window_id;
    let new_win = ws.clients[new_index].window_id;

    unfocus(app, old_win);
    focus(app, new_win);
    
    raise_window(app.core.display, new_win);
    suppress_notify_strict(app);
}

pub fn pop_push_stack(app: &mut Application, current: bool) {
    let workspace = &mut app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace];

    match workspace.arrange {
        ArrangeEngine::Tiled => {}
        _ => {
            return;
        }
    }

    // No need to rotate single window;
    if workspace.clients.len() < 2 {
        return;
    }

    // Choose index 0 vs current
    let client_index = if current {
        match app.runtime.current_client {
            Some(c) => c,
            None => return,
        }
    } else {
        0
    };

    // Pop client (current or bottom)
    let popped_client = workspace.clients.remove(client_index);

    // Place on top of stack (upper left)
    workspace.clients.push(popped_client);

    // Update tracking
    workspace.current_client = Some(workspace.clients.len() - 1);
    app.runtime.current_client = workspace.current_client;

    arrange_visible(app);
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );

    suppress_notify(app);

    arrange_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
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
        update_active_window(app);
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

    suppress_notify(app);
}

pub fn update_master_width(app: &mut Application, w: f64) {
    let workspace = &mut app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace];

    match workspace.arrange {
        ArrangeEngine::Tiled => {}
        _ => {
            return;
        }
    }

    // Update master width
    let mw = &mut workspace.master_width;
    if f64::abs(w) < *mw + w && *mw + w < 1.0 {
        *mw += w;
    }
    // Rearrange windows
    arrange_visible(app);
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
    suppress_notify(app);
}

pub fn update_master_capacity(app: &mut Application, i: i64) {
    let workspace = &mut app.runtime.screens[app.runtime.current_screen].workspaces
        [app.runtime.current_workspace];

    match workspace.arrange {
        ArrangeEngine::Tiled => {}
        _ => {
            return;
        }
    }
    // Change master size
    workspace.master_capacity += i;
    // Rearrange windows
    arrange_visible(app);
    show_workspace(
        app,
        app.runtime.current_screen,
        app.runtime.current_workspace,
    );
    suppress_notify(app);
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

        arrange_visible(app);
        show_workspace(
            app,
            app.runtime.current_screen,
            app.runtime.current_workspace,
        );
    }
    suppress_notify(app);
}

pub fn focus_on_screen_index(app: &mut Application, n: usize) {
    log!("Focusing on screen");
    if let Some(cw) = get_current_client_id(app) {
        log!("unfocusing {}", cw);
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
        log!("SETTING INPUT FOCUS");
        // set_input_focus(app.core.display, win, RevertToPointerRoot, CurrentTime);
        focus(app, win);
    }
    // update_active_window(app);
    // if let Some(cw) = get_current_client_id(app) {
    //     set_window_border(
    //         app.core.display,
    //         cw,
    //         argb_to_int(app.config.active_border_color),
    //     );
    // }
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
