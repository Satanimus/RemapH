// ======================================================
// 🎛️ Pulsadores
// ======================================================
// ETAPA 0 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Mantiene el diccionario único de todos
// los pulsadores compatibles con RemapH.
//
// Traduce entre:
//
// • Nativo.
// • Interno.
// • Interception.
// • UI.
//
// También identifica el tipo de dispositivo
// al que pertenece cada pulsador.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Carga pulsadores.tsv.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Cualquier módulo que necesite traducir
// nombres de pulsadores.
//
// Principalmente:
//
// • Backends.
// • Runtime.
// • Compilador.
// • UI.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve:
//
// • Pulsador.
// • Conversiones entre formatos.
// • Fuente del dispositivo.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// cargar()
//     Carga el diccionario.
//
// por_nativo()
//     Busca un pulsador por código nativo.
//
// por_interno()
//     Busca un pulsador por nombre interno.
//
// por_interception()
//     Busca un pulsador por nombre
//     Interception.
//
// por_ui()
//     Busca un pulsador por nombre visible.
//
// por_scancode()
//     Busca un pulsador por scan code crudo Set 1 +
//     es_extendida (columnas "scancode"/"extendida" — ver
//     comentario de esas columnas en pulsadores.tsv). Usado
//     por Modo Portable (back_windows.rs) para resolver la
//     entrada SIN pasar por "nativo"/VK, que resultó no ser
//     confiable para teclas OEM en layouts no-US. Si
//     es_extendida=true y no hay fila con extendida=1 para
//     ese scancode, cae a la fila normal (extendida=0) con
//     el mismo scancode — mismo criterio de fallback que
//     nombre_interception() en back_teclas.rs.
//
// todos()
//     Devuelve el diccionario completo.
//
// nativo_a_interno()
// interno_a_nativo()
// interno_a_interception()
// interno_a_fuente()
// interno_a_ui()
// interception_a_interno()
// ui_a_interno()
// scancode_a_interno()
//     Conversiones entre formatos.
//
// ui_desde_interno()
//     Devuelve el nombre visible.
//
// nombre_ui_efectivo()
//     Devuelve el nombre visible EFECTIVO: el override de
//     usuario (Etapa 5 de la Ventana de Configuración,
//     pestaña Teclas) si existe, si no el de fábrica.
// ------------------------------------------------------
// Transformación:
//
// fuente
//      ↓
// nativo
//      ↓
// Pulsadores
//      ↓
// interno
//      ↓
// interception
//
// o
//
// ui
// ======================================================

use std::sync::OnceLock;

use crate::configuracion_usuario;

// ======================================================
// 📦 MODELO PULSADOR
// ======================================================

#[derive(Clone, Debug)]
pub struct Pulsador {
    pub fuente: String,

    pub nativo: String,

    pub interno: String,

    pub interception: String,

    pub ui: String,

    // Set 1 crudo (ver columna "scancode" de pulsadores.tsv). None
    // para fuente=mouse, que no tiene fila de 7 columnas (no aplica).
    pub scancode: Option<u16>,

    // Ver columna "extendida" de pulsadores.tsv. false por defecto
    // para las filas de 5 columnas (mouse), donde no aplica.
    pub extendida: bool,
}

// ======================================================
// 🗂️ DICCIONARIO
// ======================================================

static PULSADORES: OnceLock<Vec<Pulsador>> = OnceLock::new();

// ======================================================
// 📖 CARGAR DICCIONARIO
// ======================================================

fn cargar() -> &'static Vec<Pulsador> {
    PULSADORES.get_or_init(|| {
        let texto = include_str!("pulsadores.tsv");

        let mut pulsadores: Vec<Pulsador> = Vec::new();

        for (numero_linea, linea) in texto.lines().enumerate() {
            let linea = linea.trim();

            if linea.is_empty() || linea.starts_with('#') {
                continue;
            }

            let columnas: Vec<&str> = linea.split('\t').collect();

            // 5 columnas: fila "mouse" (scancode/extendida no aplican,
            // no tienen scan code Set 1 de teclado). 7 columnas: fila
            // "keyboard", con scancode/extendida al final. Cualquier
            // otro largo es un error real de formato.
            if columnas.len() != 5 && columnas.len() != 7 {
                panic!(
                    "❌ Error interno en pulsadores.tsv. Línea {}",
                    numero_linea + 1
                );
            }

            // Fila de encabezado ("fuente  nativo  interno  interception  ui
            // scancode  extendida"). No empieza con '#' (así se ve en un
            // editor tabular como columnas reales), así que hay que
            // descartarla explícitamente por contenido — igual que
            // configuracion.tsv/apariencia.tsv descartan su fila
            // "clave  nombre_ui  ...". Antes se hacía por número de línea
            // (`numero_linea == 0`), pero esa cuenta incluye los
            // comentarios de arriba del archivo, así que el salto caía en
            // la primera línea de comentario y no en el encabezado real:
            // el header terminaba procesado como un pulsador más (interno
            // "interno", ui "ui").
            if columnas[0].trim() == "fuente" {
                continue;
            }

            let fuente = columnas[0].trim();

            let nativo = columnas[1].trim();

            let interno = columnas[2].trim();

            let interception = columnas[3].trim();

            let ui = columnas[4].trim();

            // scancode/extendida solo existen en filas de 7 columnas.
            // Un scancode vacío en una fila de 7 columnas se trata como
            // "no tiene" (None) en vez de forzar el parseo hex — por
            // robustez, no porque hoy exista ese caso en el archivo.
            let (scancode, extendida) = if columnas.len() == 7 {
                let scancode_txt = columnas[5].trim();

                let scancode = if scancode_txt.is_empty() {
                    None
                } else {
                    Some(
                        u16::from_str_radix(scancode_txt.trim_start_matches("0x"), 16)
                            .unwrap_or_else(|_| {
                                panic!(
                                    "❌ Scancode inválido \"{}\". Línea {}",
                                    scancode_txt,
                                    numero_linea + 1
                                )
                            }),
                    )
                };

                let extendida = columnas[6].trim() == "1";

                (scancode, extendida)
            } else {
                (None, false)
            };

            if fuente.is_empty() {
                panic!("❌ Pulsador sin fuente. Línea {}", numero_linea + 1);
            }

            if interno.is_empty() {
                panic!("❌ Pulsador sin interno. Línea {}", numero_linea + 1);
            }

            if pulsadores.iter().any(|p: &Pulsador| p.interno == interno) {
                panic!("❌ Interno duplicado: {}", interno);
            }

            if !nativo.is_empty() && pulsadores.iter().any(|p: &Pulsador| p.nativo == nativo) {
                panic!("❌ Nativo duplicado: {}", nativo);
            }

            // scancode SÍ se repite a propósito (ej. 0x47 en NumPad7 y en
            // Home) — lo que debe ser único es el PAR (scancode,
            // extendida), no el scancode solo.
            if let Some(codigo) = scancode {
                if pulsadores
                    .iter()
                    .any(|p: &Pulsador| p.scancode == Some(codigo) && p.extendida == extendida)
                {
                    panic!(
                        "❌ Scancode duplicado: {:#04X} (extendida={}). Línea {}",
                        codigo,
                        extendida,
                        numero_linea + 1
                    );
                }
            }

            pulsadores.push(Pulsador {
                fuente: fuente.to_string(),

                nativo: nativo.to_string(),

                interno: interno.to_string(),

                interception: interception.to_string(),

                ui: ui.to_string(),

                scancode,

                extendida,
            });
        }

        pulsadores
    })
}

// ======================================================
// 🔍 BUSCAR POR NATIVO
// ======================================================

pub fn por_nativo(nativo: &str) -> Option<&'static Pulsador> {
    cargar().iter().find(|pulsador| pulsador.nativo == nativo)
}

// ======================================================
// 🔍 BUSCAR POR INTERNO
// ======================================================

pub fn por_interno(interno: &str) -> Option<&'static Pulsador> {
    cargar().iter().find(|pulsador| pulsador.interno == interno)
}

// ======================================================
// 🔍 BUSCAR POR INTERCEPTION
// ======================================================

pub fn por_interception(interception: &str) -> Option<&'static Pulsador> {
    cargar()
        .iter()
        .find(|pulsador| pulsador.interception == interception)
}

// ======================================================
// 🔍 BUSCAR POR UI
// ======================================================

pub fn por_ui(ui: &str) -> Option<&'static Pulsador> {
    cargar().iter().find(|pulsador| pulsador.ui == ui)
}

// ======================================================
// 🔍 BUSCAR POR SCANCODE
// ------------------------------------------------------
// Ver nota completa en la sección 5 del encabezado del
// archivo. Si extendida=true no encuentra fila con
// extendida=1 para ese scancode, cae a la fila normal
// (extendida=0) con el mismo scancode — mismo fallback que
// nombre_interception() en back_teclas.rs, para las teclas
// extendidas que no tienen fila propia (ej. Enter de numpad).
// ======================================================

pub fn por_scancode(scancode: u16, extendida: bool) -> Option<&'static Pulsador> {
    let todos = cargar();

    if extendida {
        if let Some(pulsador) = todos
            .iter()
            .find(|p| p.extendida && p.scancode == Some(scancode))
        {
            return Some(pulsador);
        }
    }

    todos
        .iter()
        .find(|p| !p.extendida && p.scancode == Some(scancode))
}

// ======================================================
// 📋 TODOS
// ======================================================

pub fn todos() -> &'static [Pulsador] {
    cargar()
}

// ======================================================
// 🔄 CONVERSIONES
// ======================================================

pub fn interno_a_interception(interno: &str) -> Option<&'static str> {
    por_interno(interno).map(|p| p.interception.as_str())
}

pub fn interception_a_interno(interception: &str) -> Option<&'static str> {
    por_interception(interception).map(|p| p.interno.as_str())
}

pub fn scancode_a_interno(scancode: u16, extendida: bool) -> Option<&'static str> {
    por_scancode(scancode, extendida).map(|p| p.interno.as_str())
}

// ======================================================
// 🔤 INTERNO → UI
// ======================================================

pub fn ui_desde_interno(nombre: &str) -> String {
    por_interno(nombre)
        .map(|p| p.ui.to_string())
        .unwrap_or_else(|| nombre.to_string())
}

// ======================================================
// 🎨 NOMBRE UI EFECTIVO (override de usuario o fábrica)
// ------------------------------------------------------
// Consulta el override de Configuracion_Usuario.txt
// (prefijo "pulsador.", ver configuracion_usuario.rs)
// antes de caer al nombre de fábrica. Un override vacío
// (no debería llegar a persistirse así, pero por robustez)
// también cae a fábrica. Si no se puede leer el archivo de
// overrides, cae a fábrica sin propagar el error — traducir
// nombres de teclas nunca puede romper el resto de la app.
// ======================================================

pub fn nombre_ui_efectivo(interno: &str) -> String {
    if let Ok(overrides) = configuracion_usuario::leer_overrides_pulsador() {
        if let Some(nombre) = overrides.get(interno) {
            if !nombre.trim().is_empty() {
                return nombre.clone();
            }
        }
    }

    ui_desde_interno(interno)
}

// ======================================================
// 🌐 TRADUCCIÓN GENÉRICA POR COLUMNA
// ------------------------------------------------------
// Único punto de entrada para traducir entre columnas del
// diccionario desde afuera (comandos.rs → UI). Pensado para
// que la UI nunca tenga que conocer la estructura de
// pulsadores.tsv, solo pedir "de esta columna a esta otra".
//
// La columna "usuario" (nombre personalizado del usuario)
// no es un campo propio de Pulsador: como destino, resuelve
// a nombre_ui_efectivo() (override de Configuracion_Usuario.txt
// si existe, si no cae al "ui" de fábrica — ver Etapa 5 de la
// Ventana de Configuración). Como origen se sigue tratando
// igual que "interno" (no hay necesidad de buscar por nombre
// personalizado hasta ahora).
// ======================================================

pub fn traducir(valor: &str, origen: &str, destino: &str) -> Option<String> {
    let pulsador = match origen {
        "nativo" => por_nativo(valor),
        "interno" => por_interno(valor),
        "interception" => por_interception(valor),
        "ui" => por_ui(valor),
        "usuario" => por_interno(valor),
        _ => None,
    }?;

    let resultado = match destino {
        "nativo" => pulsador.nativo.clone(),
        "interno" => pulsador.interno.clone(),
        "interception" => pulsador.interception.clone(),
        "ui" => pulsador.ui.clone(),
        "usuario" => nombre_ui_efectivo(&pulsador.interno),
        _ => return None,
    };

    Some(resultado)
}

// ======================================================
// 🌐 TRADUCCIÓN EN LOTE
// ------------------------------------------------------
// Misma traducción que traducir(), pero para varios valores
// en una sola pasada — evita que la UI tenga que hacer un
// round-trip a Tauri por cada tecla al reconstruir un perfil
// completo. Los valores que no matchean ningún pulsador
// simplemente no aparecen en el mapa devuelto (quien llama
// decide el fallback, típicamente el propio valor original).
// ======================================================

pub fn traducir_lote(
    valores: &[String],
    origen: &str,
    destino: &str,
) -> std::collections::HashMap<String, String> {
    let mut mapa = std::collections::HashMap::new();

    for valor in valores {
        if let Some(traducido) = traducir(valor, origen, destino) {
            mapa.insert(valor.clone(), traducido);
        }
    }

    mapa
}
