//#![allow(non_snake_case)]
//#![allow(non_upper_case_globals)]
//#![allow(dead_code)]

mod utils;
use std::process::Command;

use utils::get_default;
use utils::grab::grab_button;
use utils::grab::grab_key;

mod wrap;

// What the fuck is going on here
fn _argb_to_int(a: u32, r: u8, g: u8, b: u8) -> u64 {
    (a as u64) << 24 | (r as u64) << 16 | (g as u64) << 8 | (b as u64)
}

fn manage_client(
    dpy: &mut Display,
    ew: u64,
    cw: &mut u64,
    ci: &mut Option<usize>,
    clients: &mut Vec<u64>,
) {
    let mut wa: XSetWindowAttributes = get_default::XSetWindowAttributes();
    wa.event_mask =
        LeaveWindowMask | EnterWindowMask | SubstructureNotifyMask | StructureNotifyMask;
    ChangeWindowAttributes(dpy, ew, CWEventMask | CWCursor, &mut wa);

    // get name
    // let mut c: *mut i8 = null_mut();
    // if XFetchName(dpy, ew, get_mut_ptr(&mut c)) == True {
    //     println!("      |- Got window name: {:?}", CStr::from_ptr(c).to_str());
    //     libc::free(c as *mut libc::c_void);
    // } else {
    //     println!("      |- Failed to get window name");
    // }
    // // get class
    // let ch: *mut XClassHint = XAllocClassHint();
    // if XGetClassHint(dpy, ew, ch) == True {
    //     println!("      |- Got window class");
    //     println!(
    //         "         |- name: {:?}",
    //         CStr::from_ptr((*ch).res_name).to_str()
    //     );
    //     println!(
    //         "         |- class: {:?}",
    //         CStr::from_ptr((*ch).res_class).to_str()
    //     );
    //     XFree((*ch).res_name as *mut libc::c_void);
    //     XFree((*ch).res_class as *mut libc::c_void);
    // } else {
    //     println!("      |- Failed To Get Window Class");
    // }

    *cw = ew;
    *ci = Some(clients.len());
    clients.push(ew);

    RaiseWindow(dpy, ew);
    MoveResizeWindow(dpy, ew, 0, 0, 1920, 1080);
    MapWindow(dpy, ew);
}

fn get_event_names_list() -> Vec<&'static str> {
    vec![
        "_",
        "_",
        "KeyPress",
        "KeyRelease",
        "ButtonPress",
        "ButtonRelease",
        "MotionNotify",
        "EnterNotify",
        "LeaveNotify",
        "FocusIn",
        "FocusOut",
        "KeymapNotify",
        "Expose",
        "GraphicsExpose",
        "NoExpose",
        "VisibilityNotify",
        "CreateNotify",
        "DestroyNotify",
        "UnmapNotify",
        "MapNotify",
        "MapRequest",
        "ReparentNotify",
        "ConfigureNotify",
        "ConfigureRequest",
        "GravityNotify",
        "ResizeRequest",
        "CirculateNotify",
        "CirculateRequest",
        "PropertyNotify",
        "SelectionClear",
        "SelectionRequest",
        "SelectionNotify",
        "ColormapNotify",
        "ClientMessage",
        "MappingNotify",
        "GenericEvent",
        "_",
    ]
}

use x11::keysym::*;
use x11::xlib::ButtonPress;
use x11::xlib::ButtonRelease;
use x11::xlib::CWCursor;
use x11::xlib::CWEventMask;
use x11::xlib::CurrentTime;
use x11::xlib::DestroyNotify;
use x11::xlib::Display;
use x11::xlib::EnterNotify;
use x11::xlib::EnterWindowMask;
use x11::xlib::IsViewable;
use x11::xlib::KeyPress;
use x11::xlib::LeaveNotify;
use x11::xlib::LeaveWindowMask;
use x11::xlib::MapRequest;
use x11::xlib::Mod1Mask as ModKey;
use x11::xlib::MotionNotify;
use x11::xlib::RevertToNone;
use x11::xlib::RevertToParent;
use x11::xlib::ShiftMask;
use x11::xlib::StructureNotifyMask;
use x11::xlib::SubstructureNotifyMask;
use x11::xlib::SubstructureRedirectMask;
use x11::xlib::XButtonEvent;
use x11::xlib::XSetWindowAttributes;
use x11::xlib::XWindowAttributes;

use crate::wrap::xinerama::XineramaQueryScreens;
use crate::wrap::xlib::ChangeWindowAttributes;
use crate::wrap::xlib::DefaultRootWindow;
use crate::wrap::xlib::GetTransientForHint;
use crate::wrap::xlib::GetWindowAttributes;
use crate::wrap::xlib::KeysymToKeycode;
use crate::wrap::xlib::KillClient;
use crate::wrap::xlib::MapWindow;
use crate::wrap::xlib::MoveResizeWindow;
use crate::wrap::xlib::NextEvent;
use crate::wrap::xlib::OpenDisplay;
use crate::wrap::xlib::QueryTree;
use crate::wrap::xlib::RaiseWindow;
use crate::wrap::xlib::SelectInput;
use crate::wrap::xlib::SetInputFocus;
use crate::wrap::xlib::SetWindowBorderWidth;

const MOD_KEY_SHIFT: u32 = ModKey | x11::xlib::ShiftMask;

fn main() {
    println!("Started Window Manager");
    //    unsafe {
    let events: Vec<&str> = get_event_names_list();
    println!("|- Created Event Look-Up Array");

    let dpy: &mut Display = OpenDisplay(None).expect("Error opening display!");
    println!("|- Opened X Display");

    let root_win: u64 = DefaultRootWindow(dpy);
    println!("|- Root window is {}", root_win);

    println!("|- Getting per monitor sizes");
    let screens = XineramaQueryScreens(dpy).expect("Running without xinerama is not supported");
    println!("|- There are {} screen connected", screens.len());
    for screen in screens {
        println!(
            "|- Screen {} has size of {}x{} pixels and originates from {},{}",
            screen.screen_number, screen.width, screen.height, screen.x_org, screen.y_org
        );
    }

    let mut attr: XWindowAttributes = get_default::XWindowAttributes();
    let mut start: XButtonEvent = get_default::XButtonEvent();
    start.subwindow = 0;

    let mut clients: Vec<u64> = Vec::new();
    let mut client_index: Option<usize> = None;
    let mut current_win: u64 = 0;

    println!("|- Created Useful Variables");

    let mut wa: XSetWindowAttributes = get_default::XSetWindowAttributes();

    // wa.event_mask = LeaveWindowMask | EnterWindowMask | SubstructureNotifyMask | StructureNotifyMask;
    wa.event_mask = SubstructureRedirectMask
        | LeaveWindowMask
        | EnterWindowMask
        | SubstructureNotifyMask
        | StructureNotifyMask;

    ChangeWindowAttributes(dpy, root_win, CWEventMask | CWCursor, &mut wa);

    SelectInput(dpy, root_win, wa.event_mask);

    println!("|- Applied Event Mask");

    let (mut rw, _, wins) = QueryTree(dpy, root_win);

    println!("|- {} windows are alredy present", clients.len());

    for win in wins {
        println!("|-- Checking window {win}");
        let res = GetWindowAttributes(dpy, win);
        if let Some(wa) = res {
            if wa.override_redirect != 0 || GetTransientForHint(dpy, win, &mut rw) != 0 {
                println!("|---- Window is transient. Skipping");
                continue;
            }
            if wa.map_state == IsViewable {
                println!("|---- Window is viewable. Managing");
                manage_client(dpy, win, &mut current_win, &mut client_index, &mut clients);
                continue;
            }
        }
        println!("|---- Can't manage window")
    }

    grab_key(dpy, XK_Return, ModKey | ShiftMask); // Move to top
    grab_key(dpy, XK_Return, ModKey); // Spawn alacritty
    grab_key(dpy, XK_Q, ModKey | ShiftMask); // Exit rust-wm
    grab_key(dpy, XK_p, ModKey); // Run dmenu
    grab_key(dpy, XK_Page_Up, ModKey); // maximize window
    grab_key(dpy, XK_C, ModKey | ShiftMask); // close window
    grab_key(dpy, XK_Tab, ModKey); // Cycle Through Windows
    grab_key(dpy, XK_l, ModKey); // Query current window information

    grab_button(dpy, 1, ModKey); // Move window
    grab_button(dpy, 2, ModKey); // Focus window
    grab_button(dpy, 3, ModKey); // Resize window

    println!("|- Grabbed Shortcuts");
    println!("|- Starting Main Loop");
    loop {
        let ev = NextEvent(dpy);
        println!("   |- Got Event Of Type \"{}\"", events[ev.type_ as usize]);
        if ev.type_ == KeyPress {
            let key = ev.key.unwrap();
            let _ew: u64 = key.window;

            if key.state == ModKey {
                if key.keycode == KeysymToKeycode(dpy, XK_Return) {
                    println!("   |- Spawning Terminal");
                    let mut handle = Command::new("kitty").spawn().expect("can't run kitty");
                    std::thread::spawn(move || {
                        handle.wait().expect("can't run process");
                    });
                }
                if key.keycode == KeysymToKeycode(dpy, XK_p) {
                    println!("   |- Spawning Dmenu");
                    Command::new("dmenu_run").spawn().unwrap().wait().unwrap();
                }
                if key.keycode == KeysymToKeycode(dpy, XK_Page_Up) {
                    println!("   |- Maximazing Window: {current_win}");
                    MoveResizeWindow(dpy, current_win, 0, 0, 1920, 1080);
                    SetWindowBorderWidth(dpy, current_win, 0);
                }
                if key.keycode == KeysymToKeycode(dpy, XK_Tab) {
                    if clients.len() > 1 {
                        println!("   |- Cycling to previous windows...(Hopefully)");
                        println!("   |- Current clients are {:?}", clients);
                        let index = client_index.unwrap();
                        // XMoveWindow(dpy, clients[index], -1920, -1080);
                        client_index = Some((index + 1) % clients.len());
                        let index = client_index.unwrap();
                        RaiseWindow(dpy, clients[index]);
                        // XMoveWindow(dpy, clients[index], 0, 0);
                    } else {
                        println!("   |- No windows. Skipping")
                    }
                }
                if key.keycode == KeysymToKeycode(dpy, XK_l) {
                    println!("   |- Current window is {current_win}");
                    println!("   |- Current Clients are {clients:?}")
                }
            }
            if key.state == MOD_KEY_SHIFT {
                if key.keycode == KeysymToKeycode(dpy, XK_C) {
                    println!("   |- Killing Window: {current_win}");
                    clients.retain(|&client| client != current_win);
                    KillClient(dpy, current_win);
                };
                if key.keycode == KeysymToKeycode(dpy, XK_Q) {
                    println!("   |- Exiting Window Manager");
                    break;
                }
            }
        }
        if ev.type_ == ButtonPress {
            let button = ev.button.unwrap();
            let ew = button.subwindow;
            if button.subwindow != 0 {
                if button.button == 2 {
                    println!("   |- Selecting Window: {ew}");
                    RaiseWindow(dpy, ew);
                    SetInputFocus(dpy, ew, RevertToParent, CurrentTime);
                    // add window decoration
                    // XSetWindowBorderWidth(dpy, ew, 2);
                    // XSetWindowBorder(dpy, ew, argb_to_int(0, 98, 114, 164));
                } else {
                    println!("   |- Started Grabbing Window: {ew}");
                    attr = GetWindowAttributes(dpy, button.subwindow).unwrap();
                    start = button;
                }
            }
        }
        if ev.type_ == MotionNotify {
            let motion = ev.motion.unwrap();
            let button = ev.button.unwrap();
            let ew: u64 = motion.window;

            println!("   |- Window id: {ew}");

            if button.subwindow != 0 && start.subwindow != 0 {
                println!("   |- Resizing OR Moving Window");
                let x_diff: i32 = button.x_root - start.x_root;
                let y_diff: i32 = button.y_root - start.y_root;
                MoveResizeWindow(
                    dpy,
                    start.subwindow,
                    attr.x + {
                        if start.button == 1 {
                            x_diff
                        } else {
                            0
                        }
                        // Get u32 keycode from keysym
                    },
                    attr.y + {
                        if start.button == 1 {
                            y_diff
                        } else {
                            0
                        }
                    },
                    1.max(
                        (attr.width + {
                            if start.button == 3 {
                                x_diff
                            } else {
                                0
                            }
                        }) as u32,
                    ),
                    1.max(
                        (attr.height + {
                            if start.button == 3 {
                                y_diff
                            } else {
                                0
                            }
                        }) as u32,
                    ),
                );
            } else {
                println!("   |- Just Moving");
                // XSetInputFocus(dpy, win, RevertToNone, CurrentTime);
            }
        }
        if ev.type_ == ButtonRelease {
            start.subwindow = 0;
        }
        if ev.type_ == MapRequest {
            let ew: u64 = ev.map_request.unwrap().window;
            manage_client(dpy, ew, &mut current_win, &mut client_index, &mut clients);
            println!("   |- Request From Window: {ew}");
        }

        if ev.type_ == EnterNotify {
            let ew: u64 = ev.crossing.unwrap().window;

            println!("      |- Window Id: {}", ew);

            // let mut c: *mut i8 = null_mut();
            // if XFetchName(dpy, ew, get_mut_ptr(&mut c)) == True {
            //     println!(
            //         "         |- Got Window Name: {:?}",
            //         CStr::from_ptr(c).to_str()
            //     );
            //     libc::free(c as *mut libc::c_void);
            // } else {
            //     println!("         |- Failed to get window name");
            // }

            // println!("         |- Raising window");
            // XRaiseWindow(dpy, ew);

            println!("         |- Setting focus to window");
            SetInputFocus(dpy, ew, RevertToNone, CurrentTime);

            current_win = ew;
        }
        if ev.type_ == LeaveNotify {
            let ew: u64 = ev.crossing.unwrap().window;

            println!("      |- Window id: {}", ew);
        }
        if ev.type_ == DestroyNotify {
            let ew: u64 = ev.destroy_window.unwrap().window;

            println!("      |- Window [{}] destroyed", ew);
            clients.retain(|&c| c != ew);

            if clients.len() > 0 {
                client_index = Some(client_index.unwrap() % clients.len());
            } else {
                client_index = None;
            }
        }
    }
}
