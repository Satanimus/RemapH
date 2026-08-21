// ======================================================
// 🪟 Back_Windows
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Único punto de contacto con la API de Windows en Modo
// Portable (sin driver Interception).
//
// ENTRADA: instala hooks globales WH_KEYBOARD_LL y
// WH_MOUSE_LL vía SetWindowsHookExW, corre un loop de
// mensajes (GetMessage) en el mismo hilo que los instaló
// (requisito de la API de Windows), traduce cada evento
// crudo a InputEvent y se lo entrega a quien lo inició.
//
// SALIDA: recibe un InputEvent y lo emite físicamente
// vía SendInput/WinAPI. Sin concepto de "dispositivo
// destino" — SendInput inyecta a nivel de sistema (ver
// Regla 6 del plan de Modo Portable).
//
// No decide qué hacer con un evento. No conoce Runtime,
// Cache ni AnalizadorTrigger.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// lib.rs (o el punto de despacho unificado, Etapa B)
// llama iniciar() una sola vez, pasándole el callback
// de procesamiento y el predicado debe_tragar_no_traducible,
// con la misma firma que usa back_interception.rs.
// entrada.rs / Runtime llaman emitir_evento() para
// devolver un evento sin consumir o ejecutar una acción
// remapeada.
// detener() puede ser llamado desde cualquier hilo para
// pedir el cierre limpio del loop de hooks.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// iniciar(procesar, debe_tragar_no_traducible): un
// callback FnMut(InputEvent) llamado por cada evento
// traducido, y un predicado Fn() -> bool que indica si
// un evento no traducible debe tragarse (true) o
// descartarse silenciosamente (false). Bloquea hasta
// que detener() sea llamado.
// emitir_evento(evento): un InputEvent completo.
// detener(): sin parámetros.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// iniciar() no retorna valor — bloquea hasta recibir
// WM_QUIT (vía detener()).
// emitir_evento() no retorna nada.
// detener() no retorna nada.
// ------------------------------------------------------
// Limitaciones conocidas y aceptadas (Modo Portable):
//
// - SendInput no logra que algunas apps UWP (ej. Paint)
//   reaccionen a eventos inyectados, aunque WinAPI reporte
//   éxito. Sin solución en Modo Portable (Interception no
//   disponible). Documentado, no bloqueante.
//
// - Algunos anti-cheat bloquean hooks globales de WinAPI.
//
// - Los hooks corren en el hilo de mensajes: más latencia
//   que un driver de kernel.
//
// - Pueden no capturar eventos de procesos con más
//   privilegios que RemapH si este no corre elevado.
// ------------------------------------------------------
// 5. Funciones del archivo
// iniciar()
//     Instala WH_KEYBOARD_LL y WH_MOUSE_LL, corre el
//     loop GetMessage en el hilo actual. Llama al callback
//     con cada InputEvent traducido. Sale al recibir
//     WM_QUIT (enviado por detener()). Desinstala los
//     hooks al salir.
// detener()
//     Pide el cierre limpio del loop: guarda el ID del
//     hilo de hooks y le envía WM_QUIT vía
//     PostThreadMessageW.
// hook_teclado() [privada, unsafe extern "system"]
//     Callback de WH_KEYBOARD_LL. Traduce KBDLLHOOKSTRUCT
//     a InputEvent. Filtra eventos inyectados propios.
//     Retorna 1 si el evento fue consumido, llama
//     CallNextHookEx si no.
// hook_mouse() [privada, unsafe extern "system"]
//     Callback de WH_MOUSE_LL. Traduce MSLLHOOKSTRUCT
//     a InputEvent. Filtra eventos inyectados propios.
//     Retorna 1 si el evento fue consumido, llama
//     CallNextHookEx si no.
// traducir_teclado() [privada]
//     KBDLLHOOKSTRUCT → Option<InputEvent>, consultando
//     pulsadores::por_nativo() con el vkCode recibido.
// traducir_mouse() [privada]
//     MSLLHOOKSTRUCT + wparam → Option<InputEvent>,
//     consultando pulsadores::por_nativo() con el código
//     de botón/rueda correspondiente.
// evaluar() [privada]
//     Extrae el procesador del estado de hilo, lo llama
//     con el evento y el cierre de emitir, lo devuelve al
//     estado. Retorna true si el evento fue consumido.
// emitir_evento()
//     InputEvent → INPUT(s) físicos vía SendInput.
//     Teclado: KEYBDINPUT con wVk. Mouse: MOUSEINPUT con
//     los flags del botón/rueda correspondiente.
// emitir_teclado() [privada]
//     Construye y envía un KEYBDINPUT (down o up).
// emitir_mouse() [privada]
//     Construye y envía un MOUSEINPUT (botón o rueda).
// emitir_mouse_button() [privada]
//     Construye y envía un MOUSEINPUT para un botón
//     con los flags dados.
// teclado_control() [privada]
//     vkCode → Option<String> (nombre interno del
//     control), consultando pulsadores::por_nativo().
// interno_nativo() [privada]
//     Nombre interno → Option<String> con el código
//     nativo Windows ("0xXX"), vía
//     pulsadores::interno_a_nativo().
// mouse_flags() [privada]
//     Nombre interno del control → Option<(u32, u32)>
//     con los flags de down y up para MOUSEINPUT.
// enviar() [privada]
//     Llama SendInput con un INPUT ya construido.
// ------------------------------------------------------
// Transformación:
//
// ENTRADA:
// Evento físico (WH_KEYBOARD_LL / WH_MOUSE_LL)
//     ↓
// traducir_teclado() / traducir_mouse()
//     ↓
// InputEvent
//     ↓
// callback(evento)   [quien llamó a iniciar()]
//
// SALIDA:
// InputEvent
//     ↓
// emitir_evento()
//     ↓
// SendInput (KEYBDINPUT / MOUSEINPUT)
// ======================================================

use crate::eventos::InputEvent;
use crate::instante;
use crate::pulsadores;
use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN,
    MOUSEEVENTF_XUP, MOUSEINPUT,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEWHEEL, WM_QUIT,
    WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

use crate::eventos::{InputId, InputState};

// ======================================================
// 🧵 ID DEL HILO DE HOOKS
// ------------------------------------------------------
// Guardado al entrar en iniciar() para que detener()
// pueda enviarle WM_QUIT desde cualquier otro hilo.
// 0 = no hay hilo activo.
// ======================================================

static HILO_HOOKS: AtomicU32 = AtomicU32::new(0);

// ======================================================
// 🧠 ESTADO DEL HILO (procesador + predicado)
// ======================================================

struct Estado {
    procesar: Box<dyn FnMut(InputEvent)>,
    debe_tragar_no_traducible: Box<dyn Fn() -> bool>,
}

thread_local! {
    static ESTADO: RefCell<Option<Estado>> = RefCell::new(None);
}

// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar(
    mut procesar: impl FnMut(InputEvent) + 'static,
    debe_tragar_no_traducible: impl Fn() -> bool + 'static,
) {
    ESTADO.with(|estado| {
        *estado.borrow_mut() = Some(Estado {
            procesar: Box::new(procesar),
            debe_tragar_no_traducible: Box::new(debe_tragar_no_traducible),
        });
    });

    let id_hilo = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
    HILO_HOOKS.store(id_hilo, Ordering::SeqCst);

    let hook_teclado =
        unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_teclado), std::ptr::null_mut(), 0) };

    if hook_teclado.is_null() {
        panic!("[back_windows] No se pudo instalar hook de teclado");
    }

    let hook_mouse =
        unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(hook_mouse), std::ptr::null_mut(), 0) };

    if hook_mouse.is_null() {
        unsafe { UnhookWindowsHookEx(hook_teclado) };
        panic!("[back_windows] No se pudo instalar hook de mouse");
    }

    println!("📥 Backend portable iniciado (WinAPI hooks).");

    let mut mensaje: MSG = unsafe { std::mem::zeroed() };

    loop {
        let resultado = unsafe { GetMessageW(&mut mensaje, std::ptr::null_mut(), 0, 0) };

        if resultado <= 0 {
            break;
        }
    }

    unsafe {
        UnhookWindowsHookEx(hook_teclado);
        UnhookWindowsHookEx(hook_mouse);
    }

    HILO_HOOKS.store(0, Ordering::SeqCst);

    ESTADO.with(|estado| {
        *estado.borrow_mut() = None;
    });

    println!("[back_windows] Finalizado.");
}

// ======================================================
// 🛑 DETENER
// ======================================================

pub fn detener() {
    let id = HILO_HOOKS.load(Ordering::SeqCst);

    if id != 0 {
        unsafe { PostThreadMessageW(id, WM_QUIT, 0, 0) };
    }
}

// ======================================================
// 🎹 HOOK TECLADO
// ======================================================

unsafe extern "system" fn hook_teclado(codigo: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if codigo < 0 {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let datos = &*(lparam as *const KBDLLHOOKSTRUCT);

    // Filtrar eventos inyectados por este mismo proceso
    if datos.flags & 0x10 != 0 {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let presionado = wparam == WM_KEYDOWN as usize || wparam == WM_SYSKEYDOWN as usize;
    let liberado = wparam == WM_KEYUP as usize || wparam == WM_SYSKEYUP as usize;

    if !presionado && !liberado {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    match traducir_teclado(datos.vkCode, presionado) {
        Some(evento) => {
            if evaluar(evento) {
                return 1;
            }
        }
        None => {
            // No traducible: tragar si hay captura activa, pasar si no
            let debe_tragar = ESTADO.with(|estado| {
                estado
                    .borrow()
                    .as_ref()
                    .map(|e| (e.debe_tragar_no_traducible)())
                    .unwrap_or(false)
            });

            if debe_tragar {
                return 1;
            }
        }
    }

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

    // Filtrar eventos inyectados por este mismo proceso
    if datos.flags & 0x01 != 0 {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    match traducir_mouse(wparam, datos) {
        Some(evento) => {
            if evaluar(evento) {
                return 1;
            }
        }
        None => {
            let debe_tragar = ESTADO.with(|estado| {
                estado
                    .borrow()
                    .as_ref()
                    .map(|e| (e.debe_tragar_no_traducible)())
                    .unwrap_or(false)
            });

            if debe_tragar {
                return 1;
            }
        }
    }

    CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam)
}

// ======================================================
// 🧠 EVALUAR
// ======================================================

fn evaluar(evento: InputEvent) -> bool {
    let mut procesar: Option<Box<dyn FnMut(InputEvent)>> = None;

    ESTADO.with(|estado| {
        if let Some(actual) = estado.borrow_mut().take() {
            procesar = Some(actual.procesar);

            // Devolver el estado sin el procesador temporalmente
            // (se restaura abajo)
            *estado.borrow_mut() = Some(Estado {
                procesar: Box::new(|_| {}),
                debe_tragar_no_traducible: actual.debe_tragar_no_traducible,
            });
        }
    });

    // Nota: el procesador de back_interception no retorna bool —
    // llama a procesar(evento) sin valor de retorno. En back_windows
    // los hooks siempre pasan el evento (nunca lo consumen solos;
    // el procesador decide si emitir o no). Retornamos false para
    // que CallNextHookEx siga la cadena — la lógica de bloqueo
    // real vendrá del Runtime en etapas posteriores.
    //
    // TODO (Etapa B): cuando el punto de despacho unificado esté
    // listo, ajustar si procesar() necesita retornar bool aquí
    // para bloquear eventos.
    if let Some(mut f) = procesar {
        f(evento);

        ESTADO.with(|estado| {
            let mut guard = estado.borrow_mut();
            if let Some(actual) = guard.take() {
                *guard = Some(Estado {
                    procesar: f,
                    debe_tragar_no_traducible: actual.debe_tragar_no_traducible,
                });
            }
        });
    }

    false
}

// ======================================================
// 🎹 TRADUCIR TECLADO
// ======================================================

fn traducir_teclado(vk: u32, presionado: bool) -> Option<InputEvent> {
    let nativo = format!("0x{:X}", vk);
    let pulsador = pulsadores::por_nativo(&nativo)?;
    let input = InputId::new("keyboard", &pulsador.interno);

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
        let pulsador = pulsadores::por_nativo(nativo)?;
        return Some(InputEvent::pulse(
            InputId::new("mouse", &pulsador.interno),
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

    let pulsador = pulsadores::por_nativo(nativo)?;
    let input = InputId::new("mouse", &pulsador.interno);

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
// 📤 EMITIR EVENTO
// ======================================================

pub fn emitir_evento(evento: InputEvent) {
    // Implementado en A4
    let _ = evento;
}

// ======================================================
// (resto de funciones de salida — A4)
// ======================================================
