//! \*Safe\* wrap for x11

pub mod sys {
    pub fn set_locale(c: i32, l: &str) {
        unsafe {
            let locale = std::ffi::CString::new(l).unwrap();
            libc::setlocale(c, locale.as_ptr());
        }
    }

    /// Remove zombie processes after spawning with shortcuts
    pub fn no_zombies() {
        use nix::sys::signal::*;
        unsafe {
            let sa = SigAction::new(
                SigHandler::SigIgn,
                SaFlags::SA_NOCLDSTOP | SaFlags::SA_NOCLDWAIT | SaFlags::SA_RESTART,
                SigSet::empty(),
            );
            let _ = sigaction(SIGCHLD, &sa);
        }
    }
}

pub mod xlib {
    const EVENT_LOOKUP: [&str; 37] = [
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
        "LASTEvent",
    ];

    use x11::xlib::{
        Atom, ButtonPress, ButtonRelease, ClientMessage, ConfigureNotify, ConfigureRequest,
        DestroyNotify, EnterNotify, KeyPress, KeyRelease, LeaveNotify, MapRequest, MotionNotify,
        PropertyNotify, UnmapNotify, XEvent,
    };

    unsafe extern "C" fn handler_func(
        _d: *mut x11::xlib::Display,
        _e: *mut x11::xlib::XErrorEvent,
    ) -> i32 {
        0
    }

    pub fn set_error_handler() {
        unsafe {
            x11::xlib::XSetErrorHandler(Some(handler_func));
        }
    }

    pub fn open_display(display_name: Option<&str>) -> Option<&mut x11::xlib::Display> {
        unsafe {
            let result = match display_name {
                Some(dn) => {
                    let name_ptr = dn.as_ptr() as *const i8;
                    x11::xlib::XOpenDisplay(name_ptr)
                }
                None => x11::xlib::XOpenDisplay(std::ptr::null::<i8>()),
            };
            result.as_mut()
        }
    }

    pub fn default_root_window(display: &mut x11::xlib::Display) -> u64 {
        unsafe { x11::xlib::XDefaultRootWindow(display as *mut x11::xlib::Display) }
    }

    pub fn set_window_border(display: &mut x11::xlib::Display, w: u64, border_pixel: u64) {
        unsafe {
            x11::xlib::XSetWindowBorder(display as *mut x11::xlib::Display, w, border_pixel);
        }
    }

    pub fn delete_property(display: &mut x11::xlib::Display, w: u64, property: x11::xlib::Atom) {
        unsafe {
            x11::xlib::XDeleteProperty(display as *mut x11::xlib::Display, w, property);
        }
    }

    pub fn default_screen(display: &mut x11::xlib::Display) -> i32 {
        unsafe { x11::xlib::XDefaultScreen(display as *mut x11::xlib::Display) }
    }

    pub fn grab_key(dpy: &mut x11::xlib::Display, keysym: u32, mask: u32) {
        unsafe {
            x11::xlib::XGrabKey(
                dpy,
                x11::xlib::XKeysymToKeycode(dpy, keysym as u64) as i32,
                mask,
                x11::xlib::XDefaultRootWindow(dpy),
                1,
                x11::xlib::GrabModeAsync,
                x11::xlib::GrabModeAsync,
            );
        }
    }

    pub fn grab_button(dpy: &mut x11::xlib::Display, win: u64, button: u32, mask: u32) {
        unsafe {
            x11::xlib::XGrabButton(
                dpy,
                button,
                mask,
                win,
                1,
                (x11::xlib::ButtonPressMask
                    | x11::xlib::ButtonReleaseMask
                    | x11::xlib::PointerMotionMask) as u32,
                x11::xlib::GrabModeAsync,
                x11::xlib::GrabModeAsync,
                0,
                0,
            );
        }
    }

    pub fn ungrab_button(dpy: &mut x11::xlib::Display, button: u32, mask: u32, win: u64) {
        unsafe {
            x11::xlib::XUngrabButton(dpy as *mut x11::xlib::Display, button, mask, win);
        }
    }

    pub fn warp_pointer_win(dpy: &mut x11::xlib::Display, win: u64, dx: i32, dy: i32) {
        unsafe {
            x11::xlib::XWarpPointer(dpy as *mut x11::xlib::Display, 0, win, 0, 0, 0, 0, dx, dy);
        }
    }

    pub fn change_window_attributes(
        display: &mut x11::xlib::Display,
        w: u64,
        valuemask: u64,
        attributes: &mut x11::xlib::XSetWindowAttributes,
    ) -> i32 {
        unsafe {
            x11::xlib::XChangeWindowAttributes(
                display as *mut x11::xlib::Display,
                w,
                valuemask,
                attributes as *mut x11::xlib::XSetWindowAttributes,
            )
        }
    }

    pub fn select_input(display: &mut x11::xlib::Display, w: u64, event_mask: i64) -> i32 {
        unsafe { x11::xlib::XSelectInput(display as *mut x11::xlib::Display, w, event_mask) }
    }

    pub fn configure_window(
        display: &mut x11::xlib::Display,
        w: u64,
        valuemask: u32,
        values: &mut x11::xlib::XWindowChanges,
    ) {
        unsafe {
            x11::xlib::XConfigureWindow(
                display as *mut x11::xlib::Display,
                w,
                valuemask,
                values as *mut x11::xlib::XWindowChanges,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_simple_window(
        display: &mut x11::xlib::Display,
        parent: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        border: u64,
        background: u64,
    ) -> u64 {
        unsafe {
            x11::xlib::XCreateSimpleWindow(
                display as *mut x11::xlib::Display,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                border,
                background,
            )
        }
    }

    pub fn set_class_hints(
        display: &mut x11::xlib::Display,
        w: u64,
        class_hints: &mut x11::xlib::XClassHint,
    ) {
        unsafe {
            x11::xlib::XSetClassHint(
                display as *mut x11::xlib::Display,
                w,
                class_hints as *mut x11::xlib::XClassHint,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_window(
        display: &mut x11::xlib::Display,
        parent: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        depth: i32,
        class: u32,
        visual: &mut x11::xlib::Visual,
        valuemask: u64,
        attributes: &mut x11::xlib::XSetWindowAttributes,
    ) -> u64 {
        unsafe {
            x11::xlib::XCreateWindow(
                display as *mut x11::xlib::Display,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                depth,
                class,
                visual as *mut x11::xlib::Visual,
                valuemask,
                attributes as *mut x11::xlib::XSetWindowAttributes,
            )
        }
    }

    pub fn default_depth(display: &mut x11::xlib::Display, number: i32) -> i32 {
        unsafe { x11::xlib::XDefaultDepth(display as *mut x11::xlib::Display, number) }
    }

    pub fn default_visual(display: &mut x11::xlib::Display, number: i32) -> x11::xlib::Visual {
        unsafe { *x11::xlib::XDefaultVisual(display as *mut x11::xlib::Display, number) }
    }

    pub fn query_tree(display: &mut x11::xlib::Display, w: u64) -> (u64, u64, Vec<u64>) {
        unsafe {
            let mut root_return: u64 = 0;
            let mut parent_return: u64 = 0;
            let mut nchildren_return: u32 = 0;
            let mut children_return: *mut u64 = std::ptr::null_mut::<u64>();

            x11::xlib::XQueryTree(
                display as *mut x11::xlib::Display,
                w,
                &mut root_return as *mut u64,
                &mut parent_return as *mut u64,
                &mut children_return as *mut *mut u64,
                &mut nchildren_return as *mut u32,
            );

            (
                0,
                0,
                std::slice::from_raw_parts_mut(children_return, nchildren_return as usize).to_vec(),
            )
        }
    }

    pub fn grab_server(display: &mut x11::xlib::Display) {
        unsafe {
            x11::xlib::XGrabServer(display as *mut x11::xlib::Display);
        }
    }

    pub fn ungrab_server(display: &mut x11::xlib::Display) {
        unsafe {
            x11::xlib::XUngrabServer(display as *mut x11::xlib::Display);
        }
    }

    pub fn set_close_down_mode(display: &mut x11::xlib::Display, mode: i32) {
        unsafe {
            x11::xlib::XSetCloseDownMode(display as *mut x11::xlib::Display, mode);
        }
    }

    pub fn get_window_attributes(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<x11::xlib::XWindowAttributes> {
        unsafe {
            let mut wa: x11::xlib::XWindowAttributes = x11::xlib::XWindowAttributes {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                border_width: 0,
                depth: 0,
                visual: std::ptr::null_mut(),
                root: 0,
                class: 0,
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                save_under: 0,
                colormap: 0,
                map_installed: 0,
                map_state: 0,
                all_event_masks: 0,
                your_event_mask: 0,
                do_not_propagate_mask: 0,
                override_redirect: 0,
                screen: std::ptr::null_mut(),
            };
            if x11::xlib::XGetWindowAttributes(
                display as *mut x11::xlib::Display,
                w,
                &mut wa as *mut x11::xlib::XWindowAttributes,
            ) != 0
            {
                Some(wa)
            } else {
                None
            }
        }
    }

    pub fn get_wm_protocols(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<Vec<x11::xlib::Atom>> {
        unsafe {
            let mut protocols_return: *mut x11::xlib::Atom = std::ptr::null_mut::<u64>();
            let mut count_return: i32 = 0;
            if x11::xlib::XGetWMProtocols(
                display as *mut x11::xlib::Display,
                w,
                &mut protocols_return as *mut *mut x11::xlib::Atom,
                &mut count_return as *mut i32,
            ) != 0
            {
                Some(std::slice::from_raw_parts(protocols_return, count_return as usize).to_vec())
            } else {
                None
            }
        }
    }

    pub fn intern_atom(
        display: &mut x11::xlib::Display,
        atom_name: String,
        oie: bool,
    ) -> x11::xlib::Atom {
        unsafe {
            let name_ptr = std::ffi::CString::new(atom_name).unwrap();
            x11::xlib::XInternAtom(
                display as *mut x11::xlib::Display,
                name_ptr.as_ptr(),
                oie as i32,
            )
        }
    }

    pub fn send_event(
        display: &mut x11::xlib::Display,
        w: u64,
        p: bool,
        event_mask: i64,
        event: EEvent,
    ) -> bool {
        unsafe {
            let mut xe = XEvent { type_: 0 };
            match event {
                EEvent::KeyPress { key } => {
                    xe.type_ = KeyPress;
                    xe.key = key
                }
                EEvent::KeyRelease { key } => {
                    xe.type_ = KeyRelease;
                    xe.key = key
                }
                EEvent::ButtonPress { button, motion } => {
                    xe.type_ = ButtonPress;
                    xe.button = button;
                    xe.motion = motion
                }
                EEvent::ButtonRelease { button, motion } => {
                    xe.type_ = ButtonRelease;
                    xe.button = button;
                    xe.motion = motion
                }
                EEvent::MotionNotify { button, motion } => {
                    xe.type_ = MotionNotify;
                    xe.button = button;
                    xe.motion = motion
                }
                EEvent::MapRequest { map_request_event } => {
                    xe.type_ = MapRequest;
                    xe.map_request = map_request_event
                }
                EEvent::EnterNotify { crossing } => {
                    xe.type_ = EnterNotify;
                    xe.crossing = crossing
                }
                EEvent::LeaveNotify { crossing } => {
                    xe.type_ = LeaveNotify;
                    xe.crossing = crossing
                }
                EEvent::DestroyNotify { destroy_window } => {
                    xe.type_ = DestroyNotify;
                    xe.destroy_window = destroy_window
                }
                EEvent::UnmapNotify { unmap } => {
                    xe.type_ = UnmapNotify;
                    xe.unmap = unmap
                }
                EEvent::PropertyNotify { property } => {
                    xe.type_ = PropertyNotify;
                    xe.property = property
                }
                EEvent::ConfigureNotify { configure } => {
                    xe.type_ = ConfigureNotify;
                    xe.configure = configure
                }
                EEvent::ClientMessage {
                    client_message_event,
                } => {
                    xe.type_ = ClientMessage;
                    xe.client_message = client_message_event
                }
                EEvent::ConfigureRequest {
                    configure_request_event,
                } => {
                    xe.type_ = ConfigureRequest;
                    xe.configure_request = configure_request_event
                }
                EEvent::Unmanaged { .. } => {}
            };

            x11::xlib::XSendEvent(
                display as *mut x11::xlib::Display,
                w,
                p as i32,
                event_mask,
                &mut xe as *mut XEvent,
            ) != 0
        }
    }

    pub fn get_transient_for_hint(
        display: &mut x11::xlib::Display,
        w: u64,
        prop_window_return: &mut u64,
    ) -> i32 {
        unsafe {
            x11::xlib::XGetTransientForHint(
                display as *mut x11::xlib::Display,
                w,
                prop_window_return as *mut u64,
            )
        }
    }

    pub fn next_event(display: &mut x11::xlib::Display) -> EEvent {
        unsafe {
            let mut ev: XEvent = XEvent { type_: 0 };
            x11::xlib::XNextEvent(display as *mut x11::xlib::Display, &mut ev as *mut XEvent);
            match ev.type_ {
                x11::xlib::KeyPress => EEvent::KeyPress { key: ev.key },
                x11::xlib::KeyRelease => EEvent::KeyRelease { key: ev.key },
                x11::xlib::ButtonPress => EEvent::ButtonPress {
                    button: ev.button,
                    motion: ev.motion,
                },
                x11::xlib::ButtonRelease => EEvent::ButtonRelease {
                    button: ev.button,
                    motion: ev.motion,
                },
                x11::xlib::MotionNotify => EEvent::MotionNotify {
                    button: ev.button,
                    motion: ev.motion,
                },
                x11::xlib::MapRequest => EEvent::MapRequest {
                    map_request_event: ev.map_request,
                },
                x11::xlib::EnterNotify => EEvent::EnterNotify {
                    crossing: ev.crossing,
                },
                x11::xlib::LeaveNotify => EEvent::LeaveNotify {
                    crossing: ev.crossing,
                },
                x11::xlib::DestroyNotify => EEvent::DestroyNotify {
                    destroy_window: ev.destroy_window,
                },
                x11::xlib::UnmapNotify => EEvent::UnmapNotify { unmap: ev.unmap },
                x11::xlib::PropertyNotify => EEvent::PropertyNotify {
                    property: ev.property,
                },
                x11::xlib::ConfigureNotify => EEvent::ConfigureNotify {
                    configure: ev.configure,
                },
                x11::xlib::ClientMessage => EEvent::ClientMessage {
                    client_message_event: ev.client_message,
                },
                x11::xlib::ConfigureRequest => EEvent::ConfigureRequest {
                    configure_request_event: ev.configure_request,
                },
                _ => EEvent::Unmanaged {
                    type_: ev.type_,
                    name: EVENT_LOOKUP[ev.type_ as usize],
                },
            }
        }
    }

    pub fn get_wm_normal_hints(
        display: &mut x11::xlib::Display,
        w: u64,
    ) -> Option<(x11::xlib::XSizeHints, i64)> {
        unsafe {
            let mut supplied_return: i64 = 0;
            let mut hints_return: x11::xlib::XSizeHints =
                std::mem::MaybeUninit::zeroed().assume_init();
            if x11::xlib::XGetWMNormalHints(
                display as *mut x11::xlib::Display,
                w,
                &mut hints_return as *mut x11::xlib::XSizeHints,
                &mut supplied_return as *mut i64,
            ) != 0
            {
                Some((hints_return, supplied_return))
            } else {
                None
            }
        }
    }

    pub fn move_resize_window(
        display: &mut x11::xlib::Display,
        w: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) {
        unsafe {
            x11::xlib::XMoveResizeWindow(
                display as *mut x11::xlib::Display,
                w,
                x,
                y,
                width,
                height,
            );
        }
    }

    pub fn get_atom_name(display: &mut x11::xlib::Display, a: Atom) -> String {
        unsafe {
            let ret = x11::xlib::XGetAtomName(display as *mut x11::xlib::Display, a);
            let sr = std::ffi::CString::from_raw(ret).into_string();
            match sr {
                Ok(s) => s,
                Err(_) => "Invalid Atom".to_string(),
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_window_border_width(display: &mut x11::xlib::Display, w: u64, width: u32) {
        unsafe {
            x11::xlib::XSetWindowBorderWidth(display as *mut x11::xlib::Display, w, width);
        }
    }

    pub fn raise_window(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XRaiseWindow(display as *mut x11::xlib::Display, w);
        }
    }

    #[allow(dead_code)]
    pub fn x_kill_client(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XKillClient(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn set_input_focus(
        display: &mut x11::xlib::Display,
        focus: u64,
        revert_to: i32,
        time: u64,
    ) {
        unsafe {
            x11::xlib::XSetInputFocus(display as *mut x11::xlib::Display, focus, revert_to, time);
        }
    }

    pub fn map_window(display: &mut x11::xlib::Display, w: u64) {
        unsafe {
            x11::xlib::XMapWindow(display as *mut x11::xlib::Display, w);
        }
    }

    pub fn keysym_to_keycode(display: &mut x11::xlib::Display, keysym: u32) -> u32 {
        unsafe {
            x11::xlib::XKeysymToKeycode(display as *mut x11::xlib::Display, keysym as u64) as u32
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn change_property(
        display: &mut x11::xlib::Display,
        w: u64,
        property: x11::xlib::Atom,
        type_: x11::xlib::Atom,
        format: i32,
        mode: i32,
        data: *mut u8,
        nelements: i32,
    ) {
        unsafe {
            x11::xlib::XChangeProperty(
                display as *mut x11::xlib::Display,
                w,
                property,
                type_,
                format,
                mode,
                data,
                nelements,
            );
        }
    }

    pub fn get_text_property(
        display: &mut x11::xlib::Display,
        w: u64,
        a: x11::xlib::Atom,
    ) -> Option<String> {
        unsafe {
            let mut tr: x11::xlib::XTextProperty = x11::xlib::XTextProperty {
                value: std::ptr::null_mut::<u8>(),
                encoding: 0,
                format: 0,
                nitems: 0,
            };
            let mut strings_return = std::ptr::null_mut::<*mut i8>();
            let mut amount = 0;

            if x11::xlib::XGetTextProperty(
                display as *mut x11::xlib::Display,
                w,
                &mut tr as *mut x11::xlib::XTextProperty,
                a,
            ) == 0
                || tr.nitems == 0
            {
                return None;
            }

            let mut name: Option<String> = None;

            if tr.encoding == x11::xlib::XA_STRING {
                name = Some(
                    match std::ffi::CStr::from_ptr(tr.value as *const i8).to_string_lossy() {
                        std::borrow::Cow::Borrowed(s) => s.to_string(),
                        std::borrow::Cow::Owned(s) => s,
                    },
                );
            } else if x11::xlib::XmbTextPropertyToTextList(
                display as *mut x11::xlib::Display,
                &mut tr as *mut x11::xlib::XTextProperty,
                &mut strings_return as *mut *mut *mut i8,
                &mut amount as *mut i32,
            ) >= x11::xlib::Success as i32
                && amount > 0
                && !(*strings_return).is_null()
            {
                name = Some(
                    match std::ffi::CStr::from_ptr(*strings_return).to_string_lossy() {
                        std::borrow::Cow::Borrowed(s) => s.to_string(),
                        std::borrow::Cow::Owned(s) => s,
                    },
                );

                x11::xlib::XFreeStringList(strings_return);
            }

            x11::xlib::XFree(tr.value as *mut libc::c_void);
            name
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn get_window_property(
        dpy: &mut x11::xlib::Display,
        win: u64,
        property: x11::xlib::Atom,
        long_offset: i64,
        long_length: i64,
        delete: bool,
        req_type: x11::xlib::Atom,
        actual_return_type: &mut x11::xlib::Atom,
        actual_format_return: &mut i32,
        nitemns_return: &mut u64,
        bytes_after_return: &mut u64,
        prop_return: &mut *mut u8,
    ) -> i32 {
        unsafe {
            x11::xlib::XGetWindowProperty(
                dpy,
                win,
                property,
                long_offset,
                long_length,
                delete as i32,
                req_type,
                actual_return_type,
                actual_format_return,
                nitemns_return,
                bytes_after_return,
                prop_return as *mut *mut u8,
            )
        }
    }

    pub fn get_class_hint(
        dpy: &mut x11::xlib::Display,
        win: u64,
        class_hint_return: &mut ClassHint,
    ) -> i32 {
        let mut dummy_class_hint: x11::xlib::XClassHint = x11::xlib::XClassHint {
            res_name: std::ptr::null_mut(),
            res_class: std::ptr::null_mut(),
        };
        let status = unsafe {
            x11::xlib::XGetClassHint(
                dpy as *mut x11::xlib::Display,
                win,
                &mut dummy_class_hint as *mut x11::xlib::XClassHint,
            )
        };
        unsafe {
            class_hint_return.res_name = if dummy_class_hint.res_name.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(dummy_class_hint.res_name)
                        .to_string_lossy()
                        .into_owned(),
                )
            };
            class_hint_return.res_class = if dummy_class_hint.res_class.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr(dummy_class_hint.res_class)
                        .to_string_lossy()
                        .into_owned(),
                )
            };
        }
        status
    }

    pub enum EEvent {
        KeyPress {
            key: x11::xlib::XKeyEvent,
        },
        KeyRelease {
            key: x11::xlib::XKeyEvent,
        },
        ButtonPress {
            button: x11::xlib::XButtonEvent,
            motion: x11::xlib::XMotionEvent,
        },
        ButtonRelease {
            button: x11::xlib::XButtonEvent,
            motion: x11::xlib::XMotionEvent,
        },
        MotionNotify {
            button: x11::xlib::XButtonEvent,
            motion: x11::xlib::XMotionEvent,
        },
        MapRequest {
            map_request_event: x11::xlib::XMapRequestEvent,
        },
        EnterNotify {
            crossing: x11::xlib::XCrossingEvent,
        },
        LeaveNotify {
            crossing: x11::xlib::XCrossingEvent,
        },
        DestroyNotify {
            destroy_window: x11::xlib::XDestroyWindowEvent,
        },
        UnmapNotify {
            unmap: x11::xlib::XUnmapEvent,
        },
        PropertyNotify {
            property: x11::xlib::XPropertyEvent,
        },
        ConfigureNotify {
            configure: x11::xlib::XConfigureEvent,
        },
        ClientMessage {
            client_message_event: x11::xlib::XClientMessageEvent,
        },
        ConfigureRequest {
            configure_request_event: x11::xlib::XConfigureRequestEvent,
        },
        Unmanaged {
            type_: i32,
            name: &'static str,
        },
    }

    #[derive(Default)]
    pub struct ClassHint {
        pub res_name: Option<String>,
        pub res_class: Option<String>,
    }
}

pub mod xinerama {
    pub fn xinerama_query_screens(
        display: &mut x11::xlib::Display,
    ) -> Option<Vec<x11::xinerama::XineramaScreenInfo>> {
        unsafe {
            let mut screens_amount: i32 = 0;
            x11::xinerama::XineramaQueryScreens(
                display as *mut x11::xlib::Display,
                &mut screens_amount as *mut i32,
            )
            .as_mut()
            .map(|xqs| std::slice::from_raw_parts_mut(xqs, screens_amount as usize).to_vec())
        }
    }
}
