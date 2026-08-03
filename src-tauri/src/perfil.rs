// ======================================================
// 👤 Perfil
// ======================================================
//
// Gestiona perfiles almacenados.
//
// Perfil NO:
//
// - Captura eventos.
// - Analiza triggers.
// - Ejecuta acciones.
// - Conoce Runtime.
//
// Responsabilidad:
//
// - Crear perfiles.
// - Cargar perfiles.
// - Guardar perfiles.
// - Cambiar perfil actual.
// - Eliminar perfiles.
// - Clonar perfiles.
// - Renombrar perfiles.
//
// Flujo:
//
// UI
// ↓
// comandos
// ↓
// perfil
// ↓
// perfil_json
//
// ======================================================
// ======================================================
//
// Funciones:
//
// activar_perfil()
//     Carga perfil actual y activa cache.
//
// desactivar_perfil()
//     Desactiva perfil actual.
//
// guardar_perfil()
//     Guarda perfil actual y recompila cache.
//     Recibe perfil ya convertido desde perfil_ui.
//
// obtener_perfil_actual()
//     Obtiene perfil actual.
//
// obtener_perfiles()
//     Lista perfiles disponibles.
//
// obtener_nombre_actual()
//     Obtiene nombre perfil actual.
//
// obtener_estado_cache()
//     Devuelve si existe cache activo.
//
// restaurar_perfil_actual()
//     Recupera perfil guardado.
//
// clonar_perfil()
//     Crea copia de perfil.
//     Recibe perfil ya convertido desde perfil_ui.
//
// renombrar_perfil()
//     Cambia nombre perfil.
//
// eliminar_perfil_actual()
//     Elimina perfil actual. Si quedan otros perfiles, pasa al
//     primero en orden alfabético. Si no queda ninguno, crea un
//     Default vacío (misma lógica que crear_perfil_nuevo()).
//
// crear_perfil_nuevo()
//     Crea perfil vacío.
//
// seleccionar_perfil()
//     Cambia perfil activo.
//
// resultado_perfil()
//     Construye respuesta completa para UI.
//
// siguiente_nombre()
//     Genera nombre disponible.
//
// guardar_en_disco()
//     Guarda perfil json en disco
//
// cargar_desde_disco()
//     Carga perfil json en disco
// ======================================================

use crate::cache;
use crate::compilador;
use crate::perfil_json::perfil_json;
use crate::usuario;
use std::fs;
use std::path::Path;

use crate::perfil_ui::ResultadoPerfil;

// ======================================================
// 🟢 ACTIVAR PERFIL
// ======================================================

pub fn activar_perfil() -> Result<bool, String> {
    let ruta = usuario::perfil_actual()?;

    let perfil = cargar_desde_disco(&ruta)?;

    compilador::compilar(&perfil);

    Ok(!cache::esta_vacia())
}

// ======================================================
// 🔴 DESACTIVAR PERFIL
// ======================================================

pub fn desactivar_perfil() {
    cache::borrar_cache();
}

// ======================================================
// 📂 OBTENER PERFIL ACTUAL
// ======================================================

pub fn obtener_perfil_actual() -> Result<perfil_json, String> {
    let ruta = usuario::perfil_actual()?;

    if !ruta.exists() {
        let perfil = perfil_json::nuevo();

        guardar_en_disco(&perfil, &ruta)?;

        compilador::compilar(&perfil);

        return Ok(perfil);
    }

    let perfil = cargar_desde_disco(&ruta)?;

    compilador::compilar(&perfil);

    Ok(perfil)
}

// ======================================================
// 💾 GUARDAR PERFIL
// ======================================================

pub fn guardar_perfil(perfil: perfil_json) -> Result<bool, String> {
    let ruta = usuario::perfil_actual()?;

    guardar_en_disco(&perfil, &ruta)?;

    compilador::compilar(&perfil);

    Ok(!cache::esta_vacia())
}

// ======================================================
// 📋 OBTENER PERFILES
// ======================================================

pub fn obtener_perfiles() -> Result<Vec<String>, String> {
    usuario::perfiles()
}

// ======================================================
// 🆔 OBTENER NOMBRE ACTUAL
// ======================================================

pub fn obtener_nombre_actual() -> Result<String, String> {
    usuario::nombre_actual()
}

// ======================================================
// 🟢 ESTADO CACHE
// ======================================================

pub fn obtener_estado_cache() -> bool {
    !cache::esta_vacia()
}

// ======================================================
// 🔄 RESTAURAR PERFIL
// ======================================================

pub fn restaurar_perfil_actual() -> Result<ResultadoPerfil, String> {
    let ruta = usuario::perfil_actual()?;

    if !ruta.exists() {
        let perfil = perfil_json::nuevo();

        guardar_en_disco(&perfil, &ruta)?;
    }

    let perfil = cargar_desde_disco(&ruta)?;

    let nombre = usuario::nombre_actual()?;

    resultado_perfil(perfil, nombre)
}

// ======================================================
// 📋 CLONAR PERFIL
// ======================================================

pub fn clonar_perfil(perfil: perfil_json) -> Result<ResultadoPerfil, String> {
    let nombre_actual = usuario::nombre_actual()?;

    let nombre = siguiente_nombre(&nombre_actual)?;

    cache::borrar_cache();

    let ruta = usuario::ruta_perfil(&nombre)?;

    guardar_en_disco(&perfil, &ruta)?;

    compilador::compilar(&perfil);

    resultado_perfil(perfil, nombre)
}

// ======================================================
// ✏️ RENOMBRAR PERFIL
// ======================================================

pub fn renombrar_perfil(nuevo_nombre: String) -> Result<ResultadoPerfil, String> {
    let nombre_actual = usuario::nombre_actual()?;

    let nuevo_nombre = nuevo_nombre.trim();

    if nuevo_nombre.is_empty() {
        return Err("El nombre del perfil está vacío".into());
    }

    if nuevo_nombre == nombre_actual {
        return Err("El perfil ya tiene ese nombre".into());
    }

    let nuevo_nombre = siguiente_nombre(nuevo_nombre)?;

    let ruta_actual = usuario::perfil_actual()?;

    let nueva_ruta = usuario::ruta_perfil(&nuevo_nombre)?;

    cache::borrar_cache();

    fs::rename(&ruta_actual, &nueva_ruta).map_err(|error| error.to_string())?;

    let perfil = cargar_desde_disco(&nueva_ruta)?;

    compilador::compilar(&perfil);

    resultado_perfil(perfil, nuevo_nombre)
}

// ======================================================
// 🗑️ ELIMINAR PERFIL
// ======================================================

pub fn eliminar_perfil_actual() -> Result<ResultadoPerfil, String> {
    let ruta_actual = usuario::perfil_actual()?;

    cache::borrar_cache();

    if ruta_actual.exists() {
        fs::remove_file(ruta_actual).map_err(|error| error.to_string())?;
    }

    // ¿Queda algún otro perfil? usuario::perfiles() ya los devuelve
    // ordenados alfabéticamente — el primero de la lista pasa a ser el
    // nuevo actual, sin más criterio que ese.
    let restantes = usuario::perfiles()?;

    let Some(siguiente_nombre) = restantes.into_iter().next() else {
        // No quedó ninguno: mismo camino que crear_perfil_nuevo().
        return crear_perfil_nuevo();
    };

    let ruta = usuario::ruta_perfil(&siguiente_nombre)?;

    let perfil = cargar_desde_disco(&ruta)?;

    compilador::compilar(&perfil);

    resultado_perfil(perfil, siguiente_nombre)
}

// ======================================================
// 🆕 CREAR PERFIL
// ======================================================

pub fn crear_perfil_nuevo() -> Result<ResultadoPerfil, String> {
    cache::borrar_cache();

    let nombre = siguiente_nombre("Default")?;

    let perfil = perfil_json::nuevo();

    let ruta = usuario::ruta_perfil(&nombre)?;

    guardar_en_disco(&perfil, &ruta)?;

    resultado_perfil(perfil, nombre)
}

// ======================================================
// 🔄 SELECCIONAR PERFIL
// ======================================================

pub fn seleccionar_perfil(nombre: String) -> Result<ResultadoPerfil, String> {
    let ruta = usuario::ruta_perfil(&nombre)?;

    if !ruta.exists() {
        return Err("El perfil seleccionado no existe".into());
    }

    cache::borrar_cache();

    let perfil = cargar_desde_disco(&ruta)?;

    compilador::compilar(&perfil);

    resultado_perfil(perfil, nombre)
}

// ======================================================
// 📦 RESULTADO
// ======================================================

fn resultado_perfil(perfil: perfil_json, nombre: String) -> Result<ResultadoPerfil, String> {
    Ok(ResultadoPerfil {
        perfil,

        nombre,

        perfiles: usuario::perfiles()?,

        cache_activo: !cache::esta_vacia(),
    })
}

// ======================================================
// 🔢 NOMBRE DISPONIBLE
// ======================================================

fn siguiente_nombre(base: &str) -> Result<String, String> {
    let ruta = usuario::ruta_perfil(base)?;

    if !ruta.exists() {
        return Ok(base.to_string());
    }

    let mut numero = 2;

    loop {
        let nombre = format!("{} ({})", base, numero);

        let ruta = usuario::ruta_perfil(&nombre)?;

        if !ruta.exists() {
            return Ok(nombre);
        }

        numero += 1;
    }
}

// ======================================================
// 💾 GUARDAR EN DISCO
// ======================================================
fn guardar_en_disco(perfil: &perfil_json, ruta: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(perfil).map_err(|error| error.to_string())?;

    fs::write(ruta, json).map_err(|error| error.to_string())?;

    Ok(())
}

// ======================================================
// 📂 CARGAR DESDE DISCO
// ======================================================
fn cargar_desde_disco(ruta: &Path) -> Result<perfil_json, String> {
    let json = fs::read_to_string(ruta).map_err(|error| error.to_string())?;

    serde_json::from_str(&json).map_err(|error| error.to_string())
}
