//! Functions for mouse support

use x11::xlib::{PropModeReplace, XMotionEvent, XA_CARDINAL};

use crate::helper::{find_window_indexes, update_client_desktop};
use crate::log;
use crate::logic::shift_current_client;
use crate::structs::Application;
use crate::wrapper::xlib::change_property;
use crate::wrapper::xlib::move_resize_window;

pub fn move_mouse(app: &mut Application, motion_event: XMotionEvent) {
    let moving_window: u64 = app.runtime.mouse_state.win;

    if let Some((mut s, mut w, mut c)) = find_window_indexes(app, moving_window) {
        let (mouse_x, mouse_y) = (motion_event.x_root as i64, motion_event.y_root as i64);
        let (pos_x, pos_y) = app.runtime.mouse_state.pos;
        let (dx, dy) = (mouse_x - pos_x, mouse_y - pos_y);

        if !(app.runtime.screens[s].x <= mouse_x
            && mouse_x < app.runtime.screens[s].x + app.runtime.screens[s].width
            && app.runtime.screens[s].y <= mouse_y
            && mouse_y < app.runtime.screens[s].y + app.runtime.screens[s].height)
        {
            let mut new_screen = s;
            for index in 0..app.runtime.screens.len() {
                let screen = &app.runtime.screens[index];
                if screen.x <= mouse_x
                    && mouse_x < screen.x + screen.width
                    && screen.y <= mouse_y
                    && mouse_y < screen.y + screen.height
                {
                    new_screen = index;
                }
            }
            let client = app.runtime.screens[s].workspaces[w].clients.remove(c);

            // Update workspace
            let new_workspace: usize = app.runtime.screens[new_screen].current_workspace
                + new_screen * crate::config::NUMBER_OF_DESKTOPS;
            update_client_desktop(app, client.window_id, new_workspace as u64);

            change_property(
                app.core.display,
                app.core.root_win,
                app.atoms.net_current_desktop,
                XA_CARDINAL,
                32,
                PropModeReplace,
                &new_workspace as *const usize as *mut usize as *mut u8,
                1,
            );

            // Update client tracker on current screen
            shift_current_client(app, None, None);

            // Add window to stack of another display
            let nw = app.runtime.screens[new_screen].current_workspace;
            app.runtime.screens[new_screen].workspaces[nw]
                .clients
                .push(client);
            s = new_screen;
            app.runtime.current_screen = s;

            w = nw;
            app.runtime.current_workspace = w;

            c = app.runtime.screens[s].workspaces[w].clients.len() - 1;
            app.runtime.current_client = Some(c);

            log!("CHANGED SCREEEEN");
        }

        // let screen_x = app.runtime.screens[s].x;
        // let screen_y = app.runtime.screens[s].y;
        // let screen_w = app.runtime.screens[s].width;
        // let screen_h = app.runtime.screens[s].height;
        //
        // // Screen Bar Left/Up/Right/Down
        // let screen_rect = {
        //     let screen = &app.runtime.screens[s];
        //     let sbl = screen.bar_offsets.left as i64;
        //     let sbu = screen.bar_offsets.up as i64;
        //     let sbr = screen.width - screen.bar_offsets.right as i64;
        //     let sbd = screen.height - screen.bar_offsets.down as i64;
        //     (sbl, sbu, sbr, sbd)
        // };

        let client = &mut app.runtime.screens[s].workspaces[w].clients[c];
        let new_x = client.x + dx as i32;
        let new_y = client.y + dy as i32;

        // Stick to screen border
        // let stick = 50;
        // let unstick = 30 as i64;

        // if (new_x < sbl as i32 + stick)
        //     && (dx < 0 && new_x > (sbl - unstick) as i32
        //         || new_x > sbl as i32 && new_x < (sbl + unstick) as i32)
        // {
        //     new_x = sbl as i32;
        // }
        // if (new_x + client.w as i32) > sbr as i32 - stick
        //     && (dx > 0 && (new_x + client.w as i32) < (sbr + unstick) as i32
        //         || (new_x + client.w as i32) < sbr as i32
        //             && (new_x + client.w as i32) > (sbr - unstick) as i32)
        // {
        //     new_x = sbr as i32 - client.w as i32 - 2 * app.config.border_size as i32;
        // }
        //
        // if new_y < sbu as i32 + stick
        //     && (dy < 0 && new_y > (sbu - unstick) as i32
        //         || new_y > sbu as i32 && new_y < (sbu + unstick) as i32)
        // {
        //     new_y = sbu as i32;
        // }
        // if (new_y + client.h as i32) > sbd as i32 - stick
        //     && (dy > 0 && (new_y + client.h as i32) < (sbd + unstick) as i32
        //         || (new_y + client.h as i32) < sbd as i32
        //             && (new_y + client.h as i32) > (sbd - unstick) as i32)
        // {
        //     new_y = sbd as i32 - client.h as i32 - 2 * app.config.border_size as i32;
        // }

        // Unstick from border

        if client.x != new_x {
            app.runtime.mouse_state.pos.0 = mouse_x;
        }

        if client.y != new_y {
            app.runtime.mouse_state.pos.1 = mouse_y;
        }

        client.x = new_x;
        client.y = new_y;

        move_resize_window(
            app.core.display,
            moving_window,
            client.x,
            client.y,
            client.w,
            client.h,
        );
    }
}

pub fn resize_mouse(app: &mut Application, motion_event: XMotionEvent) {
    let (mouse_x, mouse_y) = (motion_event.x_root as i64, motion_event.y_root as i64);

    let (pos_x, pos_y) = app.runtime.mouse_state.pos;
    let (dx, dy) = (mouse_x - pos_x, mouse_y - pos_y);
    app.runtime.mouse_state.pos = (mouse_x, mouse_y);
    let mw: u64 = app.runtime.mouse_state.win;

    if let Some((s, w, c)) = find_window_indexes(app, mw) {
        let screen = &mut app.runtime.screens[s];

        let client = &mut screen.workspaces[w].clients[c];
        let mut nw = client.w as i32;
        let mut nh = client.h as i32;
        if (nw + dx as i32) > client.minw && (client.maxw == 0 || client.maxw > 0 && (nw + dx as i32) < client.maxw) {
            nw += dx as i32;
        };
        if (nh + dy as i32) > client.minh && (client.maxh == 0 || client.maxh > 0 && (nh + dy as i32) < client.maxh) {
            nh += dy as i32;
        }
        client.w = nw as u32;
        client.h = nh as u32;

        move_resize_window(
            app.core.display,
            mw,
            client.x,
            client.y,
            client.w,
            client.h,
        );
    }
}

pub fn screen_mouse(app: &mut Application, me: XMotionEvent) {
    let (mx, my) = (me.x_root as i64, me.y_root as i64);

    for screen in &app.runtime.screens {
        if screen.x <= mx
            && mx < screen.x + screen.width
            && screen.y <= my
            && my < screen.y + screen.height
        {
            // Update trackers
            app.runtime.current_screen = screen.number as usize;
            app.runtime.current_workspace =
                app.runtime.screens[app.runtime.current_screen].current_workspace;
            app.runtime.current_client = app.runtime.screens[app.runtime.current_screen].workspaces
                [app.runtime.current_workspace]
                .current_client;
            let w = app.runtime.current_workspace
                + app.runtime.current_screen * crate::config::NUMBER_OF_DESKTOPS;

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
    }
}
