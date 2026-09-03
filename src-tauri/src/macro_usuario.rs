// ======================================================
// 🧩 macro_usuario
// ------------------------------------------------------
// Dueño de las rutas y archivos de Macro del usuario:
//
// Usuario/
//   └── Macros/
//         ├── macro_001.json
//         ├── Copiar celda.json
//         └── ...
//
// Mismo criterio que usuario.rs, pero para /Macros en vez de
// /Usuario directo. A diferencia de los perfiles, acá no hace
// falta un prefijo tipo "perfil_" para distinguir del resto
// de la carpeta — /Macros es una carpeta propia, cualquier
// .json que contenga es una macro. Tampoco hay noción de
// "macro actual": cada fila con tipo == "macro" referencia
// una por nombre (ver accion_referencia en compilador.rs).
//
// No conoce Runtime. No conoce Cache. No compila nada.
// ------------------------------------------------------
// Funciones del archivo
// carpeta()
//     Resuelve (y crea si no existe) la carpeta Macros.
// macros()
//     Lista los nombres de todas las macros guardadas.
// ruta_macro()
//     Arma la ruta de una macro a partir de su nombre.
// nombre_disponible()
//     Nombre libre a partir de uno pedido, agregando
//     " (2)", " (3)"... si ya existe (mismo criterio que
//     perfil.rs::siguiente_nombre).
// siguiente_numero_automatico()
//     Nombre "macro_NNN" libre, para cuando no se pide un
//     nombre personalizado ("Nueva Macro" sin escribir nada).
// ======================================================

use crate::config;
use std::fs;
use std::path::{Path, PathBuf};

// ======================================================
// 📁 OBTENER CARPETA MACROS
// ======================================================

fn carpeta() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|error| error.to_string())?;

    let carpeta = PathBuf::from(appdata)
        .join(config::NOMBRE_APP)
        .join("Usuario")
        .join("Macros");

    fs::create_dir_all(&carpeta).map_err(|error| error.to_string())?;

    Ok(carpeta)
}

// ======================================================
// 📋 LISTAR MACROS
// ======================================================

pub fn macros() -> Result<Vec<String>, String> {
    let carpeta = carpeta()?;

    let mut nombres = Vec::new();

    let entradas = fs::read_dir(&carpeta).map_err(|error| error.to_string())?;

    for entrada in entradas {
        let ruta = entrada.map_err(|error| error.to_string())?.path();

        if let Some(nombre) = nombre_desde_ruta(&ruta) {
            nombres.push(nombre);
        }
    }

    nombres.sort();

    Ok(nombres)
}

// ======================================================
// 🆔 NOMBRE DESDE RUTA
// ======================================================

fn nombre_desde_ruta(ruta: &Path) -> Option<String> {
    if !ruta.is_file() {
        return None;
    }

    let nombre = ruta.file_name()?.to_str()?;

    let nombre = nombre.strip_suffix(".json")?;

    Some(nombre.to_string())
}

// ======================================================
// 📍 RUTA POR NOMBRE
// ======================================================

pub fn ruta_macro(nombre: &str) -> Result<PathBuf, String> {
    if nombre.trim().is_empty() {
        return Err("El nombre de la macro está vacío".to_string());
    }

    if nombre.contains('/') || nombre.contains('\\') || nombre == "." || nombre == ".." {
        return Err("Nombre de macro inválido".to_string());
    }

    Ok(carpeta()?.join(format!("{}.json", nombre)))
}

// ======================================================
// 🔢 NOMBRE DISPONIBLE (personalizado)
// ------------------------------------------------------
// Mismo criterio que perfil.rs::siguiente_nombre — usado para
// "Nueva Macro" con nombre elegido por el usuario, y como
// nombre por defecto al clonar ("Clonar Macro").
// ======================================================

pub fn nombre_disponible(base: &str) -> Result<String, String> {
    let ruta = ruta_macro(base)?;

    if !ruta.exists() {
        return Ok(base.to_string());
    }

    let mut numero = 2;

    loop {
        let nombre = format!("{} ({})", base, numero);

        let ruta = ruta_macro(&nombre)?;

        if !ruta.exists() {
            return Ok(nombre);
        }

        numero += 1;
    }
}

// ======================================================
// 🔢 SIGUIENTE NÚMERO AUTOMÁTICO
// ------------------------------------------------------
// "Nueva Macro" sin nombre personalizado — mismo espíritu que
// perfil_default() pero numerado en vez de un nombre fijo
// ("Default"), porque acá SIEMPRE hay que generar uno (no
// existe un nombre por defecto natural para una macro).
// ======================================================

pub fn siguiente_numero_automatico() -> Result<String, String> {
    let mut numero = 1;

    loop {
        let nombre = format!("macro_{:03}", numero);

        let ruta = ruta_macro(&nombre)?;

        if !ruta.exists() {
            return Ok(nombre);
        }

        numero += 1;
    }
}
