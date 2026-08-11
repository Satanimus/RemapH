// ======================================================
// 🖥️ Perfil UI
// ======================================================
//
// Modelos de comunicación entre:
//
// TypeScript
//      ↓
// perfil_ui
//      ↓
// comandos
//      ↓
// perfil
//
// Responsabilidad:
//
// - Recibir estructuras enviadas por UI.
// - Convertir datos de tabla.
// - Entregar estructuras serializables.
//
// PerfilUI NO:
//
// - Guarda archivos.
// - Compila perfiles.
// - Modifica cache.
// - Ejecuta acciones.
//
// EXCEPCIÓN: sí guarda el estado transitorio de una
// captura en curso (CapturaEnCurso / RESULTADO_CAPTURA) —
// es el destino de AnalizadorTrigger en modo Captura. No
// toca el perfil guardado ni la cache; solo arma el
// TriggerCapturaUI final y lo deja listo para que la UI
// lo retire.
//
// ======================================================
//
// Estructuras:
//
// AppUI
//     Datos de columna App.
//
// FilaUI
//     Representación completa de una fila.
//
// TriggerUI
//     Trigger recibido desde interfaz.
//
// EntradaUI
//     Entrada individual desde interfaz.
//
// EntradaCapturaUI
//     Entrada preparada para mostrar captura.
//
// TriggerCapturaUI
//     Trigger preparado para mostrar captura.
//
// ResultadoPerfil
//     Resultado completo para actualizar UI.
//
// EstadoCachePerfil
//     Estado de cache de un perfil.
//
//
//
// Funciones:
//
// convertir_perfil()
//     Convierte filas UI a perfil_json.
//
// convertir_fila()
//     Convierte una fila UI.
//
// convertir_app()
//     Convierte configuración App.
//
// convertir_trigger()
//     Convierte trigger UI.
//
// convertir_condicion()
//     Convierte condición UI (String) a CondicionTrigger.
//
// convertir_entrada()
//     Convierte entrada UI.
//
// convertir_fuente()
//     Convierte tipo UI a fuente interna.
//
// convertir_input_captura()
//     Convierte InputId para mostrar en UI.
//
// convertir_trigger_captura()
//     Convierte trigger capturado.
//
// iniciar_captura()
//     Abre una captura nueva: fila/columna destino, secuencia vacía.
//     Activa el modo Captura en cache.rs (a partir de acá, entrada.rs
//     empieza a consumir todo hacia este archivo).
//
// recibir_down()
//     Agrega un Down a la secuencia en curso (llamada por
//     cache.rs, modo Captura).
//
// recibir_condicion()
//     Arma el TriggerCapturaUI final con la condición recibida y
//     cierra la captura (llamada por cache.rs). Si columna
//     es "Trigger" y el resultado es únicamente Click izquierdo sin
//     ningún modificador (remapearía TODO clic izquierdo de
//     Windows), se descarta: el resultado queda con el trigger en
//     None en vez de Some(...) — es una señal distinta de "todavía
//     nada" (ver obtener_captura), para que la UI pueda resetear el
//     botón a "Capturar" con feedback en vez de quedarse esperando
//     en silencio.
//
// obtener_captura()
//     La UI la consulta (polling) para retirar el resultado
//     cuando ya esté listo.
// ======================================================

use serde::{Deserialize, Serialize};

use crate::perfil_json::{
    perfil_json, AppJson, CoordenadaJson, MenuAccionJson, MenuExpressExtraJson,
    PortapapelesAccionJson, PortapapelesExtraJson, RemapeoJson, TriggerJson,
};

// ======================================================
// 🖥️ APP UI
// ======================================================

#[derive(Deserialize)]
pub struct AppUI {
    pub programa: Option<String>,

    #[serde(rename = "segundoPlano")]
    pub segundo_plano: bool,
}

// ======================================================
// 📄 FILA UI
// ======================================================

#[derive(Deserialize)]
pub struct FilaUI {
    pub id: String,

    pub estado: String,

    pub app: AppUI,

    pub trigger: TriggerUI,

    pub tipo: String,

    pub accion: Option<TriggerUI>,

    // Referencia de Acción para tipos que no son tecla_mouse (hoy:
    // Multimedia — el comando elegido: "volumen_subir", etc.). Sigue
    // siendo None para el resto de los tipos (macro/archivo/ui
    // todavía no conectados desde la UI).
    #[serde(rename = "accionReferencia", default)]
    pub accion_referencia: Option<String>,

    pub extra: String,

    // Alcance de la Acción Multimedia ("global" | "en_app"). Ver
    // nota en perfil_json.rs — campo propio, no reusa `extra`.
    #[serde(rename = "extraMultimedia", default = "extra_multimedia_default")]
    pub extra_multimedia: String,

    pub coordenada: CoordenadaJson,

    // Solo relevantes cuando tipo == "menu_express". El id de esta
    // misma fila ES el id del menú. Ver MenuAccionJson /
    // MenuExpressExtraJson en perfil_json.rs. #[serde(default)] para
    // filas creadas antes de esta feature (no debería pasar desde la
    // UI, que siempre los manda, pero es la misma red de seguridad
    // que ya usa extra_multimedia).
    #[serde(rename = "menuAccion", default)]
    pub menu_accion: MenuAccionJson,

    #[serde(rename = "menuExtra", default)]
    pub menu_extra: MenuExpressExtraJson,

    // Solo relevantes cuando tipo == "portapapeles". El id de esta
    // misma fila ES el id del Portapapeles. Ver PortapapelesAccionJson
    // / PortapapelesExtraJson en perfil_json.rs. #[serde(default)]
    // mismo criterio que menu_accion/menu_extra.
    #[serde(rename = "portapapelesAccion", default)]
    pub portapapeles_accion: PortapapelesAccionJson,

    #[serde(rename = "portapapelesExtra", default)]
    pub portapapeles_extra: PortapapelesExtraJson,

    pub color: String,

    pub nota: String,
}

// Default de `extra_multimedia` cuando el JSON entrante no lo trae
// (perfiles viejos, o filas que no son multimedia todavía) — mismo
// significado que "Global".
fn extra_multimedia_default() -> String {
    "global".to_string()
}

// ======================================================
// 🎯 TRIGGER UI
// ======================================================

#[derive(Deserialize)]
pub struct TriggerUI {
    pub modificadores: Vec<EntradaUI>,

    pub gatillo: Option<EntradaUI>,

    pub condicion: String,
}

// ======================================================
// 🆔 ENTRADA UI
// ======================================================

#[derive(Deserialize)]
pub struct EntradaUI {
    pub tipo: String,

    pub codigo: String,
}

// ======================================================
// 🎹 CAPTURA UI
// ======================================================

#[derive(Serialize)]
pub struct EntradaCapturaUI {
    pub tipo: String,

    pub codigo: String,

    pub nombre: String,
}

// ======================================================
// 🎯 TRIGGER CAPTURA UI
// ======================================================

#[derive(Serialize)]
pub struct TriggerCapturaUI {
    pub modificadores: Vec<EntradaCapturaUI>,

    pub gatillo: Option<EntradaCapturaUI>,

    pub condicion: String,
}

// ======================================================
// 📦 RESULTADO PERFIL
// ======================================================

#[derive(Serialize)]
pub struct ResultadoPerfil {
    pub perfil: perfil_json,

    pub nombre: String,

    pub perfiles: Vec<String>,

    pub cache_activo: bool,
}

// ======================================================
// 🟢🔴 ESTADO CACHE PERFIL
// ======================================================

#[derive(Serialize)]
pub struct EstadoCachePerfil {
    pub nombre: String,

    pub cache_activo: bool,
}

// ======================================================
// 🔄 CONVERTIR PERFIL
// ======================================================

pub fn convertir_perfil(filas: Vec<FilaUI>) -> perfil_json {
    let remapeos = filas.into_iter().map(convertir_fila).collect();

    perfil_json { remapeos }
}

// ======================================================
// 🧩 CONVERTIR FILA
// ======================================================

fn convertir_fila(fila: FilaUI) -> RemapeoJson {
    RemapeoJson {
        id: fila.id,

        estado: fila.estado,

        app: convertir_app(fila.app),

        trigger: convertir_trigger(fila.trigger),

        tipo: fila.tipo,

        accion_trigger: fila.accion.map(convertir_trigger),

        // macro/archivo/ui todavía no están conectados desde la UI
        // (siguen sin poder mandar accion_referencia) — Multimedia sí,
        // ya viaja tal cual desde FilaUI.
        accion_referencia: fila.accion_referencia,

        extra: fila.extra,

        extra_multimedia: fila.extra_multimedia,

        coordenada: fila.coordenada,

        menu_accion: fila.menu_accion,

        menu_extra: fila.menu_extra,

        portapapeles_accion: fila.portapapeles_accion,

        portapapeles_extra: fila.portapapeles_extra,

        color: fila.color,

        nota: fila.nota,
    }
}

// ======================================================
// 🖥️ CONVERTIR APP
// ======================================================

fn convertir_app(app: AppUI) -> AppJson {
    AppJson {
        programa: app.programa,

        segundo_plano: app.segundo_plano,
    }
}

// ======================================================
// 🎯 CONVERTIR TRIGGER
// ======================================================

fn convertir_trigger(trigger: TriggerUI) -> TriggerJson {
    TriggerJson {
        modificadores: trigger
            .modificadores
            .into_iter()
            .map(convertir_entrada)
            .collect(),

        gatillo: trigger.gatillo.map(convertir_entrada),

        condicion: convertir_condicion(&trigger.condicion),
    }
}

// ======================================================
// 🎯 CONVERTIR CONDICIÓN
// ======================================================

fn convertir_condicion(condicion: &str) -> crate::perfil_cache::CondicionTrigger {
    match condicion {
        "simple" => crate::perfil_cache::CondicionTrigger::Simple,

        "doble" => crate::perfil_cache::CondicionTrigger::Doble,

        "triple" => crate::perfil_cache::CondicionTrigger::Triple,

        "mantenido" => crate::perfil_cache::CondicionTrigger::Mantenido,

        _ => panic!("Condición no soportada: {}", condicion),
    }
}

// ======================================================
// 🆔 CONVERTIR ENTRADA
// ======================================================

fn convertir_entrada(entrada: EntradaUI) -> crate::perfil_json::Input {
    crate::perfil_json::Input::nuevo(convertir_fuente(&entrada.tipo), &entrada.codigo)
}

// ======================================================
// 🌐 TIPO UI → FUENTE
// ======================================================

fn convertir_fuente(tipo: &str) -> &'static str {
    match tipo {
        "Teclado" => "keyboard",

        "Mouse" => "mouse",

        "Multimedia" => "multimedia",

        "Joystick" => "joystick",

        _ => "unknown",
    }
}

// ======================================================
// 🔄 INPUTID → UI
// ======================================================

pub fn convertir_input_captura(input: &crate::eventos::InputId) -> EntradaCapturaUI {
    let fuente = match input.fuente().unwrap_or("") {
        "keyboard" => "Teclado",

        "mouse" => "Mouse",

        "multimedia" => "Multimedia",

        "joystick" => "Joystick",

        _ => "Desconocido",
    };

    let codigo = input.control().unwrap_or("").to_string();

    let nombre = crate::pulsadores::ui_desde_interno(&codigo);

    EntradaCapturaUI {
        tipo: fuente.to_string(),

        codigo,

        nombre,
    }
}

// ======================================================
// 🔄 EVENTOTRIGGER → UI
// ======================================================

pub fn convertir_trigger_captura(
    modificadores: Vec<crate::eventos::InputId>,
    gatillo: crate::eventos::InputId,
    condicion: crate::perfil_cache::CondicionTrigger,
) -> TriggerCapturaUI {
    TriggerCapturaUI {
        modificadores: modificadores.iter().map(convertir_input_captura).collect(),

        gatillo: Some(convertir_input_captura(&gatillo)),

        condicion: condicion_a_texto(&condicion),
    }
}

// ======================================================
// 🎯 CONDICIÓN → TEXTO (para la UI)
// ------------------------------------------------------
// Conversión explícita, en minúscula (coherente con el
// resto del proyecto) — no se usa format!("{:?}", ...)
// porque eso ata la UI al nombre interno del enum en Rust.
// ======================================================

fn condicion_a_texto(condicion: &crate::perfil_cache::CondicionTrigger) -> String {
    match condicion {
        crate::perfil_cache::CondicionTrigger::Simple => "simple".to_string(),

        crate::perfil_cache::CondicionTrigger::Doble => "doble".to_string(),

        crate::perfil_cache::CondicionTrigger::Triple => "triple".to_string(),

        crate::perfil_cache::CondicionTrigger::Mantenido => "mantenido".to_string(),
    }
}

// ======================================================
// 🎬 ESTADO DE CAPTURA (modo Captura de AnalizadorTrigger)
// ------------------------------------------------------
// Único estado con vida propia de este archivo: mientras
// dura una captura, guarda la secuencia de Down que van
// llegando y a qué fila/columna del perfil corresponden.
// Se cierra solo al recibir la condición final.
// ======================================================

struct CapturaEnCurso {
    fila_id: String,
    columna: String,
    secuencia: Vec<crate::eventos::InputId>,
}

static CAPTURA: std::sync::Mutex<Option<CapturaEnCurso>> = std::sync::Mutex::new(None);

// El trigger interno es Option: None es una señal distinta de "todavía
// no hay nada" (ver obtener_captura) — significa "hubo un resultado,
// pero se descartó" (ver recibir_condicion, filtro de Click izquierdo
// solo). La UI usa esa diferencia para resetear el botón en vez de
// seguir esperando.
static RESULTADO_CAPTURA: std::sync::Mutex<Option<(String, String, Option<TriggerCapturaUI>)>> =
    std::sync::Mutex::new(None);

/// Llamada por el botón de captura. Arma el estado transitorio y
/// activa el modo Captura en cache.rs — desde ese momento,
/// entrada.rs empieza a consumir todo lo que llegue y reenviarlo acá.
pub fn iniciar_captura(fila_id: String, columna: String) {
    *CAPTURA.lock().unwrap() = Some(CapturaEnCurso {
        fila_id,
        columna,
        secuencia: Vec::new(),
    });
    *RESULTADO_CAPTURA.lock().unwrap() = None;

    crate::cache::activar_captura();
}

/// Llamada por AnalizadorTrigger (modo Captura) con cada Down nuevo.
pub fn recibir_down(input: crate::eventos::InputId) {
    if let Some(captura) = CAPTURA.lock().unwrap().as_mut() {
        captura.secuencia.push(input);
    }
}

/// Llamada por AnalizadorTrigger (modo Captura) al terminar el gesto.
/// Arma el TriggerCapturaUI final y lo deja listo para que la UI lo
/// retire (ver obtener_captura). Cierra la captura.
pub fn recibir_condicion(condicion: crate::perfil_cache::CondicionTrigger) {
    let Some(captura) = CAPTURA.lock().unwrap().take() else {
        return;
    };

    let Some((modificadores, gatillo)) = captura
        .secuencia
        .split_last()
        .map(|(g, m)| (m.to_vec(), g.clone()))
    else {
        return;
    };

    // El Trigger no puede ser únicamente Click izquierdo sin ningún
    // modificador — de lo contrario el perfil activo remapea TODO clic
    // izquierdo de Windows. Se descarta: el resultado va con trigger en
    // None, no con el TriggerCapturaUI armado.
    let es_click_izquierdo_solo = captura.columna == "Trigger"
        && modificadores.is_empty()
        && gatillo == crate::eventos::InputId::new("mouse", "LeftButton");

    if es_click_izquierdo_solo {
        *RESULTADO_CAPTURA.lock().unwrap() = Some((captura.fila_id, captura.columna, None));
        return;
    }

    let trigger = convertir_trigger_captura(modificadores, gatillo, condicion);

    *RESULTADO_CAPTURA.lock().unwrap() = Some((captura.fila_id, captura.columna, Some(trigger)));
}

/// La UI lo consulta (polling) para saber si ya hay un resultado listo.
pub fn obtener_captura() -> Option<(String, String, Option<TriggerCapturaUI>)> {
    RESULTADO_CAPTURA.lock().unwrap().take()
}
