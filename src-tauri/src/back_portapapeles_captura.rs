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
// Este módulo NO escribe archivos, NO sabe qué filas están en
// modo Registro, y NO conoce ACTIVOS — por ahora, ante cada
// cambio, solo lo LOGGEA por consola. Queda listo para que la
// Etapa F conecte el aviso real con el pool de archivos.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// Todavía nadie. El módulo está declarado en lib.rs (necesario
// para que este archivo forme parte de la compilación), pero no
// se agrega la llamada a iniciar_monitor() desde lib.rs/setup()
// en esta etapa — se conecta recién en la Etapa F, cuando el
// primer Portapapeles entra en modo Registro (ver
// back_portapapeles.rs). Hasta entonces, el compilador va a
// avisar con warnings de "función nunca usada" para las
// funciones públicas de este archivo — es esperable y
// desaparece solo en la Etapa F.
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
// Por ahora solo loggea — Etapa F lo conecta con el pool de
// archivos real (ver header, sección 1).
// ======================================================

fn en_cambio_portapapeles() {
    match leer_portapapeles() {
        Some(ContenidoPortapapeles::Texto(texto)) => {
            println!(
                "📋 Portapapeles: texto ({} caracteres)",
                texto.chars().count()
            );
        }

        Some(ContenidoPortapapeles::Imagen { ancho, alto, .. }) => {
            println!("📋 Portapapeles: imagen {}x{}", ancho, alto);
        }

        None => {
            println!("📋 Portapapeles: cambio detectado sin contenido legible.");
        }
    }
}
