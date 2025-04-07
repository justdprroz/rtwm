use crate::structs::*;
use crate::utils::*;

/// Arrange windows of specified workspace in specified layout
/// 1. Get structs by index
/// 2. Calculate usable screen sizes, gaps, borders etc
/// 3. Get amount of clients to be tiled
/// 4. Check if all client go to master
/// 5. Iterate all clients in current workspace and calculate geometry
/// 6. Show maximized clients
/// 7. Show master clients
/// 8. Show stack clients
/// 9. Update calculated geometry
/// 10. Fullscreen window if needed
/// 11. Update borders
/// 12. Position windows
pub fn arrange_workspace(app: &mut Application, screen: usize, workspace: usize) {
    log!("======ARRANGING S: {}, W: {}", screen, workspace);
    // 1. Get actual structures
    let screen = &mut app.runtime.screens[screen];
    let workspace = &mut screen.workspaces[workspace];
    // 2. Calculate usable screen sizes, gaps, borders etc
    let bar_offsets = screen.bar_offsets;
    let screen_height = screen.height - (bar_offsets.up + bar_offsets.down) as i64;
    let gap = app.config.gap_width as i32;
    let border = app.config.border_size as u32;
    let mut master_width = ((screen.width as i32 - gap * 3) as f64 * workspace.master_width) as u32;
    let stack_width = (screen.width as i32 - gap * 3) - master_width as i32;
    let mut master_capacity = workspace.master_capacity;

    // 3. Get amount of clients to be tiled
    let stack_size = workspace.clients.iter().filter(|&c| !c.floating).count();
    // 4. Check if all client go to master
    if master_capacity <= 0 || master_capacity >= stack_size as i64 {
        master_capacity = stack_size as i64;
        master_width = screen.width as u32 - gap as u32 * 2;
    }
    log!("   |- Arranging {} tilable window", stack_size);
    // 5. Iterate all clients in current workspace and calculate geometry
    for (index, client) in workspace
        .clients
        .iter_mut()
        .rev()
        .filter(|c| !c.floating && !c.fullscreen)
        .enumerate()
    {
        // 6. Show maximized clients
        if stack_size == 1 {
            client.x = screen.x as i32;
            client.y = screen.y as i32 + bar_offsets.up as i32;
            client.w = screen.width as u32;
            client.h = screen_height as u32;
            client.border = 0;
        } else {
            if (index as i64) < master_capacity {
                // 7. Show master clients
                let win_height =
                    (screen_height - gap as i64 - master_capacity * gap as i64) / master_capacity;
                client.x = gap + screen.x as i32;
                client.y = bar_offsets.up as i32
                    + gap
                    + (win_height as i32 + gap) * index as i32
                    + screen.y as i32;
                client.w = master_width - 2 * border;
                client.h = if index as i64 != master_capacity - 1 {
                    win_height as u32 - 2 * border
                } else {
                    (screen_height as i32 - gap - client.y + bar_offsets.up as i32) as u32
                        - 2 * border
                };
            } else {
                // 8. Show stack clients
                let win_height = (screen_height
                    - gap as i64
                    - (stack_size as i64 - master_capacity) * gap as i64)
                    / (stack_size as i64 - master_capacity);
                client.x = master_width as i32 + (gap * 2) + screen.x as i32;
                client.y = bar_offsets.up as i32
                    + gap
                    + (win_height as i32 + gap) * (index as i64 - master_capacity) as i32
                    + screen.y as i32;
                client.w = stack_width as u32 - 2 * border;
                client.h = if index != stack_size - 1 {
                    win_height as u32 - 2 * border
                } else {
                    (screen_height as i32 - gap - client.y + bar_offsets.up as i32) as u32
                        - 2 * border
                };
            }
            client.border = app.config.border_size as u32;
        }

        // client.x += screen.x as i32;
        // client.y += screen.y as i32;
    }
}

//pub fn tiled(app: &mut Application) {
//
//}

/// Arrange windows of current workspace in specified layout
/// 1. Iterate over all screens
/// 2. Arrange current workspace
pub fn arrange_visible(app: &mut Application) {
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
