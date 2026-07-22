// ======================================================
// 📦 PerfilCache RemapH V3
// ------------------------------------------------------
// Modelo interno compilado.
//
// PerfilJson
//     ↓
// Compilador
//     ↓
// PerfilCache
//     ↓
// Cache
//     ↓
// Runtime
// ======================================================

use crate::eventos::InputId;

// ======================================================
// 🖥️ APP CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCache {
    Global,

    Programa { nombre: String, segundo_plano: bool },
}

// ======================================================
// 🎯 CONDICIÓN CACHE
// ======================================================

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CondicionTrigger {
    Simple,

    Doble,

    Mantenido,
}

// ======================================================
// 🧩 REMAPEO CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct RemapeoCache {
    pub app: AppCache,

    pub trigger: TriggerCache,

    pub accion: AccionCache,
}

// ======================================================
// ⌨️ TRIGGER CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct TriggerCache {
    pub modificadores: Vec<InputId>,

    pub gatillo: InputId,

    pub condicion: CondicionTrigger,
}

// ======================================================
// ⚡ ACCIÓN CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum AccionCache {
    Emitir(InputId),
}
