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

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasoMacroJson {
    pub tipo: String,

    pub marcador: Option<String>,

    pub tecla_accion: TriggerJson,

    pub tecla_extra: String,

    pub tecla_duracion_ms: Option<u64>,

    pub espera_ms: u64,

    pub bucle_marcador_destino: Option<String>,

    pub bucle_veces: u32,

    pub coord_posicion_inicial: bool,

    pub coord_ubicacion: String,

    pub coord_modo_ventana: String,

    pub coord_punto_referencia: String,

    pub coord_x: Option<f64>,

    pub coord_y: Option<f64>,

    pub coord_nota: String,

    pub coord_aplicacion: String,

    pub pegar_ruta: Option<String>,

    pub abrir_ruta: Option<String>,

    pub abrir_iniciar: String,

    pub abrir_instancias: String,

    pub abrir_con: Option<String>,

    pub abrir_argumento: String,

    pub multimedia_comando: Option<String>,

    pub multimedia_alcance: String,

    pub nota: String,
}
