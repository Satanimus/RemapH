// ======================================================
// 🧩 macros
// ------------------------------------------------------
// Gestiona archivos de Macro almacenados en /Macros.
//
// macros NO:
// - Ejecuta macros (eso es runtime.rs / Etapa 8).
// - Conoce Cache ni AccionCache::Macro.
// - Sabe qué fila del perfil referencia qué macro.
//
// Responsabilidad:
// - Listar macros guardadas.
// - Crear una macro nueva (vacía).
// - Clonar una macro existente.
// - Abrir (cargar) una macro.
// - Guardar una macro editada.
//
// Flujo:
// UI
//   ↓
// comandos
//   ↓
// macros
//   ↓
// macro_json / macro_usuario
//
// Nombre del módulo en plural ("macros", no "macro") porque
// `macro` es palabra reservada en Rust.
// ======================================================

use std::fs;
use std::path::Path;

use crate::macro_json::MacroArchivoJson;
use crate::macro_usuario;

// ======================================================
// 📋 LISTAR MACROS
// ======================================================

pub fn listar_macros() -> Result<Vec<String>, String> {
    macro_usuario::macros()
}

// ======================================================
// 🆕 CREAR MACRO NUEVA
// ------------------------------------------------------
// nombre: None o "" → se genera "macro_NNN" automático (ver
// macro_usuario::siguiente_numero_automatico). Con nombre
// personalizado, se resuelve una colisión igual que un perfil
// nuevo (" (2)", " (3)"...).
// ======================================================

pub fn crear_macro_nueva(nombre: Option<String>) -> Result<MacroArchivoJson, String> {
    let nombre = match nombre.as_deref().map(str::trim) {
        Some(nombre) if !nombre.is_empty() => macro_usuario::nombre_disponible(nombre)?,
        _ => macro_usuario::siguiente_numero_automatico()?,
    };

    let macro_archivo = MacroArchivoJson::nueva(nombre);

    guardar_en_disco(
        &macro_archivo,
        &macro_usuario::ruta_macro(&macro_archivo.nombre)?,
    )?;

    Ok(macro_archivo)
}

// ======================================================
// 📋 CLONAR MACRO
// ------------------------------------------------------
// nombre_nuevo: None o "" → usa el nombre de origen como base
// (mismo criterio de colisión que arriba).
// ======================================================

pub fn clonar_macro(
    nombre_origen: String,
    nombre_nuevo: Option<String>,
) -> Result<MacroArchivoJson, String> {
    let ruta_origen = macro_usuario::ruta_macro(&nombre_origen)?;

    let mut macro_archivo = cargar_desde_disco(&ruta_origen)?;

    let base = match nombre_nuevo.as_deref().map(str::trim) {
        Some(nombre) if !nombre.is_empty() => nombre,
        _ => &nombre_origen,
    };

    macro_archivo.nombre = macro_usuario::nombre_disponible(base)?;

    guardar_en_disco(
        &macro_archivo,
        &macro_usuario::ruta_macro(&macro_archivo.nombre)?,
    )?;

    Ok(macro_archivo)
}

// ======================================================
// 📂 ABRIR MACRO
// ======================================================

pub fn abrir_macro(nombre: String) -> Result<MacroArchivoJson, String> {
    let ruta = macro_usuario::ruta_macro(&nombre)?;

    if !ruta.exists() {
        return Err("La macro seleccionada no existe".to_string());
    }

    cargar_desde_disco(&ruta)
}

// ======================================================
// 💾 GUARDAR MACRO
// ------------------------------------------------------
// Guarda bajo macro_archivo.nombre tal cual viene — sin
// manejo de "renombrar mientras se edita" todavía (queda para
// la Etapa 6, el editor completo).
// ======================================================

pub fn guardar_macro(macro_archivo: MacroArchivoJson) -> Result<(), String> {
    let ruta = macro_usuario::ruta_macro(&macro_archivo.nombre)?;

    guardar_en_disco(&macro_archivo, &ruta)
}

// ======================================================
// 💾 GUARDAR EN DISCO
// ======================================================

fn guardar_en_disco(macro_archivo: &MacroArchivoJson, ruta: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(macro_archivo).map_err(|error| error.to_string())?;

    fs::write(ruta, json).map_err(|error| error.to_string())?;

    Ok(())
}

// ======================================================
// 📂 CARGAR DESDE DISCO
// ======================================================

fn cargar_desde_disco(ruta: &Path) -> Result<MacroArchivoJson, String> {
    let json = fs::read_to_string(ruta).map_err(|error| error.to_string())?;

    serde_json::from_str(&json).map_err(|error| error.to_string())
}
