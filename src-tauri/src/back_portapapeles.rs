// ======================================================
// 📋 back_portapapeles
// ======================================================
// ETAPA E DEL PLAN "PORTAPAPELES"
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Dueño de la carpeta física del pool compartido de Portapapeles:
//
// %APPDATA%/RemapH/Portapapeles/
//   ├── abc123-...-def_MiLink.txt        (fijado)
//   ├── Imagen_13.45.55.png              (rotativo)
//   └── la ciudad es.txt                 (rotativo)
//
// Funciones PURAS de manejo de archivos: nombrar, guardar un
// elemento nuevo como rotativo, listar (rotativos / fijados de un
// id), aplicar el límite del pool, fijar/desfijar, renombrar, editar
// contenido de texto, eliminar. Ninguna función de este archivo
// sabe qué filas están en modo Registro, ni conoce ACTIVOS, ni
// escucha el portapapeles del sistema — solo entiende la carpeta y
// sus archivos. Testeable con datos de prueba (ver tests al final),
// sin ventana ni listener de por medio.
//
// Pool de rotativos: GLOBAL y compartido (una sola carpeta, un solo
// listado) — cada fila tipo "portapapeles" es solo un visualizador
// de ese pool. Solo los FIJADOS son exclusivos de cada fila, vía el
// prefijo {id_portapapeles}_ en el nombre del archivo.
//
// Distinción fijado / rotativo: se lee directo del nombre físico del
// archivo. Un fijado tiene el id de su fila (RemapeoCache::id, un
// UUID de 36 caracteres — ver core_perfil.ts::crypto.randomUUID())
// como prefijo exacto seguido de "_". Un rotativo es cualquier
// archivo cuyo nombre NO empiece con ese patrón exacto — el texto
// copiado por el usuario puede traer guiones bajos sueltos sin que
// eso lo confunda con un fijado, porque se exige que el prefijo
// tenga el largo y la forma exacta de un UUID (ver
// es_id_portapapeles()).
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// Todavía nadie — Etapa F lo conecta con el listener de
// back_portapapeles_captura.rs y el estado ACTIVOS (modo Registro);
// Etapa G lo conecta con la ventana; Etapa H lo expone como
// comandos Tauri.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// Contenido de portapapeles (ContenidoPortapapeles, de
// back_portapapeles_captura.rs), rutas de archivos ya existentes en
// el pool, e ids de Portapapeles (String, el mismo
// RemapeoCache::id de la fila).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// ElementoPortapapeles (ruta, nombre, extensión, fijado/no,
// id dueño si aplica, fecha de modificación) — listas de estos para
// listar_rotativos()/listar_fijados(), o una ruta sola para las
// operaciones de escritura (guardar_rotativo, fijar, desfijar,
// renombrar).
// ------------------------------------------------------
// 5. Reglas / decisiones
//
// • Nombrar texto: primeros 20 caracteres del contenido copiado
//   (spec: "Si el texto copiado excede los 20 caracteres solo se
//   muestran los primeros 20"), saneados para ser un nombre de
//   archivo válido en Windows (se reemplazan \ / : * ? " < > | y
//   caracteres de control por espacio, se recorta espacio/punto al
//   final; si queda vacío, "Sin título" — no está en el spec letra
//   por letra, pero es necesario: el contenido copiado es
//   arbitrario y estos caracteres son inválidos en un nombre de
//   archivo de Windows).
// • Nombrar imagen: "Imagen_HH.MM.SS" con la hora LOCAL del sistema
//   (spec: "Imagen_13.45.55.png") — usa GetLocalTime (windows-sys),
//   mismo criterio de la app de apoyarse en la API de Windows en vez
//   de reinventar manejo de zona horaria.
// • Conflicto de nombres: se prueba el nombre pedido tal cual: si ya
//   existe un archivo con ese nombre completo (incluida la
//   extensión, e incluido el prefijo de id si es fijado), se agrega
//   " (1)", " (2)", etc. hasta encontrar uno libre — mismo criterio
//   para rotativos, fijados y renombrados (spec lo pide en los tres
//   casos por separado con el mismo comportamiento).
// • Fijar / Desfijar / Renombrar no tocan el contenido del archivo,
//   solo cambian el nombre físico (rename) — como Windows NO
//   actualiza la fecha de modificación en un rename, estas tres
//   operaciones fuerzan la fecha a "ahora" a mano (tocar_ahora) para
//   que el elemento suba arriba de la lista, tal como pide el spec
//   ("se ordenan por fecha"). Editar SÍ cambia el contenido
//   (fs::write ya actualiza la fecha solo) y NO cambia el nombre
//   físico — coincide con la nota del spec ("a menos que solo se
//   haya editado").
// • Fijar es también el mecanismo para "reemplazar el ID" de un fijado
//   (spec: "Si hubiera un caso donde un elemento fijado se vuelve a
//   guardar en otro Portapapel, se vuelve a guardar y se reemplaza
//   el ID"): fijar() parte siempre del nombre "limpio" (sin importar
//   si el archivo ya estaba fijado con OTRO id) y arma el nuevo
//   nombre con el id pedido — no hace falta una función aparte.
// • aplicar_limite() opera sobre el pool global de rotativos (no por
//   fila) — el límite EFECTIVO entre varios Portapapeles activos a
//   la vez lo calcula Etapa F; acá solo se aplica el número que
//   llega.
// • editar_texto() rechaza archivos que no sean .txt.
// ------------------------------------------------------
// 6. Funciones del archivo
//
// carpeta()
//     Resuelve (y crea si no existe) %APPDATA%/RemapH/Portapapeles/.
// listar_rotativos()
//     Todos los rotativos del pool, más reciente primero.
// listar_fijados()
//     Los fijados de un id de Portapapeles, más reciente primero.
// guardar_rotativo()
//     Guarda un ContenidoPortapapeles nuevo como rotativo.
// aplicar_limite()
//     Borra los rotativos más antiguos que sobran sobre un límite.
// fijar() / desfijar()
//     Renombran un elemento agregando/quitando el prefijo de id.
// renombrar()
//     Cambia el nombre visible de un elemento (máx 20 caracteres).
// editar_texto()
//     Sobrescribe el contenido de un elemento de texto.
// eliminar()
//     Borra un elemento del pool.
// ======================================================

use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;

use crate::back_portapapeles_captura::ContenidoPortapapeles;

// ======================================================
// 📏 CONSTANTES
// ======================================================

const LONGITUD_NOMBRE: usize = 20;
const CARACTERES_INVALIDOS: [char; 9] = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

// ======================================================
// 📦 ELEMENTO DEL POOL
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ElementoPortapapeles {
    pub ruta: PathBuf,
    // Nombre "limpio": sin extensión y sin el prefijo de id si es
    // fijado (lo que se muestra en el botón).
    pub nombre: String,
    pub extension: String,
    pub fijado: bool,
    pub id_portapapeles: Option<String>,
    pub modificado: SystemTime,
}

// ======================================================
// 📁 CARPETA DEL POOL
// ======================================================

fn carpeta() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|error| error.to_string())?;

    let carpeta = PathBuf::from(appdata).join("RemapH").join("Portapapeles");

    fs::create_dir_all(&carpeta).map_err(|error| error.to_string())?;

    Ok(carpeta)
}

// ======================================================
// 🆔 ¿ES UN PREFIJO DE ID VÁLIDO?
// ------------------------------------------------------
// Un id de Portapapeles es un UUID de 36 caracteres (8-4-4-4-12,
// hexadecimal + guiones) — ver core_perfil.ts::crypto.randomUUID().
// ======================================================

fn es_id_portapapeles(segmento: &str) -> bool {
    if segmento.len() != 36 {
        return false;
    }

    for (indice, caracter) in segmento.char_indices() {
        let debe_ser_guion = matches!(indice, 8 | 13 | 18 | 23);

        if debe_ser_guion {
            if caracter != '-' {
                return false;
            }
        } else if !caracter.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

// ======================================================
// 🔎 ELEMENTO DESDE RUTA
// ------------------------------------------------------
// Parsea un archivo del pool a partir de su ruta física. None si la
// ruta no tiene extensión, no tiene nombre, o no se pudo leer su
// fecha de modificación.
// ======================================================

fn elemento_desde_ruta(ruta: PathBuf) -> Option<ElementoPortapapeles> {
    let extension = ruta.extension()?.to_str()?.to_string();
    let stem = ruta.file_stem()?.to_str()?;

    let metadata = fs::metadata(&ruta).ok()?;
    let modificado = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    let es_fijado =
        stem.len() > 37 && stem.as_bytes()[36] == b'_' && es_id_portapapeles(&stem[..36]);

    let (fijado, id_portapapeles, nombre) = if es_fijado {
        (true, Some(stem[..36].to_string()), stem[37..].to_string())
    } else {
        (false, None, stem.to_string())
    };

    Some(ElementoPortapapeles {
        ruta,
        nombre,
        extension,
        fijado,
        id_portapapeles,
        modificado,
    })
}

// ======================================================
// 📋 LISTAR TODOS
// ======================================================

fn listar_todos() -> Result<Vec<ElementoPortapapeles>, String> {
    let carpeta = carpeta()?;

    let mut elementos = Vec::new();

    for entrada in fs::read_dir(&carpeta).map_err(|error| error.to_string())? {
        let ruta = entrada.map_err(|error| error.to_string())?.path();

        if !ruta.is_file() {
            continue;
        }

        if let Some(elemento) = elemento_desde_ruta(ruta) {
            elementos.push(elemento);
        }
    }

    Ok(elementos)
}

// ======================================================
// 🔄 LISTAR ROTATIVOS
// ======================================================

pub fn listar_rotativos() -> Result<Vec<ElementoPortapapeles>, String> {
    let mut elementos: Vec<ElementoPortapapeles> =
        listar_todos()?.into_iter().filter(|e| !e.fijado).collect();

    elementos.sort_by(|a, b| b.modificado.cmp(&a.modificado));

    Ok(elementos)
}

// ======================================================
// 📌 LISTAR FIJADOS (de un Portapapeles)
// ======================================================

pub fn listar_fijados(id_portapapeles: &str) -> Result<Vec<ElementoPortapapeles>, String> {
    let mut elementos: Vec<ElementoPortapapeles> = listar_todos()?
        .into_iter()
        .filter(|e| e.fijado && e.id_portapapeles.as_deref() == Some(id_portapapeles))
        .collect();

    elementos.sort_by(|a, b| b.modificado.cmp(&a.modificado));

    Ok(elementos)
}

// ======================================================
// 🧼 SANEAR NOMBRE
// ------------------------------------------------------
// Recorta a 20 caracteres y reemplaza caracteres inválidos en un
// nombre de archivo de Windows por espacio. "Sin título" si el
// resultado queda vacío.
// ======================================================

fn sanear_nombre(texto: &str) -> String {
    let limpio: String = texto
        .chars()
        .take(LONGITUD_NOMBRE)
        .map(|caracter| {
            if CARACTERES_INVALIDOS.contains(&caracter) || caracter.is_control() {
                ' '
            } else {
                caracter
            }
        })
        .collect();

    let limpio = limpio.trim().trim_end_matches('.').trim();

    if limpio.is_empty() {
        "Sin título".to_string()
    } else {
        limpio.to_string()
    }
}

// ======================================================
// 🕒 NOMBRE DE IMAGEN (hora local)
// ======================================================

fn nombre_hora_imagen() -> String {
    let mut hora: SYSTEMTIME = unsafe { std::mem::zeroed() };

    unsafe {
        GetLocalTime(&mut hora);
    }

    format!(
        "Imagen_{:02}.{:02}.{:02}",
        hora.wHour, hora.wMinute, hora.wSecond
    )
}

// ======================================================
// 🔀 NOMBRE SIN CONFLICTO
// ------------------------------------------------------
// Prueba "base.ext"; si ya existe, "base (1).ext", "base (2).ext",
// etc. Devuelve el nombre (sin extensión) que quedó libre.
// ======================================================

fn nombre_sin_conflicto(carpeta: &Path, base: &str, extension: &str) -> String {
    let mut candidato = base.to_string();
    let mut contador = 1;

    while carpeta
        .join(format!("{}.{}", candidato, extension))
        .exists()
    {
        candidato = format!("{} ({})", base, contador);
        contador += 1;
    }

    candidato
}

// ======================================================
// 🖼️ GUARDAR PNG
// ======================================================

fn guardar_png(ruta: &Path, ancho: usize, alto: usize, pixeles: &[u8]) -> Result<(), String> {
    let archivo = fs::File::create(ruta).map_err(|error| error.to_string())?;
    let escritor = BufWriter::new(archivo);

    let mut codificador = png::Encoder::new(escritor, ancho as u32, alto as u32);
    codificador.set_color(png::ColorType::Rgba);
    codificador.set_depth(png::BitDepth::Eight);

    let mut escritor_imagen = codificador
        .write_header()
        .map_err(|error| error.to_string())?;

    escritor_imagen
        .write_image_data(pixeles)
        .map_err(|error| error.to_string())?;

    Ok(())
}

// ======================================================
// 💾 GUARDAR ROTATIVO
// ------------------------------------------------------
// Guarda un ContenidoPortapapeles nuevo como archivo rotativo.
// Devuelve la ruta final (ya con el conflicto de nombre resuelto).
// ======================================================

pub fn guardar_rotativo(contenido: &ContenidoPortapapeles) -> Result<PathBuf, String> {
    let carpeta = carpeta()?;

    match contenido {
        ContenidoPortapapeles::Texto(texto) => {
            let base = sanear_nombre(texto);
            let nombre_final = nombre_sin_conflicto(&carpeta, &base, "txt");
            let ruta = carpeta.join(format!("{}.txt", nombre_final));

            fs::write(&ruta, texto.as_bytes()).map_err(|error| error.to_string())?;

            Ok(ruta)
        }

        ContenidoPortapapeles::Imagen {
            ancho,
            alto,
            pixeles,
        } => {
            let base = nombre_hora_imagen();
            let nombre_final = nombre_sin_conflicto(&carpeta, &base, "png");
            let ruta = carpeta.join(format!("{}.png", nombre_final));

            guardar_png(&ruta, *ancho, *alto, pixeles)?;

            Ok(ruta)
        }
    }
}

// ======================================================
// ✂️ APLICAR LÍMITE
// ------------------------------------------------------
// Borra los rotativos más antiguos que sobren sobre el límite dado.
// Los fijados no cuentan ni se tocan.
// ======================================================

pub fn aplicar_limite(limite: u32) -> Result<(), String> {
    let rotativos = listar_rotativos()?;
    let limite = limite as usize;

    if rotativos.len() > limite {
        for elemento in &rotativos[limite..] {
            fs::remove_file(&elemento.ruta).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

// ======================================================
// 🕒 TOCAR AHORA
// ------------------------------------------------------
// Fuerza la fecha de modificación de un archivo a "ahora". Se usa
// después de un rename (fijar/desfijar/renombrar), porque Windows no
// actualiza esa fecha solo con cambiar el nombre.
// ======================================================

fn tocar_ahora(ruta: &Path) -> Result<(), String> {
    let archivo = fs::OpenOptions::new()
        .write(true)
        .open(ruta)
        .map_err(|error| error.to_string())?;

    archivo
        .set_modified(SystemTime::now())
        .map_err(|error| error.to_string())
}

// ======================================================
// 📌 FIJAR
// ------------------------------------------------------
// Le agrega (o reemplaza) el prefijo de id al elemento en `ruta`.
// Sirve tanto para fijar un rotativo como para re-fijar un fijado
// bajo otro id (spec: "se reemplaza el ID").
// ======================================================

pub fn fijar(ruta: &Path, id_portapapeles: &str) -> Result<PathBuf, String> {
    let elemento = elemento_desde_ruta(ruta.to_path_buf())
        .ok_or_else(|| "No se pudo leer el elemento".to_string())?;

    let carpeta = carpeta()?;

    let base = format!("{}_{}", id_portapapeles, elemento.nombre);
    let nombre_final = nombre_sin_conflicto(&carpeta, &base, &elemento.extension);
    let nueva_ruta = carpeta.join(format!("{}.{}", nombre_final, elemento.extension));

    fs::rename(&elemento.ruta, &nueva_ruta).map_err(|error| error.to_string())?;
    tocar_ahora(&nueva_ruta)?;

    Ok(nueva_ruta)
}

// ======================================================
// 📌 DESFIJAR
// ------------------------------------------------------
// Le quita el prefijo de id al elemento en `ruta`, pasa a rotativo.
// ======================================================

pub fn desfijar(ruta: &Path) -> Result<PathBuf, String> {
    let elemento = elemento_desde_ruta(ruta.to_path_buf())
        .ok_or_else(|| "No se pudo leer el elemento".to_string())?;

    let carpeta = carpeta()?;

    let nombre_final = nombre_sin_conflicto(&carpeta, &elemento.nombre, &elemento.extension);
    let nueva_ruta = carpeta.join(format!("{}.{}", nombre_final, elemento.extension));

    fs::rename(&elemento.ruta, &nueva_ruta).map_err(|error| error.to_string())?;
    tocar_ahora(&nueva_ruta)?;

    Ok(nueva_ruta)
}

// ======================================================
// ✏️ RENOMBRAR
// ------------------------------------------------------
// Cambia el nombre visible de un elemento (máx 20 caracteres),
// conservando su prefijo de id si estaba fijado.
// ======================================================

pub fn renombrar(ruta: &Path, nuevo_nombre: &str) -> Result<PathBuf, String> {
    let elemento = elemento_desde_ruta(ruta.to_path_buf())
        .ok_or_else(|| "No se pudo leer el elemento".to_string())?;

    let carpeta = carpeta()?;
    let nombre_saneado = sanear_nombre(nuevo_nombre);

    let base = match &elemento.id_portapapeles {
        Some(id) => format!("{}_{}", id, nombre_saneado),
        None => nombre_saneado,
    };

    let nombre_final = nombre_sin_conflicto(&carpeta, &base, &elemento.extension);
    let nueva_ruta = carpeta.join(format!("{}.{}", nombre_final, elemento.extension));

    fs::rename(&elemento.ruta, &nueva_ruta).map_err(|error| error.to_string())?;
    tocar_ahora(&nueva_ruta)?;

    Ok(nueva_ruta)
}

// ======================================================
// 📝 EDITAR TEXTO
// ------------------------------------------------------
// Sobrescribe el contenido de un elemento de texto. No cambia su
// nombre físico — fs::write ya actualiza la fecha de modificación
// sola, no hace falta tocar_ahora().
// ======================================================

pub fn editar_texto(ruta: &Path, contenido: &str) -> Result<(), String> {
    if ruta.extension().and_then(|e| e.to_str()) != Some("txt") {
        return Err("Solo se puede editar contenido de texto".to_string());
    }

    fs::write(ruta, contenido.as_bytes()).map_err(|error| error.to_string())
}

// ======================================================
// 🗑️ ELIMINAR
// ======================================================

pub fn eliminar(ruta: &Path) -> Result<(), String> {
    fs::remove_file(ruta).map_err(|error| error.to_string())
}

// ======================================================
// 🧪 TESTS
// ------------------------------------------------------
// Usan std::env::temp_dir() para los archivos sueltos (no dependen
// de carpeta()/APPDATA), salvo el último grupo, que sí apunta
// APPDATA a una carpeta temporal — por eso conviene correr los
// tests de este archivo con `cargo test -- --test-threads=1`
// (mutar una variable de entorno de proceso no es seguro en
// paralelo).
// ======================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanea_y_recorta_a_20_caracteres() {
        let resultado =
            sanear_nombre("Esto es un texto bastante largo, mucho más de 20 caracteres");
        assert_eq!(resultado.chars().count(), 20);
    }

    #[test]
    fn sanea_caracteres_invalidos() {
        let resultado = sanear_nombre("a/b\\c:d");
        assert!(!resultado.contains(['/', '\\', ':']));
    }

    #[test]
    fn nombre_vacio_cae_a_sin_titulo() {
        assert_eq!(sanear_nombre("   "), "Sin título");
    }

    #[test]
    fn reconoce_prefijo_de_id_valido() {
        assert!(es_id_portapapeles("3fa85f64-5717-4562-b3fc-2c963f66afa6"));
    }

    #[test]
    fn no_confunde_texto_con_guion_bajo_con_un_id() {
        assert!(!es_id_portapapeles("la ciudad es"));
    }

    #[test]
    fn elemento_desde_ruta_detecta_fijado() {
        let carpeta = std::env::temp_dir().join("remaph_test_fijado");
        fs::create_dir_all(&carpeta).unwrap();

        let ruta = carpeta.join("3fa85f64-5717-4562-b3fc-2c963f66afa6_MiLink.txt");
        fs::write(&ruta, "hola").unwrap();

        let elemento = elemento_desde_ruta(ruta.clone()).unwrap();

        assert!(elemento.fijado);
        assert_eq!(elemento.nombre, "MiLink");
        assert_eq!(
            elemento.id_portapapeles.as_deref(),
            Some("3fa85f64-5717-4562-b3fc-2c963f66afa6")
        );

        fs::remove_dir_all(&carpeta).unwrap();
    }

    #[test]
    fn elemento_desde_ruta_detecta_rotativo() {
        let carpeta = std::env::temp_dir().join("remaph_test_rotativo");
        fs::create_dir_all(&carpeta).unwrap();

        let ruta = carpeta.join("la ciudad es.txt");
        fs::write(&ruta, "hola").unwrap();

        let elemento = elemento_desde_ruta(ruta.clone()).unwrap();

        assert!(!elemento.fijado);
        assert_eq!(elemento.nombre, "la ciudad es");
        assert_eq!(elemento.id_portapapeles, None);

        fs::remove_dir_all(&carpeta).unwrap();
    }

    #[test]
    fn resuelve_conflicto_de_nombre() {
        let carpeta = std::env::temp_dir().join("remaph_test_conflicto");
        fs::create_dir_all(&carpeta).unwrap();
        fs::write(carpeta.join("hola.txt"), "a").unwrap();
        fs::write(carpeta.join("hola (1).txt"), "b").unwrap();

        let resultado = nombre_sin_conflicto(&carpeta, "hola", "txt");
        assert_eq!(resultado, "hola (2)");

        fs::remove_dir_all(&carpeta).unwrap();
    }

    #[test]
    fn flujo_guardar_listar_fijar_desfijar_renombrar_eliminar() {
        let base = std::env::temp_dir().join("remaph_test_appdata");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        // Limpieza por si quedó algo de una corrida anterior.
        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        let contenido = ContenidoPortapapeles::Texto("Hola mundo".to_string());
        let ruta = guardar_rotativo(&contenido).unwrap();
        assert!(ruta.exists());

        let rotativos = listar_rotativos().unwrap();
        assert_eq!(rotativos.len(), 1);

        let id = "3fa85f64-5717-4562-b3fc-2c963f66afa6";
        let ruta_fijada = fijar(&ruta, id).unwrap();
        assert!(ruta_fijada.exists());
        assert!(!ruta.exists());

        let fijados = listar_fijados(id).unwrap();
        assert_eq!(fijados.len(), 1);
        assert_eq!(fijados[0].nombre, "Hola mundo");

        let ruta_desfijada = desfijar(&ruta_fijada).unwrap();
        assert!(ruta_desfijada.exists());

        let ruta_renombrada = renombrar(&ruta_desfijada, "Otro nombre").unwrap();
        assert!(ruta_renombrada.exists());

        editar_texto(&ruta_renombrada, "Contenido nuevo").unwrap();
        assert_eq!(
            fs::read_to_string(&ruta_renombrada).unwrap(),
            "Contenido nuevo"
        );

        eliminar(&ruta_renombrada).unwrap();
        assert!(!ruta_renombrada.exists());

        let _ = fs::remove_dir_all(&base);
    }
}
