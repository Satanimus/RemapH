// ======================================================
// 👤 USUARIO
// ======================================================
// 1. ¿Qué hace este archivo?
// Dueño de las rutas y archivos del usuario:
//
// Usuario/
//   ├── perfil_Default.json
//   ├── perfil_Juegos.json
//   └── ...
//
// Resuelve la carpeta Usuario, busca perfiles guardados
// en disco y decide cuál es el perfil actual (siempre el
// JSON modificado más recientemente; si no existe ninguno,
// usa Default).
//
// No conoce Runtime.
// No conoce Cache.
// No compila perfiles.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// perfil.rs (todas las operaciones de perfil pasan por
// acá para resolver rutas y nombres)
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Nombres de perfil (String) para ubicar o validar una
// ruta, o ninguna información (para listar/detectar el
// perfil actual).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Rutas (PathBuf) y nombres (String) de perfiles.
// Ejemplo:
// ruta_perfil("Juegos")
//     → Usuario/perfil_Juegos.json
// ------------------------------------------------------
// 5. Funciones del archivo
// carpeta()
//     Resuelve (y crea si no existe) la carpeta Usuario.
// rutas_perfiles()
//     Lista las rutas de todos los perfiles guardados.
// perfiles()
//     Lista los nombres de todos los perfiles guardados.
// es_perfil()
//     Determina si una ruta es un archivo de perfil.
// nombre_desde_ruta()
//     Extrae el nombre de perfil desde su ruta.
// ruta_perfil()
//     Arma la ruta de un perfil a partir de su nombre.
// perfil_default()
//     Ruta del perfil "Default".
// perfil_actual()
//     Ruta del perfil modificado más recientemente
//     (o Default si no existe ninguno).
// nombre_actual()
//     Nombre del perfil actual.
// ------------------------------------------------------
// Transformación que realiza
// %APPDATA%
//     ↓
// Usuario/
//     ↓
// perfil_actual() → perfil_Juegos.json
// ======================================================

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ======================================================
// 📁 OBTENER CARPETA USUARIO
// ======================================================

pub(crate) fn carpeta() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|error| error.to_string())?;

    let carpeta = PathBuf::from(appdata).join("RemapH").join("Usuario");

    fs::create_dir_all(&carpeta).map_err(|error| error.to_string())?;

    Ok(carpeta)
}

// ======================================================
// 📄 BUSCAR PERFILES
// ======================================================

fn rutas_perfiles() -> Result<Vec<PathBuf>, String> {
    let carpeta = carpeta()?;

    let mut perfiles = Vec::new();

    let entradas = fs::read_dir(&carpeta).map_err(|error| error.to_string())?;

    for entrada in entradas {
        let ruta = entrada.map_err(|error| error.to_string())?.path();

        if !es_perfil(&ruta) {
            continue;
        }

        perfiles.push(ruta);
    }

    Ok(perfiles)
}

// ======================================================
// 📋 LISTAR PERFILES
// ======================================================

pub fn perfiles() -> Result<Vec<String>, String> {
    let mut nombres = rutas_perfiles()?
        .into_iter()
        .filter_map(|ruta| nombre_desde_ruta(&ruta))
        .collect::<Vec<_>>();

    nombres.sort();

    Ok(nombres)
}

// ======================================================
// 🔎 ES PERFIL
// ======================================================

fn es_perfil(ruta: &Path) -> bool {
    let Some(nombre) = ruta.file_name().and_then(|nombre| nombre.to_str()) else {
        return false;
    };

    nombre.starts_with("perfil_") && nombre.ends_with(".json")
}

// ======================================================
// 🆔 NOMBRE DESDE RUTA
// ======================================================

fn nombre_desde_ruta(ruta: &Path) -> Option<String> {
    let nombre = ruta.file_name()?.to_str()?;

    let nombre = nombre.strip_prefix("perfil_")?.strip_suffix(".json")?;

    Some(nombre.to_string())
}

// ======================================================
// 📍 RUTA POR NOMBRE
// ======================================================

pub fn ruta_perfil(nombre: &str) -> Result<PathBuf, String> {
    if nombre.trim().is_empty() {
        return Err("El nombre del perfil está vacío".to_string());
    }

    if nombre.contains('/') || nombre.contains('\\') || nombre == "." || nombre == ".." {
        return Err("Nombre de perfil inválido".to_string());
    }

    Ok(carpeta()?.join(format!("perfil_{}.json", nombre)))
}

// ======================================================
// 🆕 PERFIL DEFAULT
// ======================================================

fn perfil_default() -> Result<PathBuf, String> {
    ruta_perfil("Default")
}

// ======================================================
// 🕒 PERFIL ACTUAL
// ======================================================

pub fn perfil_actual() -> Result<PathBuf, String> {
    let perfiles = rutas_perfiles()?;

    let Some(perfil) = perfiles.into_iter().max_by_key(|ruta| {
        fs::metadata(ruta)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }) else {
        return perfil_default();
    };

    Ok(perfil)
}

// ======================================================
// 🆔 NOMBRE PERFIL ACTUAL
// ======================================================

pub fn nombre_actual() -> Result<String, String> {
    let ruta = perfil_actual()?;

    nombre_desde_ruta(&ruta).ok_or_else(|| "No se pudo determinar el nombre del perfil".to_string())
}
