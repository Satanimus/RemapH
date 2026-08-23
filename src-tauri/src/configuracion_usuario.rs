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
//                    styl_variables.css).
// • "pulsador."   → Teclas (nombre visible de
//                    pulsadores.tsv).
// • "dispositivo." → NO es una pestaña de la Ventana de
//                    Configuración — es el teclado/mouse
//                    primario que back_interception.rs
//                    aprende de un evento físico real (ver
//                    ese archivo). Se guarda acá para
//                    reusar el mismo archivo/mecanismo de
//                    persistencia, no porque sea editable
//                    por el usuario. Por eso no tiene
//                    catálogo de fábrica ni validación de
//                    tipo — son dos claves fijas
//                    ("dispositivo.teclado"/"dispositivo.mouse"),
//                    leídas/escritas directo como número.
//
// Este archivo conoce las 3 secciones de UI + esta cuarta
// clave interna. Para General carga el
// catálogo de fábrica (configuracion.tsv), aplica overrides
// sobre config.rs y persiste cambios. Para Teclas valida
// contra pulsadores::por_interno() (el catálogo lo posee
// pulsadores.rs, no este archivo) y persiste, pero no
// aplica nada en caliente — pulsadores.rs lee el override
// en el momento (nombre_ui_efectivo()). Para Apariencia
// carga su propio catálogo de fábrica (apariencia.tsv) y
// valida contra él, pero tampoco aplica nada en caliente
// acá: no hay setter Rust, el frontend (core_apariencia.ts)
// pide los overrides ya guardados y los inyecta como estilo
// inline al arrancar cada ventana; guardar un cambio (o
// cargar un tema) hace que comandos.rs recargue esas
// ventanas para que vuelvan a pedirlos.
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
//
// cargar_catalogo_css()
//     Carga (una sola vez) apariencia.tsv.
//
// leer_overrides_css()
//     Devuelve solo los overrides con prefijo "css."
//     (Apariencia), con la clave ya sin el prefijo (variable
//     CSS → valor personalizado).
//
// guardar_lote_css(cambios)
//     Igual que guardar_lote_pulsadores() pero para
//     Apariencia: valida cada clave contra apariencia.tsv y
//     cada valor según su tipo (color "#RRGGBB" o pixeles
//     "Npx"); no hay setter que aplicar en caliente.
//
// exportar_tema(ruta) / importar_tema(ruta)
//     Vuelcan/leen los overrides de Apariencia como archivo
//     .theme (mismo formato "clave=valor", sin el prefijo
//     "css."). importar_tema reusa guardar_lote_css(), así
//     que también es todo o nada.
//
// leer_dispositivo_teclado() / leer_dispositivo_mouse()
//     Devuelven el número de dispositivo (Device = i32 en
//     Interception) guardado la sesión anterior, si hay
//     uno ("dispositivo.teclado"/"dispositivo.mouse"). None
//     si nunca se guardó (primera vez que corre el programa).
//
// guardar_dispositivo_teclado(device) / guardar_dispositivo_mouse(device)
//     Persisten el número de dispositivo confirmado por un
//     evento físico real (ver back_interception::registrar_
//     teclado/registrar_mouse). Sin catálogo ni validación —
//     a diferencia de guardar_lote()/guardar_lote_pulsadores()/
//     guardar_lote_css(), esto no viene de un formulario de la
//     Ventana de Configuración, así que no hace falta el
//     mecanismo de "todo o nada" de esos tres.
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
         # Prefijo pulsador. → nombre visible de pulsadores.tsv.\n\
         # Prefijo dispositivo. → teclado/mouse primario aprendido por\n\
         #   back_interception.rs (no editable desde la Ventana de\n\
         #   Configuración).\n\n",
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
// 🖱️⌨️ DISPOSITIVO PRIMARIO (teclado/mouse)
// ------------------------------------------------------
// Ver decisión en la sección 1 del header: reusa el mismo
// archivo/mecanismo (leer_mapa_completo/escribir_mapa_completo)
// que el resto de este módulo, con el prefijo "dispositivo.",
// pero sin catálogo ni pestaña de UI — son dos claves fijas.
// ======================================================

const PREFIJO_DISPOSITIVO: &str = "dispositivo.";

fn leer_dispositivo(clave: &str) -> Result<Option<i32>, String> {
    let mapa = leer_mapa_completo()?;

    Ok(mapa
        .get(clave)
        .and_then(|valor| valor.trim().parse::<i32>().ok()))
}

fn guardar_dispositivo(clave: &str, device: i32) -> Result<(), String> {
    let mut mapa = leer_mapa_completo()?;

    mapa.insert(clave.to_string(), device.to_string());

    escribir_mapa_completo(&mapa)
}

pub fn leer_dispositivo_teclado() -> Result<Option<i32>, String> {
    leer_dispositivo(&format!("{}teclado", PREFIJO_DISPOSITIVO))
}

pub fn leer_dispositivo_mouse() -> Result<Option<i32>, String> {
    leer_dispositivo(&format!("{}mouse", PREFIJO_DISPOSITIVO))
}

pub fn guardar_dispositivo_teclado(device: i32) -> Result<(), String> {
    guardar_dispositivo(&format!("{}teclado", PREFIJO_DISPOSITIVO), device)
}

pub fn guardar_dispositivo_mouse(device: i32) -> Result<(), String> {
    guardar_dispositivo(&format!("{}mouse", PREFIJO_DISPOSITIVO), device)
}

// ======================================================
// 🔀 MODO DE MOTOR
// ======================================================

const CLAVE_MODO_MOTOR: &str = "motor.modo";

pub fn guardar_modo_motor(modo: &str) -> Result<(), String> {
    let mut mapa = leer_mapa_completo()?;

    mapa.insert(CLAVE_MODO_MOTOR.to_string(), modo.to_string());

    escribir_mapa_completo(&mapa)
}

pub fn leer_modo_motor() -> Result<Option<String>, String> {
    let mapa = leer_mapa_completo()?;

    Ok(mapa.get(CLAVE_MODO_MOTOR).map(|v| v.trim().to_string()))
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

        "tiempo_inactividad_captura" => {
            config::establecer_tiempo_inactividad_captura(parsear_numero(valor)?)
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

        "tiempo_simple_teclas" => config::establecer_tiempo_simple_teclas(parsear_numero(valor)?),

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

// ======================================================
// 🎨 APARIENCIA (Etapa 6) — prefijo "css."
// ------------------------------------------------------
// Mismo espíritu que el catálogo de General (cargar_catalogo/
// EntradaCatalogo), pero para las variables de
// styl_variables.css: tipo propio (color/pixeles, no numero/
// numero_par/texto) y ningún setter de config.rs que aplicar
// — el valor vive únicamente en Configuracion_Usuario.txt y
// lo consume el frontend (ver comandos.rs, sección Apariencia,
// y core_apariencia.ts).
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum TipoValorCss {
    Color,
    Pixeles,
    Texto,
}

#[derive(Clone, Debug)]
pub struct EntradaCatalogoCss {
    pub clave: String,

    pub nombre_ui: String,

    pub valor_defecto: String,

    pub tipo: TipoValorCss,
}

static CATALOGO_CSS: OnceLock<Vec<EntradaCatalogoCss>> = OnceLock::new();

pub fn cargar_catalogo_css() -> &'static Vec<EntradaCatalogoCss> {
    CATALOGO_CSS.get_or_init(|| {
        let texto = include_str!("apariencia.tsv");

        let mut catalogo: Vec<EntradaCatalogoCss> = Vec::new();

        for (numero_linea, linea) in texto.lines().enumerate() {
            let linea = linea.trim();

            if linea.is_empty() || linea.starts_with('#') {
                continue;
            }

            let columnas: Vec<&str> = linea.split('\t').collect();

            if columnas.len() != 4 {
                panic!(
                    "❌ Error interno en apariencia.tsv. Línea {}",
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
                    "❌ Entrada de apariencia sin clave. Línea {}",
                    numero_linea + 1
                );
            }

            if clave.contains('.') {
                panic!(
                    "❌ Clave de apariencia.tsv no puede contener un punto (reservado para el prefijo css.): \"{}\"",
                    clave
                );
            }

            let tipo = match tipo_texto {
                "color" => TipoValorCss::Color,
                "pixeles" => TipoValorCss::Pixeles,
                "texto" => TipoValorCss::Texto,
                _ => panic!(
                    "❌ Tipo desconocido \"{}\" en apariencia.tsv. Línea {}",
                    tipo_texto,
                    numero_linea + 1
                ),
            };

            if catalogo.iter().any(|entrada: &EntradaCatalogoCss| entrada.clave == clave) {
                panic!("❌ Clave duplicada en apariencia.tsv: {}", clave);
            }

            catalogo.push(EntradaCatalogoCss {
                clave: clave.to_string(),

                nombre_ui: nombre_ui.to_string(),

                valor_defecto: valor_defecto.to_string(),

                tipo,
            });
        }

        catalogo
    })
}

const PREFIJO_CSS: &str = "css.";

// ======================================================
// 📥 LEER OVERRIDES (solo Apariencia, prefijo "css.")
// ======================================================

pub fn leer_overrides_css() -> Result<HashMap<String, String>, String> {
    let mapa = leer_mapa_completo()?;

    Ok(mapa
        .into_iter()
        .filter_map(|(clave, valor)| {
            clave
                .strip_prefix(PREFIJO_CSS)
                .map(|variable| (variable.to_string(), valor))
        })
        .collect())
}

// ======================================================
// ✅ VALIDAR SEGÚN TIPO (Apariencia, sin aplicar)
// ======================================================

fn validar_css_segun_tipo(tipo: &TipoValorCss, valor: &str) -> Result<(), String> {
    match tipo {
        TipoValorCss::Color => {
            let valor = valor.trim();

            let valido = valor.len() == 7
                && valor.starts_with('#')
                && valor[1..]
                    .chars()
                    .all(|caracter| caracter.is_ascii_hexdigit());

            if valido {
                Ok(())
            } else {
                Err(format!(
                    "Color inválido, debe tener el formato #RRGGBB: \"{}\"",
                    valor
                ))
            }
        }

        TipoValorCss::Pixeles => {
            let valor = valor.trim();

            let Some(numero) = valor.strip_suffix("px") else {
                return Err(format!(
                    "Debe ser un tamaño en píxeles, ej. \"16px\": \"{}\"",
                    valor
                ));
            };

            numero
                .parse::<u64>()
                .map(|_| ())
                .map_err(|_| format!("Debe ser un tamaño en píxeles, ej. \"16px\": \"{}\"", valor))
        }

        TipoValorCss::Texto => {
            if valor.trim().is_empty() {
                Err("El valor no puede estar vacío".to_string())
            } else {
                Ok(())
            }
        }
    }
}

// ======================================================
// 📦 GUARDAR LOTE — APARIENCIA (todo o nada)
// ------------------------------------------------------
// Misma mecánica que guardar_lote()/guardar_lote_pulsadores():
// valida TODO el lote primero contra apariencia.tsv; si algo
// falla no aplica ni persiste nada. No hay aplicar_valor():
// no existe setter de Rust para variables CSS, el override
// solo se persiste — quien lo "aplica" es el frontend, al leer
// obtener_overrides_apariencia() en el arranque de cada
// ventana (ver comandos.rs).
// ======================================================

pub fn guardar_lote_css(cambios: &[(String, String)]) -> Result<(), Vec<(String, String)>> {
    let catalogo = cargar_catalogo_css();

    let mut errores: Vec<(String, String)> = Vec::new();

    for (clave, valor) in cambios {
        match catalogo.iter().find(|entrada| &entrada.clave == clave) {
            None => errores.push((
                clave.clone(),
                format!("Variable CSS desconocida: \"{}\"", clave),
            )),

            Some(entrada) => {
                if let Err(mensaje) = validar_css_segun_tipo(&entrada.tipo, valor) {
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
        mapa.insert(
            format!("{}{}", PREFIJO_CSS, clave),
            valor.trim().to_string(),
        );
    }

    escribir_mapa_completo(&mapa).map_err(|error| vec![(String::new(), error)])
}

// ======================================================
// 🖼️ TEMAS (.theme) — exportar/importar overrides de Apariencia
// ------------------------------------------------------
// Formato idéntico a Configuracion_Usuario.txt ("clave=valor",
// comentarios con #) pero SOLO con las variables CSS y SIN el
// prefijo "css." — un archivo de tema es portable entre
// instalaciones de RemapH, no debe depender del formato interno
// del archivo de usuario.
// ======================================================

pub fn exportar_tema(ruta: &std::path::Path) -> Result<(), String> {
    let overrides = leer_overrides_css()?;

    // Se exporta el valor EFECTIVO de cada variable del catálogo
    // (override si existe, si no el de fábrica) — no solo las
    // claves con override guardado. Si no fuera así, guardar un
    // tema sin haber tocado nada en esta sesión produciría un
    // archivo .theme vacío, aunque en pantalla se vea una paleta
    // completa (la de fábrica).
    let catalogo = cargar_catalogo_css();

    let mut contenido = String::from(
        "# Tema de Apariencia — RemapH.\n\
         # Generado desde la Ventana de Configuración (pestaña Apariencia).\n\
         # Formato: variable=valor (una línea por variable CSS, sin el\n\
         # prefijo \"css.\" que usa Configuracion_Usuario.txt).\n\n",
    );

    for entrada in catalogo {
        let valor_efectivo = overrides
            .get(&entrada.clave)
            .unwrap_or(&entrada.valor_defecto);

        contenido.push_str(&format!("{}={}\n", entrada.clave, valor_efectivo));
    }

    fs::write(ruta, contenido).map_err(|error| error.to_string())
}

pub fn importar_tema(ruta: &std::path::Path) -> Result<(), Vec<(String, String)>> {
    let texto = fs::read_to_string(ruta).map_err(|error| {
        vec![(
            String::new(),
            format!("No se pudo leer el archivo de tema: {}", error),
        )]
    })?;

    let mut cambios: Vec<(String, String)> = Vec::new();

    for linea in texto.lines() {
        let linea = linea.trim();

        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }

        let Some((clave, valor)) = linea.split_once('=') else {
            continue;
        };

        cambios.push((clave.trim().to_string(), valor.trim().to_string()));
    }

    if cambios.is_empty() {
        return Err(vec![(
            String::new(),
            "El archivo de tema no contiene variables válidas".to_string(),
        )]);
    }

    guardar_lote_css(&cambios)
}
