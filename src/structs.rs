//! All newly defined structs used by window manager

use std::ffi::CString;

use crate::config::NUMBER_OF_DESKTOPS;

pub struct Application {
    pub config: Configuration,
    pub core: WmCore,
    pub runtime: Runtime,
    pub atoms: Atoms,
}

pub struct Configuration {
    pub key_actions: Vec<KeyAction>,
    pub gap_width: usize,
    pub border_size: usize,
    pub normal_border_color: Color,
    pub active_border_color: Color,
    pub urgent_border_color: Color,
    pub desktops: DesktopsConfig,
    pub autostart: Vec<AutostartRuleCMD>,
    pub placements: Vec<PlacementRule>,
}

#[derive(Debug, Clone)]
pub struct AutostartRuleCMD {
    pub cmd: Vec<CString>,
    pub rule: Option<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct PlacementRule {
    pub instance: Option<String>,
    pub class: Option<String>,
    pub title: Option<String>,
    pub rule_screen: Option<usize>,
    pub rule_workspace: Option<usize>,
}

#[derive(Clone)]
pub struct KeyAction {
    pub keysym: u32,
    pub modifier: u32,
    pub result: ActionResult,
}

#[derive(Debug, Clone)]
pub enum ActionResult {
    // Global management
    KillClient,
    Spawn(Vec<CString>),
    DumpInfo,
    Quit,
    // Screen management
    MoveToScreen(ScreenSwitching),
    FocusOnScreen(ScreenSwitching),
    // Workspace management
    MoveToWorkspace(u64),
    FocusOnWorkspace(u64),
    SetArrangeEngine(ArrangeEngine),
    // Tiled engine 
    UpdateMasterCapacity(i64),
    UpdateMasterWidth(f64),
    RotateStack,     // Pops and pushes LAST window
    PopPushStack,    // Pops and pushes CURRENT window
    // Any Engine
    ToggleFloat,
    CycleStack(i64), // Simply moves focus
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenSwitching {
    Next,
    Previous,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub struct DesktopsConfig {
    pub keysyms: [u32; NUMBER_OF_DESKTOPS],
    pub names: Vec<[String; NUMBER_OF_DESKTOPS]>,
    pub splits: Vec<[f64; NUMBER_OF_DESKTOPS]>,
}

impl DesktopsConfig {
    pub fn new() -> DesktopsConfig {
        DesktopsConfig {
            keysyms: [0; NUMBER_OF_DESKTOPS],
            names: vec![],
            splits: vec![],
        }
    }
}

impl Default for DesktopsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Atoms {
    pub utf8string: u64,
    pub wm_protocols: u64,
    pub wm_delete: u64,
    pub wm_state: u64,
    pub wm_take_focus: u64,
    pub wm_name: u64,
    pub net_active_window: u64,
    pub net_supported: u64,
    pub net_wm_name: u64,
    pub net_wm_state: u64,
    pub net_wm_state_demands_attention: u64,
    pub net_wm_check: u64,
    pub net_wm_fullscreen: u64,
    pub net_wm_window_type: u64,
    pub net_wm_window_type_dialog: u64,
    pub net_wm_window_type_dock: u64,
    pub net_client_list: u64,
    pub net_number_of_desktops: u64,
    pub net_current_desktop: u64,
    pub net_wm_desktop: u64,
    pub net_desktop_names: u64,
    pub net_desktop_viewport: u64,
    pub net_wm_pid: u64,
}

pub struct WmCore {
    pub display: &'static mut x11::xlib::Display,
    pub root_win: u64,
    pub wm_check_win: u64,
    pub running: bool,
}

impl std::fmt::Debug for WmCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WmCore")
            .field("display", &"Can't be printed")
            .field("root_win", &self.root_win)
            .field("wm_check_win", &self.wm_check_win)
            .field("running", &self.running)
            .finish()
    }
}

#[derive(Debug)]
pub struct Runtime {
    pub screens: Vec<Screen>,
    pub current_screen: usize,
    pub current_workspace: usize,
    pub current_client: Option<usize>,
    pub mouse_state: MouseState, // win, button, pos
    pub bars: Vec<Bar>, // Not in screens since logically bars are not limited to specific screen
    pub autostart_rules: Vec<AutostartRulePID>,
}

#[derive(Debug)]
pub struct AutostartRulePID {
    pub pid: i32,
    pub screen: usize,
    pub workspace: usize,
}

#[derive(Debug)]
pub struct Screen {
    pub number: i64,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize,
    pub bar_offsets: BarOffsets,
}

#[derive(Debug, Clone)]
pub enum ArrangeEngine {
    Tiled,
    Mono    
}

#[derive(Debug)]
pub struct Workspace {
    pub number: u64,
    pub clients: Vec<Client>,
    pub current_client: Option<usize>,
    pub master_capacity: i64,
    pub master_width: f64,
    pub arrange: ArrangeEngine,
}

#[derive(Debug, Default)]
pub struct Client {
    // Basic info
    pub window_id: u64,
    pub window_name: String,
    // Geometry
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub ow: u32,
    pub oh: u32,
    pub ox: i32,
    pub oy: i32,
    pub border: u32,
    // Flags
    pub visible: bool,
    pub floating: bool,
    pub fullscreen: bool,
    pub fixed: bool,
    pub urgent: bool,
    // Restrictions
    pub minw: i32,
    pub minh: i32,
    pub maxw: i32,
    pub maxh: i32,
}

#[derive(Debug)]
pub struct MouseState {
    pub win: u64,
    pub button: u32,
    pub pos: (i64, i64),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BarOffsets {
    pub left: usize,
    pub up: usize,
    pub right: usize,
    pub down: usize,
}

#[derive(Debug, Clone)]
pub struct Bar {
    pub window_id: u64,
    pub x: i64,
    pub y: i64,
    pub w: usize,
    pub h: usize,
}
