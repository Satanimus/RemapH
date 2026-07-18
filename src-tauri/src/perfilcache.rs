// ======================================================
// 🧠 PerfilCache RemapH V3
// ------------------------------------------------------
// Modelo interno compilado.
//
// Este modelo:
//   - No se serializa.
//   - No conoce JSON.
//   - No conoce UI.
//
// Está preparado para Runtime y Cache.
//
// PerfilJson
//      ↓
// Compilador
//      ↓
// PerfilCache
// ======================================================

use crate::eventos::InputId;


// ======================================================
// 🎯 REMAPEO CACHE
// ======================================================

#[derive(
    Clone,
    Debug,
    PartialEq,
)]
pub struct RemapeoCache {

    pub trigger:
        TriggerCache,

    pub accion:
        AccionCache,

}


// ======================================================
// ⌨️ TRIGGER CACHE
// ======================================================

#[derive(
    Clone,
    Debug,
    PartialEq,
)]
pub struct TriggerCache {

    pub modificadores:
        Vec<InputId>,

    pub gatillo:
        InputId,

}


// ======================================================
// 📤 ACCIÓN CACHE
// ======================================================

#[derive(
    Clone,
    Debug,
    PartialEq,
)]
pub enum AccionCache {

    Emitir(
        InputId
    ),

}