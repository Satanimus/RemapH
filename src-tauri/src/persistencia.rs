// ======================================================
// 💾 Persistencia RemapH V3
// ------------------------------------------------------
// Guarda y carga perfil_json.
//
// Este módulo:
//   - Escribe JSON.
//   - Lee JSON.
//
// No decide rutas.
// No busca perfiles.
// No compila.
// No toca Cache.
// No toca Runtime.
// ======================================================

use crate::perfil_json::perfil_json;

use std::fs;

use std::path::Path;

// ======================================================
// 💾 GUARDAR
// ======================================================

pub fn guardar(perfil: &perfil_json, ruta: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(perfil).map_err(|error| error.to_string())?;

    fs::write(ruta, json).map_err(|error| error.to_string())?;

    Ok(())
}

// ======================================================
// 📂 CARGAR
// ======================================================

pub fn cargar(ruta: &Path) -> Result<perfil_json, String> {
    let json = fs::read_to_string(ruta).map_err(|error| error.to_string())?;

    serde_json::from_str(&json).map_err(|error| error.to_string())
}
