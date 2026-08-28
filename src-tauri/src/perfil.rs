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
//
// Excepción puntual (Etapa 8C): SÍ llama a runtime::detener_todo()
// junto a cada punto donde ya vacía la cache (activar/desactivar/
// guardar/clonar/renombrar/eliminar/crear/seleccionar perfil) — red
// de seguridad para que cambiar de perfil corte cualquier ejecución
// activa y suelte cualquier tecla que haya quedado físicamente abajo
// por culpa del motor. No conoce nada más de Runtime más allá de esa
// única función.
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
//     Carga perfil actual y activa cache. Devuelve
//     ResultadoCompilacion (antes solo un bool) con las advertencias
//     de esta compilación.
//
// desactivar_perfil()
//     Desactiva perfil actual.
//
// guardar_perfil()
//     Guarda perfil actual y recompila cache.
//     Recibe perfil ya convertido desde perfil_ui.
//
// obtener_perfil_actual()
//     Obtiene perfil actual (compila automáticamente). Devuelve
//     ResultadoPerfilInicial (perfil + advertencias de esa
//     compilación).
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
// guardar_perfil_como()
//     Guarda la versión de perfil que muestra la UI (con o sin
//     cambios sin guardar) como un perfil nuevo, con el nombre
//     pedido al usuario, sin tocar el archivo del perfil actual.
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
//     Construye respuesta completa para UI. advertencias es
//     Option: None si la operación no recompiló (restaurar_perfil_actual).
//
// siguiente_nombre()
//     Genera nombre disponible.
//
// guardar_en_disco()
//     Guarda perfil json en disco
//
// cargar_desde_disco()
//     Carga perfil json en disco
//
// buscar_nombres_portapapeles()
//     Busca en todos los perfiles guardados el nombre de fila
//     portapapeles_accion.nombre para cada id pedida (Cambio 2).
// ======================================================

use crate::cache;
use crate::compilador;
use crate::perfil_json::{perfil_json, ItemFilaJson};
use crate::runtime;
use crate::usuario;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::compilador::{AdvertenciaCompilacion, ResultadoCompilacion};
use crate::perfil_ui::{ResultadoPerfil, ResultadoPerfilInicial};

// ======================================================
// 🟢 ACTIVAR PERFIL
// ------------------------------------------------------
// Devuelve ResultadoCompilacion completo (no solo el bool de antes)
// para que el botón "Activar perfil" de la UI pueda refrescar el
// "OFF ⚠️" de cada fila y el statusbar con las advertencias de ESTA
// compilación — antes se perdían en silencio (ver
// ui_toolbar.ts::botonEstado).
// ======================================================

pub fn activar_perfil() -> Result<ResultadoCompilacion, String> {
    let ruta = usuario::perfil_actual()?;

    let perfil = cargar_desde_disco(&ruta)?;

    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    Ok(compilador::compilar(&perfil))
}

// ======================================================
// 🔴 DESACTIVAR PERFIL
// ======================================================

pub fn desactivar_perfil() {
    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    cache::borrar_cache();
}

// ======================================================
// 📂 OBTENER PERFIL ACTUAL
// ------------------------------------------------------
// Devuelve también las advertencias de esta compilación automática
// (antes se perdían — main.ts::iniciarApp() nunca se enteraba de
// una fila "abrir" con ruta que ya no existe al abrir el programa).
// ======================================================

pub fn obtener_perfil_actual() -> Result<ResultadoPerfilInicial, String> {
    let ruta = usuario::perfil_actual()?;

    if !ruta.exists() {
        let perfil = perfil_json::nuevo();

        guardar_en_disco(&perfil, &ruta)?;

        let resultado = compilador::compilar(&perfil);

        return Ok(ResultadoPerfilInicial {
            perfil,
            advertencias: resultado.advertencias,
        });
    }

    let perfil = cargar_desde_disco(&ruta)?;

    let resultado = compilador::compilar(&perfil);

    Ok(ResultadoPerfilInicial {
        perfil,
        advertencias: resultado.advertencias,
    })
}

// ======================================================
// 💾 GUARDAR PERFIL
// ======================================================

pub fn guardar_perfil(perfil: perfil_json) -> Result<ResultadoCompilacion, String> {
    let ruta = usuario::perfil_actual()?;

    guardar_en_disco(&perfil, &ruta)?;

    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    Ok(compilador::compilar(&perfil))
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
// ------------------------------------------------------
// A propósito NO recompila (no llama compilador::compilar): revertir
// ediciones sin guardar no debe reiniciar la cache ni lo que ya esté
// corriendo (cerrar MenuExpress/Portapapeles abiertos, etc. — ver
// compilador::compilar). advertencias viaja en None para que la UI
// sepa que debe dejar las advertencias vigentes tal como están, no
// pisarlas con una lista vacía.
// ======================================================

pub fn restaurar_perfil_actual() -> Result<ResultadoPerfil, String> {
    let ruta = usuario::perfil_actual()?;

    if !ruta.exists() {
        let perfil = perfil_json::nuevo();

        guardar_en_disco(&perfil, &ruta)?;
    }

    let perfil = cargar_desde_disco(&ruta)?;

    let nombre = usuario::nombre_actual()?;

    resultado_perfil(perfil, nombre, None)
}

// ======================================================
// 💾📋 GUARDAR PERFIL COMO
// ------------------------------------------------------
// A diferencia de guardar_perfil(), no escribe sobre el archivo del
// perfil actual: guarda el perfil recibido (la versión que muestra
// la UI, editada o no) bajo el nombre pedido al usuario. El archivo
// del perfil actual queda intacto. siguiente_nombre() resuelve
// colisiones igual que en renombrar_perfil().
// ======================================================

pub fn guardar_perfil_como(nombre: String, perfil: perfil_json) -> Result<ResultadoPerfil, String> {
    let nombre = nombre.trim();

    if nombre.is_empty() {
        return Err("El nombre del perfil está vacío".into());
    }

    let nombre = siguiente_nombre(nombre)?;

    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    cache::borrar_cache();

    let ruta = usuario::ruta_perfil(&nombre)?;

    guardar_en_disco(&perfil, &ruta)?;

    let resultado = compilador::compilar(&perfil);

    resultado_perfil(perfil, nombre, Some(resultado.advertencias))
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

    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    cache::borrar_cache();

    fs::rename(&ruta_actual, &nueva_ruta).map_err(|error| error.to_string())?;

    let perfil = cargar_desde_disco(&nueva_ruta)?;

    let resultado = compilador::compilar(&perfil);

    resultado_perfil(perfil, nuevo_nombre, Some(resultado.advertencias))
}

// ======================================================
// 🗑️ ELIMINAR PERFIL
// ======================================================

pub fn eliminar_perfil_actual() -> Result<ResultadoPerfil, String> {
    let ruta_actual = usuario::perfil_actual()?;

    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

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

    let resultado = compilador::compilar(&perfil);

    resultado_perfil(perfil, siguiente_nombre, Some(resultado.advertencias))
}

// ======================================================
// 🆕 CREAR PERFIL
// ------------------------------------------------------
// No llama compilador::compilar (perfil nuevo siempre vacío, nada
// que compilar) pero SÍ vació la cache arriba — Some(vec![]) refleja
// eso con precisión: se compiló (en los hechos, a nada) y no hay
// advertencias, a diferencia de restaurar_perfil_actual que ni
// siquiera toca la cache.
// ======================================================

pub fn crear_perfil_nuevo() -> Result<ResultadoPerfil, String> {
    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    cache::borrar_cache();

    let nombre = siguiente_nombre("Default")?;

    let perfil = perfil_json::nuevo();

    let ruta = usuario::ruta_perfil(&nombre)?;

    guardar_en_disco(&perfil, &ruta)?;

    resultado_perfil(perfil, nombre, Some(Vec::new()))
}

// ======================================================
// 🔄 SELECCIONAR PERFIL
// ======================================================

pub fn seleccionar_perfil(nombre: String) -> Result<ResultadoPerfil, String> {
    let ruta = usuario::ruta_perfil(&nombre)?;

    if !ruta.exists() {
        return Err("El perfil seleccionado no existe".into());
    }

    // Etapa 8C: ver excepción documentada en el header del archivo.
    runtime::detener_todo();

    cache::borrar_cache();

    let perfil = cargar_desde_disco(&ruta)?;

    // usuario::perfil_actual() decide cuál es "el perfil actual" mirando
    // qué archivo .json fue modificado más recientemente en disco — no
    // hay ningún otro estado que lo registre. Si acá solo leyéramos el
    // archivo sin re-escribirlo, su fecha de modificación no cambiaría
    // y el perfil recién seleccionado NO pasaría a ser "el actual": el
    // sistema seguiría apuntando al perfil anterior (el último que sí
    // se guardó). Eso es lo que causaba que, tras cambiar de perfil en
    // el listado, "Renombrar" (y guardar_perfil, eliminar, etc., que
    // también dependen de usuario::perfil_actual()) siguiera operando
    // sobre el perfil viejo. Re-guardamos el mismo contenido para
    // "tocar" el archivo y que su mtime quede al día.
    guardar_en_disco(&perfil, &ruta)?;

    let resultado = compilador::compilar(&perfil);

    resultado_perfil(perfil, nombre, Some(resultado.advertencias))
}

// ======================================================
// 📦 RESULTADO
// ======================================================

fn resultado_perfil(
    perfil: perfil_json,
    nombre: String,
    advertencias: Option<Vec<AdvertenciaCompilacion>>,
) -> Result<ResultadoPerfil, String> {
    Ok(ResultadoPerfil {
        perfil,

        nombre,

        perfiles: usuario::perfiles()?,

        cache_activo: !cache::esta_vacia(),

        advertencias,
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

// ======================================================
// 🆔📋 BUSCAR NOMBRES DE PORTAPAPELES — Cambio 2 (Etapa 2B)
// ------------------------------------------------------
// Recorre TODOS los perfiles guardados en la carpeta Usuario (no
// solo el perfil actual — una ID de Portapapeles puede venir de
// cualquier perfil guardado, no necesariamente el activo) buscando
// filas tipo == "portapapeles" cuya id esté en `ids`. Devuelve un
// mapa id -> nombre (RemapeoJson::portapapeles_accion.nombre).
//
// Las IDs que no aparecen en ningún perfil simplemente no están en
// el mapa devuelto — el llamador (back_portapapeles::
// listar_otras_ids_con_fijados + comandos::portapapeles_listar_otros)
// decide qué mostrar en ese caso (spec Cambio 2: la ID cruda).
//
// Un solo recorrido de todos los .json de Usuario (no uno por id) —
// barato aun con varios perfiles guardados. Si una misma id
// apareciera en más de un perfil (no debería pasar en uso normal),
// se queda con la primera que encuentra.
// ======================================================

pub fn buscar_nombres_portapapeles(ids: &[String]) -> HashMap<String, String> {
    let mut resultado = HashMap::new();

    let Ok(nombres_perfil) = usuario::perfiles() else {
        return resultado;
    };

    for nombre_perfil in nombres_perfil {
        let Ok(ruta) = usuario::ruta_perfil(&nombre_perfil) else {
            continue;
        };

        let Ok(perfil) = cargar_desde_disco(&ruta) else {
            continue;
        };

        for item in perfil.filas {
            let ItemFilaJson::Fila(remapeo) = item else {
                continue;
            };

            if remapeo.tipo != "portapapeles" {
                continue;
            }

            if ids.contains(&remapeo.id) {
                resultado
                    .entry(remapeo.id)
                    .or_insert(remapeo.portapapeles_accion.nombre);
            }
        }
    }

    resultado
}
