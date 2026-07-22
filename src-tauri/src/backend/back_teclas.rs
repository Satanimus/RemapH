// ======================================================
// 🎹 Back_Teclas RemapH V3
// ------------------------------------------------------
// Traduce teclado físico a InputEvent genérico.
// También convierte outputs genéricos a ScanCode.
// ======================================================

use crate::eventos::{Evento, InputId};
use crate::instante;
use interception::ScanCode;

// ======================================================
// 📥 ENTRADA
// ======================================================

pub fn convertir(code: ScanCode, presionado: bool) -> Evento {
    let input = InputId::new("keyboard", &format!("{:?}", code));

    if presionado {
        Evento::down(input, instante::ahora())
    } else {
        Evento::up(input, instante::ahora())
    }
}

// ======================================================
// 📤 SALIDA
// ======================================================

pub fn convertir_salida(input: &InputId) -> Option<ScanCode> {
    if input.fuente() != Some("keyboard") {
        return None;
    }

    let tecla = input.control()?;

    match tecla {
        // ----------------------------------------------
        // 🔤 LETRAS
        // ----------------------------------------------
        "A" => Some(ScanCode::A),
        "B" => Some(ScanCode::B),
        "C" => Some(ScanCode::C),
        "D" => Some(ScanCode::D),
        "E" => Some(ScanCode::E),
        "F" => Some(ScanCode::F),
        "G" => Some(ScanCode::G),
        "H" => Some(ScanCode::H),
        "I" => Some(ScanCode::I),
        "J" => Some(ScanCode::J),
        "K" => Some(ScanCode::K),
        "L" => Some(ScanCode::L),
        "M" => Some(ScanCode::M),
        "N" => Some(ScanCode::N),
        "O" => Some(ScanCode::O),
        "P" => Some(ScanCode::P),
        "Q" => Some(ScanCode::Q),
        "R" => Some(ScanCode::R),
        "S" => Some(ScanCode::S),
        "T" => Some(ScanCode::T),
        "U" => Some(ScanCode::U),
        "V" => Some(ScanCode::V),
        "W" => Some(ScanCode::W),
        "X" => Some(ScanCode::X),
        "Y" => Some(ScanCode::Y),
        "Z" => Some(ScanCode::Z),

        // ----------------------------------------------
        // 🔢 NÚMEROS
        // ----------------------------------------------
        "Num1" => Some(ScanCode::Num1),
        "Num2" => Some(ScanCode::Num2),
        "Num3" => Some(ScanCode::Num3),
        "Num4" => Some(ScanCode::Num4),
        "Num5" => Some(ScanCode::Num5),
        "Num6" => Some(ScanCode::Num6),
        "Num7" => Some(ScanCode::Num7),
        "Num8" => Some(ScanCode::Num8),
        "Num9" => Some(ScanCode::Num9),
        "Num0" => Some(ScanCode::Num0),

        // ----------------------------------------------
        // ⌨️ BÁSICAS
        // ----------------------------------------------
        "Enter" => Some(ScanCode::Enter),
        "Esc" => Some(ScanCode::Esc),
        "Backspace" => Some(ScanCode::Backspace),
        "Tab" => Some(ScanCode::Tab),
        "Space" => Some(ScanCode::Space),

        // ----------------------------------------------
        // 🔣 SÍMBOLOS
        // ----------------------------------------------
        "Minus" => Some(ScanCode::Minus),
        "Equals" => Some(ScanCode::Equals),
        "LeftBracket" => Some(ScanCode::LeftBracket),
        "RightBracket" => Some(ScanCode::RightBracket),
        "BackSlash" => Some(ScanCode::BackSlash),
        "SemiColon" => Some(ScanCode::SemiColon),
        "Apostrophe" => Some(ScanCode::Apostrophe),
        "Grave" => Some(ScanCode::Grave),
        "Comma" => Some(ScanCode::Comma),
        "Period" => Some(ScanCode::Period),
        "Slash" => Some(ScanCode::Slash),

        // ----------------------------------------------
        // 🔒 BLOQUEO
        // ----------------------------------------------
        "CapsLock" => Some(ScanCode::CapsLock),
        "NumLock" => Some(ScanCode::NumLock),
        "ScrollLock" => Some(ScanCode::ScrollLock),

        // ----------------------------------------------
        // ⚙️ FUNCIÓN
        // ----------------------------------------------
        "F1" => Some(ScanCode::F1),
        "F2" => Some(ScanCode::F2),
        "F3" => Some(ScanCode::F3),
        "F4" => Some(ScanCode::F4),
        "F5" => Some(ScanCode::F5),
        "F6" => Some(ScanCode::F6),
        "F7" => Some(ScanCode::F7),
        "F8" => Some(ScanCode::F8),
        "F9" => Some(ScanCode::F9),
        "F10" => Some(ScanCode::F10),
        "F11" => Some(ScanCode::F11),
        "F12" => Some(ScanCode::F12),

        // ----------------------------------------------
        // 🎛️ MODIFICADORES
        // ----------------------------------------------
        "LeftControl" => Some(ScanCode::LeftControl),
        "LeftShift" => Some(ScanCode::LeftShift),
        "LeftAlt" => Some(ScanCode::LeftAlt),

        // ----------------------------------------------
        // ❌ NO SOPORTADO
        // ----------------------------------------------
        _ => None,
    }
}
