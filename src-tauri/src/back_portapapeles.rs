// ======================================================
// 📋 back_portapapeles
// ======================================================
// ETAPAS E, F Y G DEL PLAN "PORTAPAPELES"
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
// (Etapa E) Funciones PURAS de manejo de archivos: nombrar, guardar
// un elemento nuevo como rotativo, listar (rotativos / fijados de un
// id), aplicar el límite del pool, fijar/desfijar, renombrar, editar
// contenido de texto, eliminar. Estas funciones no saben qué filas
// están en modo Registro ni escuchan el portapapeles del sistema —
// solo entienden la carpeta y sus archivos.
//
// (Etapa F) Dueño también del estado ACTIVOS (qué ids de fila están
// en modo Registro ahora mismo y con qué límite cada uno), del
// límite EFECTIVO entre todos ellos, y del arranque (una sola vez
// por proceso) del listener real de back_portapapeles_captura.rs.
// en_cambio_del_sistema() es el punto donde un aviso real de cambio
// del portapapeles termina convirtiéndose (o no, según ACTIVOS) en
// un archivo nuevo del pool.
//
// (Etapa G) Dueño también de las ventanas flotantes nativas de
// Portapapeles — mismo patrón que back_menu_express.rs (registro
// ABIERTOS_VENTANAS + AppHandle global + posicionamiento Persistente/
// Cursor + WS_EX_NOACTIVATE). abrir_o_alternar() aplica las reglas
// de apertura del plan (según ACTIVOS, ver construir_datos) para
// decidir si la ventana abre en modo Registro (mostrando todo el
// pool) o en Simple (mostrando/generando un solo rotativo actual).
//
// Testeable con datos de prueba (ver tests al final), sin ventana
// real de Windows detrás (crear_ventana() sí la necesita, pero el
// armado de datos — construir_datos()/resolver_elemento_simple() —
// se puede probar aparte).
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
// back_portapapeles_captura::en_cambio_portapapeles() (Etapa D) ya
// llama a en_cambio_del_sistema() acá abajo cada vez que Windows
// avisa un cambio real del portapapeles — es la única conexión entre
// el listener y el pool de archivos.
//
// ETAPA G: todavía nadie más llama a abrir_o_alternar()/inicializar()/
// cerrar()/cerrar_todas()/obtener_datos() — Etapa I conecta
// abrir_o_alternar() con el brazo AccionCache::Portapapeles de
// runtime.rs (mismo criterio que back_menu_express.rs), lib.rs/
// setup() llama inicializar() recién cuando se agregue ahí, y Etapa H
// expone el resto (toggle Registro, fijar, etc.) como comandos Tauri
// finos que delegan acá. Hasta entonces, el compilador va a avisar
// con warnings de "función/struct nunca usada" para varias de las
// funciones públicas de esta etapa (abrir_o_alternar, inicializar,
// cerrar_todas, obtener_datos) — es esperable, mismo caso que pasó
// con back_portapapeles_captura.rs en la Etapa D, y desaparece solo
// a medida que Etapas H/I las conecten.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// Contenido de portapapeles (ContenidoPortapapeles, de
// back_portapapeles_captura.rs), rutas de archivos ya existentes en
// el pool, ids de Portapapeles (String, el mismo RemapeoCache::id de
// la fila), y — Etapa G — un PortapapelesPaquete (espejo de
// AccionCache::Portapapeles) para abrir_o_alternar()/inicializar(app)
// para fijar el AppHandle global.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// ElementoPortapapeles (ruta, nombre, extensión, fijado/no,
// id dueño si aplica, fecha de modificación) — listas de estos para
// listar_rotativos()/listar_fijados(), o una ruta sola para las
// operaciones de escritura (guardar_rotativo, fijar, desfijar,
// renombrar). Etapa F agrega: esta_activo()/hay_algun_activo() (bool)
// y limite_efectivo() (u32) que Etapa G ya usa para decidir cómo
// abrir cada ventana según las reglas del plan. Etapa G agrega:
// PortapapelesDatosUI (paquete completo ya serializable para la
// ventana — nombre/comportamiento/ubicacion/tamaños/límite/color +
// fijados/rotativos en vocabulario UI) vía obtener_datos(id).
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
//   la vez lo calcula Etapa F (ver limite_efectivo() más abajo);
//   aplicar_limite() en sí solo aplica el número que llega.
// • editar_texto() rechaza archivos que no sean .txt.
//
// ETAPA F — ACTIVOS, arranque/parada de la captura real:
// • ACTIVOS es el conjunto de ids de fila en modo Registro en este
//   momento, con el límite que CADA una pidió (plan: "El número de
//   cada Portapapeles indica solo el número que muestra, es
//   individual"). Vive solo en memoria — igual que ABIERTOS en
//   back_menu_express.rs, no sobrevive a un reinicio (spec: "Al
//   reiniciar el programa, vuelve a Simple").
// • limite_efectivo() es el MAYOR límite entre todos los ids
//   activos ahora mismo (plan: "de todos los que están en modo
//   registro en ese momento, el mayor es el que manda"). Con
//   ACTIVOS vacío no importa (no se escribe nada), así que se
//   define en 0 para ese caso.
// • activar_registro() es idempotente por id: si el id ya estaba en
//   ACTIVOS, solo actualiza su límite pedido (no reinicia nada).
// • ETAPA J.1 (reemplaza el diseño original "el listener arranca una
//   vez y nunca se detiene"): ahora hay UN SOLO listener nativo
//   (back_portapapeles_captura::asegurar_listener() / detener_
//   listener()) cuya existencia sigue a debe_existir_listener() —
//   true si hay algún id en ACTIVOS O alguna ventana de Portapapeles
//   abierta (en cualquier modo). activar_registro(), desactivar_
//   registro(), abrir_o_alternar() y cerrar() (más el cierre por
//   [x]/Alt+F4, en crear_ventana()) llaman asegurar_listener() o
//   detener_listener_si_no_hace_falta() según corresponda en cada
//   uno de esos 4 puntos donde el estado puede cambiar. Ambas
//   funciones del listener son idempotentes, así que llamarlas de
//   más nunca es un problema.
// • en_cambio_del_sistema() es lo que back_portapapeles_captura::
//   en_cambio_portapapeles() llama en cada aviso real de Windows.
//   Dos ramas (ver su propio comentario más abajo): si hay algún
//   Registro activo, guarda rotativo + aplica el límite efectivo
//   (comportamiento original, sin cambios); si no hay Registro pero
//   sí alguna ventana Simple abierta, reusa el rotativo más reciente
//   si el contenido no cambió o guarda uno nuevo si cambió (mismo
//   criterio que resolver_elemento_simple() al abrir), sin aplicar
//   ningún límite. En ambos casos, al final notifica (Tauri emit) a
//   cada ventana de Portapapeles abierta con sus datos ya
//   recalculados — así ninguna necesita cerrarse/reabrirse para
//   verse actualizada.
//
// ETAPA G — ventana real:
// • Reglas de apertura (según ACTIVOS), ver construir_datos():
//   1) id ya en ACTIVOS → modo Registro: se listan TODOS los
//      rotativos del pool (ya recortados al límite por
//      en_cambio_del_sistema() en su momento) — no se genera nada
//      nuevo al abrir, solo se listan.
//   2) ACTIVOS no vacío pero con OTRO id → Simple, mostrando el
//      último rotativo YA EXISTENTE sin generar uno nuevo (evita
//      duplicar mientras otro Portapapeles está registrando).
//   3) ACTIVOS vacío → Simple normal: resolver_elemento_simple()
//      lee el portapapeles del sistema y reusa el rotativo más
//      reciente si su contenido coincide (mismo_contenido()), o
//      genera uno nuevo si no. Si el portapapeles del sistema no
//      tiene nada legible, se muestra igual el rotativo más
//      reciente que ya hubiera (no se vacía la ventana solo porque
//      AHORA el portapapeles tiene, por ejemplo, un archivo copiado
//      del explorador — algo que este tipo no guarda).
// • Los FIJADOS de la fila se listan siempre, en cualquiera de los 3
//   casos — viven aparte de ACTIVOS, son exclusivos de ese id.
// • mismo_contenido() compara CONTENIDO real (texto UTF-8 tal cual;
//   imagen decodificando el .png guardado de vuelta a RGBA8 y
//   comparando píxeles), nunca por nombre — el nombre de un texto ya
//   viene recortado a 20 caracteres, así que dos textos distintos
//   con el mismo prefijo no deben confundirse.
// • Portapapeles siempre es una lista vertical — a diferencia de
//   MenuExpress no existe una variante "Radial", así que
//   ubicar_en_monitor() acá no recibe es_radial (siempre cuadrícula/
//   esquina).
// • comportamiento (Toggle/Efímero) viaja en PortapapelesPaquete
//   porque así compila AccionCache::Portapapeles, pero esta etapa no
//   le da ningún efecto propio todavía (Portapapeles no tiene
//   botones ejecutables adentro como MenuExpress — click en un
//   elemento pega, no dispara un remapeo con down/up propio).
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
// activar_registro() / desactivar_registro()
//     Agregan/sacan un id de ACTIVOS y aseguran/revisan el listener
//     según haga falta (Etapa F, arranque/parada real en Etapa J.1).
// esta_activo() / hay_algun_activo()
//     Consultas de ACTIVOS para que Etapa G decida cómo abrir la
//     ventana (Etapa F).
// limite_efectivo()
//     El mayor límite pedido entre los ids activos ahora (Etapa F).
// hay_alguna_ventana_abierta() / debe_existir_listener() /
// debe_procesar_cambio() / detener_listener_si_no_hace_falta()
//     Condición y helpers de arranque/parada del listener único
//     (Etapa J.1).
// en_cambio_del_sistema()
//     Reacciona a un cambio real del portapapeles: guarda/reusa el
//     rotativo según haya Registro activo o solo ventana Simple
//     abierta, y notifica a las ventanas abiertas (Etapa F, ETAPA
//     J.1).
// notificar_ventanas_abiertas()
//     Recalcula y emite (Tauri event) los datos de cada ventana de
//     Portapapeles abierta (Etapa J.1).
// inicializar(app)
//     Guarda el AppHandle global — llamado una sola vez desde
//     setup() de tauri::Builder, cuando lib.rs lo agregue (Etapa G,
//     conexión real en Etapa I).
// abrir_o_alternar(id, paquete)
//     Si ya hay ventana abierta para ese id, la cierra (toggle a
//     nivel de trigger); si no, arma los datos según ACTIVOS y crea
//     la ventana (Etapa G).
// crear_ventana(app, id, paquete)
//     Arma y muestra la ventana nativa real, en el hilo principal
//     (Etapa G).
// cerrar(id) / cerrar_todas()
//     Cierran una ventana puntual, o todas (para Etapa L) (Etapa G).
// obtener_datos(id)
//     Consulta de sólo lectura del registro de ventanas abiertas —
//     la propia ventana la llama al cargar (Etapa G).
// construir_datos(id, paquete) / resolver_elemento_simple() /
// mismo_contenido()
//     Lógica interna de armado de datos según ACTIVOS y de
//     reuso/generación del rotativo en modo Simple (Etapa G).
// ======================================================

use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

use serde::Serialize;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use windows_sys::Win32::Foundation::SYSTEMTIME;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL,
};

use crate::back_portapapeles_captura::{self, ContenidoPortapapeles};
use crate::perfil_cache::{ComportamientoMenu, TamanoBotonPortapapeles, TamanoMenu, UbicacionMenu};

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
// 🕒 MARCAR RECIENTE (silencioso) — ETAPA J.2
// ------------------------------------------------------
// spec: al entrar a editar un archivo de texto, antes de abrir el
// popup se le actualiza la fecha de modificación a "ahora" para que
// quede primero en el orden de rotativos — así, si en paralelo
// aplicar_limite() recorta los más antiguos (modo Registro activo
// en otra fila), el archivo que se está editando queda a salvo.
// A propósito NO llama a refrescar_datos() ni notificar_ventanas_
// abiertas(): no debe disparar ninguna actualización de UI, el
// nuevo orden se ve solo, naturalmente, la próxima vez que algo
// dispare un refresco (spec: "sin dar la orden de actualizar la ui").
// ======================================================

pub fn marcar_reciente(ruta: &Path) -> Result<(), String> {
    tocar_ahora(ruta)
}

// ======================================================
// 🗑️ ELIMINAR
// ======================================================

pub fn eliminar(ruta: &Path) -> Result<(), String> {
    fs::remove_file(ruta).map_err(|error| error.to_string())
}

// ======================================================
// 🟢 ACTIVOS (ids en modo Registro) — ETAPA F
// ------------------------------------------------------
// id de fila -> límite que ESA fila pidió (columna Extra,
// AccionCache::Portapapeles::limite). La presencia de un id acá ES
// "está en modo Registro" — mismo criterio que ABIERTOS en
// back_menu_express.rs (la existencia de la entrada es la fuente de
// verdad). Solo en memoria: no persiste (spec: "Al reiniciar el
// programa, vuelve a Simple").
// ======================================================

static ACTIVOS: Mutex<Option<HashMap<String, u32>>> = Mutex::new(None);

fn con_activos<R>(f: impl FnOnce(&mut HashMap<String, u32>) -> R) -> R {
    let mut guardia = ACTIVOS.lock().unwrap();
    let mapa = guardia.get_or_insert_with(HashMap::new);
    f(mapa)
}

/// Agrega (o actualiza el límite de) un id en modo Registro y se
/// asegura de que el listener esté corriendo (ETAPA J.1 — ya no
/// "una sola vez para siempre": asegurar_listener() es idempotente,
/// así que llamarla de más no tiene costo).
pub fn activar_registro(id_portapapeles: &str, limite: u32) {
    con_activos(|activos| {
        activos.insert(id_portapapeles.to_string(), limite);
    });

    back_portapapeles_captura::asegurar_listener();
}

/// Saca un id de modo Registro. Si ya no queda ningún motivo para
/// que el listener siga corriendo (ni Registro activo ni ventana
/// abierta — ver debe_existir_listener()), lo detiene.
pub fn desactivar_registro(id_portapapeles: &str) {
    con_activos(|activos| {
        activos.remove(id_portapapeles);
    });

    detener_listener_si_no_hace_falta();
}

/// ¿Esta fila está en modo Registro ahora mismo?
pub fn esta_activo(id_portapapeles: &str) -> bool {
    con_activos(|activos| activos.contains_key(id_portapapeles))
}

/// ¿Hay ALGÚN Portapapeles en modo Registro ahora mismo (de
/// cualquier id)? — lo usa Etapa G para decidir si una ventana que
/// se abre para un id distinto al activo debe mostrarse en Simple
/// sin generar un rotativo nuevo (regla de apertura del plan).
pub fn hay_algun_activo() -> bool {
    con_activos(|activos| !activos.is_empty())
}

/// El mayor límite pedido entre todos los ids activos ahora mismo
/// (plan: "el mayor es el que manda"). 0 si no hay ninguno activo
/// (en ese caso en_cambio_del_sistema() no llega a usarlo, porque no
/// escribe nada).
pub fn limite_efectivo() -> u32 {
    con_activos(|activos| activos.values().copied().max().unwrap_or(0))
}

// ======================================================
// 👁️ CONDICIÓN DEL LISTENER — ETAPA J.1
// ------------------------------------------------------
// "Debe existir" el listener nativo mientras haya algún motivo para
// mirar el portapapeles del sistema: algún Registro activo (de
// cualquier id) O alguna ventana de Portapapeles abierta ahora mismo
// (en cualquier modo — una Simple también necesita el listener para
// poder actualizarse sola, ver en_cambio_del_sistema() más abajo).
// debe_procesar_cambio() es la misma condición, expuesta para que
// back_portapapeles_captura::en_cambio_portapapeles() pueda cortar
// temprano sin leer el portapapeles si por algún motivo el listener
// siguiera vivo un instante después de que la condición ya dio
// false (ver detener_listener_si_no_hace_falta()).
// ======================================================

fn hay_alguna_ventana_abierta() -> bool {
    con_ventanas(|mapa| !mapa.is_empty())
}

fn debe_existir_listener() -> bool {
    hay_algun_activo() || hay_alguna_ventana_abierta()
}

pub fn debe_procesar_cambio() -> bool {
    debe_existir_listener()
}

/// Detiene el listener si, tras el cambio que se acaba de aplicar
/// (cerrar una ventana o sacar un id de ACTIVOS), ya no queda ningún
/// motivo para que siga corriendo. Sin efecto si sigue haciendo
/// falta, o si ya estaba detenido.
fn detener_listener_si_no_hace_falta() {
    if !debe_existir_listener() {
        back_portapapeles_captura::detener_listener();
    }
}

// ======================================================
// 🔔 EN CAMBIO DEL SISTEMA — ETAPA F, reescrita en ETAPA J.1
// ------------------------------------------------------
// Llamado por back_portapapeles_captura::en_cambio_portapapeles()
// en cada aviso real de Windows (WM_CLIPBOARDUPDATE). Dos ramas:
//
// • Si hay algún Registro activo (de cualquier id): comportamiento
//   original, sin cambios — guarda el contenido como rotativo nuevo
//   y recién DESPUÉS aplica el límite efectivo (para no borrar por
//   error el elemento que se acaba de crear).
// • Si no hay Registro pero sí alguna ventana Simple abierta: mismo
//   criterio que resolver_elemento_simple() al abrir una ventana —
//   si el contenido nuevo es igual al rotativo más reciente, no
//   hace nada (ya está reflejado); si es distinto, guarda uno nuevo.
//   Sin aplicar ningún límite acá (el límite es un concepto
//   exclusivo de Registro — spec: "los fijados no se eliminan ni
//   cuentan para el límite", y un Simple puro ni siquiera tiene uno
//   configurado con sentido).
// • Si no hay ni Registro ni ventana abierta, no se toca el pool —
//   no debería llegar a pasar (el listener no estaría corriendo),
//   pero queda como red de seguridad.
//
// En cualquiera de los dos primeros casos, termina notificando a
// todas las ventanas de Portapapeles abiertas con sus datos ya
// recalculados (ver notificar_ventanas_abiertas() más abajo) — así
// una ventana Registro ve aparecer el elemento nuevo arriba de su
// lista, y una ventana Simple abierta mientras OTRO id está en
// Registro ve el último rotativo sin haber generado nada ella misma.
// ======================================================

pub fn en_cambio_del_sistema(contenido: &ContenidoPortapapeles) {
    if ignorar_proximo_cambio() {
        return;
    }

    if hay_algun_activo() {
        if guardar_rotativo(contenido).is_ok() {
            let _ = aplicar_limite(limite_efectivo());
        }
    } else if hay_alguna_ventana_abierta() {
        guardar_si_distinto_del_mas_reciente(contenido);
    } else {
        return;
    }

    notificar_ventanas_abiertas();
}

/// Guarda `contenido` como rotativo nuevo solo si es distinto al
/// rotativo más reciente que ya hay en el pool (mismo criterio de
/// comparación que resolver_elemento_simple(), vía mismo_contenido())
/// — evita duplicar el mismo contenido si el aviso de Windows llega
/// más de una vez, o si ya estaba reflejado.
fn guardar_si_distinto_del_mas_reciente(contenido: &ContenidoPortapapeles) {
    let mas_reciente = listar_rotativos()
        .ok()
        .and_then(|lista| lista.into_iter().next());

    let ya_coincide = mas_reciente
        .as_ref()
        .map(|elemento| mismo_contenido(elemento, contenido))
        .unwrap_or(false);

    if !ya_coincide {
        let _ = guardar_rotativo(contenido);
    }
}

// ======================================================
// 📣 NOTIFICAR VENTANAS ABIERTAS — ETAPA J.1
// ------------------------------------------------------
// Recalcula PortapapelesDatosUI para cada ventana de Portapapeles
// abierta (mismo camino que refrescar_datos(), Etapa H, que ya usan
// los comandos de mutación manual) y le emite un evento Tauri con
// esos datos — la propia ventana (Etapa J.2) escucha ese evento y se
// vuelve a pintar sola, sin que el usuario tenga que hacer nada.
// emit_to() por label (no un emit() global) para no forzar a cada
// ventana a filtrar eventos ajenos a su propio id.
// ======================================================

fn notificar_ventanas_abiertas() {
    let Some(app) = app_handle() else {
        return;
    };

    let ids: Vec<String> = con_ventanas(|mapa| mapa.keys().cloned().collect());

    for id in ids {
        if let Some(datos) = refrescar_datos(&id) {
            let _ = app.emit_to(label_de(&id).as_str(), "portapapeles-actualizado", datos);
        }
    }
}

// ======================================================
// 🔒 BLOQUEO ANTI-DUPLICADO (tras pegar) — ETAPA H
// ------------------------------------------------------
// spec: "Al clickear en [el nombre de un elemento] se pega el
// contenido del archivo al portapapeles y a la ventana activa. Hacer
// esto debe bloquear el modo automático de crear archivo por
// modificación de portapapeles, para no generar un duplicado."
//
// pegar() (más abajo) escribe al portapapeles del sistema con
// arboard antes de simular Ctrl+V — eso por sí solo ya dispara
// WM_CLIPBOARDUPDATE en el listener de back_portapapeles_captura.rs,
// como cualquier otro cambio real. Sin este bloqueo, en_cambio_del_
// sistema() (si hay algún Registro activo) o resolver_elemento_
// simple() (si no hay ninguno) tratarían ese aviso como un cambio
// nuevo del usuario y generarían un rotativo duplicado del mismo
// contenido que ya existía.
//
// Se usa un timestamp con expiración corta (no solo un booleano) en
// vez de "bloqueado hasta que llegue el próximo aviso": si por lo
// que sea el aviso de Windows nunca llega (falla puntual del
// listener, foco robado a mitad de camino, etc.), un booleano sin
// vencimiento dejaría el pool bloqueado para siempre. Con
// expiración, como mucho se pierde un aviso real dentro de esa
// ventana muy corta — mismo tipo de trade-off que IGNORAR_MOVED_MS
// más abajo (Etapa G).
// ======================================================

const IGNORAR_PROXIMO_CAMBIO_MS: u128 = 400;

static IGNORAR_HASTA: Mutex<Option<std::time::Instant>> = Mutex::new(None);

fn marcar_ignorar_proximo_cambio() {
    let vencimiento = std::time::Instant::now()
        + std::time::Duration::from_millis(IGNORAR_PROXIMO_CAMBIO_MS as u64);

    *IGNORAR_HASTA.lock().unwrap() = Some(vencimiento);
}

fn ignorar_proximo_cambio() -> bool {
    let mut guardia = IGNORAR_HASTA.lock().unwrap();

    match *guardia {
        Some(vencimiento) if std::time::Instant::now() < vencimiento => {
            // Se consume: el bloqueo cubre el próximo aviso, no
            // todos los avisos hasta que venza el timer — si el
            // usuario copia algo nuevo de verdad un instante después
            // de pegar, ese cambio sí debe registrarse.
            *guardia = None;
            true
        }
        _ => false,
    }
}

// ======================================================
// 📌➡️📋 PEGAR — ETAPA H
// ------------------------------------------------------
// spec: click en el nombre de un elemento → "se pega el contenido
// del archivo al portapapeles y a la ventana activa". Dos pasos:
//
// 1) Lee el archivo (texto o imagen, según extensión) y lo escribe
//    al portapapeles del sistema (back_portapapeles_captura::
//    escribir_portapapeles) — esto es lo que "pega al portapapeles".
// 2) Simula Ctrl+V con SendInput — esto es lo que "pega a la ventana
//    activa". La ventana de Portapapeles nunca tiene foco (creada
//    con WS_EX_NOACTIVATE, Etapa G) así que Ctrl+V le llega a la
//    ventana que el usuario tenía activa antes de abrir el
//    Portapapeles, que sigue siéndolo.
//
// Antes de escribir al portapapeles se marca el bloqueo anti-
// duplicado (ver arriba) — el propio paso 1 dispara
// WM_CLIPBOARDUPDATE en el listener, y sin el bloqueo eso generaría
// un rotativo duplicado del mismo contenido que ya existía en el
// pool.
// ======================================================

pub fn pegar(ruta: &Path) -> Result<(), String> {
    let contenido = contenido_desde_archivo(ruta)?;

    marcar_ignorar_proximo_cambio();

    back_portapapeles_captura::escribir_portapapeles(&contenido)?;

    simular_ctrl_v();

    Ok(())
}

fn contenido_desde_archivo(ruta: &Path) -> Result<ContenidoPortapapeles, String> {
    let extension = ruta
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_lowercase();

    match extension.as_str() {
        "txt" => {
            let texto = fs::read_to_string(ruta).map_err(|error| error.to_string())?;
            Ok(ContenidoPortapapeles::Texto(texto))
        }

        "png" => {
            let (ancho, alto) = dimensiones_png(ruta)
                .ok_or_else(|| "no se pudo leer el tamaño de la imagen".to_string())?;

            let pixeles = decodificar_png_rgba8(ruta)
                .ok_or_else(|| "no se pudo decodificar la imagen".to_string())?;

            Ok(ContenidoPortapapeles::Imagen {
                ancho: ancho as usize,
                alto: alto as usize,
                pixeles,
            })
        }

        _ => Err(format!("extensión no soportada: {extension}")),
    }
}

fn dimensiones_png(ruta: &Path) -> Option<(u32, u32)> {
    let archivo = fs::File::open(ruta).ok()?;
    // png::Decoder exige R: Read + BufRead — fs::File solo da Read,
    // hace falta envolverlo en BufReader (mismo motivo en
    // decodificar_png_rgba8() más abajo).
    let decodificador = png::Decoder::new(BufReader::new(archivo));
    let lector = decodificador.read_info().ok()?;
    let info = lector.info();

    Some((info.width, info.height))
}

// ======================================================
// ⌨️ SIMULAR CTRL+V (SendInput crudo)
// ------------------------------------------------------
// Mismo patrón que back_multimedia.rs::enviar_vk — down de Ctrl,
// down de V, up de V, up de Ctrl, todo en un solo SendInput (4
// eventos). Sin KEYEVENTF_EXTENDEDKEY acá: Ctrl y V no son teclas
// extendidas (a diferencia de las teclas multimedia de
// back_multimedia.rs), así que no corresponde marcarlas así.
//
// VK_V no existe como constante en windows-sys (ni en la Win32 API
// en general): Winuser.h no define constantes para las teclas
// alfanuméricas A-Z/0-9, sus virtual-key codes son directamente su
// valor ASCII en mayúscula (documentación oficial de Microsoft,
// "Keyboard Input" — "there is no constant named VK_A [...] just
// use the numeric value"). 0x56 es 'V'.
// ======================================================

const VK_V: VIRTUAL_KEY = 0x56;

fn simular_ctrl_v() {
    let mut eventos = [
        input_teclado(VK_CONTROL, false),
        input_teclado(VK_V, false),
        input_teclado(VK_V, true),
        input_teclado(VK_CONTROL, true),
    ];

    unsafe {
        SendInput(
            eventos.len() as u32,
            eventos.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

fn input_teclado(vk: VIRTUAL_KEY, soltar: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if soltar { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

// ======================================================
// 🪟 VENTANA DE PORTAPAPELES — ETAPA G
// ------------------------------------------------------
// Ciclo de vida de la ventana real, paralelo a
// back_menu_express.rs (mismo registro ABIERTOS + AppHandle global,
// mismo criterio de posicionamiento Persistente/Cursor y
// WS_EX_NOACTIVATE). A diferencia de MenuExpress, acá no hay
// "botones" que compilar de antemano: el contenido (fijados/
// rotativos) se lee del pool de archivos (Etapa E) recién al armar
// los datos de la ventana, según el modo que le toque abrir según
// ACTIVOS (reglas del plan, ver más abajo).
// ======================================================

// ======================================================
// 📦 PAQUETE RECIBIDO DESDE RUNTIME
// ------------------------------------------------------
// Espejo exacto de los campos de AccionCache::Portapapeles (ver
// perfil_cache.rs) — separado en su propio struct acá, mismo
// criterio que MenuExpressPaquete en back_menu_express.rs. El campo
// `comportamiento` viaja igual que en MenuExpress pero, a
// diferencia de ahí, Portapapeles no tiene botones ejecutables
// adentro (click en un elemento pega contenido, no dispara un
// remapeo) — por ahora no tiene efecto propio en esta ventana; se
// conserva en el paquete solo porque así compila el AccionCache, no
// se descarta por si acaso Etapa H/J le encuentra un uso (ej. cerrar
// sola tras pegar, análogo a un menú Efímero).
//
// Deriva Clone: además de crear_ventana(), lo necesita
// refrescar_datos() (Etapa H) para reconstruir PortapapelesDatosUI
// cada vez que una operación (fijar/renombrar/editar/eliminar/
// toggle Registro) cambia el pool — el paquete se guarda junto a los
// datos en ABIERTOS_VENTANAS precisamente para eso, ver más abajo.
// ======================================================

#[derive(Clone)]
pub struct PortapapelesPaquete {
    pub nombre: String,
    pub comportamiento: ComportamientoMenu,
    pub ubicacion: UbicacionMenu,
    pub tamano_boton: TamanoBotonPortapapeles,
    pub tamano_texto: TamanoMenu,
    pub limite: u32,
    pub color: String,
}

// ======================================================
// 🖥️ DATOS SERIALIZABLES PARA LA VENTANA (TS)
// ------------------------------------------------------
// Mismo vocabulario string que va a usar core_portapapeles.ts /
// portapapeles_main.ts (Etapa J) — la ventana no conoce los enums de
// Rust. camelCase vía rename_all, mismo criterio que
// MenuExpressDatosUI.
// ======================================================

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementoPortapapelesUI {
    // Ruta absoluta como String — identificador único y estable
    // frente a la ventana (fijar/renombrar/editar/eliminar la usan
    // para saber sobre qué archivo operar, ver Etapa H). Cambia si
    // el archivo se renombra/fija/desfija (rename físico), momento
    // en el que obtener_datos() ya sirve la ruta nueva de todos
    // modos (Etapa H vuelve a pedir los datos tras cada operación).
    pub ruta: String,

    pub nombre: String,

    pub extension: String,

    pub fijado: bool,

    // Timestamp Unix en milisegundos — más simple de ordenar/mostrar
    // del lado TS que un SystemTime. La ventana no reordena sola (ya
    // llega ordenado, ver construir_datos), esto es solo para que
    // pueda mostrar "hace X" si quiere.
    pub modificado_ms: u64,
}

fn modificado_a_ms(momento: SystemTime) -> u64 {
    momento
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duracion| duracion.as_millis() as u64)
        .unwrap_or(0)
}

fn elemento_a_ui(elemento: &ElementoPortapapeles) -> ElementoPortapapelesUI {
    ElementoPortapapelesUI {
        ruta: elemento.ruta.to_string_lossy().into_owned(),
        nombre: elemento.nombre.clone(),
        extension: elemento.extension.clone(),
        fijado: elemento.fijado,
        modificado_ms: modificado_a_ms(elemento.modificado),
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortapapelesDatosUI {
    pub nombre: String,

    pub comportamiento: String,

    pub ubicacion: String,

    pub tamano_boton: String,

    pub tamano_texto: String,

    // Límite que ESTA fila pide — la ventana lo muestra en su popup
    // Extra (Etapa J), no necesariamente el límite EFECTIVO del pool
    // (ese es un detalle interno de back_portapapeles.rs, no de la
    // fila que el usuario está editando).
    pub limite: u32,

    pub color: String,

    // Si está true, esta ventana es la que está en modo Registro
    // ahora mismo (esta_activo(id) — ver más abajo). La propia
    // ventana usa esto para pintar el toggle "Modo Registro" ya
    // encendido al cargar, sin depender de un segundo viaje.
    pub registro_activo: bool,

    pub fijados: Vec<ElementoPortapapelesUI>,

    // Rotativos a mostrar — nunca vacío y "sin nada" a la vez: si no
    // hay ningún rotativo (pool vacío y portapapeles del sistema
    // tampoco tiene nada legible), esta lista queda vacía y la
    // ventana muestra "Portapapel vacío" (spec) en vez de una fila.
    pub rotativos: Vec<ElementoPortapapelesUI>,
}

// ======================================================
// 🌐 APPHANDLE GLOBAL
// ------------------------------------------------------
// Mismo criterio que back_menu_express.rs::APP — el trigger que abre
// una ventana Portapapeles llega desde el hilo de entrada física
// (Etapa I), no desde un comando Tauri, así que no hay forma de
// recibirlo como parámetro en ese momento. Se fija en el setup() de
// tauri::Builder (ver lib.rs), una sola vez.
// ======================================================

static APP: OnceLock<AppHandle> = OnceLock::new();

/// Llamado una sola vez desde el setup() de tauri::Builder.
pub fn inicializar(app: AppHandle) {
    let _ = APP.set(app);
}

fn app_handle() -> Option<&'static AppHandle> {
    APP.get()
}

// ======================================================
// 🗃️ REGISTRO DE VENTANAS ABIERTAS
// ------------------------------------------------------
// id de la fila -> (paquete original + datos ya convertidos a
// vocabulario UI). Igual que ABIERTOS en back_menu_express.rs, la
// existencia de una entrada acá ES la fuente de verdad de "hay una
// ventana abierta para este id". Nombre propio (ABIERTOS_VENTANAS,
// no ABIERTOS a secas) para no chocar si algún día ambos módulos se
// funden — no hace falta acá, pero cuesta nada.
//
// Se guarda también el PAQUETE original (no solo los datos ya
// convertidos) porque los comandos de mutación (Etapa H: fijar/
// renombrar/editar/eliminar/limpiar/toggle Registro) necesitan poder
// reconstruir PortapapelesDatosUI después de cada operación —
// construir_datos() pide un &PortapapelesPaquete, y esos comandos
// solo reciben un id de String desde JS, no el paquete completo (la
// ventana no lo re-envía en cada click). Ver refrescar_datos() más
// abajo.
// ======================================================

struct VentanaAbierta {
    paquete: PortapapelesPaquete,
    datos: PortapapelesDatosUI,
}

static ABIERTOS_VENTANAS: Mutex<Option<HashMap<String, VentanaAbierta>>> = Mutex::new(None);

fn con_ventanas<R>(f: impl FnOnce(&mut HashMap<String, VentanaAbierta>) -> R) -> R {
    let mut guardia = ABIERTOS_VENTANAS.lock().unwrap();
    let mapa = guardia.get_or_insert_with(HashMap::new);
    f(mapa)
}

fn label_de(id: &str) -> String {
    format!("portapapeles_{id}")
}

// ======================================================
// 📍 ÚLTIMA POSICIÓN (ubicacion = Persistente)
// ------------------------------------------------------
// Mismo mecanismo que ULTIMA_POSICION en back_menu_express.rs — solo
// en memoria, por id, actualizado al mover/cerrar la ventana.
// ======================================================

static ULTIMA_POSICION: Mutex<Option<HashMap<String, (i32, i32)>>> = Mutex::new(None);

fn recordar_posicion(id: &str, x: i32, y: i32) {
    let mut guardia = ULTIMA_POSICION.lock().unwrap();
    guardia
        .get_or_insert_with(HashMap::new)
        .insert(id.to_string(), (x, y));
}

fn ultima_posicion(id: &str) -> Option<(i32, i32)> {
    ULTIMA_POSICION
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|mapa| mapa.get(id).copied())
}

// Mismo problema/solución que IGNORAR_MOVED_MS en
// back_menu_express.rs — Windows manda un Moved "de fábrica" al
// crear la ventana, que no es un arrastre real del usuario.
const IGNORAR_MOVED_MS: u128 = 250;

static VENTANA_LISTA_DESDE: Mutex<Option<HashMap<String, std::time::Instant>>> = Mutex::new(None);

fn marcar_ventana_lista(id: &str) {
    let mut guardia = VENTANA_LISTA_DESDE.lock().unwrap();
    guardia
        .get_or_insert_with(HashMap::new)
        .insert(id.to_string(), std::time::Instant::now());
}

fn moved_es_de_creacion(id: &str) -> bool {
    VENTANA_LISTA_DESDE
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|mapa| mapa.get(id))
        .map(|listo_desde| listo_desde.elapsed().as_millis() < IGNORAR_MOVED_MS)
        .unwrap_or(false)
}

fn olvidar_ventana_lista(id: &str) {
    if let Some(mapa) = VENTANA_LISTA_DESDE.lock().unwrap().as_mut() {
        mapa.remove(id);
    }
}

// ======================================================
// 📐 UBICAR EN MONITOR
// ------------------------------------------------------
// Idéntico a back_menu_express.rs (mismo bug histórico evitado:
// mezclar coordenadas físicas y lógicas sin convertir). Portapapeles
// solo usa la variante "cuadrícula" (es_radial = false) — no existe
// una forma Radial para esta ventana, siempre es una lista vertical.
// ======================================================

fn monitor_para_punto(app: &AppHandle, x: i32, y: i32) -> Option<(i32, i32, i32, i32, f64)> {
    let monitores = app.available_monitors().ok()?;

    let contenedor = monitores.iter().find(|m| {
        let pos = m.position();
        let size = m.size();
        x >= pos.x && x < pos.x + size.width as i32 && y >= pos.y && y < pos.y + size.height as i32
    });

    let elegido = contenedor.or_else(|| monitores.first())?;

    let pos = elegido.position();
    let size = elegido.size();

    Some((
        pos.x,
        pos.y,
        size.width as i32,
        size.height as i32,
        elegido.scale_factor(),
    ))
}

fn clamp_esquina_a_monitor(
    app: &AppHandle,
    punto: (i32, i32),
    tamano_ventana_logico: (f64, f64),
) -> (f64, f64) {
    let (px, py) = punto;
    let (ancho_log, alto_log) = tamano_ventana_logico;

    let Some((mon_x, mon_y, mon_ancho, mon_alto, escala)) = monitor_para_punto(app, px, py) else {
        return (px as f64, py as f64);
    };

    let ancho = (ancho_log * escala).round() as i32;
    let alto = (alto_log * escala).round() as i32;

    let mon_derecha = mon_x + mon_ancho;
    let mon_abajo = mon_y + mon_alto;

    let x = px.clamp(mon_x, (mon_derecha - ancho).max(mon_x));
    let y = py.clamp(mon_y, (mon_abajo - alto).max(mon_y));

    (x as f64 / escala, y as f64 / escala)
}

fn ubicar_en_monitor(
    app: &AppHandle,
    punto: (i32, i32),
    tamano_ventana_logico: (f64, f64),
) -> (f64, f64) {
    let (px, py) = punto;

    let Some((mon_x, mon_y, mon_ancho, mon_alto, escala)) = monitor_para_punto(app, px, py) else {
        return (px as f64, py as f64);
    };

    let ancho = (tamano_ventana_logico.0 * escala).round() as i32;
    let alto = (tamano_ventana_logico.1 * escala).round() as i32;

    let mon_derecha = mon_x + mon_ancho;
    let mon_abajo = mon_y + mon_alto;

    // Siempre "cuadrícula" (Portapapeles nunca es Radial): se elige
    // la esquina de la ventana que coincide con el punto según en
    // qué mitad del monitor cae ese punto — mismo criterio que
    // back_menu_express.rs::ubicar_en_monitor con es_radial = false.
    let hacia_izquierda = px > mon_x + mon_ancho / 2;
    let hacia_arriba = py > mon_y + mon_alto / 2;

    let x = if hacia_izquierda { px - ancho } else { px };
    let y = if hacia_arriba { py - alto } else { py };

    let x = x.clamp(mon_x, (mon_derecha - ancho).max(mon_x));
    let y = y.clamp(mon_y, (mon_abajo - alto).max(mon_y));

    (x as f64 / escala, y as f64 / escala)
}

// ======================================================
// 🚫 DESACTIVAR ACTIVACIÓN (WS_EX_NOACTIVATE)
// ------------------------------------------------------
// Idéntico criterio que back_menu_express.rs::desactivar_activacion
// — la ventana no debe robarle el foco a la app activa, ni al
// abrirse ni al clickear un elemento de adentro (spec: pegar
// contenido en la ventana activa del usuario, que tiene que seguir
// siendo la app de antes, no esta ventana).
// ======================================================

fn desactivar_activacion(ventana: &tauri::WebviewWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    };

    let Ok(handle) = ventana.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return;
    };

    let hwnd: HWND = win32.hwnd.get() as *mut core::ffi::c_void;

    unsafe {
        let estilo_actual = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            estilo_actual | (WS_EX_NOACTIVATE as isize),
        );
    }
}

// ======================================================
// 🔍 ¿ESTE ARCHIVO YA TIENE ESTE CONTENIDO?
// ------------------------------------------------------
// Usado por resolver_elemento_simple() para decidir si el rotativo
// más reciente del pool YA es una copia de lo que hay ahora mismo en
// el portapapeles del sistema — si lo es, se reusa en vez de generar
// un duplicado (spec: "Al cerrar la ventana y volver a abrirla se
// debería mostrar ese archivo", no uno nuevo con el mismo texto).
// Compara bytes crudos en ambos casos (texto: UTF-8 tal cual;
// imagen: los mismos bytes RGBA8 que ya guardó guardar_png) — nunca
// compara por nombre, porque el nombre de texto ya viene recortado a
// 20 caracteres y dos textos distintos podrían compartir ese
// prefijo.
// ======================================================

fn mismo_contenido(elemento: &ElementoPortapapeles, contenido: &ContenidoPortapapeles) -> bool {
    match contenido {
        ContenidoPortapapeles::Texto(texto) => {
            if elemento.extension != "txt" {
                return false;
            }

            fs::read_to_string(&elemento.ruta)
                .map(|actual| actual == *texto)
                .unwrap_or(false)
        }

        ContenidoPortapapeles::Imagen { pixeles, .. } => {
            if elemento.extension != "png" {
                return false;
            }

            // Decodifica el .png ya guardado y compara sus píxeles
            // RGBA8 contra los del portapapeles actual — más honesto
            // que comparar el archivo .png byte a byte (dos
            // codificaciones del mismo bitmap no tienen por qué dar
            // el mismo binario). Si por lo que sea no se puede
            // decodificar, se asume que no coincide (mejor un
            // duplicado ocasional que reusar algo que no es lo
            // mismo).
            decodificar_png_rgba8(&elemento.ruta)
                .map(|actual| actual == *pixeles)
                .unwrap_or(false)
        }
    }
}

fn decodificar_png_rgba8(ruta: &Path) -> Option<Vec<u8>> {
    let archivo = fs::File::open(ruta).ok()?;
    let mut decodificador = png::Decoder::new(BufReader::new(archivo));

    // El .png del pool siempre se guardó como RGBA8 sin paleta (ver
    // guardar_png() más arriba) — se fuerza la misma transformación
    // acá para no depender del default de la versión del crate, y
    // así comparar siempre RGBA8 contra RGBA8.
    decodificador.set_transformations(png::Transformations::EXPAND);

    let mut lector = decodificador.read_info().ok()?;

    // Se copian ancho/alto a variables propias (u32, no una
    // referencia) para soltar el préstamo de lector.info() antes de
    // pedir el préstamo mutable de next_frame() más abajo.
    let info = lector.info();
    let (ancho, alto) = (info.width, info.height);

    // Buffer calculado a mano (ancho × alto × 4 canales RGBA8) en
    // vez de Reader::output_buffer_size(): esa API cambió de firma
    // entre versiones del crate (usize en unas, Option<usize> en
    // otras — ver CHANGES.md de image-png), y el archivo siempre es
    // RGBA8 sin interlace (así lo escribió guardar_png(), con
    // set_color(Rgba) + set_depth(Eight) y sin set_interlaced), así
    // que el tamaño exacto es simplemente ancho * alto * 4 — no hace
    // falta preguntarle al reader.
    let tamano = ancho as usize * alto as usize * 4;
    let mut buffer = vec![0u8; tamano];

    let info_frame = lector.next_frame(&mut buffer).ok()?;

    buffer.truncate(info_frame.buffer_size());

    Some(buffer)
}

// ======================================================
// 🔄 RESOLVER ELEMENTO ACTUAL DEL MODO SIMPLE
// ------------------------------------------------------
// Regla de "Comportamiento Simple" del spec: al abrir en Simple
// normal (ACTIVOS vacío, ver construir_datos), se lee el
// portapapeles del sistema; si coincide con el rotativo más
// reciente que ya existe, se reusa ese archivo (no se genera uno
// nuevo); si no coincide (o no había ninguno), se guarda uno nuevo.
// Si el portapapeles del sistema no tiene nada legible, se usa el
// rotativo más reciente que ya exista (spec: "el elemento actual
// del portapapeles (si hay algo) o vacío si no hay nada" — nada
// legible AHORA no borra lo que ya estaba mostrado la última vez).
// None solo si no hay ni portapapeles legible ni rotativo previo —
// ahí la ventana muestra "Portapapel vacío".
// ======================================================

fn resolver_elemento_simple() -> Option<ElementoPortapapeles> {
    let mas_reciente = listar_rotativos()
        .ok()
        .and_then(|lista| lista.into_iter().next());

    let Some(contenido) = back_portapapeles_captura::leer_portapapeles() else {
        return mas_reciente;
    };

    if let Some(elemento) = &mas_reciente {
        if mismo_contenido(elemento, &contenido) {
            return mas_reciente;
        }
    }

    guardar_rotativo(&contenido)
        .ok()
        .and_then(|ruta| elemento_desde_ruta(ruta))
        .or(mas_reciente)
}

// ======================================================
// 🏗️ CONSTRUIR DATOS DE LA VENTANA
// ------------------------------------------------------
// Arma el PortapapelesDatosUI completo aplicando las reglas de
// apertura del plan, según el estado actual de ACTIVOS:
//
// • Si `id` ya está en ACTIVOS → modo Registro: la lista de
//   rotativos son TODOS los que hay en el pool (hasta el límite ya
//   aplicado por en_cambio_del_sistema()/aplicar_limite() en su
//   momento) — no se genera nada nuevo acá, solo se listan.
// • Si ACTIVOS no está vacío pero con OTRO id activo → Simple, pero
//   mostrando el último rotativo YA EXISTENTE sin generar uno nuevo
//   (evita duplicar mientras otro Portapapeles está registrando) —
//   spec: "abre en Simple, mostrando el último rotativo ya
//   existente, sin crear uno nuevo".
// • Si ACTIVOS está vacío → Simple normal: resolver_elemento_simple()
//   (lee el portapapeles del sistema, genera o reusa).
//
// Los FIJADOS de esta fila se listan siempre, sin importar el modo
// (viven aparte de ACTIVOS, exclusivos de este id).
// ======================================================

fn construir_datos(id: &str, paquete: &PortapapelesPaquete) -> PortapapelesDatosUI {
    let fijados = listar_fijados(id)
        .unwrap_or_default()
        .iter()
        .map(elemento_a_ui)
        .collect();

    let registro_activo = esta_activo(id);

    let rotativos: Vec<ElementoPortapapelesUI> = if registro_activo {
        listar_rotativos()
            .unwrap_or_default()
            .iter()
            .map(elemento_a_ui)
            .collect()
    } else if hay_algun_activo() {
        listar_rotativos()
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|elemento| vec![elemento_a_ui(&elemento)])
            .unwrap_or_default()
    } else {
        resolver_elemento_simple()
            .map(|elemento| vec![elemento_a_ui(&elemento)])
            .unwrap_or_default()
    };

    PortapapelesDatosUI {
        nombre: paquete.nombre.clone(),
        comportamiento: paquete.comportamiento.como_str().to_string(),
        ubicacion: paquete.ubicacion.como_str().to_string(),
        tamano_boton: paquete.tamano_boton.como_str().to_string(),
        tamano_texto: paquete.tamano_texto.como_str().to_string(),
        limite: paquete.limite,
        color: paquete.color.clone(),
        registro_activo,
        fijados,
        rotativos,
    }
}

// ======================================================
// ⚡🪟 ABRIR O ALTERNAR
// ------------------------------------------------------
// Único punto de entrada que va a llamar runtime.rs (Etapa I) —
// mismo criterio que back_menu_express.rs::abrir_o_alternar: alterna
// A NIVEL DE TRIGGER (volver a presionar el mismo trigger cierra SU
// ventana), sin importar comportamiento/modo.
// ======================================================

pub fn abrir_o_alternar(id: String, paquete: PortapapelesPaquete) {
    let ya_abierto = con_ventanas(|mapa| mapa.contains_key(&id));

    if ya_abierto {
        cerrar(&id);
        return;
    }

    let Some(app) = app_handle() else {
        return;
    };

    let datos = construir_datos(&id, &paquete);

    con_ventanas(|mapa| {
        mapa.insert(
            id.clone(),
            VentanaAbierta {
                paquete: paquete.clone(),
                datos,
            },
        );
    });

    // ETAPA J.1: esta ventana recién insertada ya hace que
    // debe_existir_listener() sea true — asegurar_listener() es
    // idempotente, así que no importa si ya estaba corriendo por
    // algún Registro activo en otra fila.
    back_portapapeles_captura::asegurar_listener();

    crear_ventana(app.clone(), id, paquete);
}

// ======================================================
// 📐 TAMAÑO DE VENTANA
// ------------------------------------------------------
// Portapapeles siempre es una lista vertical (nunca Radial/
// Cuadrícula como MenuExpress): ancho fijo por tamaño de botón,
// alto según cantidad de filas a mostrar (fijados + rotativos, más
// el header y la barra de botones inferior "Modo Registro"/"Limpiar
// todo" — ver spec, diagrama "VENTANA PORTAPAPELES"). Mismo criterio
// que back_menu_express.rs::calcular_tamano_ventana: los px vienen
// de config.rs (Etapa C), único lugar con el valor real — si cambia
// ahí, este cálculo ya lo respeta solo.
// ======================================================

const ALTO_HEADER: f64 = 32.0;
const ALTO_BARRA_INFERIOR: f64 = 40.0;
const ALTO_SEPARADOR: f64 = 9.0;
const ESPACIO_ENTRE_FILAS: f64 = 4.0;
const PADDING_CUERPO: f64 = 12.0;
const ANCHO_EXTRA_BOTON_ACCIONES: f64 = 90.0;
const MAX_FILAS_VISIBLES: f64 = 8.0;

fn tamano_boton_px(tamano: &TamanoBotonPortapapeles) -> (f64, f64) {
    let (ancho, alto) = match tamano {
        TamanoBotonPortapapeles::Pequeno => crate::config::portapapeles_boton_pequeno(),
        TamanoBotonPortapapeles::Mediano => crate::config::portapapeles_boton_mediano(),
        TamanoBotonPortapapeles::Grande => crate::config::portapapeles_boton_grande(),
    };

    (ancho as f64, alto as f64)
}

fn calcular_tamano_ventana(
    paquete: &PortapapelesPaquete,
    datos: &PortapapelesDatosUI,
) -> (f64, f64) {
    let (ancho_boton, alto_boton) = tamano_boton_px(&paquete.tamano_boton);

    let total_filas = datos.fijados.len() + datos.rotativos.len();
    // Al menos 1 fila de alto para el caso "Portapapel vacío" (spec:
    // se muestra un texto en el lugar donde iría la fila rotativa).
    let filas_visibles = (total_filas.max(1) as f64).min(MAX_FILAS_VISIBLES);

    let hay_separador = !datos.fijados.is_empty() && !datos.rotativos.is_empty();

    let alto_filas =
        filas_visibles * alto_boton + (filas_visibles - 1.0).max(0.0) * ESPACIO_ENTRE_FILAS;

    let alto = ALTO_HEADER
        + PADDING_CUERPO
        + alto_filas
        + if hay_separador { ALTO_SEPARADOR } else { 0.0 }
        + ALTO_BARRA_INFERIOR;

    let ancho = ancho_boton + ANCHO_EXTRA_BOTON_ACCIONES;

    (ancho.clamp(220.0, 600.0), alto.clamp(140.0, 700.0))
}

// ======================================================
// 🏗️ CREAR VENTANA
// ------------------------------------------------------
// Corre en el hilo principal (run_on_main_thread) — mismo motivo que
// back_menu_express.rs::crear_ventana: este disparo va a llegar
// desde el hilo de entrada física (Etapa I), no desde un comando
// Tauri async, así que WebviewWindowBuilder::build() necesita
// marshalling explícito al hilo principal (WebView2 lo exige).
// ======================================================

fn crear_ventana(app: AppHandle, id: String, paquete: PortapapelesPaquete) {
    let label = label_de(&id);

    let datos = con_ventanas(|mapa| mapa.get(&id).map(|ventana| ventana.datos.clone()));

    let Some(datos) = datos else {
        // No debería pasar (abrir_o_alternar ya insertó antes de
        // llamar acá) — por las dudas, no hay datos que mostrar.
        con_ventanas(|mapa| {
            mapa.remove(&id);
        });

        return;
    };

    let tamano_ventana = calcular_tamano_ventana(&paquete, &datos);
    let (ancho, alto) = tamano_ventana;

    let posicion = match &paquete.ubicacion {
        UbicacionMenu::Cursor => {
            let cursor = crate::back_coordenada::obtener_cursor();
            Some(ubicar_en_monitor(&app, cursor, tamano_ventana))
        }
        UbicacionMenu::Persistente => {
            ultima_posicion(&id).map(|punto| clamp_esquina_a_monitor(&app, punto, tamano_ventana))
        }
    };

    let id_interno = id.clone();
    let label_interno = label.clone();
    let app_interno = app.clone();

    let resultado = app.run_on_main_thread(move || {
        let mut builder = WebviewWindowBuilder::new(
            &app_interno,
            &label_interno,
            WebviewUrl::App(format!("portapapeles.html?id={id_interno}").into()),
        )
        .title("RemapH — Portapapeles")
        .inner_size(ancho, alto)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .devtools(true);

        if let Some((x, y)) = posicion {
            builder = builder.position(x, y);
        }

        match builder.build() {
            Ok(ventana) => {
                desactivar_activacion(&ventana);

                let id_cierre = id_interno.clone();

                marcar_ventana_lista(&id_cierre);

                let ventana_para_evento = ventana.clone();

                ventana.on_window_event(move |evento| {
                    if let tauri::WindowEvent::Moved(posicion) = evento {
                        if !moved_es_de_creacion(&id_cierre) {
                            recordar_posicion(&id_cierre, posicion.x, posicion.y);
                        }
                    }

                    if let tauri::WindowEvent::CloseRequested { .. } = evento {
                        if let Ok(posicion) = ventana_para_evento.outer_position() {
                            recordar_posicion(&id_cierre, posicion.x, posicion.y);
                        }
                    }

                    if let tauri::WindowEvent::CloseRequested { .. }
                    | tauri::WindowEvent::Destroyed = evento
                    {
                        olvidar_ventana_lista(&id_cierre);

                        con_ventanas(|mapa| {
                            mapa.remove(&id_cierre);
                        });

                        // ETAPA J.1: cierre por [x]/Alt+F4 (no pasa
                        // por back_portapapeles::cerrar()) — mismo
                        // chequeo ahí, para no dejar el listener
                        // corriendo de más si esta era la última
                        // ventana y no hay ningún Registro activo.
                        detener_listener_si_no_hace_falta();
                    }
                });
            }

            Err(error) => {
                eprintln!("[Portapapeles] no se pudo crear la ventana: {error}");

                con_ventanas(|mapa| {
                    mapa.remove(&id_interno);
                });
            }
        }
    });

    if let Err(error) = resultado {
        eprintln!("[Portapapeles] run_on_main_thread falló para {label}: {error}");

        con_ventanas(|mapa| {
            mapa.remove(&id);
        });
    }
}

// ======================================================
// 🚪 CERRAR / CERRAR TODAS
// ------------------------------------------------------
// Mismo criterio que back_menu_express.rs. cerrar_todas() la va a
// llamar compilador.rs en cada recompilación (Etapa L) — acá ya
// queda lista para eso, aunque todavía nadie la invoque.
// ======================================================

pub fn cerrar(id: &str) {
    if let Some(app) = app_handle() {
        if let Some(ventana) = app.get_webview_window(&label_de(id)) {
            let _ = ventana.close();
        }
    }

    con_ventanas(|mapa| {
        mapa.remove(id);
    });

    // ETAPA J.1: el cierre real de la ventana (arriba, ventana.close())
    // dispara CloseRequested/Destroyed de forma asíncrona — este
    // remove() y el chequeo de acá ya dejan el estado correcto de
    // inmediato, sin esperar a que ese evento llegue.
    detener_listener_si_no_hace_falta();
}

pub fn cerrar_todas() {
    let ids: Vec<String> = con_ventanas(|mapa| mapa.keys().cloned().collect());

    for id in ids {
        cerrar(&id);
    }
}

// ======================================================
// 📤 OBTENER DATOS
// ------------------------------------------------------
// Consulta de sólo lectura — la propia ventana la llama al cargar
// (Etapa J), con el id que vino en la URL (?id=...), y de nuevo tras
// cada operación que cambie el pool (fijar/renombrar/editar/
// eliminar/limpiar/toggle Registro — Etapa H), para no tener que
// duplicar en TS la lógica de qué mostrar según el modo.
// ======================================================

pub fn obtener_datos(id: &str) -> Option<PortapapelesDatosUI> {
    con_ventanas(|mapa| mapa.get(id).map(|ventana| ventana.datos.clone()))
}

// ======================================================
// 🔄 REFRESCAR DATOS — ETAPA H
// ------------------------------------------------------
// Reconstruye PortapapelesDatosUI desde cero (misma lógica que
// abrir_o_alternar: construir_datos() según ACTIVOS) y actualiza la
// entrada en ABIERTOS_VENTANAS, reusando el PortapapelesPaquete que
// ya se guardó al abrir la ventana. Los comandos Tauri de mutación
// (fijar/desfijar/renombrar/editar/eliminar/limpiar_todo/toggle
// Registro, en comandos.rs) llaman esto después de aplicar el cambio
// real sobre el pool, y le devuelven el resultado a la ventana en la
// misma respuesta — así la ventana no necesita un segundo viaje
// (mutar + obtener_datos por separado).
//
// Si `id` no tiene ventana abierta (alguien llamó a un comando de
// mutación para un Portapapeles que ya se cerró, ej. una operación
// en vuelo que resuelve tarde), no hace nada — no hay a quién
// refrescarle nada.
// ======================================================

pub fn refrescar_datos(id: &str) -> Option<PortapapelesDatosUI> {
    let paquete = con_ventanas(|mapa| mapa.get(id).map(|ventana| ventana.paquete.clone()))?;

    let datos = construir_datos(id, &paquete);

    con_ventanas(|mapa| {
        if let Some(ventana) = mapa.get_mut(id) {
            ventana.datos = datos.clone();
        }
    });

    Some(datos)
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

    // --------------------------------------------------
    // ETAPA F — ACTIVOS / límite efectivo
    // --------------------------------------------------
    // No prueban el arranque/parada real del listener (eso exige un
    // entorno Windows con mensajes de verdad, ver back_portapapeles_
    // captura.rs) — solo la lógica pura de ACTIVOS, que es lo que
    // puede quedar mal sin depender de Windows.
    // --------------------------------------------------

    #[test]
    fn activar_agrega_y_desactivar_saca_del_registro() {
        let id = "id-test-activos-1";

        assert!(!esta_activo(id));

        con_activos(|activos| {
            activos.insert(id.to_string(), 10);
        });
        assert!(esta_activo(id));
        assert!(hay_algun_activo());

        con_activos(|activos| {
            activos.remove(id);
        });
        assert!(!esta_activo(id));
    }

    #[test]
    fn limite_efectivo_es_el_mayor_entre_los_activos() {
        con_activos(|activos| {
            activos.clear();
            activos.insert("a".to_string(), 5);
            activos.insert("b".to_string(), 15);
            activos.insert("c".to_string(), 10);
        });

        assert_eq!(limite_efectivo(), 15);

        con_activos(|activos| activos.clear());
        assert_eq!(limite_efectivo(), 0);
    }

    #[test]
    fn en_cambio_del_sistema_no_escribe_sin_activos() {
        let base = std::env::temp_dir().join("remaph_test_en_cambio_sin_activos");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        con_activos(|activos| activos.clear());

        let contenido = ContenidoPortapapeles::Texto("no debería guardarse".to_string());
        en_cambio_del_sistema(&contenido);

        assert_eq!(listar_rotativos().unwrap().len(), 0);

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn en_cambio_del_sistema_guarda_y_aplica_limite_con_activos() {
        let base = std::env::temp_dir().join("remaph_test_en_cambio_con_activos");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        con_activos(|activos| {
            activos.clear();
            activos.insert("id-test-en-cambio".to_string(), 2);
        });

        for texto in ["uno", "dos", "tres"] {
            en_cambio_del_sistema(&ContenidoPortapapeles::Texto(texto.to_string()));
            // Windows no garantiza resolución de sub-milisegundo en
            // la fecha de modificación de un archivo recién creado;
            // sin esta pequeña espera, dos guardados muy seguidos
            // podrían empatar en fecha y volver el orden de
            // aplicar_limite() no determinístico para este test.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let rotativos = listar_rotativos().unwrap();
        assert_eq!(rotativos.len(), 2);
        assert_eq!(rotativos[0].nombre, "tres");
        assert_eq!(rotativos[1].nombre, "dos");

        con_activos(|activos| activos.clear());
        let _ = fs::remove_dir_all(&base);
    }

    // --------------------------------------------------
    // ETAPA G — mismo_contenido / resolver_elemento_simple /
    // construir_datos / calcular_tamano_ventana
    // --------------------------------------------------
    // Ninguno de estos tests crea una ventana real (eso exige
    // Windows + Tauri corriendo) — solo prueban la lógica de armado
    // de datos, que es pura sobre el pool de archivos + ACTIVOS.
    // --------------------------------------------------

    fn paquete_de_prueba(nombre: &str, limite: u32) -> PortapapelesPaquete {
        PortapapelesPaquete {
            nombre: nombre.to_string(),
            comportamiento: ComportamientoMenu::Toggle,
            ubicacion: UbicacionMenu::Persistente,
            tamano_boton: TamanoBotonPortapapeles::Mediano,
            tamano_texto: TamanoMenu::Mediano,
            limite,
            color: "cyan".to_string(),
        }
    }

    #[test]
    fn mismo_contenido_detecta_texto_igual_y_distinto() {
        let base = std::env::temp_dir().join("remaph_test_mismo_contenido_texto");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        let ruta = guardar_rotativo(&ContenidoPortapapeles::Texto("hola".to_string())).unwrap();
        let elemento = elemento_desde_ruta(ruta).unwrap();

        assert!(mismo_contenido(
            &elemento,
            &ContenidoPortapapeles::Texto("hola".to_string())
        ));
        assert!(!mismo_contenido(
            &elemento,
            &ContenidoPortapapeles::Texto("chau".to_string())
        ));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn mismo_contenido_detecta_imagen_igual_y_distinta() {
        let base = std::env::temp_dir().join("remaph_test_mismo_contenido_imagen");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        let pixeles_a = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let pixeles_b = vec![0, 0, 255, 255, 0, 0, 0, 255];

        let ruta = guardar_rotativo(&ContenidoPortapapeles::Imagen {
            ancho: 2,
            alto: 1,
            pixeles: pixeles_a.clone(),
        })
        .unwrap();
        let elemento = elemento_desde_ruta(ruta).unwrap();

        assert!(mismo_contenido(
            &elemento,
            &ContenidoPortapapeles::Imagen {
                ancho: 2,
                alto: 1,
                pixeles: pixeles_a,
            }
        ));
        assert!(!mismo_contenido(
            &elemento,
            &ContenidoPortapapeles::Imagen {
                ancho: 2,
                alto: 1,
                pixeles: pixeles_b,
            }
        ));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn construir_datos_modo_simple_normal_sin_activos() {
        let base = std::env::temp_dir().join("remaph_test_construir_simple");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        con_activos(|activos| activos.clear());

        // Sin portapapeles del sistema legible en este entorno de
        // test (no hay Windows real detrás de arboard acá), y sin
        // ningún rotativo previo → debe quedar vacío, no romper.
        let paquete = paquete_de_prueba("Mi Portapapeles", 10);
        let datos = construir_datos("id-construir-simple", &paquete);

        assert!(!datos.registro_activo);
        assert_eq!(datos.rotativos.len(), 0);
        assert_eq!(datos.fijados.len(), 0);
        assert_eq!(datos.nombre, "Mi Portapapeles");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn construir_datos_modo_registro_lista_todos_los_rotativos() {
        let base = std::env::temp_dir().join("remaph_test_construir_registro");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        let id = "id-construir-registro";

        con_activos(|activos| {
            activos.clear();
            activos.insert(id.to_string(), 10);
        });

        for texto in ["uno", "dos"] {
            guardar_rotativo(&ContenidoPortapapeles::Texto(texto.to_string())).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let paquete = paquete_de_prueba("Registro", 10);
        let datos = construir_datos(id, &paquete);

        assert!(datos.registro_activo);
        assert_eq!(datos.rotativos.len(), 2);

        con_activos(|activos| activos.clear());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn construir_datos_modo_simple_con_otro_id_activo_no_duplica() {
        let base = std::env::temp_dir().join("remaph_test_construir_otro_activo");
        fs::create_dir_all(&base).unwrap();
        std::env::set_var("APPDATA", &base);

        if let Ok(carpeta) = carpeta() {
            let _ = fs::remove_dir_all(&carpeta);
            fs::create_dir_all(&carpeta).unwrap();
        }

        guardar_rotativo(&ContenidoPortapapeles::Texto("ya existente".to_string())).unwrap();

        con_activos(|activos| {
            activos.clear();
            activos.insert("otro-id-activo".to_string(), 10);
        });

        let paquete = paquete_de_prueba("Simple pasivo", 10);
        let datos = construir_datos("id-que-no-esta-activo", &paquete);

        assert!(!datos.registro_activo);
        assert_eq!(datos.rotativos.len(), 1);
        assert_eq!(datos.rotativos[0].nombre, "ya existente");

        // No debe haber generado un rotativo nuevo — sigue habiendo
        // exactamente 1 en el pool.
        assert_eq!(listar_rotativos().unwrap().len(), 1);

        con_activos(|activos| activos.clear());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn calcular_tamano_ventana_no_es_cero_con_lista_vacia() {
        let paquete = paquete_de_prueba("Vacío", 10);
        let datos = PortapapelesDatosUI {
            nombre: paquete.nombre.clone(),
            comportamiento: "toggle".to_string(),
            ubicacion: "persistente".to_string(),
            tamano_boton: "mediano".to_string(),
            tamano_texto: "mediano".to_string(),
            limite: 10,
            color: "cyan".to_string(),
            registro_activo: false,
            fijados: vec![],
            rotativos: vec![],
        };

        let (ancho, alto) = calcular_tamano_ventana(&paquete, &datos);

        assert!(ancho > 0.0);
        assert!(alto > 0.0);
    }
}
