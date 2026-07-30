// ======================================================
// 📦 EVENTOS RemapH V3
// ======================================================
// ETAPA 1 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Recibe información proveniente del backend de captura y del módulo instante, y la estructura en un formato estándar
// (InputEvent) que será utilizado por todo el motor.
// Los constructores permiten instanciar fácilmente estos formatos.
// Aquí todavía NO existe el concepto de trigger. Sólo existen eventos físicos.
// + Incluye Display, una limpieza de lectura para los println!("{}", input);
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// Backend de captura
// (Interception / rdev)
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Construye un InputEvent a partir de información proporcionada por el backend de captura.
// El Instante ya viene generado por el backend mediante instante::ahora().
// Eventos físicos:
// InputId / Estado (Down / Up / Pulse) / Instante
//
// Ejemplo:
// keyboard:A
// Down
// 105263 ms
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// InputEvent
// Ejemplo:
// InputEvent {
//     input: keyboard:A,
//     state: Down,
//     instante: 105263,
// }
// ------------------------------------------------------
// 5. Funciones del archivo
// Display::fmt()
//     Permite imprimir InputId con println!("{}", input).
// InputId::new()
//     Construye un identificador interno.
// fuente()
//     Devuelve el dispositivo.
// control()
//     Devuelve el botón o tecla.
//InputEvent::down()
//    Da formato a un evento físico Down.
//InputEvent::up()
//    Da formato a un evento físico Up.
//InputEvent::pulse()
//    Da formato a un evento Pulse.
// ------------------------------------------------------
// Transformación que realiza
// Windows
//     ↓
// VK_A Down
//     ↓
// InputEvent {
//     input: keyboard:A,
//     state: Down,
//     instante: 105263,}
// ======================================================
use serde::{Deserialize, Serialize};
use std::hash::Hash;

// ======================================================
// ⏱️ INSTANTE
// ======================================================

pub type Instante = u64;

// ======================================================
// 🆔 IDENTIDAD DE INPUT
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputId(String);

// ======================================================
// 🏗️ CONSTRUCTOR
// ======================================================

impl InputId {
    #[inline]
    pub fn new(fuente: &str, control: &str) -> Self {
        Self(format!("{}:{}", fuente, control))
    }

    // ==================================================
    // 🧩 FUENTE
    // ==================================================

    #[inline]
    pub fn fuente(&self) -> Option<&str> {
        self.0.split_once(':').map(|(fuente, _)| fuente)
    }

    // ==================================================
    // 🎛️ CONTROL
    // ==================================================

    #[inline]
    pub fn control(&self) -> Option<&str> {
        self.0.split_once(':').map(|(_, control)| control)
    }
}

// ======================================================
// 🖨️ DISPLAY
// ------------------------------------------------------
// Permite imprimir InputId directamente:   println!("{}", input);
// En lugar de:                             println!("{:?}", input);
// ======================================================

impl std::fmt::Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ======================================================
// 🔄 ESTADO FÍSICO
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputState {
    Down,

    Up,

    Pulse,
}

// ======================================================
// 📡 EVENTO FÍSICO
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEvent {
    pub input: InputId,

    pub state: InputState,

    pub instante: Instante,
}

// ======================================================
// 🧱 CONSTRUCTORES
// ======================================================

impl InputEvent {
    // ==================================================
    // ⬇️ DOWN
    // ==================================================

    #[inline]
    pub fn down(input: InputId, instante: Instante) -> Self {
        Self {
            input,

            state: InputState::Down,

            instante,
        }
    }

    // ==================================================
    // ⬆️ UP
    // ==================================================

    #[inline]
    pub fn up(input: InputId, instante: Instante) -> Self {
        Self {
            input,

            state: InputState::Up,

            instante,
        }
    }

    // ==================================================
    // ⚡ PULSE
    // ==================================================

    #[inline]
    pub fn pulse(input: InputId, instante: Instante) -> Self {
        Self {
            input,

            state: InputState::Pulse,

            instante,
        }
    }
}
