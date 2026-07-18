// ======================================================
// 🪟 Back_Windows RemapH V3
// ------------------------------------------------------
// Backend Portable.
//
// Usa:
//   - WH_KEYBOARD_LL
//   - WH_MOUSE_LL
//   - SendInput
//
// No conoce:
//   - Cache.
//   - Runtime.
//   - Remapeos.
//   - Accion.
//
// Solo:
//   - Captura input físico.
//   - Traduce a InputEvent.
//   - Emite InputEvent genérico.
// ======================================================

use std::cell::RefCell;
use std::mem::size_of;

use windows_sys::Win32::Foundation::{
    LPARAM,
    LRESULT,
    WPARAM,
};

use windows_sys::Win32::System::LibraryLoader::{
    GetModuleHandleW,
};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    INPUT,
    INPUT_0,
    INPUT_KEYBOARD,
    INPUT_MOUSE,
    KEYBDINPUT,
    KEYEVENTF_KEYUP,
    MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_WHEEL,
    MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP,
    MOUSEINPUT,
    SendInput,
    VK_0,
    VK_9,
    VK_A,
    VK_BACK,
    VK_CAPITAL,
    VK_ESCAPE,
    VK_F1,
    VK_LCONTROL,
    VK_LMENU,
    VK_LSHIFT,
    VK_NUMLOCK,
    VK_RETURN,
    VK_RCONTROL,
    VK_RMENU,
    VK_RSHIFT,
    VK_SCROLL,
    VK_SPACE,
    VK_TAB,
    VK_OEM_1,
    VK_OEM_2,
    VK_OEM_3,
    VK_OEM_4,
    VK_OEM_5,
    VK_OEM_6,
    VK_OEM_7,
    VK_OEM_COMMA,
    VK_OEM_MINUS,
    VK_OEM_PERIOD,
    VK_OEM_PLUS,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx,
    GetMessageW,
    KBDLLHOOKSTRUCT,
    MSLLHOOKSTRUCT,
    MSG,
    SetWindowsHookExW,
    UnhookWindowsHookEx,
    WH_KEYBOARD_LL,
    WH_MOUSE_LL,
    WM_KEYDOWN,
    WM_KEYUP,
    WM_LBUTTONDOWN,
    WM_LBUTTONUP,
    WM_MBUTTONDOWN,
    WM_MBUTTONUP,
    WM_MOUSEWHEEL,
    WM_RBUTTONDOWN,
    WM_RBUTTONUP,
    WM_SYSKEYDOWN,
    WM_SYSKEYUP,
    WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

use crate::eventos::{
    InputEvent,
    InputId,
    InputState,
};


// ======================================================
// 🧠 PROCESADOR
// ======================================================

type Procesador = Box<dyn FnMut(
    InputEvent,
    &mut dyn FnMut(InputEvent),
) -> bool>;

struct Estado {

    procesar:
        Procesador,

}


// ======================================================
// 🧵 ESTADO DEL HILO
// ======================================================

thread_local! {

    static ESTADO:

        RefCell<Option<Estado>>

        = RefCell::new(None);

}


// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar<F>(

    procesar:
        F,

)
where

    F: FnMut(

        InputEvent,

        &mut dyn FnMut(InputEvent),

    ) -> bool

        + Send
        + 'static,

{

    ESTADO.with(

        |estado| {

            *estado.borrow_mut() = Some(

                Estado {

                    procesar:

                        Box::new(

                            procesar

                        ),

                }

            );

        }

    );


    let modulo = unsafe {

        GetModuleHandleW(

            std::ptr::null()

        )

    };


    let teclado = unsafe {

        SetWindowsHookExW(

            WH_KEYBOARD_LL,

            Some(teclado),

            modulo,

            0,

        )

    };


    if teclado.is_null() {

        panic!(

            "No se pudo instalar hook de teclado"

        );

    }


    let mouse = unsafe {

        SetWindowsHookExW(

            WH_MOUSE_LL,

            Some(mouse),

            modulo,

            0,

        )

    };


    if mouse.is_null() {

        unsafe {

            UnhookWindowsHookEx(

                teclado

            );

        }


        panic!(

            "No se pudo instalar hook de mouse"

        );

    }


    println!(

        "🪟 Backend Portable iniciado."

    );


    let mut mensaje:

        MSG = unsafe {

            std::mem::zeroed()

        };


    loop {

        let resultado = unsafe {

            GetMessageW(

                &mut mensaje,

                std::ptr::null_mut(),

                0,

                0,

            )

        };


        if resultado <= 0 {

            break;

        }

    }


    unsafe {

        UnhookWindowsHookEx(

            teclado

        );

        UnhookWindowsHookEx(

            mouse

        );

    }


    ESTADO.with(

        |estado| {

            *estado.borrow_mut() =
                None;

        }

    );

}


// ======================================================
// 🎹 HOOK TECLADO
// ======================================================

unsafe extern "system" fn teclado(

    codigo:
        i32,

    wparam:
        WPARAM,

    lparam:
        LPARAM,

) -> LRESULT {

    if codigo < 0 {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    }


    let datos =

        &*(

            lparam

                as *const KBDLLHOOKSTRUCT

        );


    if datos.flags & 0x10 != 0 {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    }


    let presionado =

        wparam
            == WM_KEYDOWN as usize

        || wparam
            == WM_SYSKEYDOWN as usize;


    let liberado =

        wparam
            == WM_KEYUP as usize

        || wparam
            == WM_SYSKEYUP as usize;


    if !presionado
        && !liberado
    {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    }

    let Some(evento) =

        traducir_teclado(

            datos.vkCode,

            datos.scanCode,

            datos.flags,

            presionado,

        )

    else {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    };


    if evaluar(evento) {

        return 1;

    }


    CallNextHookEx(

        std::ptr::null_mut(),

        codigo,

        wparam,

        lparam,

    )

}


// ======================================================
// 🖱️ HOOK MOUSE
// ======================================================

unsafe extern "system" fn mouse(

    codigo:
        i32,

    wparam:
        WPARAM,

    lparam:
        LPARAM,

) -> LRESULT {

    if codigo < 0 {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    }


    let datos =

        &*(

            lparam

                as *const MSLLHOOKSTRUCT

        );


    if datos.flags & 0x01 != 0 {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    }


    let Some(evento) =

        traducir_mouse(

            wparam,

            datos,

        )

    else {

        return CallNextHookEx(

            std::ptr::null_mut(),

            codigo,

            wparam,

            lparam,

        );

    };


    if evaluar(evento) {

        return 1;

    }


    CallNextHookEx(

        std::ptr::null_mut(),

        codigo,

        wparam,

        lparam,

    )

}


// ======================================================
// 🧠 EVALUAR
// ======================================================

fn evaluar(

    evento:
        InputEvent,

) -> bool {

    ESTADO.with(

        |estado| {

            let mut estado =

                estado.borrow_mut();


            let Some(actual) =

                estado.as_mut()

            else {

                return false;

            };


            let mut emitir =

                |evento:

                    InputEvent|

                {

                    emitir_evento(

                        evento

                    );

                };


            (actual.procesar)(

                evento,

                &mut emitir,

            )

        }

    )

}


// ======================================================
// 🎹 TRADUCIR TECLADO
// ======================================================

fn traducir_teclado(

    vk:
        u32,

    scan:
        u32,

    flags:
        u32,

    presionado:
        bool,

) -> Option<InputEvent> {

    let control =

        teclado_control(

            vk,

            scan,

            flags,

        )?;


    let input =

        InputId::new(

            "keyboard",

            &control,

        );


    if presionado {

        Some(

            InputEvent::down(

                input

            )

        )

    }

    else {

        Some(

            InputEvent::up(

                input

            )

        )

    }

}


// ======================================================
// 🖱️ TRADUCIR MOUSE
// ======================================================

fn traducir_mouse(

    mensaje:
        WPARAM,

    datos:
        &MSLLHOOKSTRUCT,

) -> Option<InputEvent> {

    let mensaje =
        mensaje as u32;


    if mensaje
        == WM_MOUSEWHEEL
    {

        let delta =

            (

                (

                    datos.mouseData
                        >> 16

                ) as u16

            ) as i16;


        return Some(

            InputEvent::pulse(

                InputId::new(

                    "mouse",

                    if delta > 0 {

                        "WheelUp"

                    }

                    else {

                        "WheelDown"

                    },

                )

            )

        );

    }


    let control =

        match mensaje {

            WM_LBUTTONDOWN
            | WM_LBUTTONUP => "LeftButton",

            WM_RBUTTONDOWN
            | WM_RBUTTONUP => "RightButton",

            WM_MBUTTONDOWN
            | WM_MBUTTONUP => "MiddleButton",

            WM_XBUTTONDOWN
            | WM_XBUTTONUP => {

                match

                    (datos.mouseData >> 16) as u16

                {

                    1 => "Button4",

                    2 => "Button5",

                    _ => return None,

                }

            }

            _ => return None,

        };


    let input =

        InputId::new(

            "mouse",

            control,

        );


    match mensaje {

        WM_LBUTTONDOWN
        | WM_RBUTTONDOWN
        | WM_MBUTTONDOWN
        | WM_XBUTTONDOWN => {

            Some(

                InputEvent::down(

                    input

                )

            )

        }


        WM_LBUTTONUP
        | WM_RBUTTONUP
        | WM_MBUTTONUP
        | WM_XBUTTONUP => {

            Some(

                InputEvent::up(

                    input

                )

            )

        }


        _ => None,

    }

}


// ======================================================
// 🎹 NOMBRE DE TECLA
// ======================================================

fn teclado_control(

    vk:
        u32,

    scan:
        u32,

    flags:
        u32,

) -> Option<String> {

    let vk =
        vk as u16;


    if (VK_A..=VK_A + 25)
        .contains(&vk)
    {

        return Some(

            char::from_u32(

                vk as u32

            )?

            .to_string()

        );

    }


    if (VK_0..=VK_9)
        .contains(&vk)
    {

        return Some(

            format!(

                "Num{}",

                char::from_u32(

                    vk as u32

                )?

            )

        );

    }


    if vk == 0x10 {

        return Some(

            if scan == 0x2A {

                "LeftShift"

            }

            else {

                "RightShift"

            }

            .to_string()

        );

    }


    match vk {

        VK_LCONTROL => {

            Some(

                "LeftControl"

                    .to_string()

            )

        }


        VK_RCONTROL => {

            Some(

                "RightControl"

                    .to_string()

            )

        }


        0x12 => {

            Some(

                if flags & 0x01 != 0 {

                    "RightAlt"

                }

                else {

                    "LeftAlt"

                }

                .to_string()

            )

        }


        VK_RETURN => Some(

            "Enter".to_string()

        ),

        VK_ESCAPE => Some(

            "Esc".to_string()

        ),

        VK_BACK => Some(

            "Backspace".to_string()

        ),

        VK_TAB => Some(

            "Tab".to_string()

        ),

        VK_SPACE => Some(

            "Space".to_string()

        ),

        VK_CAPITAL => Some(

            "CapsLock".to_string()

        ),

        VK_NUMLOCK => Some(

            "NumLock".to_string()

        ),

        VK_SCROLL => Some(

            "ScrollLock".to_string()

        ),

        VK_OEM_MINUS => Some(

            "Minus".to_string()

        ),

        VK_OEM_PLUS => Some(

            "Equals".to_string()

        ),

        VK_OEM_4 => Some(

            "LeftBracket".to_string()

        ),

        VK_OEM_6 => Some(

            "RightBracket".to_string()

        ),

        VK_OEM_5 => Some(

            "BackSlash".to_string()

        ),

        VK_OEM_1 => Some(

            "SemiColon".to_string()

        ),

        VK_OEM_7 => Some(

            "Apostrophe".to_string()

        ),

        VK_OEM_3 => Some(

            "Grave".to_string()

        ),

        VK_OEM_COMMA => Some(

            "Comma".to_string()

        ),

        VK_OEM_PERIOD => Some(

            "Period".to_string()

        ),

        VK_OEM_2 => Some(

            "Slash".to_string()

        ),

        _ => {

            if vk >= VK_F1
                && vk <= VK_F1 + 11
            {

                Some(

                    format!(

                        "F{}",

                        vk - VK_F1 + 1

                    )

                )

            }

            else {

                None

            }

        }

    }

}


// ======================================================
// 📤 EMITIR INPUT
// ======================================================

pub fn emitir_evento(

    evento:
        InputEvent,

) {

    let Some(control) =

        evento.input.control()

    else {

        return;

    };


    match evento.input.fuente() {

        Some("keyboard") => {

            let Some(vk) =

                tecla_vk(control)

            else {

                return;

            };


            match evento.state {

                InputState::Down => {

                    emitir_teclado(

                        vk,

                        false,

                    );

                }


                InputState::Up => {

                    emitir_teclado(

                        vk,

                        true,

                    );

                }


                InputState::Pulse => {

                    emitir_teclado(

                        vk,

                        false,

                    );


                    emitir_teclado(

                        vk,

                        true,

                    );

                }

            }

        }


        Some("mouse") => {

            emitir_mouse(

                control,

                evento.state,

            );

        }


        _ => {}

    }

}


// ======================================================
// 🎹 EMITIR TECLADO
// ======================================================

fn emitir_teclado(

    vk:
        u16,

    arriba:
        bool,

) {

    let flags =

        if arriba {

            KEYEVENTF_KEYUP

        }

        else {

            0

        };


    let input =

        INPUT {

            r#type:
                INPUT_KEYBOARD,

            Anonymous:

                INPUT_0 {

                    ki:

                        KEYBDINPUT {

                            wVk:
                                vk,

                            wScan:
                                0,

                            dwFlags:
                                flags,

                            time:
                                0,

                            dwExtraInfo:
                                0,

                        },

                },

        };


    enviar(input);

}


// ======================================================
// 🖱️ EMITIR MOUSE
// ======================================================

fn emitir_mouse(

    control:
        &str,

    estado:
        InputState,

) {

    if control == "WheelUp"
        || control == "WheelDown"
    {

        let delta:

            i32 =

            if control == "WheelUp" {

                120

            }

            else {

                -120

            };


        let input =

            INPUT {

                r#type:
                    INPUT_MOUSE,

                Anonymous:

                    INPUT_0 {

                        mi:

                            MOUSEINPUT {

                                dx:
                                    0,

                                dy:
                                    0,

                                mouseData:
                                    delta as u32,

                                dwFlags:
                                    MOUSEEVENTF_WHEEL,

                                time:
                                    0,

                                dwExtraInfo:
                                    0,

                            },

                    },

            };


        enviar(input);

        return;

    }


    let Some((down, up)) =

        mouse_flags(control)

    else {

        return;

    };


    match estado {

        InputState::Down => {

            emitir_mouse_button(

                control,

                down,

            );

        }


        InputState::Up => {

            emitir_mouse_button(

                control,

                up,

            );

        }


        InputState::Pulse => {

            emitir_mouse_button(

                control,

                down,

            );


            emitir_mouse_button(

                control,

                up,

            );

        }

    }

}


// ======================================================
// 🖱️ EMITIR BOTÓN
// ======================================================

fn emitir_mouse_button(

    control:
        &str,

    flags:
        u32,

) {

    let input =

        INPUT {

            r#type:
                INPUT_MOUSE,

            Anonymous:

                INPUT_0 {

                    mi:

                        MOUSEINPUT {

                            dx:
                                0,

                            dy:
                                0,

                            mouseData:

                                if control == "Button4" {

                                    1

                                }

                                else if control == "Button5" {

                                    2

                                }

                                else {

                                    0

                                },

                            dwFlags:
                                flags,

                            time:
                                0,

                            dwExtraInfo:
                                0,

                        },

                },

        };


    enviar(input);

}


// ======================================================
// 🎹 TECLA → VK
// ======================================================

fn tecla_vk(

    control:
        &str,

) -> Option<u16> {

    if control.len() == 1 {

        let byte =
            control.as_bytes()[0];


        if byte >= b'A'
            && byte <= b'Z'
        {

            return Some(

                byte as u16

            );

        }

    }


    if control.starts_with("Num")
        && control.len() == 4
    {

        let numero =
            control.as_bytes()[3];


        if numero >= b'0'
            && numero <= b'9'
        {

            return Some(

                VK_0
                    + (numero - b'0') as u16

            );

        }

    }


    match control {

        "Enter" =>
            Some(VK_RETURN),

        "Esc" =>
            Some(VK_ESCAPE),

        "Backspace" =>
            Some(VK_BACK),

        "Tab" =>
            Some(VK_TAB),

        "Space" =>
            Some(VK_SPACE),

        "CapsLock" =>
            Some(VK_CAPITAL),

        "NumLock" =>
            Some(VK_NUMLOCK),

        "ScrollLock" =>
            Some(VK_SCROLL),

        "LeftControl" =>
            Some(VK_LCONTROL),

        "RightControl" =>
            Some(VK_RCONTROL),

        "LeftShift" =>
            Some(VK_LSHIFT),

        "RightShift" =>
            Some(VK_RSHIFT),

        "LeftAlt" =>
            Some(VK_LMENU),

        "RightAlt" =>
            Some(VK_RMENU),

        "Minus" =>
            Some(VK_OEM_MINUS),

        "Equals" =>
            Some(VK_OEM_PLUS),

        "LeftBracket" =>
            Some(VK_OEM_4),

        "RightBracket" =>
            Some(VK_OEM_6),

        "BackSlash" =>
            Some(VK_OEM_5),

        "SemiColon" =>
            Some(VK_OEM_1),

        "Apostrophe" =>
            Some(VK_OEM_7),

        "Grave" =>
            Some(VK_OEM_3),

        "Comma" =>
            Some(VK_OEM_COMMA),

        "Period" =>
            Some(VK_OEM_PERIOD),

        "Slash" =>
            Some(VK_OEM_2),

        _ => {

            if let Some(numero) =

                control
                    .strip_prefix("F")
                    .and_then(

                        |valor|

                            valor
                                .parse::<u16>()
                                .ok()

                    )

            {

                if numero >= 1
                    && numero <= 12
                {

                    return Some(

                        VK_F1
                            + numero
                            - 1

                    );

                }

            }


            None

        }

    }

}


// ======================================================
// 🖱️ FLAGS MOUSE
// ======================================================

fn mouse_flags(

    control:
        &str,

) -> Option<(u32, u32)> {

    match control {

        "LeftButton" => Some(

            (

                MOUSEEVENTF_LEFTDOWN,

                MOUSEEVENTF_LEFTUP,

            )

        ),

        "RightButton" => Some(

            (

                MOUSEEVENTF_RIGHTDOWN,

                MOUSEEVENTF_RIGHTUP,

            )

        ),

        "MiddleButton" => Some(

            (

                MOUSEEVENTF_MIDDLEDOWN,

                MOUSEEVENTF_MIDDLEUP,

            )

        ),

        "Button4" => Some(

            (

                MOUSEEVENTF_XDOWN,

                MOUSEEVENTF_XUP,

            )

        ),

        "Button5" => Some(

            (

                MOUSEEVENTF_XDOWN,

                MOUSEEVENTF_XUP,

            )

        ),

        _ => None,

    }

}


// ======================================================
// 📤 SEND INPUT
// ======================================================

fn enviar(

    input:
        INPUT,

) {

    unsafe {

        SendInput(

            1,

            &input,

            size_of::<INPUT>() as i32,

        );

    }

}