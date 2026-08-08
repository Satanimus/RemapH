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
// iniciar_monitor() todavía no se llama desde lib.rs/setup() — se
// arranca recién la primera vez que un Portapapeles entra en modo
// Registro, vía back_portapapeles::activar_registro() (ETAPA F).
// Hasta que eso ocurra por primera vez en una sesión, el hilo
// listener de este archivo simplemente no existe todavía.
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
// • El hilo nunca termina (mismo criterio que back_app::
//   iniciar_monitor() — corre de por vida, no hay orden de
//   detenerlo).
// • Solo se llama una vez: RegisterClassExW fallaría en una
//   segunda llamada con el mismo nombre de clase (no hay guarda
//   explícita acá, mismo criterio que back_app::iniciar_monitor(),
//   que tampoco se protege contra llamados repetidos).
// ------------------------------------------------------
// 6. Funciones del archivo
//
// ContenidoPortapapeles
//     Texto(String) | Imagen{ancho, alto, pixeles RGBA8}.
// leer_portapapeles()
//     Lee el contenido actual del portapapeles bajo demanda.
// iniciar_monitor()
//     Crea la ventana mensaje-only, instala
//     AddClipboardFormatListener y arranca el loop de mensajes
//     en un hilo dedicado.
// wndproc_portapapeles()
//     Procedimiento de ventana: intercepta WM_CLIPBOARDUPDATE,
//     delega el resto a DefWindowProcW.
// en_cambio_portapapeles()
//     Reacciona a cada aviso — por ahora, solo loggea.
// ======================================================

use windows_sys::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};

use windows_sys::Win32::System::DataExchange::AddClipboardFormatListener;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW,
    TranslateMessage, HWND_MESSAGE, MSG, WM_CLIPBOARDUPDATE, WNDCLASSEXW,
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
// 👁️ MONITOR DE PORTAPAPELES
// ======================================================

pub fn iniciar_monitor() {
    std::thread::spawn(|| unsafe {
        let nombre_clase: Vec<u16> = "RemapHPortapapelesListener"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let instancia = GetModuleHandleW(std::ptr::null()) as HINSTANCE;

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
            println!("⚠️ No se pudo registrar la clase de ventana del listener de Portapapeles.");

            return;
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

            return;
        }

        if AddClipboardFormatListener(hwnd) == 0 {
            println!("⚠️ No se pudo instalar el listener de Portapapeles.");

            return;
        }

        let mut mensaje: MSG = std::mem::zeroed();

        while GetMessageW(&mut mensaje, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&mensaje);

            DispatchMessageW(&mensaje);
        }
    });
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

    DefWindowProcW(hwnd, mensaje, wparam, lparam)
}

// ======================================================
// 🔔 EN CAMBIO DE PORTAPAPELES
// ------------------------------------------------------
// ETAPA F: además de loggear (se mantiene, es útil para depurar),
// delega en back_portapapeles::en_cambio_del_sistema() — ese
// archivo decide si hay algún Portapapeles en modo Registro y, si
// lo hay, guarda el rotativo y aplica el límite. Si el portapapeles
// cambió a algo no legible (None — un formato que no es ni texto ni
// imagen, ej. copiar un archivo del explorador), no se llama a nada
// más: el spec solo pide guardar texto e imágenes.
//
// Corta antes de leer el portapapeles si no hay ningún Portapapeles
// en modo Registro — evita crear un arboard::Clipboard y copiar
// bytes de imagen en el caso normal (el usuario copiando cosas sin
// tener ninguna ventana Portapapeles en modo Registro abierta), que
// va a ser el caso más frecuente. Esto es solo una optimización: si
// justo se activa un Registro entre este chequeo y el próximo aviso
// real, ese próximo aviso sí se procesa normal — no se pierde nada
// más que un aviso puntual mientras nadie estaba mirando.
// ======================================================

fn en_cambio_portapapeles() {
    if !crate::back_portapapeles::hay_algun_activo() {
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
