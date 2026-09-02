// ======================================================
// 🖱️ Back_Coordenada
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Traduce entre Windows y el tipo "Click en coordenada":
// posición del cursor físico, rect + título de la ventana
// activa, mover el cursor, y calcular a qué punto de
// pantalla corresponde una UbicacionCache ya compilada
// (con clip a los bordes si la ventana cambió de tamaño y
// la coordenada guardada quedó afuera).
//
// No conoce perfiles, Cache ni Runtime más allá de recibir
// UbicacionCache/PostAccionCache ya resueltos — no
// interpreta strings de UI (eso ya lo hizo compilador.rs).
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// runtime.rs (ejecutar_click_coordenada, ejecución real del
//     trigger).
// comandos.rs (mientras la ventana de captura está abierta,
//     para mostrar datos en vivo y resolver el guardado).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// obtener_cursor() / obtener_ventana_activa(): nada.
// mover_cursor(x, y, debe_detenerse): coordenada absoluta de
//     pantalla + callback para cortar el movimiento en curso.
// calcular_destino(ubicacion): una UbicacionCache ya
//     compilada (ver perfil_cache.rs).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// obtener_cursor() -> (i32, i32).
// obtener_ventana_activa() -> Option<VentanaActiva>.
// calcular_destino() -> (i32, i32), siempre clippeado a dentro
//     de la ventana activa si la ubicación es relativa a
//     ventana.
// ------------------------------------------------------
// 5. Funciones del archivo
//
// obtener_cursor()
//     GetCursorPos.
// obtener_ventana_activa()
//     GetForegroundWindow + GetWindowRect + GetWindowTextW.
//     None si no hay ventana en foreground (caso raro, ej.
//     el escritorio).
// mover_cursor(x, y, debe_detenerse)
//     Delega al motor activo (Interception o Portable);
//     movimiento interpolado, no SetCursorPos. Corta antes
//     si debe_detenerse() da true.
// calcular_destino(ubicacion)
//     Resuelve Absoluta / RelativaCursor / RelativaVentana
//     (Porcentaje o Píxeles) a una coordenada de pantalla.
//     Relativa a cursor usa obtener_cursor() como origen.
//     Relativa a ventana usa obtener_ventana_activa() — si no
//     hay ventana activa, cae en (0, 0) del punto de
//     referencia correspondiente. Clip: si el resultado cae
//     fuera del rect de la ventana activa, se recorta al
//     borde más cercano.
// ======================================================

use crate::perfil_cache::{PuntoReferenciaCache, UbicacionCache};

use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowRect, GetWindowTextW,
};

// ======================================================
// 🖥️ VENTANA ACTIVA
// ======================================================

pub struct VentanaActiva {
    pub x: i32,

    pub y: i32,

    pub ancho: i32,

    pub alto: i32,

    pub titulo: String,
}

// ======================================================
// 📍 OBTENER CURSOR
// ======================================================

pub fn obtener_cursor() -> (i32, i32) {
    unsafe {
        let mut punto: POINT = std::mem::zeroed();

        if GetCursorPos(&mut punto) == 0 {
            return (0, 0);
        }

        (punto.x, punto.y)
    }
}

// ======================================================
// 🪟 OBTENER VENTANA ACTIVA
// ======================================================

pub fn obtener_ventana_activa() -> Option<VentanaActiva> {
    unsafe {
        let ventana = GetForegroundWindow();

        if ventana.is_null() {
            return None;
        }

        let mut rect: RECT = std::mem::zeroed();

        if GetWindowRect(ventana, &mut rect) == 0 {
            return None;
        }

        let mut buffer = [0u16; 256];

        let largo = GetWindowTextW(ventana, buffer.as_mut_ptr(), buffer.len() as i32);

        let titulo = String::from_utf16_lossy(&buffer[..largo.max(0) as usize]);

        Some(VentanaActiva {
            x: rect.left,
            y: rect.top,
            ancho: rect.right - rect.left,
            alto: rect.bottom - rect.top,
            titulo,
        })
    }
}

// ======================================================
// 🚚 MOVER CURSOR
// ------------------------------------------------------
// Delega al motor activo (mismo patrón que
// motor::emitir_evento) para que el movimiento se emita por
// el mismo canal que clicks/teclas — interpolado en pasos
// cortos con WM_MOUSEMOVE reales, no SetCursorPos (que
// teleporta sin generar movimiento intermedio y varias apps
// no reconocen como arrastre: Explorer/drag, selección de
// texto, cuadro selector del escritorio, Paint).
// ======================================================

pub fn mover_cursor(x: i32, y: i32, debe_detenerse: &dyn Fn() -> bool) {
    match crate::motor::modo_activo() {
        crate::motor::Modo::Interception => {
            crate::back_interception::mover_cursor(x, y, debe_detenerse)
        }
        crate::motor::Modo::Portable => crate::back_windows::mover_cursor(x, y, debe_detenerse),
    }
}

// ======================================================
// 🎯 CALCULAR DESTINO
// ======================================================

pub fn calcular_destino(ubicacion: &UbicacionCache) -> (i32, i32) {
    match ubicacion {
        UbicacionCache::Absoluta { x, y } => (*x as i32, *y as i32),

        UbicacionCache::RelativaCursor { offset_x, offset_y } => {
            let (cx, cy) = obtener_cursor();

            (cx + *offset_x as i32, cy + *offset_y as i32)
        }

        UbicacionCache::RelativaVentanaPorcentaje { h, v } => {
            let Some(ventana) = obtener_ventana_activa() else {
                return (0, 0);
            };

            let x = ventana.x + ((ventana.ancho as f64) * (*h / 100.0)) as i32;
            let y = ventana.y + ((ventana.alto as f64) * (*v / 100.0)) as i32;

            clip_a_ventana(x, y, &ventana)
        }

        UbicacionCache::RelativaVentanaPixeles {
            offset_x,
            offset_y,
            referencia,
        } => {
            let Some(ventana) = obtener_ventana_activa() else {
                return (0, 0);
            };

            let (base_x, base_y) = punto_referencia_absoluto(referencia, &ventana);

            let x = base_x + *offset_x as i32;
            let y = base_y + *offset_y as i32;

            clip_a_ventana(x, y, &ventana)
        }
    }
}

// ======================================================
// 🧮 UBICACIÓN / DESTINO DESDE VALORES CRUDOS
// ------------------------------------------------------
// Puente entre el vocabulario de strings de UI (ubicacion/
// modo_ventana/punto_referencia, mismo que ConfigCaptura) y
// UbicacionCache — reutilizado por compilador.rs::
// convertir_coordenada() (perfil real) y por el modo
// previsualización de captura_coordenada.rs (Etapa E), para
// no duplicar el match en los dos lugares.
// ======================================================

pub fn ubicacion_desde_valores(
    ubicacion: &str,
    modo_ventana: &str,
    punto_referencia: &str,
    x: f64,
    y: f64,
) -> UbicacionCache {
    match ubicacion {
        "relativa_cursor" => UbicacionCache::RelativaCursor {
            offset_x: x,
            offset_y: y,
        },

        "relativa_ventana" => match modo_ventana {
            "porcentaje" => UbicacionCache::RelativaVentanaPorcentaje { h: x, v: y },

            _ => UbicacionCache::RelativaVentanaPixeles {
                offset_x: x,
                offset_y: y,
                referencia: punto_referencia_desde_str(punto_referencia),
            },
        },

        _ => UbicacionCache::Absoluta { x, y },
    }
}

pub fn calcular_destino_valores(
    ubicacion: &str,
    modo_ventana: &str,
    punto_referencia: &str,
    x: f64,
    y: f64,
) -> (i32, i32) {
    let ubicacion = ubicacion_desde_valores(ubicacion, modo_ventana, punto_referencia, x, y);

    calcular_destino(&ubicacion)
}

fn punto_referencia_desde_str(valor: &str) -> PuntoReferenciaCache {
    match valor {
        "sup_der" => PuntoReferenciaCache::SupDer,

        "centro" => PuntoReferenciaCache::Centro,

        "inf_izq" => PuntoReferenciaCache::InfIzq,

        "inf_der" => PuntoReferenciaCache::InfDer,

        _ => PuntoReferenciaCache::SupIzq,
    }
}



// ======================================================
// 📐 PUNTO DE REFERENCIA → COORDENADA ABSOLUTA
// ======================================================

fn punto_referencia_absoluto(
    referencia: &PuntoReferenciaCache,
    ventana: &VentanaActiva,
) -> (i32, i32) {
    match referencia {
        PuntoReferenciaCache::SupIzq => (ventana.x, ventana.y),

        PuntoReferenciaCache::SupDer => (ventana.x + ventana.ancho, ventana.y),

        PuntoReferenciaCache::Centro => (
            ventana.x + ventana.ancho / 2,
            ventana.y + ventana.alto / 2,
        ),

        PuntoReferenciaCache::InfIzq => (ventana.x, ventana.y + ventana.alto),

        PuntoReferenciaCache::InfDer => (ventana.x + ventana.ancho, ventana.y + ventana.alto),
    }
}

// ======================================================
// ✂️ CLIP A VENTANA
// ------------------------------------------------------
// Si la ventana se redimensionó entre la captura y la
// ejecución y la coordenada calculada queda afuera del rect
// actual, se recorta al borde más cercano en vez de hacer
// click fuera de la ventana.
// ======================================================

fn clip_a_ventana(x: i32, y: i32, ventana: &VentanaActiva) -> (i32, i32) {
    let x = x.clamp(ventana.x, ventana.x + ventana.ancho);
    let y = y.clamp(ventana.y, ventana.y + ventana.alto);

    (x, y)
}
