// ======================================================
// 📦 Eventos RemapH V3
// ------------------------------------------------------
// Representación genérica de entradas físicas.
//
// El Runtime no conoce:
//   - Teclado.
//   - Mouse.
//   - Joystick.
//
// Todo se representa mediante InputId.
//
// Este evento todavía es físico.
// No contiene:
//   - Simple.
//   - Doble.
//   - Mantenido.
//
// Esa transformación pertenece a:
//     AnalizadorTrigger
// ======================================================

use serde::{Deserialize, Serialize};

use std::hash::Hash;

// ======================================================
// ⏱️ INSTANTE
// ------------------------------------------------------
// Momento en que ocurrió un evento físico.
//
// Se utiliza para:
//   - Doble toque.
//   - Tiempo mantenido.
//   - Secuencias.
//
// Unidad:
// milisegundos.
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
    pub fn new(fuente: &str, control: &str) -> Self {
        Self(format!("{}:{}", fuente, control))
    }

    // ==================================================
    // 🧩 FUENTE
    // ==================================================

    pub fn fuente(&self) -> Option<&str> {
        self.0.split_once(':').map(|(fuente, _)| fuente)
    }

    // ==================================================
    // 🎛️ CONTROL
    // ==================================================

    pub fn control(&self) -> Option<&str> {
        self.0.split_once(':').map(|(_, control)| control)
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
// 📡 EVENTO FÍSICO GENÉRICO
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

    pub fn pulse(input: InputId, instante: Instante) -> Self {
        Self {
            input,

            state: InputState::Pulse,

            instante,
        }
    }
}

// ======================================================
// 🔁 COMPATIBILIDAD INTERNA
// ======================================================

pub type Evento = InputEvent;
