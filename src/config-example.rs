//! Configuration file for editing user defined settings

// Some essential imports
use std::ffi::CString;

use crate::structs::ActionResult::*;
use crate::structs::AutostartRuleCMD;
use crate::structs::Color;
use crate::structs::Configuration;
use crate::structs::DesktopsConfig;
use crate::structs::KeyAction;
use crate::structs::PlacementRule;
use crate::structs::ScreenSwitching;

use x11::keysym::*;
use x11::xlib::Mod4Mask as ModKey;
use x11::xlib::ShiftMask;

//-----------------------------------------------------------------------
//                          General rules
//-----------------------------------------------------------------------

pub const NUMBER_OF_DESKTOPS: usize = 11;
pub const FOCUS_IGNORES_GEOMETRY: bool = true;

/// Function for cunfiguring everything(actually not everything) you need
pub fn config() -> Configuration {
    //-----------------------------------------------------------------------
    //                          Local Macro Definitions
    //-----------------------------------------------------------------------
    // Macro for creating autostart rules
    macro_rules! AUTOSTART {
        ($cmd:expr) => {
            AutostartRuleCMD {
                cmd: $cmd,
                rule: None,
            }
        };
        ($cmd:expr, $s:expr, $w:expr) => {
            AutostartRuleCMD {
                cmd: $cmd,
                rule: Some(($s, $w)),
            }
        };
    }
    // Macro for creating array of strings used by nix's execvp function
    macro_rules! CMD {
        ( $( $e:expr ),* ) => {
            vec![$( CString::new($e).unwrap() ),*]
        };
    }

    fn mk_placement<S: Into<String>>(
        instance: Option<S>,
        class: Option<S>,
        title: Option<S>,
        rule_screen: Option<usize>,
        rule_workspace: Option<usize>,
    ) -> PlacementRule {
        PlacementRule {
            instance: instance.map(|s| s.into()),
            class: class.map(|s| s.into()),
            title: title.map(|s| s.into()),
            rule_screen,
            rule_workspace,
        }
    }

    //-----------------------------------------------------------------------
    //                               Visuals
    //-----------------------------------------------------------------------
    let gap_width = 4;
    let border_size = 2;
    let normal_border_color = Color {
        //#404080
        alpha: 255,
        red: 64,
        green: 64,
        blue: 128,
    };
    let active_border_color = Color {
        //#7e2487
        alpha: 255,
        red: 126,
        green: 36,
        blue: 135,
    };
    let urgent_border_color = Color {
        //#A54242
        alpha: 255,
        red: 186,
        green: 28,
        blue: 28,
    };

    //-----------------------------------------------------------------------
    //                          Shortcuts setup
    //-----------------------------------------------------------------------
    let terminal = CMD!("alacritty");
    let file_manager = CMD!("thunar");
    let app_launcher = CMD!("dmenu_run", "-p", "Open app:", "-b");

    let mut key_actions = vec![
        KeyAction {
            modifier: ModKey,
            keysym: XK_Return,
            result: Spawn(terminal),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_e,
            result: Spawn(file_manager),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_p,
            result: Spawn(app_launcher),
        },
        // You actually can find these scripts here https://github.com/pavtiger/my-dwm-desktop-enviroment/tree/master/scripts
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioRaiseVolume,
            result: Spawn(CMD!("volumeup")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioLowerVolume,
            result: Spawn(CMD!("volumedown")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioMute,
            result: Spawn(CMD!("volumemute")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioPlay,
            result: Spawn(CMD!("playerctl play-pause")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioNext,
            result: Spawn(CMD!("playerctl next")),
        },
        KeyAction {
            modifier: 0,
            keysym: XF86XK_AudioPrev,
            result: Spawn(CMD!("playerctl previous")),
        },
        KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: XK_q,
            result: Quit,
        },
        KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: XK_c,
            result: KillClient,
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_w,
            result: DumpInfo,
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_comma,
            result: FocusOnScreen(ScreenSwitching::Previous),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_period,
            result: FocusOnScreen(ScreenSwitching::Next),
        },
        KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: XK_comma,
            result: MoveToScreen(ScreenSwitching::Previous),
        },
        KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: XK_period,
            result: MoveToScreen(ScreenSwitching::Next),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_i,
            result: UpdateMasterCapacity(1),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_d,
            result: UpdateMasterCapacity(-1),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_l,
            result: UpdateMasterWidth(0.05),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_h,
            result: UpdateMasterWidth(-0.05),
        },
        KeyAction {
            modifier: ModKey | ShiftMask | ControlMask,
            keysym: XK_Return,
            result: RotateStack,
        },
        KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: XK_space,
            result: ToggleFloat,
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_j,
            result: CycleStack(-1),
        },
        KeyAction {
            modifier: ModKey,
            keysym: XK_k,
            result: CycleStack(1),
        },
    ];

    //-----------------------------------------------------------------------
    //                          Desktops Setup
    //-----------------------------------------------------------------------
    let mut desktops = DesktopsConfig::new();

    desktops.keysyms = [XK_1, XK_2, XK_3, XK_4, XK_5, XK_6, XK_7, XK_8, XK_9, XK_0];

    desktops.names =
        vec![["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"].map(|s| s.to_string())];

    // desktops.splits is a vector of arrays of NUMBER_OF_DESKTOPS float numbers where
    // each array corresponds to screen and each float corresponds to split on workspace
    // so: desktops.splits[screen index][workspace index] = split width
    desktops.splits = vec![[0.5; NUMBER_OF_DESKTOPS]]
    desktops.splits[0][0] = 0.7;

    for (i, k) in desktops.keysyms.iter().enumerate() {
        key_actions.push(KeyAction {
            modifier: ModKey,
            keysym: *k,
            result: FocusOnWorkspace(i as u64),
        });
        key_actions.push(KeyAction {
            modifier: ModKey | ShiftMask,
            keysym: *k,
            result: MoveToWorkspace(i as u64),
        });
    }

    //-----------------------------------------------------------------------
    //                        Autostart setup
    //-----------------------------------------------------------------------
    let autostart = vec![
        // Positioned
        AUTOSTART!(CMD!("alacritty"), 0, 0),
        AUTOSTART!(CMD!("firefox"), 0, 1),
        AUTOSTART!(CMD!("telegram-desktop"), 0, 3),
        // Cli
        AUTOSTART!(CMD!("picom")),
        AUTOSTART!(CMD!("polybar")),
        AUTOSTART!(CMD!(
            "setxkbmap",
            "us,ru",
            "-option",
            "grp:win_space_toggle"
        )),
        AUTOSTART!(CMD!(std::env!("HOME").to_owned() + "/.fehbg")),
        AUTOSTART!(CMD!("touch", "/tmp/rtwmrunning")),
    ];

    //-----------------------------------------------------------------------
    //                       Permanent rules setup
    //-----------------------------------------------------------------------

    // xprop(1):
    //  WM_CLASS(STRING) = instance, class
    //  WM_NAME(STRING) = title
    // mk_placement(instance, class, title, rule_screen, rule_workspace)

    let placements: Vec<PlacementRule> = vec![
        // mk_placement(None, Some("zen"), None, Some(0), Some(1)),
        // mk_placement(None, Some("Thunar"), None, Some(0), Some(2)),
        mk_placement(None, Some("pavucontrol"), None, Some(0), Some(9)),
        mk_placement(None, Some("Arandr"), None, Some(0), Some(9)),
    ];

    //-----------------------------------------------------------------------
    //                      Create config & return
    //-----------------------------------------------------------------------
    return Configuration {
        key_actions,
        gap_width,
        border_size,
        normal_border_color,
        active_border_color,
        urgent_border_color,
        desktops,
        autostart,
        placements,
    };
}
