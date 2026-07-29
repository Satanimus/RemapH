// ======================================================
// 🖥️ Perfil UI RemapH V3
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
// ======================================================

use serde::{Deserialize, Serialize};

use crate::perfil_json::{perfil_json, AppJson, RemapeoJson, TriggerJson};

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

    pub condicion: String,

    pub extra: String,

    pub color: String,

    pub nota: String,
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

        accion: fila.accion.map(convertir_trigger),

        condicion: fila.condicion,

        extra: fila.extra,

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

        condicion: trigger.condicion,
    }
}

// ======================================================
// 🆔 CONVERTIR ENTRADA
// ======================================================

fn convertir_entrada(entrada: EntradaUI) -> crate::idioma::Input {
    crate::idioma::Input::nuevo(convertir_fuente(&entrada.tipo), &entrada.codigo)
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

pub fn convertir_trigger_captura(trigger: crate::captura::EventoTrigger) -> TriggerCapturaUI {
    TriggerCapturaUI {
        modificadores: trigger
            .modificadores
            .iter()
            .map(convertir_input_captura)
            .collect(),

        gatillo: Some(convertir_input_captura(&trigger.gatillo)),

        condicion: format!("{:?}", trigger.condicion),
    }
}
