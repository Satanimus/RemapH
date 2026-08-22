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
//     Retorna 1 si el evento se tradujo (siempre lo
//     bloquea) o si debe_tragar_no_traducible() lo pide;
//     llama CallNextHookEx en cualquier otro caso.
// hook_mouse() [privada, unsafe extern "system"]
//     Callback de WH_MOUSE_LL. Traduce MSLLHOOKSTRUCT
//     a InputEvent. Filtra eventos inyectados propios.
//     Retorna 1 si el evento se tradujo (siempre lo
//     bloquea) o si debe_tragar_no_traducible() lo pide;
//     llama CallNextHookEx en cualquier otro caso.
// traducir_teclado() [privada]
//     KBDLLHOOKSTRUCT → Option<InputEvent>, consultando
//     pulsadores::por_nativo() con el vkCode recibido.
// traducir_mouse() [privada]
//     MSLLHOOKSTRUCT + wparam → Option<InputEvent>,
//     consultando pulsadores::por_nativo() con el código
//     de botón/rueda correspondiente.
// evaluar() [privada]
//     Extrae el procesador del estado de hilo, lo llama
//     con el evento, lo devuelve al estado. No decide
//     bloqueo — eso ya lo resolvió el hook que la llama
//     (siempre bloquea un evento traducido).
// emitir_evento()
//     InputEvent → INPUT(s) físicos vía SendInput.
//     Teclado: KEYBDINPUT con wVk. Mouse: MOUSEINPUT con
//     los flags del botón/rueda correspondiente.
// emitir_teclado() [privada]
//     Construye y envía un KEYBDINPUT (down o up).
// vk_desde_interno() [privada]
//     Nombre interno (InputId) → Option<u16> con el VK
//     real a usar en wVk, vía pulsadores::interno_a_nativo().
// es_extendida() [privada]
//     VK → bool. Marca los VK que Windows considera
//     "extendidos" (ver KEYEVENTF_EXTENDEDKEY más abajo).
// enviar_tecla() [privada]
//     Construye y envía un KEYBDINPUT (down o up),
//     agregando KEYEVENTF_EXTENDEDKEY cuando corresponde.
// emitir_mouse() [privada]
//     Construye y envía un MOUSEINPUT (botón o rueda).
// emitir_mouse_button() [privada]
//     Construye y envía un MOUSEINPUT para un botón
//     con los flags dados.
// mouse_flags() [privada]
//     Nombre interno del control → Option<(u32, u32)>
//     con los flags de down y up para MOUSEINPUT.
// enviar_rueda() [privada]
//     Construye y envía un MOUSEINPUT de rueda.
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
use crate::pulsadores;
use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL,
    MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
    WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
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

    // Sin perfil activo NI captura en curso, no hay nada que
    // remapear ni que grabar: se pasa TODO directo, sin traducir, sin
    // bloquear el físico, sin llamar a debe_tragar_no_traducible() ni
    // a evaluar(). Mismo criterio y mismo orden de precedencia que ya
    // usa entrada.rs::procesar_evento (captura_activa() primero,
    // esta_vacia() después) — con Modo Captura activo SIEMPRE hay que
    // seguir interceptando, aunque no haya perfil, porque ese modo
    // necesita ver/grabar cada evento físico (ver captura_coordenada.rs).
    // Aplicado ANTES de interceptar nada, para evitar que este hook
    // de baja latencia (WH_KEYBOARD_LL) haga trabajo alguno por cada
    // tecla cuando RemapH no tiene nada que hacer — causa del lag
    // general reportado (afecta más al mouse, ver hook_mouse()).
    if !crate::cache::captura_activa() && crate::cache::esta_vacia() {
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
            // Evento traducido: SIEMPRE se bloquea el físico original
            // (nunca CallNextHookEx acá) — mismo modelo que Interception,
            // que intercepta todo por default. Lo que deba pasar
            // (el mismo evento sin tocar, o una acción remapeada) lo
            // reinyecta motor::emitir_evento() más abajo en la cadena
            // (entrada.rs), vía SendInput — el filtro de "eventos
            // inyectados por este mismo proceso" de más arriba evita
            // que esa reinyección se vuelva a capturar como si fuera
            // físico.
            evaluar(evento);
            return 1;
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

    // Sin perfil activo NI captura en curso, no hay nada que
    // remapear ni que grabar: se pasa TODO directo (ver comentario
    // equivalente en hook_teclado(), mismo orden de precedencia que
    // entrada.rs::procesar_evento). Este es el caso que más importa:
    // WM_MOUSEMOVE por sí solo ya se filtra más abajo, pero botones y
    // rueda sin perfil ni captura también pasaban por
    // evaluar()/debe_tragar_no_traducible() innecesariamente.
    if !crate::cache::captura_activa() && crate::cache::esta_vacia() {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    let datos = &*(lparam as *const MSLLHOOKSTRUCT);

    // Filtrar eventos inyectados por este mismo proceso
    if datos.flags & 0x01 != 0 {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    // WM_MOUSEMOVE se descarta ANTES de traducir_mouse/evaluar: no es
    // un botón/rueda traducible (traducir_mouse() no lo contempla,
    // siempre da None), y a diferencia de un click o scroll puntual
    // llega a un ritmo de cientos/miles de eventos por segundo. Si
    // cayera en la rama "no traducible" de más abajo, cada movimiento
    // del mouse en CUALQUIER ventana del sistema llamaría a
    // debe_tragar_no_traducible() dentro de este hook de baja
    // latencia (WH_MOUSE_LL) — Windows tiene un timeout estricto para
    // este hilo, y saturarlo así es lo que causa el lag general del
    // sistema (arrastrar ventanas, rueda) visto incluso sin perfil
    // activo. Nunca se bloquea el movimiento físico: siempre pasa.
    if wparam as u32 == WM_MOUSEMOVE {
        return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
    }

    match traducir_mouse(wparam, datos) {
        Some(evento) => {
            // Ver nota equivalente en hook_teclado(): un evento
            // traducido siempre bloquea el físico original.
            evaluar(evento);
            return 1;
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

fn evaluar(evento: InputEvent) {
    let mut procesar: Option<Box<dyn FnMut(InputEvent)>> = None;
    let mut debe_tragar_no_traducible: Option<Box<dyn Fn() -> bool>> = None;

    ESTADO.with(|estado| {
        if let Some(actual) = estado.borrow_mut().take() {
            procesar = Some(actual.procesar);
            debe_tragar_no_traducible = Some(actual.debe_tragar_no_traducible);
        }
    });

    // El bloqueo del evento físico ya se decidió en el hook (siempre
    // se bloquea, ver hook_teclado()/hook_mouse()) — acá solo se
    // entrega al procesador para que decida qué reinyectar, si
    // corresponde, vía motor::emitir_evento().
    if let (Some(mut f), Some(pred)) = (procesar, debe_tragar_no_traducible) {
        f(evento);

        ESTADO.with(|estado| {
            *estado.borrow_mut() = Some(Estado {
                procesar: f,
                debe_tragar_no_traducible: pred,
            });
        });
    }
}

// ======================================================
// 🎹 TRADUCIR TECLADO
// ======================================================

fn traducir_teclado(vk: u32, presionado: bool) -> Option<InputEvent> {
    let nativo = format!("0x{:X}", vk);
    let pulsador = pulsadores::por_nativo(&nativo)?;
    let input = InputId::new("keyboard", &pulsador.interno);

    if presionado {
        Some(InputEvent::down(input))
    } else {
        Some(InputEvent::up(input))
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
        return Some(InputEvent::pulse_con_magnitud(
            InputId::new("mouse", &pulsador.interno),
            delta,
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
            Some(InputEvent::down(input))
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => Some(InputEvent::up(input)),
        _ => None,
    }
}

// ======================================================
// 📤 EMITIR EVENTO
// ======================================================
//
// InputEvent → INPUT(s) físicos vía SendInput. Sin
// concepto de "dispositivo destino" — SendInput inyecta a
// nivel de sistema (ver Regla 6 del plan de Modo Portable),
// a diferencia de back_interception::emitir_evento() que
// manda a un Device puntual (teclado_primario()/mouse_
// primario()).
// ======================================================

pub fn emitir_evento(evento: InputEvent) {
    match evento.input.fuente() {
        Some("keyboard") => emitir_teclado(&evento),
        Some("mouse") => emitir_mouse(&evento),
        _ => {}
    }
}

// ======================================================
// 🎹 EMITIR TECLADO
// ======================================================

fn emitir_teclado(evento: &InputEvent) {
    // Caso especial por paridad estructural con
    // back_interception.rs::emitir_teclado (mismo chequeo sobre
    // evento.input.control()) — pero la mecánica de fake-shift de la
    // Regla 7 no aplica acá: esa mecánica existe solo porque
    // Interception opera en Set 1/Set 2 crudo, donde Impr Pant sin el
    // par fake-shift+tecla no se traduce a nada (ver comentario en
    // back_interception.rs, líneas 513-521). SendInput ya traduce
    // Impr Pant correctamente como una tecla suelta más, vía el wVk
    // real (0x2C) + KEYEVENTF_EXTENDEDKEY (ver es_extendida()).
    if evento.input.control() == Some("PrintScreen") {
        match evento.state {
            InputState::Down => enviar_tecla(0x2C, false),
            InputState::Up => enviar_tecla(0x2C, true),
            InputState::Pulse => {
                enviar_tecla(0x2C, false);
                enviar_tecla(0x2C, true);
            }
        }

        return;
    }

    let Some(vk) = vk_desde_interno(evento) else {
        return;
    };

    match evento.state {
        InputState::Down => enviar_tecla(vk, false),
        InputState::Up => enviar_tecla(vk, true),
        InputState::Pulse => {
            enviar_tecla(vk, false);
            enviar_tecla(vk, true);
        }
    }
}

fn vk_desde_interno(evento: &InputEvent) -> Option<u16> {
    let interno = evento.input.control()?;
    let nativo = pulsadores::interno_a_nativo(interno)?;

    nativo
        .strip_prefix("0x")
        .and_then(|valor| u16::from_str_radix(valor, 16).ok())
}

// ======================================================
// 🧩 ES_EXTENDIDA
// ------------------------------------------------------
// VK que Windows considera parte del "grupo extendido"
// (ver KEYBDINPUT/KEYEVENTF_EXTENDEDKEY en MSDN): Ctrl/Alt
// derecho, el bloque de navegación (Inicio/Fin/RePág/AvPág/
// flechas/Insert/Supr), Impr Pant, Num Lock, la tecla Win, y
// el "/" del numpad (Divide). Sin esto, SendInput con solo
// wVk las manda como si fueran su par no-extendido (ej. el
// numpad), igual que el problema que Regla 7/TABLA_EXTENDIDA
// resuelve del lado de back_teclas.rs para Interception —acá
// no hace falta una tabla de ambigüedad porque el VK ya es
// único por tecla (ver columna "nativo" de pulsadores.tsv);
// solo falta marcar el flag para que Windows la trate igual
// que la manda un teclado físico real.
// ======================================================

fn es_extendida(vk: u16) -> bool {
    matches!(
        vk,
        0x21..=0x28 // RePág, AvPág, Fin, Inicio, Izquierda, Arriba, Derecha, Abajo
            | 0x2C..=0x2E // Impr Pant, Insert, Supr
            | 0x5B // Win (LeftMeta)
            | 0x6F // Divide (Numpad /)
            | 0x90 // Num Lock
            | 0xA3 // Ctrl derecho
            | 0xA5 // Alt derecho
    )
}

fn enviar_tecla(vk: u16, arriba: bool) {
    let flags = match (es_extendida(vk), arriba) {
        (true, true) => KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
        (true, false) => KEYEVENTF_EXTENDEDKEY,
        (false, true) => KEYEVENTF_KEYUP,
        (false, false) => 0,
    };

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

fn emitir_mouse(evento: &InputEvent) {
    let Some(control) = evento.input.control() else {
        return;
    };

    if control == "WheelUp" || control == "WheelDown" {
        // Si el evento trae magnitud real (rueda física, ver
        // traducir_mouse()/InputEvent::pulse_con_magnitud), se
        // reenvía tal cual entró — igual criterio que
        // back_interception::emitir_mouse(). Si no trae magnitud
        // (evento sintético de una Acción remapeada), se usa el
        // valor fijo de siempre (120/-120).
        let signo: i32 = if control == "WheelUp" { 1 } else { -1 };
        let cantidad = evento.magnitud.map(|m| m as i32).unwrap_or(signo * 120);

        enviar_rueda(cantidad);
        return;
    }

    let Some((down, up)) = mouse_flags(control) else {
        return;
    };

    match evento.state {
        InputState::Down => enviar_mouse_button(control, down),
        InputState::Up => enviar_mouse_button(control, up),
        InputState::Pulse => {
            enviar_mouse_button(control, down);
            enviar_mouse_button(control, up);
        }
    }
}

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

fn enviar_mouse_button(control: &str, flags: u32) {
    let mouse_data = match control {
        "Button4" => 1,
        "Button5" => 2,
        _ => 0,
    };

    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    enviar(input);
}

fn enviar_rueda(cantidad: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: cantidad as u32,
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    enviar(input);
}

// ======================================================
// 📤 SEND INPUT
// ======================================================

fn enviar(input: INPUT) {
    unsafe {
        SendInput(1, &input, size_of::<INPUT>() as i32);
    }
}
