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

use crate::instante;
use crate::pulsadores;
use std::cell::RefCell;
use std::mem::size_of;

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP, MOUSEINPUT,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG,
    MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

use crate::eventos::{InputEvent, InputId, InputState};

// ======================================================
// 🧠 PROCESADOR
// ======================================================

type Procesador = Box<dyn FnMut(InputEvent, &mut dyn FnMut(InputEvent)) -> bool>;

struct Estado {
    procesar: Procesador,
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

pub fn iniciar<F>(procesar: F)
where
    F: FnMut(InputEvent, &mut dyn FnMut(InputEvent)) -> bool + Send + 'static,
{
    println!("[BACK] iniciar()");

    ESTADO.with(|estado| {
        *estado.borrow_mut() = Some(Estado {
            procesar: Box::new(procesar),
        });
    });

    println!("[BACK] Estado creado");

    let modulo = unsafe { GetModuleHandleW(std::ptr::null()) };

    println!("[BACK] Módulo = {:?}", modulo);

    let teclado = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(prueba_hook_teclado),
            std::ptr::null_mut(),
            0,
        )
    };

    println!("[BACK] Hook teclado = {:?}", teclado);

    if teclado.is_null() {
        panic!("No se pudo instalar hook de teclado");
    }

    let mouse =
        unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(hook_mouse), std::ptr::null_mut(), 0) };
    println!("[BACK] Hook mouse = {:?}", mouse);

    if mouse.is_null() {
        unsafe {
            UnhookWindowsHookEx(teclado);
        }

        panic!("No se pudo instalar hook de mouse");
    }

    println!("[BACK] Hooks instalados");
    println!("[BACK] Entrando a GetMessage()");

    let mut mensaje: MSG = unsafe { std::mem::zeroed() };

    loop {
        let resultado = unsafe { GetMessageW(&mut mensaje, std::ptr::null_mut(), 0, 0) };

        println!("[BACK] GetMessage -> {}", resultado);

        if resultado <= 0 {
            println!("[BACK] Saliendo del loop");
            break;
        }
    }

    println!("[BACK] Desinstalando hooks");

    unsafe {
        UnhookWindowsHookEx(teclado);
        UnhookWindowsHookEx(mouse);
    }

    ESTADO.with(|estado| {
        *estado.borrow_mut() = None;
    });

    println!("[BACK] Finalizado");
}

// ======================================================
// 🎹 HOOK TECLADO
// ======================================================

unsafe extern "system" fn prueba_hook_teclado(
    codigo: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    println!(
        "[TECLADO RAW] Entró callback. codigo={} wparam={}",
        codigo, wparam
    );

    if codigo < 0 {
        println!("[TECLADO RAW] codigo < 0");

        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let datos = &*(lparam as *const KBDLLHOOKSTRUCT);

    println!(
        "[TECLADO RAW] vk={} scan={} flags={:#X}",
        datos.vkCode, datos.scanCode, datos.flags
    );

    if datos.flags & 0x10 != 0 {
        println!("[TECLADO RAW] Evento inyectado ignorado");

        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let presionado = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;

    let liberado = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

    println!(
        "[TECLADO RAW] presionado={} liberado={}",
        presionado, liberado
    );

    if !presionado && !liberado {
        println!("[TECLADO RAW] Mensaje ignorado");

        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let Some(evento) = traducir_teclado(datos.vkCode, datos.scanCode, datos.flags, presionado)
    else {
        println!("[TECLADO RAW] No pudo traducir teclado");

        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    };

    println!("[TECLADO RAW] Evento traducido -> {:?}", evento);

    if evaluar(evento) {
        println!("[TECLADO RAW] Consumido");

        return 1;
    }

    println!("[TECLADO RAW] Pasando a Windows");

    CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam)
}

// ======================================================
// 🖱️ HOOK MOUSE
// ======================================================

unsafe extern "system" fn hook_mouse(codigo: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if codigo < 0 {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let datos = &*(lparam as *const MSLLHOOKSTRUCT);

    if datos.flags & 0x01 != 0 {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let Some(evento) = traducir_mouse(wparam, datos) else {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    };

    if evaluar(evento) {
        return 1;
    }

    CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam)
}

// ======================================================
// 🧠 EVALUAR
// ======================================================

fn evaluar(evento: InputEvent) -> bool {
    ESTADO.with(|estado| {
        let mut estado = estado.borrow_mut();

        let Some(actual) = estado.as_mut() else {
            return false;
        };

        let mut emitir = |evento: InputEvent| {
            emitir_evento(evento);
        };

        (actual.procesar)(evento, &mut emitir)
    })
}

// ======================================================
// 🎹 TRADUCIR TECLADO
// ======================================================

fn traducir_teclado(vk: u32, scan: u32, flags: u32, presionado: bool) -> Option<InputEvent> {
    let control = teclado_control(vk, scan, flags)?;

    let input = InputId::new("keyboard", &control);

    if presionado {
        Some(InputEvent::down(input, instante::ahora()))
    } else {
        Some(InputEvent::up(input, instante::ahora()))
    }
}

// ======================================================
// 🖱️ TRADUCIR MOUSE
// ======================================================

fn traducir_mouse(mensaje: WPARAM, datos: &MSLLHOOKSTRUCT) -> Option<InputEvent> {
    let mensaje = mensaje as u32;

    if mensaje == WM_MOUSEWHEEL {
        let delta = ((datos.mouseData >> 16) as u16) as i16;

        let nativo = if delta > 0 {
            "0x020A_UP"
        } else {
            "0x020A_DOWN"
        };

        let control = pulsadores::por_nativo(nativo)?.interno.clone();

        return Some(InputEvent::pulse(
            InputId::new("mouse", &control),
            instante::ahora(),
        ));
    }

    let nativo = match mensaje {
        WM_LBUTTONDOWN | WM_LBUTTONUP => "0x0201",

        WM_RBUTTONDOWN | WM_RBUTTONUP => "0x0204",

        WM_MBUTTONDOWN | WM_MBUTTONUP => "0x0207",

        WM_XBUTTONDOWN | WM_XBUTTONUP => match (datos.mouseData >> 16) as u16 {
            1 => "0x020B",

            2 => "0x020C",

            _ => return None,
        },

        _ => return None,
    };

    let control = pulsadores::por_nativo(nativo)?.interno.clone();

    let input = InputId::new("mouse", &control);

    match mensaje {
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
            Some(InputEvent::down(input, instante::ahora()))
        }

        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
            Some(InputEvent::up(input, instante::ahora()))
        }

        _ => None,
    }
}

// ======================================================
// 🎹 NOMBRE DE TECLA
// ------------------------------------------------------
// Windows entrega vkCode.
// pulsadores.tsv contiene:
// nativo -> interno
//
// No traduce.
// Solo consulta diccionario.
// ======================================================

fn teclado_control(vk: u32, scan: u32, flags: u32) -> Option<String> {
    let nativo = format!("0x{:X}", vk);

    println!("[TECLADO] Buscando {}", nativo);

    match pulsadores::por_nativo(&nativo) {
        Some(pulsador) => {
            println!("[TECLADO] Encontrado {}", pulsador.interno);
            Some(pulsador.interno.clone())
        }

        None => {
            println!("[TECLADO] NO encontrado {}", nativo);
            None
        }
    }
}

// ======================================================
// 📤 EMITIR INPUT
// ======================================================

pub fn emitir_evento(evento: InputEvent) {
    let Some(control) = evento.input.control() else {
        return;
    };

    match evento.input.fuente() {
        Some("keyboard") => {
            let Some(nativo) = interno_nativo(control) else {
                return;
            };

            let Some(vk) = nativo
                .strip_prefix("0x")
                .and_then(|valor| u16::from_str_radix(valor, 16).ok())
            else {
                return;
            };

            match evento.state {
                InputState::Down => {
                    emitir_teclado(vk, false);
                }

                InputState::Up => {
                    emitir_teclado(vk, true);
                }

                InputState::Pulse => {
                    emitir_teclado(vk, false);

                    emitir_teclado(vk, true);
                }
            }
        }

        Some("mouse") => {
            emitir_mouse(control, evento.state);
        }

        _ => {}
    }
}

// ======================================================
// 🎹 EMITIR TECLADO
// ======================================================

fn emitir_teclado(vk: u16, arriba: bool) {
    let flags = if arriba { KEYEVENTF_KEYUP } else { 0 };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,

        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,

                wScan: 0,

                dwFlags: flags,

                time: 0,

                dwExtraInfo: 0,
            },
        },
    };

    enviar(input);
}

// ======================================================
// 🖱️ EMITIR MOUSE
// ======================================================

fn emitir_mouse(control: &str, estado: InputState) {
    if control == "WheelUp" || control == "WheelDown" {
        let delta: i32 = if control == "WheelUp" { 120 } else { -120 };

        let input = INPUT {
            r#type: INPUT_MOUSE,

            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,

                    dy: 0,

                    mouseData: delta as u32,

                    dwFlags: MOUSEEVENTF_WHEEL,

                    time: 0,

                    dwExtraInfo: 0,
                },
            },
        };

        enviar(input);

        return;
    }

    let Some((down, up)) = mouse_flags(control) else {
        return;
    };

    match estado {
        InputState::Down => {
            emitir_mouse_button(control, down);
        }

        InputState::Up => {
            emitir_mouse_button(control, up);
        }

        InputState::Pulse => {
            emitir_mouse_button(control, down);

            emitir_mouse_button(control, up);
        }
    }
}

// ======================================================
// 🖱️ EMITIR BOTÓN
// ======================================================

fn emitir_mouse_button(control: &str, flags: u32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,

        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,

                dy: 0,

                mouseData: if control == "Button4" {
                    1
                } else if control == "Button5" {
                    2
                } else {
                    0
                },

                dwFlags: flags,

                time: 0,

                dwExtraInfo: 0,
            },
        },
    };

    enviar(input);
}

// ======================================================
// 🎹 INTERNO → NATIVO WINDOWS
// ======================================================

fn interno_nativo(interno: &str) -> Option<String> {
    pulsadores::interno_a_nativo(interno).map(|valor| valor.to_string())
}

// ======================================================
// 🖱️ FLAGS MOUSE
// ======================================================

fn mouse_flags(control: &str) -> Option<(u32, u32)> {
    match control {
        "LeftButton" => Some((MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP)),

        "RightButton" => Some((MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP)),

        "MiddleButton" => Some((MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP)),

        "Button4" => Some((MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP)),

        "Button5" => Some((MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP)),

        _ => None,
    }
}

// ======================================================
// 📤 SEND INPUT
// ======================================================

fn enviar(input: INPUT) {
    unsafe {
        SendInput(1, &input, size_of::<INPUT>() as i32);
    }
}
