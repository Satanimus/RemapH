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
// No conoce AnalizadorTrigger.
// No conoce Cache.
// No conoce Runtime.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// back_interception (traducir(), para la entrada)
// back_interception, para la salida (convertir_salida())
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// convertir(): un ScanCode crudo de interception + si
// fue Down o Up.
// convertir_salida(): un InputId (interno) a emitir.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// convertir(): Option<Evento>. None si la tecla no está
// en pulsadores.tsv (no soportada).
// convertir_salida(): Option<ScanCode>. None si el
// InputId no es de teclado, o no tiene ScanCode asociado.
// ------------------------------------------------------
// 5. Funciones del archivo
// TABLA
//     Única lista de pares texto ↔ ScanCode. Fuente para
//     las dos direcciones de conversión.
// convertir()
//     ScanCode + estado → Evento (vía pulsadores.tsv).
//     Registra en consola las teclas no soportadas.
// convertir_salida()
//     InputId → ScanCode (vía pulsadores.tsv + TABLA).
// ------------------------------------------------------
// Transformación:
//
// ENTRADA:
// ScanCode (interception)
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
// TABLA
//     ↓
// ScanCode (interception)
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
// 📥 ENTRADA
// ======================================================

pub fn convertir(code: ScanCode, presionado: bool) -> Option<InputEvent> {
    let interception = format!("{:?}", code);

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

// ======================================================
// 📤 SALIDA
// ======================================================

pub fn convertir_salida(input: &InputId) -> Option<ScanCode> {
    if input.fuente() != Some("keyboard") {
        return None;
    }

    let interno = input.control()?;

    let interception = pulsadores::interno_a_interception(interno)?;

    TABLA
        .iter()
        .find(|(nombre, _)| *nombre == interception)
        .map(|(_, code)| *code)
}
