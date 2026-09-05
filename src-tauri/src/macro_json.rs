// ======================================================
// 🧩 macro_json
// ------------------------------------------------------
// Modelo persistente de un archivo de Macro (carpeta de
// usuario /Macros, *.json). No es una RemapeoJson ni vive
// dentro de perfil_json — es un documento aparte que una
// fila con tipo == "macro" solo referencia por nombre/ruta
// (ver AccionCache::Macro en perfil_cache.rs).
//
// Espejo exacto de MacroArchivo / PasoMacro en
// core_macro.ts (TS) — mismo criterio de perfil_json.rs:
// struct plana con TODOS los campos de los 7 tipos de paso
// siempre presentes, con efecto solo según `tipo`, en vez de
// un enum taggeado. camelCase en el JSON vía
// #[serde(rename_all = "camelCase")], igual que
// CoordenadaJson.
// ======================================================

use crate::perfil_cache::CondicionTrigger;
use crate::perfil_json::TriggerJson;

// ======================================================
// 🗂️ ARCHIVO DE MACRO
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MacroArchivoJson {
    pub nombre: String,

    pub pasos: Vec<PasoMacroJson>,
}

impl MacroArchivoJson {
    pub fn nueva(nombre: String) -> Self {
        Self {
            nombre,
            pasos: Vec::new(),
        }
    }
}

// ======================================================
// 📄 PASO
// ------------------------------------------------------
// tipo: "tecla_mouse" | "espera" | "bucle" | "coordenada" |
//   "pegar" | "abrir" | "multimedia".
// marcador: letra de Marcador asignada a este paso (solo
//   posible en un paso anterior a un paso "bucle" existente),
//   None si no está marcado.
//
// tecla_accion / tecla_extra / tecla_duracion_ms: solo
//   cuando tipo == "tecla_mouse". Tras el rediseño de Extra,
//   Simple/Doble/Triple/Mantenido ya no son valores de
//   tecla_extra — se leen de tecla_accion.condicion (mismo
//   criterio que RemapeoJson.trigger/accion_trigger). tecla_
//   extra queda en "" (Ninguno) | "normal" | "turbo", sin
//   "repeticion_rueda" (no hay Rueda en una Macro).
//   tecla_duracion_ms simula lo que en una Macro no hay Up
//   físico real para calcular: cuánto dura el DOWN sostenido
//   (condicion == "mantenido" con Extra Ninguno) o cuánto dura
//   el bucle de repetición (tecla_extra != ""). None mientras
//   no se configuró.
//
// espera_ms: solo cuando tipo == "espera".
//
// bucle_marcador_destino / bucle_veces: solo cuando tipo ==
//   "bucle". Un solo algoritmo (sin distinción con_fin/
//   sin_fin, ver Etapa 8B): resta 1 en cada visita; al llegar
//   a 0, resetea al valor programado y sigue de largo — listo
//   para una próxima visita si está anidado dentro de otro
//   bucle (bucles anidados).
//
// coord_*: solo cuando tipo == "coordenada". Solo mueve el
//   mouse (sin click). coord_posicion_inicial es única y
//   excluyente: en true ignora el resto de estos campos y usa
//   la posición del mouse al inicio de la ejecución de la
//   macro. coord_nota/coord_aplicacion son copia informativa de
//   la CoordenadaBanco elegida (mismo criterio que CoordenadaJson
//   en perfil_json.rs) — no se usan al ejecutar, solo para el
//   resumen del editor. Sin post_accion (no aplica desacoplado de
//   una tecla, a diferencia de CoordenadaJson).
//
// pegar_ruta: solo cuando tipo == "pegar". Misma ruta sirve
//   para un fijado del Portapapeles, cualquier archivo del
//   disco soportado, o texto literal si no matchea ningún
//   archivo (back_portapapeles::contenido_desde_archivo_o_
//   texto() decide). Formatos de archivo soportados: cualquier
//   extensión de texto plano (ver EXTENSIONES_TEXTO en
//   back_portapapeles.rs) y .png.
//
// abrir_*: solo cuando tipo == "abrir". Mismos 5 campos que
//   AbrirAccionJson/AbrirExtraJson, aplanados acá.
//
// multimedia_comando / multimedia_alcance: solo cuando
//   tipo == "multimedia". "en_app" reusa el Filtro de App de
//   la fila Macro contenedora (no tiene programa propio acá).
//
// nota: texto plano independiente del tipo, no se envía al
//   ejecutar la macro (columna Nota del editor). "" cuando no
//   tiene nota.
// ======================================================

// [FIX] Bug "un paso trae un montón de líneas basura que no aplican
// para su tipo" (ej. un paso tecla_mouse con down/up de arrastre
// mostraba también coord_*/abrir_*/multimedia_*/etc., todos en su
// valor por defecto — ruido puro). La struct y su Serialize/
// Deserialize normales NO cambian (siguen siendo derive puro, con
// TODOS los campos siempre presentes) — este tipo también viaja tal
// cual por IPC hacia el editor (ver comandos.rs: macro_abrir,
// macro_leer, etc.), que espera el "espejo exacto" completo de
// core_macro.ts; tocar Serialize acá rompería eso. El recorte de
// campos "basura" se hace aparte, solo al ESCRIBIR a disco (ver
// macros.rs::guardar_en_disco -> macro_json::json_para_disco()) sobre
// un serde_json::Value ya serializado, sin tocar esta struct.
//
// Los campos que ahora pueden faltar en un archivo *leído* de disco
// (uno ya trimeado por una versión anterior de RemapH, o uno viejo
// donde no aplica) llevan #[serde(default)] (o
// #[serde(default = "...")] para tecla_accion, que no tiene Default
// propio) para no fallar el Deserialize si no están.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasoMacroJson {
    pub tipo: String,

    #[serde(default)]
    pub marcador: Option<String>,

    #[serde(default = "tecla_accion_por_defecto")]
    pub tecla_accion: TriggerJson,

    #[serde(default)]
    pub tecla_extra: String,

    #[serde(default)]
    pub tecla_duracion_ms: Option<u64>,

    // Etapa F: arrastre diferido. "down" retiene mods+gatillo abajo
    // hasta que llegue un paso "up" posterior con la misma
    // secuencia; "up" libera. None = comportamiento normal (sin
    // retención), sin cambios respecto a lo existente.
    #[serde(default)]
    pub tecla_retencion: Option<String>,

    #[serde(default)]
    pub espera_ms: u64,

    #[serde(default)]
    pub bucle_marcador_destino: Option<String>,

    #[serde(default)]
    pub bucle_veces: u32,

    #[serde(default)]
    pub coord_posicion_inicial: bool,

    #[serde(default)]
    pub coord_ubicacion: String,

    #[serde(default)]
    pub coord_modo_ventana: String,

    #[serde(default)]
    pub coord_punto_referencia: String,

    #[serde(default)]
    pub coord_x: Option<f64>,

    #[serde(default)]
    pub coord_y: Option<f64>,

    #[serde(default)]
    pub coord_nota: String,

    #[serde(default)]
    pub coord_aplicacion: String,

    #[serde(default)]
    pub pegar_ruta: Option<String>,

    #[serde(default)]
    pub abrir_ruta: Option<String>,

    #[serde(default)]
    pub abrir_iniciar: String,

    #[serde(default)]
    pub abrir_instancias: String,

    #[serde(default)]
    pub abrir_con: Option<String>,

    #[serde(default)]
    pub abrir_argumento: String,

    #[serde(default)]
    pub multimedia_comando: Option<String>,

    #[serde(default)]
    pub multimedia_alcance: String,

    #[serde(default)]
    pub nota: String,
}

fn tecla_accion_por_defecto() -> TriggerJson {
    TriggerJson {
        modificadores: Vec::new(),
        gatillo: None,
        condicion: CondicionTrigger::Simple,
    }
}

// ======================================================
// 🧹 JSON PARA DISCO — solo los campos del `tipo` de cada paso
// ------------------------------------------------------
// Usado únicamente por macros::guardar_en_disco(), nunca por IPC (ver
// nota del FIX arriba). Serializa normal (serde_json::to_value, todos
// los campos) y después, por cada paso del array "pasos", borra las
// claves que no apliquen a su "tipo" — camelCase a mano, uno a uno,
// porque acá no hay struct de la cual derivar: son claves sueltas
// dentro de un serde_json::Value ya armado.
// ======================================================

const CAMPOS_TECLA_MOUSE: &[&str] =
    &["teclaAccion", "teclaExtra", "teclaDuracionMs", "teclaRetencion"];
const CAMPOS_ESPERA: &[&str] = &["esperaMs"];
const CAMPOS_BUCLE: &[&str] = &["bucleMarcadorDestino", "bucleVeces"];
const CAMPOS_COORDENADA: &[&str] = &[
    "coordPosicionInicial",
    "coordUbicacion",
    "coordModoVentana",
    "coordPuntoReferencia",
    "coordX",
    "coordY",
    "coordNota",
    "coordAplicacion",
];
const CAMPOS_PEGAR: &[&str] = &["pegarRuta"];
const CAMPOS_ABRIR: &[&str] = &[
    "abrirRuta",
    "abrirIniciar",
    "abrirInstancias",
    "abrirCon",
    "abrirArgumento",
];
const CAMPOS_MULTIMEDIA: &[&str] = &["multimediaComando", "multimediaAlcance"];

const TODOS_LOS_CAMPOS_POR_TIPO: &[&[&str]] = &[
    CAMPOS_TECLA_MOUSE,
    CAMPOS_ESPERA,
    CAMPOS_BUCLE,
    CAMPOS_COORDENADA,
    CAMPOS_PEGAR,
    CAMPOS_ABRIR,
    CAMPOS_MULTIMEDIA,
];

fn campos_relevantes(tipo: &str) -> &'static [&'static str] {
    match tipo {
        "tecla_mouse" => CAMPOS_TECLA_MOUSE,
        "espera" => CAMPOS_ESPERA,
        "bucle" => CAMPOS_BUCLE,
        "coordenada" => CAMPOS_COORDENADA,
        "pegar" => CAMPOS_PEGAR,
        "abrir" => CAMPOS_ABRIR,
        "multimedia" => CAMPOS_MULTIMEDIA,
        _ => &[],
    }
}

pub fn json_para_disco(macro_archivo: &MacroArchivoJson) -> Result<String, String> {
    let mut valor = serde_json::to_value(macro_archivo).map_err(|error| error.to_string())?;

    if let Some(pasos) = valor.get_mut("pasos").and_then(|p| p.as_array_mut()) {
        for paso in pasos.iter_mut() {
            let Some(objeto) = paso.as_object_mut() else {
                continue;
            };

            let tipo = objeto
                .get("tipo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let relevantes = campos_relevantes(&tipo);

            for grupo in TODOS_LOS_CAMPOS_POR_TIPO {
                for campo in grupo.iter() {
                    if !relevantes.contains(campo) {
                        objeto.remove(*campo);
                    }
                }
            }
        }
    }

    serde_json::to_string_pretty(&valor).map_err(|error| error.to_string())
}
