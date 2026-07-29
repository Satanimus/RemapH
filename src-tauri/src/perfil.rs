// ======================================================
// 👤 Perfil RemapH V3
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
// persistencia
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
//     Elimina perfil actual.
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
// sincronizar_estado_cache()
//     Actualiza estado global según perfil.
//
// ======================================================

use std::fs;

use crate::cache;
use crate::compilador;
use crate::estado;
use crate::perfil_json::perfil_json;
use crate::persistencia;
use crate::usuario;

use crate::perfil_ui::ResultadoPerfil;

// ======================================================
// 🟢 ACTIVAR PERFIL
// ======================================================

pub fn activar_perfil() -> Result<bool, String> {
    let ruta = usuario::perfil_actual()?;

    let perfil = persistencia::cargar(&ruta)?;

    compilador::compilar(&perfil);

    sincronizar_estado_cache(&perfil);

    Ok(!cache::esta_vacia())
}

// ======================================================
// 🔴 DESACTIVAR PERFIL
// ======================================================

pub fn desactivar_perfil() {
    cache::borrar();

    estado::desactivar();
}

// ======================================================
// 📂 OBTENER PERFIL ACTUAL
// ======================================================

pub fn obtener_perfil_actual() -> Result<perfil_json, String> {
    let ruta = usuario::perfil_actual()?;

    if !ruta.exists() {
        let perfil = perfil_json::nuevo();

        persistencia::guardar(&perfil, &ruta)?;

        compilador::compilar(&perfil);

        sincronizar_estado_cache(&perfil);

        return Ok(perfil);
    }

    let perfil = persistencia::cargar(&ruta)?;

    compilador::compilar(&perfil);

    sincronizar_estado_cache(&perfil);

    Ok(perfil)
}

// ======================================================
// 💾 GUARDAR PERFIL
// ======================================================

pub fn guardar_perfil(perfil: perfil_json) -> Result<bool, String> {
    let ruta = usuario::perfil_actual()?;

    persistencia::guardar(&perfil, &ruta)?;

    compilador::compilar(&perfil);

    sincronizar_estado_cache(&perfil);

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

        persistencia::guardar(&perfil, &ruta)?;
    }

    let perfil = persistencia::cargar(&ruta)?;

    let nombre = usuario::nombre_actual()?;

    resultado_perfil(perfil, nombre)
}

// ======================================================
// 📋 CLONAR PERFIL
// ======================================================

pub fn clonar_perfil(perfil: perfil_json) -> Result<ResultadoPerfil, String> {
    let nombre_actual = usuario::nombre_actual()?;

    let nombre = siguiente_nombre(&nombre_actual)?;

    cache::borrar();

    estado::desactivar();

    let ruta = usuario::ruta_perfil(&nombre)?;

    persistencia::guardar(&perfil, &ruta)?;

    compilador::compilar(&perfil);

    sincronizar_estado_cache(&perfil);

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

    cache::borrar();

    estado::desactivar();

    fs::rename(&ruta_actual, &nueva_ruta).map_err(|error| error.to_string())?;

    let perfil = persistencia::cargar(&nueva_ruta)?;

    compilador::compilar(&perfil);

    sincronizar_estado_cache(&perfil);

    resultado_perfil(perfil, nuevo_nombre)
}

// ======================================================
// 🗑️ ELIMINAR PERFIL
// ======================================================

pub fn eliminar_perfil_actual() -> Result<ResultadoPerfil, String> {
    let ruta_actual = usuario::perfil_actual()?;

    cache::borrar();

    estado::desactivar();

    if ruta_actual.exists() {
        fs::remove_file(ruta_actual).map_err(|error| error.to_string())?;
    }

    Err("Pendiente definir creación automática después de eliminar último perfil".into())
}

// ======================================================
// 🆕 CREAR PERFIL
// ======================================================

pub fn crear_perfil_nuevo() -> Result<ResultadoPerfil, String> {
    cache::borrar();

    estado::desactivar();

    let nombre = siguiente_nombre("Default")?;

    let perfil = perfil_json::nuevo();

    let ruta = usuario::ruta_perfil(&nombre)?;

    persistencia::guardar(&perfil, &ruta)?;

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

    cache::borrar();

    estado::desactivar();

    let perfil = persistencia::cargar(&ruta)?;

    compilador::compilar(&perfil);

    sincronizar_estado_cache(&perfil);

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
// 🔄 SINCRONIZAR ESTADO
// ======================================================

fn sincronizar_estado_cache(perfil: &perfil_json) {
    if perfil.remapeos.iter().any(|remapeo| remapeo.estado == "ON") {
        estado::activar();
    } else {
        estado::desactivar();
    }
}
