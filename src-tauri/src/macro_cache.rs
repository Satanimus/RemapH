// ======================================================
// 🗃️ macro_cache
// ------------------------------------------------------
// Cache en memoria de los archivos de Macro que están
// siendo editados. Mientras el editor está abierto, los
// cambios se escriben acá (no al archivo de usuario).
// Solo al hacer "Guardar" se promueve la copia al disco.
//
// La clave del mapa es el nombre de la macro (sin .json),
// igual que macro_usuario::ruta_macro.
//
// Funciones:
// escribir_cache(nombre, macro_archivo)
//     Guarda o reemplaza la copia en cache.
// leer_cache(nombre) -> Option<MacroArchivoJson>
//     Devuelve la copia en cache, o None si no existe.
// descartar_cache(nombre)
//     Elimina la copia en cache (Cancelar).
// promover_cache(nombre) -> Result<(), String>
//     Escribe la copia en cache al archivo de usuario
//     y luego la descarta.
// ======================================================

use crate::macro_json::MacroArchivoJson;
use crate::macro_usuario;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

// ======================================================
// 🗂️ ESTADO GLOBAL
// ======================================================

static CACHE: Mutex<HashMap<String, MacroArchivoJson>> = Mutex::new(HashMap::new());

// ======================================================
// ✍️ ESCRIBIR EN CACHE
// ======================================================

pub fn escribir_cache(nombre: &str, macro_archivo: MacroArchivoJson) {
    CACHE
        .lock()
        .unwrap()
        .insert(nombre.to_string(), macro_archivo);
}

// ======================================================
// 📖 LEER DE CACHE
// ======================================================

pub fn leer_cache(nombre: &str) -> Option<MacroArchivoJson> {
    CACHE.lock().unwrap().get(nombre).cloned()
}

// ======================================================
// 🗑️ DESCARTAR CACHE (Cancelar)
// ======================================================

pub fn descartar_cache(nombre: &str) {
    CACHE.lock().unwrap().remove(nombre);
}

// ======================================================
// 💾 PROMOVER CACHE → ARCHIVO DE USUARIO (Guardar)
// ------------------------------------------------------
// Escribe la copia en cache al .json de usuario y luego
// la elimina del mapa. Si el nombre en cache difiere del
// nombre del campo `nombre` dentro del MacroArchivoJson,
// se usa la clave del mapa (el nombre con el que fue
// abierto) para localizar el archivo destino, y el campo
// interno para actualizar el contenido. La ruta de
// destino siempre se deriva de la clave del mapa — el
// renombrado físico lo gestiona macros::renombrar_macro,
// no esta función.
// ======================================================

pub fn promover_cache(nombre: &str) -> Result<(), String> {
    let macro_archivo = {
        let guard = CACHE.lock().unwrap();

        guard
            .get(nombre)
            .cloned()
            .ok_or_else(|| format!("No hay cache para la macro \"{}\"", nombre))?
    };

    let ruta = macro_usuario::ruta_macro(nombre)?;

    let json = serde_json::to_string_pretty(&macro_archivo).map_err(|error| error.to_string())?;

    fs::write(&ruta, json).map_err(|error| error.to_string())?;

    descartar_cache(nombre);

    Ok(())
}
