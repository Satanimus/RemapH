// ======================================================
// 🗂️ Configuración de Usuario
// ======================================================
// ETAPA 2 DEL FLUJO — VENTANA DE CONFIGURACIÓN
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Dueño de Configuracion_Usuario.txt: un único archivo
// de overrides compartido por las 3 pestañas de la
// Ventana de Configuración.
//
// Cada línea es "clave=valor". La clave define a qué
// pestaña pertenece:
//
// • sin prefijo   → General (variable de config.rs).
// • "css."        → Apariencia (variable de
//                    styl_variables.css). Se agrega en
//                    la Etapa 6.
// • "pulsador."   → Teclas (nombre visible de
//                    pulsadores.tsv).
//
// Este archivo conoce la parte SIN prefijo (General) y la
// parte "pulsador." (Teclas): para General carga el
// catálogo de fábrica (configuracion.tsv), aplica overrides
// sobre config.rs y persiste cambios; para Teclas valida
// contra pulsadores::por_interno() (el catálogo lo posee
// pulsadores.rs, no este archivo) y persiste, pero no
// aplica nada en caliente — pulsadores.rs lee el override
// en el momento (nombre_ui_efectivo()). Las claves "css."
// solo pasan a través de él como texto plano (las lee/
// escribe el módulo de la Etapa 6).
//
// No valida en la UI. No arma la tabla que ve el usuario
// (eso es Etapa 3/4, en comandos.rs y configuracion.ts).
//
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// • lib.rs, una sola vez al arrancar (cargar_al_iniciar).
// • comandos.rs, desde los comandos Tauri de la pestaña
//   General (Etapa 3).
//
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// Pares (clave, valor) en texto plano, ya sea desde el
// archivo en disco o desde un comando Tauri.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// • El catálogo de fábrica de General (clave, nombre UI,
//   valor por defecto, tipo).
// • Los overrides de General actualmente guardados.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// cargar_catalogo()
//     Carga (una sola vez) configuracion.tsv.
//
// leer_overrides()
//     Devuelve solo los overrides SIN prefijo (General),
//     leyendo el archivo completo y descartando las
//     claves con prefijo (css./pulsador.).
//
// guardar_override(clave, valor)
//     Valida que la clave exista en el catálogo de
//     General, la aplica en caliente (aplicar_valor) y
//     persiste el archivo completo (preservando las
//     claves con prefijo que ya hubiera).
//
// aplicar_valor(clave, valor)
//     Parsea "valor" según el tipo de "clave" (numero /
//     numero_par / texto) y llama al setter de config.rs
//     correspondiente. Claves desconocidas para esta
//     sección (con o sin prefijo) se ignoran en silencio.
//
// cargar_al_iniciar()
//     Lee los overrides de General y los aplica todos,
//     uno por uno, ignorando los que fallen (ver
//     aplicar_valor).
//
// guardar_lote(cambios)
//     Valida TODOS los cambios primero (sin aplicar
//     ninguno); si alguno falla, no aplica ni persiste
//     nada y devuelve la lista de errores. Si todos son
//     válidos, los aplica y persiste juntos.
//
// restablecer_seccion(prefijo)
//     Borra del archivo todos los overrides de una
//     sección (None = General, Some("css.") = Apariencia,
//     Some("pulsador.") = Teclas). Para General, además
//     reaplica los valores de fábrica en caliente.
//
// leer_overrides_pulsador()
//     Devuelve solo los overrides con prefijo "pulsador."
//     (Teclas), con la clave ya sin el prefijo (interno →
//     nombre personalizado).
//
// guardar_lote_pulsadores(cambios)
//     Igual que guardar_lote() pero para Teclas: valida
//     que cada clave sea un "interno" real de
//     pulsadores.tsv (pulsadores::por_interno) y que el
//     valor no esté vacío; no hay setter que aplicar en
//     caliente (pulsadores::nombre_ui_efectivo() lee el
//     override en el momento).
// ------------------------------------------------------
// Transformación:
//
// configuracion.tsv (fábrica)
//      +
// Configuracion_Usuario.txt (override, sin prefijo)
//      ↓
// aplicar_valor()
//      ↓
// config.rs (setters)
// ======================================================

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::config;
use crate::usuario;

// ======================================================
// 📦 MODELO ENTRADA DE CATÁLOGO
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum TipoValor {
    Numero,
    NumeroPar,
    Texto,
}

#[derive(Clone, Debug)]
pub struct EntradaCatalogo {
    pub clave: String,

    pub nombre_ui: String,

    pub valor_defecto: String,

    pub tipo: TipoValor,
}

// ======================================================
// 🗂️ CATÁLOGO DE FÁBRICA (General)
// ======================================================

static CATALOGO: OnceLock<Vec<EntradaCatalogo>> = OnceLock::new();

// ======================================================
// 📖 CARGAR CATÁLOGO
// ======================================================

pub fn cargar_catalogo() -> &'static Vec<EntradaCatalogo> {
    CATALOGO.get_or_init(|| {
        let texto = include_str!("configuracion.tsv");

        let mut catalogo: Vec<EntradaCatalogo> = Vec::new();

        for (numero_linea, linea) in texto.lines().enumerate() {
            let linea = linea.trim();

            if linea.is_empty() || linea.starts_with('#') {
                continue;
            }

            let columnas: Vec<&str> = linea.split('\t').collect();

            if columnas.len() != 4 {
                panic!(
                    "❌ Error interno en configuracion.tsv. Línea {}",
                    numero_linea + 1
                );
            }

            // Fila de encabezado ("clave  nombre_ui  valor_defecto  tipo").
            if columnas[0].trim() == "clave" {
                continue;
            }

            let clave = columnas[0].trim();

            let nombre_ui = columnas[1].trim();

            let valor_defecto = columnas[2].trim();

            let tipo_texto = columnas[3].trim();

            if clave.is_empty() {
                panic!(
                    "❌ Entrada de configuración sin clave. Línea {}",
                    numero_linea + 1
                );
            }

            if clave.contains('.') {
                panic!(
                    "❌ Clave de configuracion.tsv no puede contener un punto (reservado para prefijos css./pulsador.): \"{}\"",
                    clave
                );
            }

            let tipo = match tipo_texto {
                "numero" => TipoValor::Numero,
                "numero_par" => TipoValor::NumeroPar,
                "texto" => TipoValor::Texto,
                _ => panic!(
                    "❌ Tipo desconocido \"{}\" en configuracion.tsv. Línea {}",
                    tipo_texto,
                    numero_linea + 1
                ),
            };

            if catalogo.iter().any(|entrada: &EntradaCatalogo| entrada.clave == clave) {
                panic!("❌ Clave duplicada en configuracion.tsv: {}", clave);
            }

            catalogo.push(EntradaCatalogo {
                clave: clave.to_string(),

                nombre_ui: nombre_ui.to_string(),

                valor_defecto: valor_defecto.to_string(),

                tipo,
            });
        }

        catalogo
    })
}

// ======================================================
// 📍 RUTA DEL ARCHIVO DE USUARIO
// ======================================================

fn ruta_archivo() -> Result<PathBuf, String> {
    Ok(usuario::carpeta()?.join("Configuracion_Usuario.txt"))
}

// ======================================================
// 📥 LEER MAPA COMPLETO (todas las claves, con o sin prefijo)
// ------------------------------------------------------
// Único punto de lectura de Configuracion_Usuario.txt. Si
// el archivo todavía no existe, devuelve un mapa vacío (no
// es un error: significa "sin overrides todavía").
// ======================================================

fn leer_mapa_completo() -> Result<HashMap<String, String>, String> {
    let ruta = ruta_archivo()?;

    if !ruta.exists() {
        return Ok(HashMap::new());
    }

    let texto = fs::read_to_string(&ruta).map_err(|error| error.to_string())?;

    let mut mapa = HashMap::new();

    for linea in texto.lines() {
        let linea = linea.trim();

        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }

        let Some((clave, valor)) = linea.split_once('=') else {
            continue;
        };

        mapa.insert(clave.trim().to_string(), valor.trim().to_string());
    }

    Ok(mapa)
}

// ======================================================
// 📤 ESCRIBIR MAPA COMPLETO
// ------------------------------------------------------
// Único punto de escritura. Reescribe el archivo entero a
// partir del mapa recibido, así que quien llama debe partir
// de leer_mapa_completo() y modificarlo, nunca escribir un
// subconjunto — de lo contrario se pierden los overrides de
// otras secciones (css./pulsador.).
// ======================================================

fn escribir_mapa_completo(mapa: &HashMap<String, String>) -> Result<(), String> {
    let ruta = ruta_archivo()?;

    let mut claves: Vec<&String> = mapa.keys().collect();

    claves.sort();

    let mut contenido = String::from(
        "# Configuracion_Usuario.txt — overrides de la Ventana de Configuración.\n\
         # Se reescribe por completo cada vez que se guarda un cambio: no editar\n\
         # a mano mientras RemapH está abierto.\n\
         #\n\
         # Formato: clave=valor (una línea por override).\n\
         # Sin prefijo   → variable de config.rs (ver configuracion.tsv).\n\
         # Prefijo css.  → variable de styl_variables.css.\n\
         # Prefijo pulsador. → nombre visible de pulsadores.tsv.\n\n",
    );

    for clave in claves {
        let valor = &mapa[clave];

        contenido.push_str(&format!("{}={}\n", clave, valor));
    }

    fs::write(&ruta, contenido).map_err(|error| error.to_string())
}

// ======================================================
// 📥 LEER OVERRIDES (solo General, sin prefijo)
// ======================================================

pub fn leer_overrides() -> Result<HashMap<String, String>, String> {
    let mapa = leer_mapa_completo()?;

    Ok(mapa
        .into_iter()
        .filter(|(clave, _)| !clave.contains('.'))
        .collect())
}

// ======================================================
// 🔢 PARSEO DE VALORES
// ======================================================

fn parsear_numero(valor: &str) -> Result<u64, String> {
    valor
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Valor no numérico: \"{}\"", valor))
}

fn parsear_numero_par(valor: &str) -> Result<(u64, u64), String> {
    let partes: Vec<&str> = valor.split(',').collect();

    if partes.len() != 2 {
        return Err(format!(
            "Se esperaban dos números separados por coma: \"{}\"",
            valor
        ));
    }

    let ancho = partes[0]
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Valor no numérico: \"{}\"", partes[0]))?;

    let alto = partes[1]
        .trim()
        .parse::<u64>()
        .map_err(|_| format!("Valor no numérico: \"{}\"", partes[1]))?;

    Ok((ancho, alto))
}

// ======================================================
// ⚙️ APLICAR VALOR (clave → setter de config.rs)
// ------------------------------------------------------
// Claves que no matchean ningún brazo (desconocidas para
// esta sección, o con prefijo css./pulsador.) se ignoran
// en silencio: pueden pertenecer a otra sección, o ser
// restos de una versión anterior del archivo de usuario.
// ======================================================

pub fn aplicar_valor(clave: &str, valor: &str) -> Result<(), String> {
    match clave {
        "tiempo_doble" => config::establecer_tiempo_doble(parsear_numero(valor)?),

        "tiempo_triple" => config::establecer_tiempo_triple(parsear_numero(valor)?),

        "tiempo_mantenido" => config::establecer_tiempo_mantenido(parsear_numero(valor)?),

        "sensibilidad_rueda" => config::establecer_sensibilidad_rueda(parsear_numero(valor)?),

        "tiempo_repeticion" => config::establecer_tiempo_repeticion(parsear_numero(valor)?),

        "tiempo_espera_normal" => config::establecer_tiempo_espera_normal(parsear_numero(valor)?),

        "tiempo_maximo_retenido" => {
            config::establecer_tiempo_maximo_retenido(parsear_numero(valor)?)
        }

        "tecla_guardar_coordenada" => {
            let valor_limpio = valor.trim();

            if valor_limpio.is_empty() {
                return Err("La tecla no puede estar vacía".to_string());
            }

            config::establecer_tecla_guardar_coordenada(valor_limpio.to_string());
        }

        "intervalo_captura_coordenada" => {
            config::establecer_intervalo_captura_coordenada(parsear_numero(valor)?)
        }

        "delay_entre_salida_doble" => {
            config::establecer_delay_entre_salida_doble(parsear_numero(valor)?)
        }

        "delay_rueda_repeticion" => {
            config::establecer_delay_rueda_repeticion(parsear_numero(valor)?)
        }

        "tiempo_salida_mantenido" => {
            config::establecer_tiempo_salida_mantenido(parsear_numero(valor)?)
        }

        "delta_volumen" => config::establecer_delta_volumen(parsear_numero(valor)?),

        "menu_boton_pequeno" => {
            let (ancho, alto) = parsear_numero_par(valor)?;

            config::establecer_menu_boton_pequeno(ancho, alto);
        }

        "menu_boton_mediano" => {
            let (ancho, alto) = parsear_numero_par(valor)?;

            config::establecer_menu_boton_mediano(ancho, alto);
        }

        "menu_boton_grande" => {
            let (ancho, alto) = parsear_numero_par(valor)?;

            config::establecer_menu_boton_grande(ancho, alto);
        }

        "menu_texto_pequeno" => config::establecer_menu_texto_pequeno(parsear_numero(valor)?),

        "menu_texto_mediano" => config::establecer_menu_texto_mediano(parsear_numero(valor)?),

        "menu_texto_grande" => config::establecer_menu_texto_grande(parsear_numero(valor)?),

        "portapapeles_boton_pequeno" => {
            let (ancho, alto) = parsear_numero_par(valor)?;

            config::establecer_portapapeles_boton_pequeno(ancho, alto);
        }

        "portapapeles_boton_mediano" => {
            let (ancho, alto) = parsear_numero_par(valor)?;

            config::establecer_portapapeles_boton_mediano(ancho, alto);
        }

        "portapapeles_boton_grande" => {
            let (ancho, alto) = parsear_numero_par(valor)?;

            config::establecer_portapapeles_boton_grande(ancho, alto);
        }

        "tiempo_ignorar_cambio_portapapeles" => {
            config::establecer_tiempo_ignorar_cambio_portapapeles(parsear_numero(valor)?)
        }

        "tiempo_espera_pegado_imagen" => {
            config::establecer_tiempo_espera_pegado_imagen(parsear_numero(valor)?)
        }

        "tiempo_espera_pegado_texto" => {
            config::establecer_tiempo_espera_pegado_texto(parsear_numero(valor)?)
        }

        "delay_imagen_photoshop" => {
            config::establecer_delay_imagen_photoshop(parsear_numero(valor)?)
        }

        _ => {}
    }

    Ok(())
}

// ======================================================
// 💾 GUARDAR OVERRIDE (una clave, valida + aplica + persiste)
// ======================================================

pub fn guardar_override(clave: &str, valor: &str) -> Result<(), String> {
    let existe = cargar_catalogo()
        .iter()
        .any(|entrada| entrada.clave == clave);

    if !existe {
        return Err(format!("Clave de configuración desconocida: \"{}\"", clave));
    }

    aplicar_valor(clave, valor)?;

    let mut mapa = leer_mapa_completo()?;

    mapa.insert(clave.to_string(), valor.to_string());

    escribir_mapa_completo(&mapa)
}

// ======================================================
// ✅ VALIDAR SEGÚN TIPO (sin aplicar)
// ======================================================

fn validar_segun_tipo(tipo: &TipoValor, valor: &str) -> Result<(), String> {
    match tipo {
        TipoValor::Numero => parsear_numero(valor).map(|_| ()),

        TipoValor::NumeroPar => parsear_numero_par(valor).map(|_| ()),

        TipoValor::Texto => {
            if valor.trim().is_empty() {
                Err("El valor no puede estar vacío".to_string())
            } else {
                Ok(())
            }
        }
    }
}

// ======================================================
// 📦 GUARDAR LOTE (varios cambios, todo o nada)
// ------------------------------------------------------
// Primero valida cada (clave, valor) del lote sin tocar
// nada; si hay al menos un error, devuelve la lista
// completa de errores (Err) sin aplicar ni persistir
// ningún cambio del lote. Si todos son válidos, los aplica
// (en caliente) y los persiste juntos en una sola
// escritura del archivo.
//
// Un error con clave "" representa un error general, no
// asociado a una fila puntual (ej. no se pudo leer/escribir
// Configuracion_Usuario.txt).
// ======================================================

pub fn guardar_lote(cambios: &[(String, String)]) -> Result<(), Vec<(String, String)>> {
    let catalogo = cargar_catalogo();

    let mut errores: Vec<(String, String)> = Vec::new();

    for (clave, valor) in cambios {
        match catalogo.iter().find(|entrada| &entrada.clave == clave) {
            None => errores.push((
                clave.clone(),
                format!("Clave de configuración desconocida: \"{}\"", clave),
            )),

            Some(entrada) => {
                if let Err(mensaje) = validar_segun_tipo(&entrada.tipo, valor) {
                    errores.push((clave.clone(), mensaje));
                }
            }
        }
    }

    if !errores.is_empty() {
        return Err(errores);
    }

    let mut mapa = leer_mapa_completo().map_err(|error| vec![(String::new(), error)])?;

    for (clave, valor) in cambios {
        // Ya validado arriba: no debería fallar acá.
        let _ = aplicar_valor(clave, valor);

        mapa.insert(clave.clone(), valor.clone());
    }

    escribir_mapa_completo(&mapa).map_err(|error| vec![(String::new(), error)])
}

// ======================================================
// ♻️ RESTABLECER SECCIÓN
// ======================================================

pub fn restablecer_seccion(prefijo: Option<&str>) -> Result<(), String> {
    let mut mapa = leer_mapa_completo()?;

    mapa.retain(|clave, _| {
        let pertenece_a_la_seccion = match prefijo {
            Some(prefijo) => clave.starts_with(prefijo),
            None => !clave.contains('.'),
        };

        !pertenece_a_la_seccion
    });

    escribir_mapa_completo(&mapa)?;

    // Solo General tiene "aplicar en caliente" acá: Apariencia se
    // resuelve leyendo CSS y Teclas leyendo pulsadores.tsv en el
    // momento (Etapas 5/6), no a través de config.rs.
    if prefijo.is_none() {
        for entrada in cargar_catalogo() {
            let _ = aplicar_valor(&entrada.clave, &entrada.valor_defecto);
        }
    }

    Ok(())
}

// ======================================================
// 🚀 CARGAR AL INICIAR
// ------------------------------------------------------
// Se llama una sola vez, desde setup() en lib.rs. Nunca
// hace panic: un override roto o una clave desconocida no
// puede impedir que RemapH arranque, solo se loguea y se
// sigue con el resto.
// ======================================================

pub fn cargar_al_iniciar() {
    let overrides = match leer_overrides() {
        Ok(mapa) => mapa,

        Err(error) => {
            eprintln!(
                "⚠️ No se pudo leer Configuracion_Usuario.txt, se usan los valores de fábrica: {}",
                error
            );

            return;
        }
    };

    for (clave, valor) in overrides {
        if let Err(error) = aplicar_valor(&clave, &valor) {
            eprintln!(
                "⚠️ Override de configuración inválido, se ignora. Clave: \"{}\", valor: \"{}\". Detalle: {}",
                clave, valor, error
            );
        }
    }
}

// ======================================================
// ⌨️ TECLAS (Etapa 5) — prefijo "pulsador."
// ------------------------------------------------------
// El catálogo de claves válidas ("interno") lo posee
// pulsadores.rs, no este archivo — por eso se lo consulta
// acá en vez de tener un segundo catálogo propio, igual
// que config.rs es el dueño de los setters para General.
// ======================================================

const PREFIJO_PULSADOR: &str = "pulsador.";

// ======================================================
// 📥 LEER OVERRIDES (solo Teclas, prefijo "pulsador.")
// ------------------------------------------------------
// Devuelve el mapa con la clave ya sin el prefijo (interno
// → nombre personalizado), listo para que pulsadores.rs lo
// use directamente por interno.
// ======================================================

pub fn leer_overrides_pulsador() -> Result<HashMap<String, String>, String> {
    let mapa = leer_mapa_completo()?;

    Ok(mapa
        .into_iter()
        .filter_map(|(clave, valor)| {
            clave
                .strip_prefix(PREFIJO_PULSADOR)
                .map(|interno| (interno.to_string(), valor))
        })
        .collect())
}

// ======================================================
// 📦 GUARDAR LOTE — TECLAS (todo o nada)
// ------------------------------------------------------
// Misma mecánica que guardar_lote(), pero validando contra
// pulsadores::por_interno() en vez del catálogo de General,
// y sin aplicar_valor() (no hay setter: el override se lee
// en el momento desde pulsadores::nombre_ui_efectivo()).
// ======================================================

pub fn guardar_lote_pulsadores(cambios: &[(String, String)]) -> Result<(), Vec<(String, String)>> {
    let mut errores: Vec<(String, String)> = Vec::new();

    for (interno, valor) in cambios {
        if crate::pulsadores::por_interno(interno).is_none() {
            errores.push((
                interno.clone(),
                format!("Pulsador interno desconocido: \"{}\"", interno),
            ));

            continue;
        }

        if valor.trim().is_empty() {
            errores.push((
                interno.clone(),
                "El nombre no puede estar vacío".to_string(),
            ));
        }
    }

    if !errores.is_empty() {
        return Err(errores);
    }

    let mut mapa = leer_mapa_completo().map_err(|error| vec![(String::new(), error)])?;

    for (interno, valor) in cambios {
        mapa.insert(
            format!("{}{}", PREFIJO_PULSADOR, interno),
            valor.trim().to_string(),
        );
    }

    escribir_mapa_completo(&mapa).map_err(|error| vec![(String::new(), error)])
}
