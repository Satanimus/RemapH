// ======================================================
// 👤 perfil_json RemapH V3
// ======================================================
// 1. ¿Qué hace este archivo?
// Modelo persistente del perfil de usuario.
// Guarda la configuración completa necesaria para:
// - Reconstruir la UI. - Guardar perfiles.
// - Cargar perfiles. - Entregar información al compilador.
//
// perfil_json NO:
// - Ejecuta remapeos. - Conoce Runtime. - Conoce dispositivos físicos.
//
// Flujo:
// UI
//   ↓
// perfil_json
//   ↓
// JSON guardado
//   ↓
// Compilador
//   ↓
// perfil_cache
// ------------------------------------------------------
// 2. ¿Qué información recibe?
// Recibe la configuración creada o modificada por la UI.
// Contiene:
// Perfil: - Lista de remapeos.
//
//Remapeo:
// - Identidad.- Estado.- Trigger.- Respuesta.- Personalización.
//
// Ejemplo:
// RemapeoJson
// {id: "001",
//   trigger:
//   { app: firefox,
//      entrada: CTRL, A,
//      condicion: Doble},
//   respuesta:
//   { tipo: tecla_mouse,
//      accion: B,
//      ejecucion: Simple }}
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
// Recibe información desde:
// - UI RemapH.
// Es utilizado por:
// - Sistema de guardado. - Sistema de carga.- Compilador hacia perfil_cache.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Entrega una estructura serializable/deserializable.
// Ejemplo:
// perfil_json
// {remapeos:
//    [RemapeoJson] }
// Esta información posteriormente será transformada
// por el compilador en una estructura optimizada
// para Runtime.
// ------------------------------------------------------
// 5. Funciones y estructuras del archivo
// perfil_json
//     Contenedor principal del perfil.
// RemapeoJson
//     Representa una fila completa de la tabla UI.
// TriggerJson
//     Representa cómo se activa un remapeo.
// Input
//     Representa una entrada física (fuente + control)
//     tal como se guarda dentro de TriggerJson.entrada.
// Input::nuevo()
//     Crea un Input a partir de fuente y control.
// AppJson
//     Representa el contexto donde existe el trigger.
// RespuestaJson
//     Representa qué debe ocurrir después del trigger.
// AccionJson
//     Representa los datos necesarios para ejecutar.
// perfil_json::nuevo()
//     Crea un perfil vacío.
// ------------------------------------------------------

use crate::perfil_cache::CondicionTrigger;

// ======================================================
// 🆔 INPUT
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Input {
    pub fuente: String,

    pub control: String,
}

impl Input {
    pub fn nuevo(fuente: &str, control: &str) -> Self {
        Self {
            fuente: fuente.to_string(),

            control: control.to_string(),
        }
    }
}

// ======================================================
// 👤 PERFIL JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct perfil_json {
    pub remapeos: Vec<RemapeoJson>,
}

// ======================================================
// 🎯 REMAPEO JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RemapeoJson {
    pub id: String,
    pub estado: String,
    pub trigger: TriggerJson,
    pub respuesta: RespuestaJson,
    pub color: String,
    pub nota: String,
}

// ======================================================
// ⌨️ TRIGGER JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TriggerJson {
    pub app: AppJson,

    pub entrada: Vec<Input>,

    pub condicion: CondicionTrigger,
}

// ======================================================
// ⚡ RESPUESTA JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RespuestaJson {
    pub tipo: String,

    pub accion: String,

    pub extra: String,
}

// ======================================================
// 🚀 CREAR PERFIL JSON
// ======================================================

impl perfil_json {
    pub fn nuevo() -> Self {
        Self {
            remapeos: Vec::new(),
        }
    }
}

// ======================================================
// 🖥️ APP JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AppJson {
    pub programa: Option<String>,

    #[serde(rename = "segundoPlano")]
    pub segundo_plano: bool,
}
