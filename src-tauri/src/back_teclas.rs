// ======================================================
// 🎹 Back_Teclas RemapH V3
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Traduce entre el teclado físico (ScanCode de la
// librería interception) y el idioma interno del motor
// (InputId).
//
// El nombre de cada tecla lo decide únicamente la
// columna "interception" de pulsadores.tsv. Este archivo
// no inventa nombres — solo sabe convertir ese nombre al
// tipo ScanCode que pide la librería, y viceversa.
//
// OJO — el crate `interception` NO tiene una variante de
// ScanCode propia para las teclas extendidas (flechas,
// Inicio/Fin, Insert/Supr, RePág/AvPág, Ctrl/Alt derecho):
// comparten el mismo código crudo que su par del teclado
// numérico (Flecha izquierda = mismo código que NumPad4,
// etc.) y la ÚNICA diferencia real es el bit E0 del
// stroke — no viaja en el ScanCode. Por eso toda función
// acá recibe/devuelve también `es_extendida: bool`, y
// TABLA_EXTENDIDA existe aparte de TABLA para resolver esa
// ambigüedad ANTES de mirar pulsadores.tsv (si no, Flecha
// izquierda se confunde con NumPad4 sin que nada avise).
//
// No conoce AnalizadorTrigger.
// No conoce Cache.
// No conoce Runtime.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// back_interception (traducir(), para la entrada)
// back_interception, para la salida (convertir_salida())
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// convertir(): un ScanCode crudo de interception + si es
// extendida (bit E0) + si fue Down o Up.
// convertir_salida(): un InputId (interno) a emitir.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// convertir(): Option<Evento>. None si la tecla no está
// en pulsadores.tsv (no soportada).
// convertir_salida(): Option<(ScanCode, bool)> — el bool
// es "es_extendida", para que quien emite sepa si tiene
// que mandar también el bit E0. None si el InputId no es
// de teclado, o no tiene ScanCode asociado.
// ------------------------------------------------------
// 5. Funciones del archivo
// TABLA
//     Pares texto ↔ ScanCode para teclas SIN ambigüedad
//     (su código crudo no lo comparte nadie más).
// TABLA_EXTENDIDA
//     Pares texto ↔ ScanCode para teclas que SÍ comparten
//     código con otra (numpad, o Ctrl/Alt izquierdo) — acá
//     el bit E0 es lo único que las distingue.
// convertir()
//     ScanCode + es_extendida + estado → Evento (vía
//     pulsadores.tsv). Registra en consola las teclas no
//     soportadas.
// convertir_salida()
//     InputId → (ScanCode, es_extendida) (vía
//     pulsadores.tsv + TABLA/TABLA_EXTENDIDA).
// ------------------------------------------------------
// Transformación:
//
// ENTRADA:
// ScanCode + es_extendida (interception)
//     ↓
// nombre_interception() [ve TABLA_EXTENDIDA primero]
//     ↓
// pulsadores::interception_a_interno()
//     ↓
// InputId (interno)
//     ↓
// Evento
//
// SALIDA:
// InputId (interno)
//     ↓
// pulsadores::interno_a_interception()
//     ↓
// TABLA_EXTENDIDA o, si no, TABLA
//     ↓
// (ScanCode, es_extendida) (interception)
// ======================================================

use crate::eventos::{InputEvent, InputId};
use crate::instante;
use crate::pulsadores;
use interception::ScanCode;

// ======================================================
// 📖 TABLA texto ↔ ScanCode
// ======================================================

const TABLA: &[(&str, ScanCode)] = &[
    // ------------------------------------------------
    // 🔤 LETRAS
    // ------------------------------------------------
    ("A", ScanCode::A),
    ("B", ScanCode::B),
    ("C", ScanCode::C),
    ("D", ScanCode::D),
    ("E", ScanCode::E),
    ("F", ScanCode::F),
    ("G", ScanCode::G),
    ("H", ScanCode::H),
    ("I", ScanCode::I),
    ("J", ScanCode::J),
    ("K", ScanCode::K),
    ("L", ScanCode::L),
    ("M", ScanCode::M),
    ("N", ScanCode::N),
    ("O", ScanCode::O),
    ("P", ScanCode::P),
    ("Q", ScanCode::Q),
    ("R", ScanCode::R),
    ("S", ScanCode::S),
    ("T", ScanCode::T),
    ("U", ScanCode::U),
    ("V", ScanCode::V),
    ("W", ScanCode::W),
    ("X", ScanCode::X),
    ("Y", ScanCode::Y),
    ("Z", ScanCode::Z),
    // ------------------------------------------------
    // 🔢 NÚMEROS
    // ------------------------------------------------
    ("Num1", ScanCode::Num1),
    ("Num2", ScanCode::Num2),
    ("Num3", ScanCode::Num3),
    ("Num4", ScanCode::Num4),
    ("Num5", ScanCode::Num5),
    ("Num6", ScanCode::Num6),
    ("Num7", ScanCode::Num7),
    ("Num8", ScanCode::Num8),
    ("Num9", ScanCode::Num9),
    ("Num0", ScanCode::Num0),
    // ------------------------------------------------
    // 🔢 NUMPAD (números y operadores) — comparten ScanCode con
    // TABLA_EXTENDIDA (Home/Up/Insert/etc.) pero acá van bajo su
    // propio nombre "Numpad*", que es como los identifica
    // pulsadores.tsv cuando NO son extendidas (es_extendida=false,
    // el caso normal de la tecla física de numpad).
    // ------------------------------------------------
    ("Numpad0", ScanCode::Numpad0),
    ("Numpad1", ScanCode::Numpad1),
    ("Numpad2", ScanCode::Numpad2),
    ("Numpad3", ScanCode::Numpad3),
    ("Numpad4", ScanCode::Numpad4),
    ("Numpad5", ScanCode::Numpad5),
    ("Numpad6", ScanCode::Numpad6),
    ("Numpad7", ScanCode::Numpad7),
    ("Numpad8", ScanCode::Numpad8),
    ("Numpad9", ScanCode::Numpad9),
    ("NumpadMultiply", ScanCode::NumpadMultiply),
    ("NumpadPlus", ScanCode::NumpadPlus),
    ("NumpadMinus", ScanCode::NumpadMinus),
    ("NumpadPeriod", ScanCode::NumpadPeriod),
    // ------------------------------------------------
    // ⌨️ BÁSICAS
    // ------------------------------------------------
    ("Enter", ScanCode::Enter),
    ("Esc", ScanCode::Esc),
    ("Backspace", ScanCode::Backspace),
    ("Tab", ScanCode::Tab),
    ("Space", ScanCode::Space),
    // ------------------------------------------------
    // 🔣 SÍMBOLOS
    // ------------------------------------------------
    ("Minus", ScanCode::Minus),
    ("Equals", ScanCode::Equals),
    ("LeftBracket", ScanCode::LeftBracket),
    ("RightBracket", ScanCode::RightBracket),
    ("BackSlash", ScanCode::BackSlash),
    ("SemiColon", ScanCode::SemiColon),
    ("Apostrophe", ScanCode::Apostrophe),
    ("Grave", ScanCode::Grave),
    ("Comma", ScanCode::Comma),
    ("Period", ScanCode::Period),
    ("Slash", ScanCode::Slash),
    // Int1: tecla ISO extra (cerca del Shift izquierdo en teclados
    // de 105 teclas / ISO, ej. layouts latinoamericanos y europeos).
    // No comparte ScanCode con nada más, no es ambigua: va en TABLA,
    // no en TABLA_EXTENDIDA.
    ("Int1", ScanCode::Int1),
    // ------------------------------------------------
    // 🔒 BLOQUEO
    // ------------------------------------------------
    ("CapsLock", ScanCode::CapsLock),
    ("NumLock", ScanCode::NumLock),
    ("ScrollLock", ScanCode::ScrollLock),
    // ------------------------------------------------
    // ⚙️ FUNCIÓN
    // ------------------------------------------------
    ("F1", ScanCode::F1),
    ("F2", ScanCode::F2),
    ("F3", ScanCode::F3),
    ("F4", ScanCode::F4),
    ("F5", ScanCode::F5),
    ("F6", ScanCode::F6),
    ("F7", ScanCode::F7),
    ("F8", ScanCode::F8),
    ("F9", ScanCode::F9),
    ("F10", ScanCode::F10),
    ("F11", ScanCode::F11),
    ("F12", ScanCode::F12),
    // ------------------------------------------------
    // 🎛️ MODIFICADORES
    // ------------------------------------------------
    ("LeftControl", ScanCode::LeftControl),
    ("LeftShift", ScanCode::LeftShift),
    ("LeftAlt", ScanCode::LeftAlt),
];

// ======================================================
// 📖 TABLA_EXTENDIDA texto ↔ ScanCode (comparten código, se
// distinguen solo por el bit E0)
// ------------------------------------------------------
// El crate `interception` no tiene un ScanCode propio para
// estas — reutiliza el mismo código crudo que la tecla del
// numpad (o, para Ctrl/Alt derecho, el mismo que su par
// izquierdo) y el bit E0 del stroke es la única diferencia
// real entre una y otra.
// ======================================================

const TABLA_EXTENDIDA: &[(&str, ScanCode)] = &[
    ("Home", ScanCode::Numpad7),
    ("Up", ScanCode::Numpad8),
    ("PageUp", ScanCode::Numpad9),
    ("Left", ScanCode::Numpad4),
    ("Right", ScanCode::Numpad6),
    ("End", ScanCode::Numpad1),
    ("Down", ScanCode::Numpad2),
    ("PageDown", ScanCode::Numpad3),
    ("Insert", ScanCode::Numpad0),
    ("Delete", ScanCode::NumpadPeriod),
    ("RightControl", ScanCode::LeftControl),
    ("RightAlt", ScanCode::LeftAlt),
    // Divide (numpad "/"): manda el mismo ScanCode crudo que el "/"
    // normal (Slash) pero con el bit E0 puesto — mismo patrón que
    // Home/Numpad7, Up/Numpad8, etc. de arriba. Sin esta entrada,
    // nombre_interception() no la distinguía de la "/" normal y caía
    // al nombre por defecto del ScanCode ("Slash"), pisando la fila
    // "Divide" que pulsadores.tsv ya tenía preparada para esta tecla.
    ("Divide", ScanCode::Slash),
    // Oem2 (Win izquierda, VK_LWIN / nativo 0x5B): la tecla física manda
    // el bit E0 (extendida). El nombre "Oem2" es el mismo que ya usa el
    // fallback de nombre_interception() para la entrada (no hay un
    // nombre inventado tipo "LeftMeta" — esa variante no existe en el
    // crate `interception`), así que agregar esta entrada NO cambia
    // nada del lado de la entrada; solo habilita la salida
    // (convertir_salida) y fija correctamente es_extendida=true para
    // cuando se emite.
    ("Oem2", ScanCode::Oem2),
];

// ======================================================
// 📥 ENTRADA
// ======================================================

pub fn convertir(code: ScanCode, es_extendida: bool, presionado: bool) -> Option<InputEvent> {
    let interception = nombre_interception(code, es_extendida);

    let Some(interno) = pulsadores::interception_a_interno(&interception) else {
        println!("⚠️ Tecla de teclado no soportada: {}", interception);

        return None;
    };

    let input = InputId::new("keyboard", interno);

    Some(if presionado {
        InputEvent::down(input, instante::ahora())
    } else {
        InputEvent::up(input, instante::ahora())
    })
}

/// Resuelve el nombre "interception" real de una tecla. Si es
/// extendida y su ScanCode está en TABLA_EXTENDIDA, ese nombre gana
/// (es la única forma de no confundirla con la tecla de numpad que
/// comparte su mismo ScanCode). Si no, es el nombre del ScanCode tal
/// cual lo entrega el crate.
fn nombre_interception(code: ScanCode, es_extendida: bool) -> String {
    if es_extendida {
        if let Some((nombre, _)) = TABLA_EXTENDIDA.iter().find(|(_, c)| *c == code) {
            return (*nombre).to_string();
        }
    }

    format!("{:?}", code)
}

// ======================================================
// 📤 SALIDA
// ======================================================

pub fn convertir_salida(input: &InputId) -> Option<(ScanCode, bool)> {
    if input.fuente() != Some("keyboard") {
        return None;
    }

    let interno = input.control()?;

    let interception = pulsadores::interno_a_interception(interno)?;

    if let Some((_, code)) = TABLA_EXTENDIDA
        .iter()
        .find(|(nombre, _)| *nombre == interception)
    {
        return Some((*code, true));
    }

    TABLA
        .iter()
        .find(|(nombre, _)| *nombre == interception)
        .map(|(_, code)| (*code, false))
}
