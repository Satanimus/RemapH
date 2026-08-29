// ======================================================
// 📍 Banco_Coordenadas
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Catálogo de coordenadas guardadas por el usuario
// (Usuario/Coordenadas.tsv), filtrable por aplicación/
// tipo/modo. Persistencia por reescritura total del
// archivo (mismo criterio que otros catálogos del
// proyecto) — no hay edición línea a línea en disco.
//
// No conoce UI ni el flujo de captura — solo modelo y
// CRUD sobre el archivo.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// comandos.rs (comandos Tauri expuestos a la UI).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// CoordenadaBanco completas (agregar/editar), o id
// (eliminar), o filtros opcionales (listar_filtrado).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Vec<CoordenadaBanco> (cargar/listar_filtrado), o
// Result<(), String> (agregar/editar/eliminar).
// ------------------------------------------------------
// 5. Funciones del archivo
//
// cargar()
//     Lee y parsea Coordenadas.tsv completo. Vec vacío si
//     el archivo no existe todavía (no lo crea).
// guardar()
//     Reescribe Coordenadas.tsv completo a partir de la
//     lista dada.
// generar_id()
//     uuid v4 para coordenadas nuevas.
// agregar() / editar() / eliminar()
//     CRUD sobre la lista completa (carga, modifica en
//     memoria, persiste). agregar() genera el id con
//     generar_id() si llega vacío y devuelve la coordenada
//     ya guardada (con el id asignado).
// listar_filtrado()
//     cargar() + filtro por aplicación (substring, sin
//     distinguir mayúsculas)/tipo/modo (coincidencia
//     exacta, None = sin filtrar esa columna).
// ======================================================

use std::fs;

use crate::usuario;

// ======================================================
// 📦 MODELO
// ------------------------------------------------------
// tipo: 1=Absoluta, 2=Cursor, 3=Ventana
// modo: 0=no aplica, 1=Píxeles, 2=Porcentaje
// punto_referencia: 0=no aplica, 1=Sup-Izq, 2=Sup-Der,
//                    3=Centro, 4=Inf-Izq, 5=Inf-Der
// ======================================================

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CoordenadaBanco {
    pub id: String,

    pub aplicacion: String,

    pub tipo: u8,

    pub modo: u8,

    pub punto_referencia: u8,

    pub x: f64,

    pub y: f64,

    pub nota: String,
}

// Columnas de Coordenadas.tsv, en orden.
const CANTIDAD_COLUMNAS: usize = 8;

// ======================================================
// 🧼 SANEAR CAMPO DE TEXTO
// ------------------------------------------------------
// aplicacion/nota son texto libre de usuario — TSV rompe
// si contienen tab o salto de línea. Se reemplazan por
// espacio antes de persistir (nunca se rechaza el guardado
// por esto).
// ======================================================

fn sanear_campo(valor: &str) -> String {
    valor.replace(['\t', '\n', '\r'], " ")
}

// ======================================================
// 📖 CARGAR
// ======================================================

pub fn cargar() -> Result<Vec<CoordenadaBanco>, String> {
    let ruta = usuario::ruta_coordenadas()?;

    if !ruta.exists() {
        return Ok(Vec::new());
    }

    let texto = fs::read_to_string(&ruta).map_err(|error| error.to_string())?;

    let mut coordenadas = Vec::new();

    for linea in texto.lines() {
        if linea.trim().is_empty() {
            continue;
        }

        if let Some(coordenada) = desde_linea(linea) {
            coordenadas.push(coordenada);
        }
    }

    Ok(coordenadas)
}

fn desde_linea(linea: &str) -> Option<CoordenadaBanco> {
    let columnas: Vec<&str> = linea.split('\t').collect();

    if columnas.len() != CANTIDAD_COLUMNAS {
        return None;
    }

    Some(CoordenadaBanco {
        id: columnas[0].to_string(),
        aplicacion: columnas[1].to_string(),
        tipo: columnas[2].parse().ok()?,
        modo: columnas[3].parse().ok()?,
        punto_referencia: columnas[4].parse().ok()?,
        x: columnas[5].parse().ok()?,
        y: columnas[6].parse().ok()?,
        nota: columnas[7].to_string(),
    })
}

// ======================================================
// 💾 GUARDAR
// ======================================================

pub fn guardar(lista: &[CoordenadaBanco]) -> Result<(), String> {
    let ruta = usuario::ruta_coordenadas()?;

    let texto = lista
        .iter()
        .map(a_linea)
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(&ruta, texto).map_err(|error| error.to_string())
}

fn a_linea(coordenada: &CoordenadaBanco) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        coordenada.id,
        sanear_campo(&coordenada.aplicacion),
        coordenada.tipo,
        coordenada.modo,
        coordenada.punto_referencia,
        coordenada.x,
        coordenada.y,
        sanear_campo(&coordenada.nota),
    )
}

// ======================================================
// 🆔 GENERAR ID
// ======================================================

pub fn generar_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ======================================================
// ➕ AGREGAR
// ======================================================

pub fn agregar(mut coordenada: CoordenadaBanco) -> Result<CoordenadaBanco, String> {
    if coordenada.id.is_empty() {
        coordenada.id = generar_id();
    }

    let mut lista = cargar()?;

    lista.push(coordenada.clone());

    guardar(&lista)?;

    Ok(coordenada)
}

// ======================================================
// ✏️ EDITAR
// ======================================================

pub fn editar(id: &str, coordenada: CoordenadaBanco) -> Result<(), String> {
    let mut lista = cargar()?;

    let Some(existente) = lista.iter_mut().find(|item| item.id == id) else {
        return Err(format!("No se encontró la coordenada con id {id}"));
    };

    *existente = coordenada;

    guardar(&lista)
}

// ======================================================
// 🖱️ ACTUALIZAR X,Y — Etapa F
// ------------------------------------------------------
// Pisa solo x/y (arrastre del marcador de previsualización),
// sin tocar el resto de los campos — a diferencia de editar(),
// que reemplaza la coordenada completa.
// ======================================================

pub fn actualizar_xy(id: &str, x: f64, y: f64) -> Result<(), String> {
    let mut lista = cargar()?;

    let Some(existente) = lista.iter_mut().find(|item| item.id == id) else {
        return Err(format!("No se encontró la coordenada con id {id}"));
    };

    existente.x = x;
    existente.y = y;

    guardar(&lista)
}

// ======================================================
// ⁝⁝ REORDENAR
// ------------------------------------------------------
// `orden` trae solo los ids VISIBLES (la fila arrastrada
// puede venir de una lista filtrada por Grupo/Tipo). Las
// coordenadas que no están en `orden`, por quedar fuera del
// filtro activo, no cambian de posición: se reasignan
// únicamente los índices que ya ocupaba el subconjunto
// afectado, en el nuevo orden recibido.
// ======================================================

pub fn reordenar(orden: &[String]) -> Result<(), String> {
    let mut lista = cargar()?;

    let indices: Vec<usize> = lista
        .iter()
        .enumerate()
        .filter(|(_, item)| orden.contains(&item.id))
        .map(|(indice, _)| indice)
        .collect();

    if indices.len() != orden.len() {
        return Err("El orden recibido no coincide con las coordenadas guardadas".into());
    }

    let mut por_id: std::collections::HashMap<String, CoordenadaBanco> = lista
        .iter()
        .filter(|item| orden.contains(&item.id))
        .map(|item| (item.id.clone(), item.clone()))
        .collect();

    for (indice, id) in indices.into_iter().zip(orden.iter()) {
        if let Some(coordenada) = por_id.remove(id) {
            lista[indice] = coordenada;
        }
    }

    guardar(&lista)
}

// ======================================================
// 🗑️ ELIMINAR
// ======================================================

pub fn eliminar(id: &str) -> Result<(), String> {
    let mut lista = cargar()?;

    let cantidad_previa = lista.len();

    lista.retain(|item| item.id != id);

    if lista.len() == cantidad_previa {
        return Err(format!("No se encontró la coordenada con id {id}"));
    }

    guardar(&lista)
}

// ======================================================
// 🔎 LISTAR FILTRADO
// ======================================================

pub fn listar_filtrado(
    aplicacion: Option<&str>,
    tipo: Option<u8>,
    modo: Option<u8>,
) -> Result<Vec<CoordenadaBanco>, String> {
    let lista = cargar()?;

    let aplicacion_filtro = aplicacion.map(|valor| valor.to_lowercase());

    Ok(lista
        .into_iter()
        .filter(|coordenada| {
            if let Some(filtro) = &aplicacion_filtro {
                if !coordenada.aplicacion.to_lowercase().contains(filtro) {
                    return false;
                }
            }

            if let Some(filtro) = tipo {
                if coordenada.tipo != filtro {
                    return false;
                }
            }

            if let Some(filtro) = modo {
                if coordenada.modo != filtro {
                    return false;
                }
            }

            true
        })
        .collect())
}

// ======================================================
// 🗂️ LISTAR GRUPOS DISTINTOS
// ------------------------------------------------------
// Valores únicos de `aplicacion` ya guardados (sin vacíos),
// orden alfabético — para poblar el desplegable de filtro
// "Grupo" ("Todos" + esta lista) en la ventana de
// Coordenadas guardadas.
// ======================================================

pub fn listar_grupos_distintos() -> Result<Vec<String>, String> {
    let lista = cargar()?;

    let mut grupos: Vec<String> = lista
        .into_iter()
        .map(|coordenada| coordenada.aplicacion)
        .filter(|aplicacion| !aplicacion.is_empty())
        .collect();

    grupos.sort();
    grupos.dedup();

    Ok(grupos)
}
