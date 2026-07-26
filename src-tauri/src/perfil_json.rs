// ======================================================
// 👤 perfil_json RemapH V3
// ------------------------------------------------------
// Modelo persistente del perfil.
//
// perfil_json representa la tabla de la UI.
//
// El número de fila no se guarda.
// El orden dentro del Vec determina el número.
//
// JSON
//   ↓
// perfil_json
// ======================================================

use crate::idioma::Input;

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

    pub app: AppJson,

    pub trigger: TriggerJson,

    pub tipo: String,

    pub accion: Option<TriggerJson>,

    pub condicion: String,

    pub ejecucion: String,

    pub color: String,

    pub nota: String,
}

// ======================================================
// ⌨️ TRIGGER JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TriggerJson {
    pub modificadores: Vec<Input>,

    pub gatillo: Option<Input>,

    pub condicion: String,
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
// APP JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AppJson {
    pub programa: Option<String>,

    #[serde(rename = "segundoPlano")]
    pub segundo_plano: bool,
}
