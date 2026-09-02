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
//     bloquea y lo ENCOLA para el hilo worker — ver COLA)
//     o si debe_tragar_no_traducible() lo pide (sincrónico,
//     no encolado); llama CallNextHookEx en cualquier otro
//     caso.
// hook_mouse() [privada, unsafe extern "system"]
//     Callback de WH_MOUSE_LL. Traduce MSLLHOOKSTRUCT
//     a InputEvent. Filtra eventos inyectados propios.
//     Mismo modelo que hook_teclado(): traducido → bloquea
//     y encola; no traducible → debe_tragar_no_traducible()
//     sincrónico decide; si no, CallNextHookEx.
// traducir_teclado() [privada]
//     KBDLLHOOKSTRUCT (scanCode + flags extendida) →
//     Option<InputEvent>: resuelve directo por posición
//     física vía pulsadores::scancode_a_interno(), sin pasar
//     por VK. Ver esa función y su nota completa.
// traducir_mouse() [privada]
//     MSLLHOOKSTRUCT + wparam → Option<InputEvent>,
//     consultando pulsadores::por_nativo() con el código
//     de botón/rueda correspondiente.
// (hilo worker, lanzado desde iniciar())
//     Consume COLA en un loop (rx.recv()) y llama a
//     procesar() (entrada::procesar_evento()) con cada
//     evento — fuera del hilo de hooks, ver nota completa
//     en la declaración de COLA. No decide bloqueo — eso ya
//     lo resolvió el hook que encoló (siempre bloqueaba un
//     evento traducido).
// emitir_evento()
//     InputEvent → INPUT(s) físicos vía SendInput.
//     Teclado: KEYBDINPUT con wScan (posición física) +
//     KEYEVENTF_SCANCODE. Mouse: MOUSEINPUT con los flags
//     del botón/rueda correspondiente.
// emitir_teclado() [privada]
//     Construye y envía un KEYBDINPUT (down o up).
// scancode_desde_interno() [privada]
//     Nombre interno (InputId) → Option<(u16, bool)> con el
//     scan code físico + es_extendida a usar en wScan, vía
//     pulsadores::por_interno(). Igual criterio que ya usa
//     Interception al emitir (por posición, no por VK) — ver
//     nota completa en enviar_tecla().
// enviar_tecla() [privada]
//     Construye y envía un KEYBDINPUT (down o up) con
//     KEYEVENTF_SCANCODE, agregando KEYEVENTF_EXTENDEDKEY
//     cuando corresponde.
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
// COLA (mpsc::channel) — el hook encola y retorna, sin
//     esperar a que se procese
//     ↓
// hilo worker: callback(evento)   [quien llamó a iniciar()]
//
// SALIDA:
// InputEvent
//     ↓
// emitir_evento()
//     ↓
// SendInput (KEYBDINPUT / MOUSEINPUT)
// ======================================================

use crate::back_coordenada;
use crate::eventos::InputEvent;
use crate::pulsadores;
use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender};

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL,
    MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, GetSystemMetrics, PostThreadMessageW, SetWindowsHookExW,
    UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, SM_CXVIRTUALSCREEN,
    SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
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
// 🧠 ESTADO DEL HOOK (predicado no-traducible, sincrónico)
// ------------------------------------------------------
// Sigue viviendo en el hilo de hooks — la decisión de bloquear
// un evento no traducible (debe_tragar_no_traducible) TIENE que
// tomarse dentro del propio hook, ver nota en COLA más abajo.
// ======================================================

struct EstadoHook {
    debe_tragar_no_traducible: Box<dyn Fn() -> bool>,
}

thread_local! {
    static ESTADO_HOOK: RefCell<Option<EstadoHook>> = RefCell::new(None);
}

// ======================================================
// 🧠 ESTADO DEL WORKER (procesador)
// ------------------------------------------------------
// El trabajo pesado — evaluar()/procesar_evento()/SendInput de
// reinyección para el camino TRADUCIBLE — se movió a un hilo
// dedicado (worker) para que el hilo de hooks (WH_KEYBOARD_LL/
// WH_MOUSE_LL) nunca lo haga: Windows exige que ese hilo
// responda con un timeout estricto, y cualquier lock o lógica
// de matching ahí dentro competía por ese presupuesto, causando
// el lag general del sistema reportado con perfil activo
// (arrastrar ventanas, rueda, etc. en cualquier app, no solo
// en RemapH).
//
// `procesar` es siempre una `fn` libre (entrada::procesar_evento
// — ver lib.rs), no una closure con estado capturado, así que es
// trivial de mover a otro hilo (Copy + Send + 'static).
// ======================================================

// Canal hacia el hilo worker. Se recrea en cada iniciar() y se
// destruye (Sender se dropea) al salir — el hilo worker termina
// solo cuando su Receiver se desconecta, sin necesitar una señal
// explícita de apagado. Mutex en vez de thread_local porque el
// hook y el worker corren en hilos distintos (mismo patrón que
// COMPILADO/RUNTIME en cache.rs).
//
// Solo pasa por acá el camino TRADUCIBLE (evaluar() -> procesar_
// evento() completo, con sus locks y posible SendInput de
// reinyección — el costo variable real). El camino NO traducible
// sigue siendo 100% sincrónico dentro del hook: la decisión de
// bloquear un evento no traducible depende de debe_tragar_no_
// traducible() (cache::captura_activa(), un solo Mutex.lock() +
// lectura de bool — barato) y esa decisión TIENE que tomarse ahí
// mismo, porque bloquear un evento físico solo es posible
// devolviendo 1 desde el propio hook, no después.
static COLA: std::sync::Mutex<Option<Sender<InputEvent>>> = std::sync::Mutex::new(None);

// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar(
    mut procesar: impl FnMut(InputEvent) + Send + 'static,
    debe_tragar_no_traducible: impl Fn() -> bool + 'static,
) {
    ESTADO_HOOK.with(|estado| {
        *estado.borrow_mut() = Some(EstadoHook {
            debe_tragar_no_traducible: Box::new(debe_tragar_no_traducible),
        });
    });

    // Hilo worker dedicado: consume la cola y llama a procesar()
    // (evaluar() -> entrada::procesar_evento()) fuera del hilo de
    // hooks — ver nota completa en la declaración de COLA más
    // arriba. join_handle no se guarda: el hilo termina solo cuando
    // el Sender se dropea al salir del loop de mensajes más abajo,
    // y no hace falta esperarlo (no toca nada que dependa del hilo
    // de hooks después de ese punto).
    let (tx, rx) = mpsc::channel::<InputEvent>();

    *COLA.lock().unwrap() = Some(tx);

    let manija_worker = std::thread::spawn(move || {
        while let Ok(evento) = rx.recv() {
            procesar(evento);
        }
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

    // Dropea el Sender: el hilo worker sale de su while let Ok(...)
    // en cuanto termina de procesar lo que ya tenía encolado, y
    // finaliza solo.
    *COLA.lock().unwrap() = None;

    // Espera a que el worker drene lo que ya tenía encolado ANTES de
    // devolver el control a motor::iniciar() (que arranca el otro
    // backend acto seguido). Sin este join(), un evento bloqueado
    // justo antes del cambio de modo (ej. el click en "Guardar
    // cambios" que disparó este mismo cambio) podía quedar sin su
    // SendInput de reinyección — los hooks ya desinstalados, el
    // worker todavía corriendo en paralelo — dejando ese botón físico
    // "abajo" para Windows indefinidamente.
    let _ = manija_worker.join();

    ESTADO_HOOK.with(|estado| {
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

    // [FIX] Ya NO hay shortcut por cache::esta_vacia() acá: con perfil
    // desactivado, el atajo global Activar/Desactivar (Regla 2 del
    // plan) tiene que seguir viendo cada evento físico para poder
    // reactivar el perfil — y esa detección vive únicamente en
    // entrada::procesar_evento (entrada.rs es el único funnel, Regla
    // 3), no acá, para no duplicarla en cada backend. El shortcut
    // viejo cortaba ANTES de llegar a ese funnel, dejando el atajo
    // muerto apenas se desactivaba el perfil (y, con él, cualquier
    // forma de reactivarlo sin reiniciar). back_interception.rs nunca
    // tuvo este shortcut y nunca tuvo este bug. El costo real de
    // encolar siempre queda en el hilo worker (ver COLA más arriba),
    // no en este hook de baja latencia — y WM_MOUSEMOVE (el caso que
    // sí importa en volumen) sigue filtrado aparte en hook_mouse().
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

    // LLKHF_EXTENDED = 0x01: distingue, por ejemplo, la flecha derecha
    // extendida del "6" de numpad, que comparten scanCode base (ver
    // nota en traducir_teclado()).
    let es_extendida = datos.flags & 0x01 != 0;

    match traducir_teclado(datos.scanCode, es_extendida, presionado) {
        Some(evento) => {
            // Evento traducido: se bloquea el físico original SOLO si
            // hay un worker activo para encolarlo y eventualmente
            // reinyectarlo (SendInput) — si COLA ya es None (backend
            // cerrándose, ver detener()/iniciar() al final del
            // archivo), no hay nadie que vaya a reinyectar este
            // evento nunca: bloquearlo lo perdería para siempre y
            // dejaría la tecla/botón físicamente "abajo" para Windows.
            // Mismo bug para teclado y mouse — ver hook_mouse() más
            // abajo.
            let cola = COLA.lock().unwrap();

            match cola.as_ref() {
                Some(tx) => {
                    let _ = tx.send(evento);
                    return 1;
                }
                None => {
                    drop(cola);
                    return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
                }
            }
        }
        None => {
            // No traducible: tragar si hay captura activa, pasar si
            // no. Esta decisión SÍ sigue siendo sincrónica (no se
            // encola) porque bloquear un evento físico solo es
            // posible devolviendo 1 desde el propio hook — ver nota
            // en la declaración de ESTADO_HOOK/COLA más arriba.
            let debe_tragar = ESTADO_HOOK.with(|estado| {
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

    // [FIX] Ver nota equivalente en hook_teclado(): ya no hay
    // shortcut por cache::esta_vacia() acá, para que el atajo global
    // toggle siga funcionando con el perfil desactivado. WM_MOUSEMOVE
    // (el volumen que de verdad importa) sigue filtrado aparte más
    // abajo, antes de traducir_mouse/encolar — así que este cambio no
    // reintroduce el lag original.
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
            // traducido solo bloquea el físico si hay worker activo
            // para reinyectarlo; si COLA ya es None, se deja pasar.
            let cola = COLA.lock().unwrap();

            match cola.as_ref() {
                Some(tx) => {
                    let _ = tx.send(evento);
                    return 1;
                }
                None => {
                    drop(cola);
                    return CallNextHookEx(std::ptr::null_mut(), codigo, wparam, lparam);
                }
            }
        }
        None => {
            let debe_tragar = ESTADO_HOOK.with(|estado| {
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
// 🎹 TRADUCIR TECLADO
// ------------------------------------------------------
// Resuelve directo por posición física (scan code Set 1 +
// bit E0), vía pulsadores::scancode_a_interno() — igual
// criterio que ya usa Interception (back_teclas.rs::convertir(),
// que también parte de un ScanCode y nunca de un VK). Antes
// pasaba por MapVirtualKeyExW + pulsadores::por_nativo(), pero
// esa API no respeta el layout activo para teclas OEM en
// layouts no-US (confirmado con logs: layout español detectado
// bien, VK devuelto correspondía a la posición equivalente en
// layout US) — ver Etapa A/B de pulsadores.tsv/pulsadores.rs
// para el detalle de las columnas scancode/extendida.
// ======================================================

fn traducir_teclado(scan_code: u32, es_extendida: bool, presionado: bool) -> Option<InputEvent> {
    let interno = pulsadores::scancode_a_interno(scan_code as u16, es_extendida)?;
    let input = InputId::new("keyboard", interno);

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
    let Some((scancode, extendida)) = scancode_desde_interno(evento) else {
        return;
    };

    match evento.state {
        InputState::Down => enviar_tecla(scancode, extendida, false),
        InputState::Up => enviar_tecla(scancode, extendida, true),
        InputState::Pulse => {
            enviar_tecla(scancode, extendida, false);
            enviar_tecla(scancode, extendida, true);
        }
    }
}

fn scancode_desde_interno(evento: &InputEvent) -> Option<(u16, bool)> {
    let interno = evento.input.control()?;
    let pulsador = pulsadores::por_interno(interno)?;

    Some((pulsador.scancode?, pulsador.extendida))
}

// ======================================================
// ⌨️ ENVIAR TECLA
// ------------------------------------------------------
// Emite por posición física (scan code Set 1 + bit E0),
// vía KEYEVENTF_SCANCODE — no por VK (wVk). Antes se armaba
// el KEYBDINPUT con wVk, pero eso le pide a Windows que
// traduzca ese VK a carácter usando el layout activo EN ESE
// MOMENTO en la ventana de destino: es la misma traducción
// dependiente de layout que ya vimos con MapVirtualKeyExW en
// la entrada (Etapa C), solo que ahora en la dirección
// contraria. Como "nativo"/"ui" en pulsadores.tsv están
// calibrados para el layout "Español (Latinoamérica)", un
// layout activo distinto (ej. Español de España) traduce ese
// mismo VK a otro carácter (confirmado: interno "Grado", VK
// pensado para imprimir "°", se emitía como "ñ"; "Interrogacion",
// pensado para "¡", se emitía como "+").
//
// Con KEYEVENTF_SCANCODE, en cambio, Windows recibe la
// posición física y la traduce con el layout que sea, dando
// siempre el carácter real de esa tecla en ese layout — igual
// que ya hace Interception al emitir (nunca tuvo este bug).
// ======================================================

fn enviar_tecla(scancode: u16, extendida: bool, arriba: bool) {
    let mut flags = KEYEVENTF_SCANCODE;

    if extendida {
        flags |= KEYEVENTF_EXTENDEDKEY;
    }

    if arriba {
        flags |= KEYEVENTF_KEYUP;
    }

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: scancode,
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
// 🚚 MOVER CURSOR (interpolado, absoluto)
// ------------------------------------------------------
// Reemplaza al SetCursorPos que usaba back_coordenada
// directo: SetCursorPos teleporta el cursor sin generar
// WM_MOUSEMOVE intermedios, y varias apps (Explorer/drag de
// archivos, selección de texto por arrastre, el cuadro
// selector del escritorio, Paint) no reconocen un arrastre
// que salta directo de un punto a otro — necesitan ver el
// recorrido.
//
// [FIX] Movimiento ABSOLUTO, no relativo. La primera versión
// mandaba MOUSEEVENTF_MOVE relativo (dx/dy) y releía la
// posición real entre pasos para no acumular error — pero
// Windows aplica aceleración de puntero (pointer ballistics)
// a los deltas relativos por defecto, y esa aceleración podía
// mantenerse activa/errática entre pasos sucesivos incluso
// releyendo la posición, dando un recorrido mucho más amplio
// que el pedido (llegaba hasta el borde de pantalla). El
// modo absoluto (MOUSEEVENTF_ABSOLUTE) no pasa por pointer
// ballistics — es un porcentaje 0..65535 del escritorio
// virtual completo (todos los monitores, ver
// MOUSEEVENTF_VIRTUALDESK), documentado por Microsoft como
// posición directa, no delta. Se sigue interpolando en pasos
// cortos (ahora cada uno es una posición absoluta intermedia,
// no un delta) para que el destino siga viendo varios
// WM_MOUSEMOVE en el trayecto. `debe_detenerse` se consulta en
// cada paso (mismo mecanismo que el resto de runt_macro.rs) para
// que el toggle/detener pueda cortar un movimiento largo en curso
// en vez de esperar a que termine — y MAX_PASOS_MOVIMIENTO es la
// salvaguarda dura para call sites que no tienen forma real de
// cancelar (pasan &|| false: nunca antes eran cancelables tampoco,
// pero ahora igual quedan protegidos por el límite de pasos).
// ======================================================

const PASO_MOVIMIENTO_PX: i32 = 12;
const PAUSA_ENTRE_PASOS_MS: u64 = 4;

// Tolerancia de llegada: el redondeo de ida y vuelta al escalar a
// 0..65536 (MOUSEEVENTF_ABSOLUTE) y que Windows lo vuelva a
// convertir a píxel no es perfectamente reversible — el cursor
// puede terminar 1-2px del destino exacto. Sin esta tolerancia, la
// comparación por igualdad exacta contra el destino nunca se
// cumplía y el loop no terminaba nunca (bug: cursor "temblando" en
// el destino, hilo de la macro bloqueado para siempre — y si se
// reintentaba la macro, se acumulaban varios de estos hilos vivos a
// la vez, cada uno moviendo el cursor a su propio destino).
const TOLERANCIA_LLEGADA_PX: i32 = 2;

// Salvaguarda dura: aunque la tolerancia de arriba cubre el caso
// normal, ningún movimiento debe poder quedar vivo para siempre por
// una condición imprevista (destino fuera del escritorio virtual,
// métricas en 0, etc.) — a este ritmo de paso/pausa, 2000 pasos son
// varios segundos de sobra para cualquier distancia real en
// pantalla.
const MAX_PASOS_MOVIMIENTO: u32 = 2000;

pub fn mover_cursor(x: i32, y: i32, debe_detenerse: &dyn Fn() -> bool) {
    for _ in 0..MAX_PASOS_MOVIMIENTO {
        if debe_detenerse() {
            return;
        }

        let (actual_x, actual_y) = back_coordenada::obtener_cursor();

        let restante_x = x - actual_x;
        let restante_y = y - actual_y;

        if restante_x.abs() <= TOLERANCIA_LLEGADA_PX && restante_y.abs() <= TOLERANCIA_LLEGADA_PX
        {
            // Último ajuste fino: un movimiento absoluto directo al
            // destino exacto, sin interpolar (la distancia restante
            // ya es mínima, no hace falta trayecto intermedio).
            enviar_movimiento_absoluto(x, y);
            return;
        }

        let distancia = (restante_x.abs()).max(restante_y.abs());
        let paso = PASO_MOVIMIENTO_PX.min(distancia).max(1);

        let dx = if restante_x == 0 {
            0
        } else {
            (restante_x.signum()) * paso.min(restante_x.abs())
        };
        let dy = if restante_y == 0 {
            0
        } else {
            (restante_y.signum()) * paso.min(restante_y.abs())
        };

        enviar_movimiento_absoluto(actual_x + dx, actual_y + dy);

        std::thread::sleep(std::time::Duration::from_millis(PAUSA_ENTRE_PASOS_MS));
    }
}

fn enviar_movimiento_absoluto(x: i32, y: i32) {
    // MOUSEEVENTF_ABSOLUTE + VIRTUALDESK: dx/dy dejan de ser delta y
    // pasan a ser un porcentaje 0..65535 del escritorio virtual
    // completo (todos los monitores) — no del monitor primario, que
    // es lo que se obtendría sin VIRTUALDESK en un setup multi-
    // monitor.
    unsafe {
        let origen_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let origen_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let ancho = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(1);
        let alto = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(1);

        let escala_x = (((x - origen_x) as i64 * 65536) / ancho as i64) as i32;
        let escala_y = (((y - origen_y) as i64 * 65536) / alto as i64) as i32;

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: escala_x,
                    dy: escala_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        enviar(input);
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
