// ======================================================
// 📋 back_portapapeles_captura
// ======================================================
// ETAPA D DEL PLAN "PORTAPAPELES"
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Backend AISLADO encargado de avisar cuándo cambió el
// contenido del portapapeles de Windows, y de leer ese
// contenido bajo demanda.
//
// Mismo patrón que back_app.rs::iniciar_monitor() (hilo
// dedicado + loop de mensajes de Windows), pero acá el aviso
// no es SetWinEventHook (foco) sino AddClipboardFormatListener
// (portapapeles) — y ESE mecanismo exige que el hilo tenga una
// ventana propia (Windows solo le puede avisar a un HWND), así
// que este archivo también crea una ventana "mensaje-only"
// (HWND_MESSAGE) invisible, solo para recibir el aviso.
//
// La LECTURA del contenido (texto/imagen) usa el crate arboard
// en vez de manejar a mano los formatos crudos del portapapeles
// (CF_UNICODETEXT / CF_DIB) — arboard ya sabe hacer esa traducción.
//
// Este módulo en sí NO escribe archivos, NO sabe qué filas están
// en modo Registro, y NO conoce ACTIVOS directamente — sigue sin
// saber nada de eso. Lo único que hace ante cada aviso es leer el
// contenido y pasárselo tal cual a
// back_portapapeles::en_cambio_del_sistema() (ETAPA F), que es
// quien decide si hay que guardarlo o no (según ACTIVOS) y aplica
// el límite. Ese único punto de contacto mantiene la separación:
// este archivo solo sabe "detectar y leer", back_portapapeles.rs
// solo sabe "decidir y guardar".
// El módulo también sabe ESCRIBIR al portapapeles del sistema
// (escribir_portapapeles(), ETAPA H) — lo usa back_portapapeles::
// pegar() para el click en un elemento (spec: "se pega el contenido
// del archivo al portapapeles"). Sigue siendo el único lugar del
// proyecto que toca arboard directamente, por simetría con
// leer_portapapeles().
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// asegurar_listener() / detener_listener() (ETAPA J.1) las llama
// back_portapapeles.rs en cada uno de sus puntos donde cambia si
// "debe existir" el listener (abrir/cerrar ventana, activar/
// desactivar Registro) — ver debe_existir_listener() ahí. Ninguno
// de los dos hace nada si el estado ya es el pedido (arrancar
// estando ya arrancado, o detener estando ya detenido).
// escribir_portapapeles() la llama back_portapapeles::pegar()
// (ETAPA H) cada vez que el usuario clickea un elemento.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// Nada desde afuera — Windows le avisa directo vía mensaje
// WM_CLIPBOARDUPDATE a la ventana propia que este módulo crea.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// leer_portapapeles() -> Option<ContenidoPortapapeles>: el
// contenido actual del portapapeles del sistema, bajo demanda
// (lo va a usar el modo Simple, ver back_portapapeles.rs etapa G,
// para leer sin esperar el próximo cambio).
// ------------------------------------------------------
// 5. Reglas / decisiones
//
// • Si el portapapeles tiene AMBOS formatos a la vez (imagen y
//   texto — pasa con algunas apps de oficina que copian una
//   "vista previa" junto al texto), se prioriza la IMAGEN. Es lo
//   más común de los dos casos reales del spec (Ctrl+C de texto
//   vs. captura de pantalla) y no debería ser ambiguo en la
//   práctica — si en Etapa E/G aparece un caso real donde esto
//   da un resultado no deseado, se ajusta el orden ahí.
// • ETAPA J.1 — el hilo YA NO corre de por vida: arranca con
//   asegurar_listener() y se detiene con detener_listener(). La
//   CLASE de ventana (RegisterClassExW) sí se registra una única
//   vez por proceso (una clase registrada dos veces falla) — un
//   AtomicBool propio (CLASE_REGISTRADA) lo garantiza, separado de
//   "está corriendo ahora" (LISTENER_CORRIENDO). Cada arranque
//   crea una ventana mensaje-only NUEVA sobre esa misma clase ya
//   registrada; cada parada la destruye. Mismo criterio de fondo
//   que back_app::iniciar_monitor() (hilo dedicado + loop de
//   mensajes), pero ahí no hace falta detenerlo nunca — acá sí,
//   porque el listener de Portapapeles no debe quedar leyendo el
//   portapapeles del sistema cuando no hay nada mirando (ni
//   Registro activo ni ventana abierta).
// • asegurar_listener() es IDEMPOTENTE: llamarla estando ya
//   corriendo no hace nada (compare_exchange en LISTENER_CORRIENDO).
//   Mismo criterio para detener_listener() estando ya detenido.
// ------------------------------------------------------
// 6. Funciones del archivo
//
// ContenidoPortapapeles
//     Texto(String) | Imagen{ancho, alto, pixeles RGBA8}.
// leer_portapapeles()
//     Lee el contenido actual del portapapeles bajo demanda.
// asegurar_listener()
//     Arranca el hilo/ventana/AddClipboardFormatListener si no
//     estaba corriendo ya. Sin efecto si ya estaba corriendo.
// detener_listener()
//     Pide al hilo que termine (PostMessageW WM_CLOSE) si estaba
//     corriendo. Sin efecto si ya estaba detenido.
// wndproc_portapapeles()
//     Procedimiento de ventana: intercepta WM_CLIPBOARDUPDATE (avisa
//     el cambio) y WM_DESTROY (corta el loop de mensajes con
//     PostQuitMessage), delega el resto a DefWindowProcW.
// en_cambio_portapapeles()
//     Reacciona a cada aviso — lee el portapapeles y delega en
//     back_portapapeles::en_cambio_del_sistema().
// ======================================================

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::System::DataExchange::{
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostMessageW, PostQuitMessage,
    RegisterClassExW, TranslateMessage, HWND_MESSAGE, MSG, WM_CLIPBOARDUPDATE, WM_CLOSE,
    WM_DESTROY, WNDCLASSEXW,
};

// ======================================================
// 📦 CONTENIDO DEL PORTAPAPELES
// ------------------------------------------------------
// Solo texto e imagen — mismo alcance que el spec ("Solo se
// guarda texto e imágenes copiadas o guardadas"). pixeles ya
// viene en RGBA8 (mismo formato que entrega arboard::ImageData),
// lista para que Etapa E la guarde como .png sin conversión
// adicional.
// ======================================================

pub enum ContenidoPortapapeles {
    Texto(String),

    Imagen {
        ancho: usize,
        alto: usize,
        pixeles: Vec<u8>,
    },
}

// ======================================================
// 📥 LEER PORTAPAPELES (bajo demanda)
// ======================================================

pub fn leer_portapapeles() -> Option<ContenidoPortapapeles> {
    let mut portapapeles = arboard::Clipboard::new().ok()?;

    if let Ok(imagen) = portapapeles.get_image() {
        return Some(ContenidoPortapapeles::Imagen {
            ancho: imagen.width,
            alto: imagen.height,
            pixeles: imagen.bytes.into_owned(),
        });
    }

    if let Ok(texto) = portapapeles.get_text() {
        if !texto.is_empty() {
            return Some(ContenidoPortapapeles::Texto(texto));
        }
    }

    None
}

// ======================================================
// 📤 ESCRIBIR PORTAPAPELES — ETAPA H
// ------------------------------------------------------
// Usado por back_portapapeles::pegar() al clickear un elemento.
// Simétrico a leer_portapapeles(): mismo arboard, mismo formato
// (ContenidoPortapapeles), sin tocar CF_UNICODETEXT/CF_DIB a mano.
// ======================================================

pub fn escribir_portapapeles(contenido: &ContenidoPortapapeles) -> Result<(), String> {
    let mut portapapeles = arboard::Clipboard::new().map_err(|error| error.to_string())?;

    match contenido {
        ContenidoPortapapeles::Texto(texto) => portapapeles
            .set_text(texto.clone())
            .map_err(|error| error.to_string()),

        ContenidoPortapapeles::Imagen {
            ancho,
            alto,
            pixeles,
        } => {
            let imagen = arboard::ImageData {
                width: *ancho,
                height: *alto,
                bytes: std::borrow::Cow::Borrowed(pixeles),
            };

            portapapeles
                .set_image(imagen)
                .map_err(|error| error.to_string())
        }
    }
}

// ======================================================
// 👁️ MONITOR DE PORTAPAPELES — arranque/parada real (ETAPA J.1)
// ------------------------------------------------------
// LISTENER_CORRIENDO: true mientras el hilo/ventana/listener están
// activos ahora mismo. HWND_ACTUAL: handle de la ventana mensaje-
// only viva (None si no hay ninguna) — HWND es `isize` en
// windows-sys ≥0.52 (no un puntero crudo), así que guardarlo en un
// Mutex normal es seguro entre hilos sin wrapper unsafe aparte.
// CLASE_REGISTRADA: aparte de LISTENER_CORRIENDO — la clase de
// ventana se registra una única vez por proceso y se reutiliza en
// cada arranque siguiente (RegisterClassExW falla si se llama dos
// veces con el mismo nombre).
// ======================================================

static LISTENER_CORRIENDO: AtomicBool = AtomicBool::new(false);
static CLASE_REGISTRADA: AtomicBool = AtomicBool::new(false);
static HWND_ACTUAL: Mutex<Option<HWND>> = Mutex::new(None);

const NOMBRE_CLASE: &str = "RemapHPortapapelesListener";

/// Arranca el listener si no estaba corriendo. Sin efecto si ya
/// estaba corriendo (lo llama back_portapapeles.rs cada vez que
/// "debe existir" pasa a ser true — puede llamarse de más sin
/// problema).
pub fn asegurar_listener() {
    if LISTENER_CORRIENDO
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    std::thread::spawn(|| unsafe {
        let nombre_clase: Vec<u16> = NOMBRE_CLASE
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let instancia = GetModuleHandleW(std::ptr::null()) as HINSTANCE;

        if CLASE_REGISTRADA
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let clase = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(wndproc_portapapeles),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instancia,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: nombre_clase.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };

            if RegisterClassExW(&clase) == 0 {
                println!(
                    "⚠️ No se pudo registrar la clase de ventana del listener de Portapapeles."
                );

                LISTENER_CORRIENDO.store(false, Ordering::SeqCst);

                return;
            }
        }

        let hwnd = CreateWindowExW(
            0,
            nombre_clase.as_ptr(),
            std::ptr::null(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            std::ptr::null_mut(),
            instancia,
            std::ptr::null(),
        );

        if hwnd.is_null() {
            println!("⚠️ No se pudo crear la ventana del listener de Portapapeles.");

            LISTENER_CORRIENDO.store(false, Ordering::SeqCst);

            return;
        }

        if AddClipboardFormatListener(hwnd) == 0 {
            println!("⚠️ No se pudo instalar el listener de Portapapeles.");

            LISTENER_CORRIENDO.store(false, Ordering::SeqCst);

            return;
        }

        *HWND_ACTUAL.lock().unwrap() = Some(hwnd);

        let mut mensaje: MSG = std::mem::zeroed();

        while GetMessageW(&mut mensaje, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&mensaje);

            DispatchMessageW(&mensaje);
        }

        // Llegó WM_QUIT (ver wndproc_portapapeles, WM_DESTROY) — el
        // hilo termina acá. Deja todo listo para un próximo
        // asegurar_listener(): la clase sigue registrada (no se
        // desregistra nunca), pero HWND_ACTUAL y LISTENER_CORRIENDO
        // vuelven a su estado "detenido".
        *HWND_ACTUAL.lock().unwrap() = None;

        LISTENER_CORRIENDO.store(false, Ordering::SeqCst);
    });
}

/// Pide al listener que termine, si estaba corriendo. Sin efecto si
/// ya estaba detenido (lo llama back_portapapeles.rs cada vez que
/// "debe existir" pasa a ser false).
pub fn detener_listener() {
    let hwnd = HWND_ACTUAL.lock().unwrap().take();

    if let Some(hwnd) = hwnd {
        unsafe {
            RemoveClipboardFormatListener(hwnd);

            // DefWindowProcW ya destruye la ventana ante WM_CLOSE;
            // wndproc_portapapeles intercepta el WM_DESTROY que eso
            // genera y llama PostQuitMessage(0) para cortar el loop
            // GetMessageW del hilo (ver arriba).
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
}

// ======================================================
// 🪟 PROCEDIMIENTO DE VENTANA
// ======================================================

unsafe extern "system" fn wndproc_portapapeles(
    hwnd: HWND,
    mensaje: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if mensaje == WM_CLIPBOARDUPDATE {
        en_cambio_portapapeles();

        return 0;
    }

    // ETAPA J.1: WM_DESTROY llega como consecuencia del WM_CLOSE que
    // manda detener_listener() (vía DefWindowProcW, más abajo).
    // PostQuitMessage(0) es lo que hace que GetMessageW() del loop en
    // asegurar_listener() devuelva 0 y el hilo termine.
    if mensaje == WM_DESTROY {
        PostQuitMessage(0);

        return 0;
    }

    DefWindowProcW(hwnd, mensaje, wparam, lparam)
}

// ======================================================
// 🔔 EN CAMBIO DE PORTAPAPELES
// ------------------------------------------------------
// Delega en back_portapapeles::en_cambio_del_sistema() — ese
// archivo decide qué hacer según el estado (algún Registro activo,
// o solo alguna ventana Simple abierta) y notifica a las ventanas
// abiertas (ETAPA J.1). Si el portapapeles cambió a algo no legible
// (None — un formato que no es ni texto ni imagen, ej. copiar un
// archivo del explorador), no se llama a nada más: el spec solo
// pide guardar texto e imágenes.
//
// Corta antes de leer el portapapeles si no hay ningún motivo para
// procesar el aviso (ni Registro activo ni ventana abierta) — evita
// crear un arboard::Clipboard y copiar bytes de imagen quedando el
// listener corriendo apenas un instante de más entre que "debe
// existir" pasa a false y detener_listener() lo corta de verdad.
// ======================================================

fn en_cambio_portapapeles() {
    if !crate::back_portapapeles::debe_procesar_cambio() {
        return;
    }

    match leer_portapapeles() {
        Some(ContenidoPortapapeles::Texto(texto)) => {
            println!(
                "📋 Portapapeles: texto ({} caracteres)",
                texto.chars().count()
            );

            crate::back_portapapeles::en_cambio_del_sistema(&ContenidoPortapapeles::Texto(texto));
        }

        Some(imagen @ ContenidoPortapapeles::Imagen { .. }) => {
            if let ContenidoPortapapeles::Imagen { ancho, alto, .. } = &imagen {
                println!("📋 Portapapeles: imagen {}x{}", ancho, alto);
            }

            crate::back_portapapeles::en_cambio_del_sistema(&imagen);
        }

        None => {
            println!("📋 Portapapeles: cambio detectado sin contenido legible.");
        }
    }
}
