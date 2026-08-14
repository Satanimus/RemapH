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
// leer_portapapeles(). Para IMAGEN puntualmente, arboard::set_image()
// dejó de usarse (Paint la rechazaba con "La información en el
// Portapapeles no se puede insertar en Paint" — ver el comentario
// arriba de procesar_escribir_imagen()). Se probaron varias
// variantes manuales (CF_DIB simple, CF_DIB+CF_DIBV5, CF_BITMAP vía
// CreateDIBSection) sin éxito — la que sí funciona es replicar
// exactamente lo que hace clipboard-win (la crate que usa arboard
// por dentro en Windows) en su código fuente real: CreateDIBitmap +
// SOLO CF_BITMAP, con EmptyClipboard llamado DESPUÉS de crear el
// bitmap, no antes.
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

use windows_sys::Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, OpenClipboard,
    RemoveClipboardFormatListener, SetClipboardData,
};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use windows_sys::Win32::Graphics::Gdi::{
    CreateDIBitmap, DeleteObject, GetDC, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostMessageW, PostQuitMessage,
    RegisterClassExW, SendMessageW, TranslateMessage, HWND_MESSAGE, MSG, WM_APP,
    WM_CLIPBOARDUPDATE, WM_CLOSE, WM_DESTROY, WNDCLASSEXW,
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
// 📤 ESCRIBIR PORTAPAPELES — ETAPA H (+ FIX CF_DIB/CF_DIBV5)
// ------------------------------------------------------
// Usado por back_portapapeles::pegar() al clickear un elemento.
//
// Texto: sigue usando arboard tal cual — sin reportes de problemas
// ahí, CF_UNICODETEXT es un formato único y sin ambigüedad.
//
// Imagen: YA NO usa arboard::set_image() (Paint la rechazaba igual).
// Se probaron varias variantes propias sin éxito (CF_DIB simple,
// CF_DIB+CF_DIBV5, CF_BITMAP vía CreateDIBSection). La que funciona:
// replicar exactamente set_bitmap_inner() de clipboard-win (la crate
// que usa arboard por dentro en Windows, código fuente real
// verificado en docs.rs) — CreateDIBitmap + SOLO CF_BITMAP, con
// EmptyClipboard llamado DESPUÉS de crear el bitmap, no antes.
//
// Por qué esto no rompe el bloqueo anti-duplicado (Etapa H, ver
// back_portapapeles.rs::marcar_ignorar_proximo_cambio()): sigue
// siendo UNA sola apertura/cierre del portapapeles → UN solo
// WM_CLIPBOARDUPDATE, exactamente como antes.
// ======================================================

pub fn escribir_portapapeles(contenido: &ContenidoPortapapeles) -> Result<(), String> {
    match contenido {
        ContenidoPortapapeles::Texto(texto) => {
            let mut portapapeles = arboard::Clipboard::new().map_err(|error| error.to_string())?;

            portapapeles
                .set_text(texto.clone())
                .map_err(|error| error.to_string())
        }

        ContenidoPortapapeles::Imagen {
            ancho,
            alto,
            pixeles,
        } => escribir_imagen_como_bitmap(*ancho, *alto, pixeles),
    }
}

/// Escribe una imagen RGBA8 al portapapeles del sistema como
/// CF_BITMAP (HBITMAP real vía CreateDIBitmap). Ver comentario
/// arriba para el porqué de este enfoque — replica clipboard-win.
///
/// IMPORTANTE — por qué esto pide al hilo del listener que lo haga
/// él mismo en vez de escribir directo acá: este método corre en el
/// threadpool de Tauri (comando síncrono, ver comandos.rs::
/// portapapeles_pegar), un hilo de trabajo genérico sin cola de
/// mensajes de Windows propia. Se comprobó con logs de diagnóstico
/// que, ejecutado ahí, OpenClipboard/EmptyClipboard/SetClipboardData
/// devuelven éxito en cada paso, pero el resultado no queda
/// realmente disponible para lectores externos (ni Paint ni el
/// Historial de Windows / Win+V podían leerlo después). La
/// asociación real hilo↔portapapeles que Windows espera necesita un
/// hilo con su propio message loop detrás — que es justo lo que ya
/// tiene el listener de Portapapeles (asegurar_listener(), más abajo:
/// su propio hilo, su propia ventana, su GetMessageW corriendo).
///
/// SendMessageW (a diferencia de PostMessageW) es SÍNCRONO: bloquea
/// este hilo hasta que wndproc_portapapeles procese
/// WM_ESCRIBIR_IMAGEN en el hilo del listener y deje la respuesta en
/// RESPUESTA_ESCRITURA — así se puede seguir devolviendo
/// Result<(), String> de forma normal, sin volver todo el módulo
/// async.
fn escribir_imagen_como_bitmap(ancho: usize, alto: usize, pixeles: &[u8]) -> Result<(), String> {
    // DIAGNÓSTICO TEMPORAL — sacar estos println! una vez resuelto
    // el problema de pegado en Paint.
    println!(
        "📋 [diag] escribir_imagen_como_bitmap: ancho={} alto={} pixeles.len()={} (esperado {})",
        ancho,
        alto,
        pixeles.len(),
        ancho * alto * 4
    );

    if ancho == 0 || alto == 0 {
        println!("📋 [diag] abortado: ancho o alto en 0");
        return Err("imagen con ancho o alto en 0".to_string());
    }

    // El DIB de Windows se guarda con las filas de ABAJO hacia
    // ARRIBA (bottom-up) y en orden de bytes BGRA, no RGBA. arboard
    // entrega RGBA8 top-down (mismo formato que ya veníamos usando
    // en el resto del archivo — ver ContenidoPortapapeles), así que
    // hay que convertir acá antes de pasárselo a Windows. Esta
    // conversión es pura CPU, sin WinAPI de por medio, así que no
    // hace falta que corra en el hilo del listener — se hace acá
    // mismo, en el hilo del comando Tauri, y solo el resultado
    // (bgra) viaja al listener.
    let bgra = rgba8_a_bgra8_bottom_up(ancho, alto, pixeles);

    println!("📋 [diag] bgra.len()={}", bgra.len());

    let hwnd_listener = HWND_ACTUAL.lock().unwrap().map(|valor| valor as HWND);

    let hwnd_listener = match hwnd_listener {
        Some(hwnd) => hwnd,
        None => {
            println!("📋 [diag] listener no está corriendo, no se puede escribir la imagen");
            return Err("el listener de portapapeles no está activo".to_string());
        }
    };

    // La solicitud vive en el stack de ESTE hilo — es seguro
    // pasarle un puntero crudo a wndproc_portapapeles porque
    // SendMessageW bloquea este hilo (y por lo tanto mantiene vivo
    // este stack frame) hasta que el otro hilo termine de usarlo.
    let solicitud = SolicitudEscribirImagen {
        ancho,
        alto,
        bgra: &bgra,
    };

    println!(
        "📋 [diag] enviando WM_ESCRIBIR_IMAGEN al listener (hwnd={:?})",
        hwnd_listener
    );

    let puntero_solicitud = &solicitud as *const SolicitudEscribirImagen as isize;

    unsafe {
        SendMessageW(
            hwnd_listener,
            WM_ESCRIBIR_IMAGEN,
            0,
            puntero_solicitud as LPARAM,
        );
    }

    // wndproc_portapapeles dejó la respuesta acá antes de retornar
    // de SendMessageW (que ya volvió en este punto, al ser
    // síncrono).
    let respuesta = RESPUESTA_ESCRITURA.lock().unwrap().take();

    match respuesta {
        Some(resultado) => {
            println!("📋 [diag] respuesta del listener: {:?}", resultado);
            resultado
        }
        None => {
            println!("📋 [diag] el listener no dejó ninguna respuesta (no debería pasar)");
            Err("el listener no procesó la solicitud de escritura".to_string())
        }
    }
}

/// Datos que escribir_imagen_como_bitmap() le pasa a
/// wndproc_portapapeles vía WM_ESCRIBIR_IMAGEN. bgra es una
/// referencia prestada — válida mientras dure el SendMessageW que
/// la acompaña (ver comentario en escribir_imagen_como_bitmap()).
struct SolicitudEscribirImagen<'a> {
    ancho: usize,
    alto: usize,
    bgra: &'a [u8],
}

/// Mensaje custom que le pide al hilo del listener que escriba una
/// imagen al portapapeles él mismo (ver comentario largo arriba de
/// escribir_imagen_como_bitmap() para el porqué). WM_APP es la base
/// que Windows reserva para mensajes definidos por la aplicación.
const WM_ESCRIBIR_IMAGEN: u32 = WM_APP + 1;

/// Resultado que wndproc_portapapeles deja acá después de procesar
/// WM_ESCRIBIR_IMAGEN, para que escribir_imagen_como_bitmap() (que
/// está bloqueada en SendMessageW mientras tanto) lo recoja.
static RESPUESTA_ESCRITURA: Mutex<Option<Result<(), String>>> = Mutex::new(None);

/// Lógica real de escritura — corre DENTRO del hilo del listener,
/// llamada desde wndproc_portapapeles al recibir WM_ESCRIBIR_IMAGEN.
///
/// Replica byte a byte lo que hace clipboard-win (la crate que usa
/// arboard por dentro en Windows, confirmado con su código fuente
/// real en docs.rs — ver set_bitmap_inner en
/// clipboard_win::raw): CreateDIBitmap + SOLO CF_BITMAP, con el
/// EmptyClipboard llamado DESPUÉS de crear el bitmap, justo antes
/// del SetClipboardData — no antes, como hacíamos nosotros. Se
/// habían probado antes CF_DIB a mano, CF_DIBV5, y CF_BITMAP vía
/// CreateDIBSection en vez de CreateDIBitmap, con EmptyClipboard
/// siempre llamado primero — ninguna variante funcionó. Esta es la
/// implementación real y confirmada de una librería que sabemos que
/// funciona en producción, replicada tal cual en vez de seguir
/// adivinando variantes propias.
unsafe fn procesar_escribir_imagen(
    solicitud: &SolicitudEscribirImagen,
    hwnd: HWND,
) -> Result<(), String> {
    if OpenClipboard(hwnd) == 0 {
        let codigo_error = GetLastError();
        println!(
            "📋 [diag] OpenClipboard: FALLÓ, GetLastError()={}",
            codigo_error
        );
        return Err("no se pudo abrir el portapapeles del sistema".to_string());
    }

    println!("📋 [diag] OpenClipboard: OK");

    // Crear el HBITMAP ANTES de EmptyClipboard — así lo hace
    // clipboard-win (set_bitmap_inner). Si esto falla, el
    // portapapeles ni se vació — se cierra tal cual estaba.
    let hbitmap = match crear_bitmap_desde_bgra(solicitud.ancho, solicitud.alto, solicitud.bgra) {
        Some(handle) => {
            println!(
                "📋 [diag] crear_bitmap_desde_bgra (CreateDIBitmap): OK, handle={:?}",
                handle
            );
            handle
        }
        None => {
            let codigo_error = GetLastError();
            println!(
                "📋 [diag] crear_bitmap_desde_bgra: FALLÓ (devolvió None), GetLastError()={}",
                codigo_error
            );
            return cerrar_y_retornar_error("no se pudo crear el bitmap de la imagen");
        }
    };

    if EmptyClipboard() == 0 {
        let codigo_error = GetLastError();
        println!(
            "📋 [diag] EmptyClipboard: FALLÓ, GetLastError()={}",
            codigo_error
        );
        DeleteObject(hbitmap);
        return cerrar_y_retornar_error("no se pudo vaciar el portapapeles del sistema");
    }

    println!("📋 [diag] EmptyClipboard: OK");

    // CF_BITMAP = 2 (constante estable de la API de Windows; no
    // tiene ítem propio en windows_sys::Win32::System::
    // DataExchange, así que se usa el valor numérico directo).
    const CF_BITMAP: u32 = 2;

    let resultado_bitmap = SetClipboardData(CF_BITMAP, hbitmap);

    if resultado_bitmap.is_null() {
        let codigo_error = GetLastError();
        println!(
            "📋 [diag] SetClipboardData(CF_BITMAP): FALLÓ (NULL), GetLastError()={}",
            codigo_error
        );
        DeleteObject(hbitmap);
        return cerrar_y_retornar_error("no se pudo escribir CF_BITMAP al portapapeles");
    }

    println!(
        "📋 [diag] SetClipboardData(CF_BITMAP): OK, resultado={:?}",
        resultado_bitmap
    );
    // Ownership pasó al sistema — NO liberar hbitmap acá.

    CloseClipboard();

    println!("📋 [diag] CloseClipboard listo — escritura terminada (en hilo del listener)");

    Ok(())
}

/// Crea un HBITMAP con CreateDIBitmap (BITMAPINFOHEADER de 32bpp
/// BI_RGB) a partir de los píxeles BGRA — replica exactamente
/// set_bitmap_inner() de clipboard-win (la crate que usa arboard por
/// dentro en Windows), confirmado con su código fuente real en
/// docs.rs. Es la misma llamada, con el mismo orden de parámetros,
/// que usa esa librería en producción.
unsafe fn crear_bitmap_desde_bgra(
    ancho: usize,
    alto: usize,
    bgra: &[u8],
) -> Option<windows_sys::Win32::Graphics::Gdi::HBITMAP> {
    let info_header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: ancho as i32,
        biHeight: alto as i32, // positivo: bottom-up, formato estándar
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let dc_pantalla = GetDC(std::ptr::null_mut());

    if dc_pantalla.is_null() {
        println!("📋 [diag] GetDC: FALLÓ (devolvió NULL)");
        return None;
    }

    let mut info: BITMAPINFO = std::mem::zeroed();
    info.bmiHeader = info_header;

    // CBM_INIT = 0x04 (constante estable de la API de Windows; no
    // tiene ítem propio expuesto en windows_sys en esta versión del
    // crate, así que se usa el valor numérico directo — mismo valor
    // que usa clipboard-win, ver const CBM_INIT arriba en su
    // raw.rs).
    const CBM_INIT: u32 = 0x04;

    let hbitmap = CreateDIBitmap(
        dc_pantalla,
        &info_header,
        CBM_INIT,
        bgra.as_ptr() as *const _,
        &info,
        DIB_RGB_COLORS,
    );

    ReleaseDC(std::ptr::null_mut(), dc_pantalla);

    if hbitmap.is_null() {
        let codigo_error = GetLastError();
        println!(
            "📋 [diag] CreateDIBitmap: FALLÓ, GetLastError()={}",
            codigo_error
        );
        return None;
    }

    Some(hbitmap)
}

// ======================================================

/// Cierra el portapapeles (dejarlo abierto lo bloquearía para el
/// resto del sistema) y devuelve el error dado. Punto de salida
/// único para los caminos de error de procesar_escribir_imagen()
/// que ocurren con el portapapeles ya abierto.
unsafe fn cerrar_y_retornar_error(mensaje: &str) -> Result<(), String> {
    CloseClipboard();
    Err(mensaje.to_string())
}

/// RGBA8 top-down (formato que usa el resto del archivo) a BGRA8
/// bottom-up (formato que espera un DIB de Windows).
fn rgba8_a_bgra8_bottom_up(ancho: usize, alto: usize, pixeles: &[u8]) -> Vec<u8> {
    let mut salida = vec![0u8; pixeles.len()];
    let ancho_en_bytes = ancho * 4;

    for fila_origen in 0..alto {
        let fila_destino = alto - 1 - fila_origen; // invierte el orden de filas

        let desde = fila_origen * ancho_en_bytes;
        let destino_desde = fila_destino * ancho_en_bytes;

        for pixel in 0..ancho {
            let o = desde + pixel * 4;
            let d = destino_desde + pixel * 4;

            // RGBA -> BGRA
            salida[d] = pixeles[o + 2]; // B
            salida[d + 1] = pixeles[o + 1]; // G
            salida[d + 2] = pixeles[o]; // R
            salida[d + 3] = pixeles[o + 3]; // A
        }
    }

    salida
}

// ======================================================
// 👁️ MONITOR DE PORTAPAPELES — arranque/parada real (ETAPA J.1)
// ------------------------------------------------------
// LISTENER_CORRIENDO: true mientras el hilo/ventana/listener están
// activos ahora mismo. HWND_ACTUAL: handle de la ventana mensaje-
// only viva (None si no hay ninguna) — se guarda como `isize` (su
// valor numérico), NO como HWND directamente: en windows-sys ≥0.52
// (este proyecto fija ">=0.59, <=0.61") HWND es `*mut c_void`, un
// puntero crudo que no implementa Send/Sync, así que un
// `Mutex<Option<HWND>>` estático no compila ("shared static
// variables must have a type that implements Sync"). `isize` es
// Copy/Send/Sync sin wrapper unsafe aparte, y como acá el handle
// nunca se dereferencia (solo se lo pasa de vuelta tal cual a las
// funciones de la API de Windows), guardar su valor numérico es
// seguro y no pierde nada.
// CLASE_REGISTRADA: aparte de LISTENER_CORRIENDO — la clase de
// ventana se registra una única vez por proceso y se reutiliza en
// cada arranque siguiente (RegisterClassExW falla si se llama dos
// veces con el mismo nombre).
// ======================================================

static LISTENER_CORRIENDO: AtomicBool = AtomicBool::new(false);
static CLASE_REGISTRADA: AtomicBool = AtomicBool::new(false);
static HWND_ACTUAL: Mutex<Option<isize>> = Mutex::new(None);

const NOMBRE_CLASE: &str = "RemapHPortapapelesListener";

/// Handle (como isize crudo) de la ventana oculta del listener de
/// Portapapeles, si está corriendo.
///
/// YA NO SE USA para robar el foco: back_portapapeles.rs::
/// forzar_relectura_portapapeles() lo intentó primero con este HWND,
/// pero al ser una ventana "mensaje-only" (HWND_MESSAGE) nunca puede
/// convertirse en primer plano — limitación estructural de ese tipo
/// de ventana, no arreglable con AttachThreadInput ni ningún truco de
/// foco. Se cambió a usar el HWND de la ventana flotante de
/// Portapapeles que esté abierta (ver hwnd_de_alguna_ventana_abierta()
/// en back_portapapeles.rs), que sí es una ventana real. DEJADA SIN
/// BORRAR a modo de referencia/rollback, mismo criterio que
/// simular_ctrl_v() en back_portapapeles.rs.
#[allow(dead_code)]
pub fn hwnd_listener() -> Option<isize> {
    *HWND_ACTUAL.lock().unwrap()
}

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

        *HWND_ACTUAL.lock().unwrap() = Some(hwnd as isize);

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
        let hwnd = hwnd as HWND;

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

    // WM_ESCRIBIR_IMAGEN: pedido de escribir_imagen_como_bitmap()
    // (corriendo en el hilo del comando Tauri, bloqueada en
    // SendMessageW) para que ESTE hilo —el del listener, con su
    // propio message loop— ejecute la escritura real al
    // portapapeles. lparam es un puntero a un SolicitudEscribirImagen
    // que vive en el stack del hilo llamante, válido mientras dure
    // este SendMessageW (ver comentario en
    // escribir_imagen_como_bitmap()).
    if mensaje == WM_ESCRIBIR_IMAGEN {
        let solicitud = &*(lparam as *const SolicitudEscribirImagen);

        let resultado = procesar_escribir_imagen(solicitud, hwnd);

        *RESPUESTA_ESCRITURA.lock().unwrap() = Some(resultado);

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
