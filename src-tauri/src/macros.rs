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
// - Abrir (cargar) una macro.
// - Guardar una macro editada.
// - Renombrar una macro (Etapa 8A).
// - Eliminar una macro (Etapa 8A).
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

use crate::macro_cache;
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
// 🆕 CREAR MACRO NUEVA → CACHE
// ------------------------------------------------------
// Igual que crear_macro_nueva (crea el archivo de usuario
// vacío en disco), pero además escribe la copia inicial en
// CACHE_MACROS para que el editor trabaje sobre ella.
// ======================================================

pub fn crear_macro_nueva_a_cache(nombre: Option<String>) -> Result<MacroArchivoJson, String> {
    let macro_archivo = crear_macro_nueva(nombre)?;

    macro_cache::escribir_cache(&macro_archivo.nombre, macro_archivo.clone());

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
// 📂 ABRIR MACRO → CACHE
// ------------------------------------------------------
// Carga la macro desde disco y escribe la copia en
// CACHE_MACROS para que el editor trabaje sobre ella.
// ======================================================

pub fn abrir_macro_a_cache(nombre: String) -> Result<MacroArchivoJson, String> {
    let macro_archivo = abrir_macro(nombre)?;

    macro_cache::escribir_cache(&macro_archivo.nombre, macro_archivo.clone());

    Ok(macro_archivo)
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
// 💾 GUARDAR DESDE CACHE
// ------------------------------------------------------
// Promueve la copia en cache al archivo de usuario (botón
// "Guardar" del editor). Delega en macro_cache::promover_cache.
// ======================================================

pub fn guardar_desde_cache(nombre: &str) -> Result<(), String> {
    macro_cache::promover_cache(nombre)
}

// ======================================================
// 🗑️ DESCARTAR CACHE (Cancelar)
// ------------------------------------------------------
// Descarta la copia en cache sin tocar el archivo de usuario
// (botón "Cancelar" del editor). Delega en macro_cache::descartar_cache.
// ======================================================

pub fn descartar_cache_macro(nombre: &str) {
    macro_cache::descartar_cache(nombre);
}

// ======================================================
// ✏️ RENOMBRAR MACRO — Etapa 8A
// ------------------------------------------------------
// Mismo patrón que perfil.rs::renombrar_perfil (nombre_disponible +
// fs::rename), pero SIN el chequeo de "¿alguna fila la referencia?"
// — decisión del usuario: renombrar/eliminar una macro referenciada
// se trata igual que una fila "abrir" cuya ruta ya no existe (aviso
// amarillo en la próxima compilación, vía convertir_macro en
// compilador.rs), no un error duro que bloquee la operación. Las
// filas que ya apuntaban al nombre viejo quedan apuntando al nombre
// viejo — el usuario las reasigna a mano si hace falta.
// ======================================================

pub fn renombrar_macro(nombre_actual: String, nombre_nuevo: String) -> Result<String, String> {
    let nombre_nuevo = nombre_nuevo.trim();

    if nombre_nuevo.is_empty() {
        return Err("El nombre de la macro está vacío".to_string());
    }

    if nombre_nuevo == nombre_actual {
        return Err("La macro ya tiene ese nombre".to_string());
    }

    let ruta_actual = macro_usuario::ruta_macro(&nombre_actual)?;

    if !ruta_actual.exists() {
        return Err("La macro seleccionada no existe".to_string());
    }

    let nombre_nuevo = macro_usuario::nombre_disponible(nombre_nuevo)?;

    let ruta_nueva = macro_usuario::ruta_macro(&nombre_nuevo)?;

    fs::rename(&ruta_actual, &ruta_nueva).map_err(|error| error.to_string())?;

    let mut macro_archivo = cargar_desde_disco(&ruta_nueva)?;

    macro_archivo.nombre = nombre_nuevo.clone();

    guardar_en_disco(&macro_archivo, &ruta_nueva)?;

    Ok(nombre_nuevo)
}

// ======================================================
// 🗑️ ELIMINAR MACRO — Etapa 8A
// ------------------------------------------------------
// Mismo criterio "no bloquear" que renombrar_macro — ver comentario
// arriba. Borra el archivo sin comprobar si alguna fila del perfil
// activo la referencia.
// ======================================================

pub fn eliminar_macro(nombre: String) -> Result<(), String> {
    let ruta = macro_usuario::ruta_macro(&nombre)?;

    if !ruta.exists() {
        return Err("La macro seleccionada no existe".to_string());
    }

    fs::remove_file(&ruta).map_err(|error| error.to_string())
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
