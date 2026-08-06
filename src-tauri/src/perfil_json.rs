// ======================================================
// 👤 perfil_json
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
//   app: firefox,
//   trigger:
//   { modificadores: [CTRL],
//      gatillo: A,
//      condicion: doble},
//   tipo: tecla_mouse,
//   accion_trigger:
//   { modificadores: [],
//      gatillo: B,
//      condicion: simple},
//   accion_referencia: null}
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
//     Representa cómo se activa un remapeo (o, reutilizada
//     en accion_trigger, cómo se ejecuta una acción de
//     tipo tecla_mouse). modificadores + gatillo + condicion.
// Input
//     Representa una entrada física (fuente + control)
//     tal como se guarda dentro de TriggerJson.
// Input::nuevo()
//     Crea un Input a partir de fuente y control.
// AppJson
//     Representa el contexto donde existe el trigger.
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
    pub app: AppJson,
    pub trigger: TriggerJson,
    pub tipo: String,
    // Caja cuyo contenido depende de `tipo`:
    // - "tecla_mouse" -> accion_trigger (mod + gatillo + condicion)
    // - "macro" / "archivo" / "ui" -> accion_referencia (ruta / valor)
    // Nunca los dos a la vez.
    pub accion_trigger: Option<TriggerJson>,
    pub accion_referencia: Option<String>,
    pub extra: String,
    // Alcance de la Acción Multimedia: "global" | "en_app". Solo
    // tiene sentido cuando tipo == "multimedia" — para el resto de
    // los tipos queda en "global" sin usarse. Campo separado de
    // `extra` a propósito (decisión del usuario): `extra` es
    // vocabulario propio de Tecla/Mouse (Simple/Mantenido/Turbo),
    // mezclarlo acá sería confuso. Snake_case a propósito, mismo
    // criterio que accion_trigger/accion_referencia: el nombre viaja
    // igual en el JSON sin traducción adicional.
    pub extra_multimedia: String,
    pub coordenada: CoordenadaJson,
    // Solo relevantes cuando tipo == "menu_express". El id de esta
    // misma fila (RemapeoJson::id) ES el id del menú — no hay id
    // aparte. menu_accion es la columna Acción (nombre del menú +
    // botones que contiene); menu_extra es la columna Extra (forma/
    // comportamiento/ubicación/tamaños). #[serde(default)] para que
    // perfiles guardados antes de esta feature sigan cargando sin
    // romper. Ver MenuAccionJson / MenuExpressExtraJson más abajo.
    #[serde(default)]
    pub menu_accion: MenuAccionJson,
    #[serde(default)]
    pub menu_extra: MenuExpressExtraJson,
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

    pub condicion: CondicionTrigger,
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

// ======================================================
// 🖱️ COORDENADA JSON
// ------------------------------------------------------
// Datos del extra "Coordenada" dentro del popup Extra de
// tecla_mouse. Ya no depende de tipo == "click_coordenada"
// (ese tipo dejó de existir) — activa es un toggle
// independiente del `extra` genérico de la fila.
//
// activa: si está prendido, se calcula y mueve el cursor
//     antes de ejecutar la acción de la fila (ver
//     compilador.rs); la repetición (Simple/Mantenido/Turbo)
//     ahora la da `extra`, no un campo propio acá.
// ubicacion: "absoluta" | "relativa_cursor" | "relativa_ventana"
// modo_ventana: "porcentaje" | "pixeles" — solo si ubicacion
//     es "relativa_ventana".
// punto_referencia: "sup_izq" | "sup_der" | "centro" |
//     "inf_izq" | "inf_der" — solo si modo_ventana es "pixeles"
//     (en "porcentaje" siempre es sup_izq, fijo).
// post_accion: "inicial" | "final".
// x / y: interpretación depende de ubicacion/modo_ventana —
//     absoluta -> coordenada de pantalla.
//     relativa_cursor -> offset (destino - origen).
//     relativa_ventana + porcentaje -> %H, %V (0-100).
//     relativa_ventana + pixeles -> offset desde punto_referencia.
//     None mientras no se haya capturado todavía.
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordenadaJson {
    pub activa: bool,

    pub ubicacion: String,

    pub modo_ventana: String,

    pub punto_referencia: String,

    pub post_accion: String,

    pub x: Option<f64>,

    pub y: Option<f64>,
}

impl CoordenadaJson {
    pub fn nueva() -> Self {
        Self {
            activa: false,
            ubicacion: "absoluta".to_string(),
            modo_ventana: "pixeles".to_string(),
            punto_referencia: "sup_izq".to_string(),
            post_accion: "final".to_string(),
            x: None,
            y: None,
        }
    }
}

// ======================================================
// ⚡ MENU EXPRESS — ACCIÓN / EXTRA
// ------------------------------------------------------
// Datos del tipo "menu_express". El id del menú es el mismo
// RemapeoJson::id de la fila (no hay id propio acá). Se guardan
// como dos objetos siempre presentes en la fila (mismo criterio que
// CoordenadaJson) — solo tienen efecto cuando RemapeoJson::tipo ==
// "menu_express".
//
// MenuBotonJson: un botón del menú.
//   fila_id: id INTERNO de la fila referenciada (no su número de
//     orden en la tabla).
//   renombrar: texto que Menú muestra sobre ese botón.
//
// MenuAccionJson (columna Acción):
//   nombre: nombre del menú, mostrado en el botón de la columna
//     Acción ("⚡ Multimedia") y en el editor.
//   botones: lista de MenuBotonJson, en el orden en que se
//     guardaron — compilador.rs los reordena por número de fila al
//     compilar (no se guarda el orden de ejecución, ver spec).
//
// MenuExpressExtraJson (columna Extra):
//   forma: "radial" | "cuadricula"
//   columnas / filas: 0 = Auto (se acomoda al número de atajos).
//     Solo uno de los dos puede ser distinto de 0 — la UI impone
//     esa regla; acá se guarda tal cual se recibe.
//   comportamiento: "toggle" | "efimero"
//   ubicacion: "persistente" | "cursor"
//   tamano_boton / tamano_texto: "pequeno" | "mediano" | "grande"
// ======================================================

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuBotonJson {
    pub fila_id: String,

    pub renombrar: String,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuAccionJson {
    pub nombre: String,

    pub botones: Vec<MenuBotonJson>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuExpressExtraJson {
    pub forma: String,

    pub columnas: u32,

    pub filas: u32,

    pub comportamiento: String,

    pub ubicacion: String,

    pub tamano_boton: String,

    pub tamano_texto: String,
}

impl Default for MenuExpressExtraJson {
    fn default() -> Self {
        Self {
            forma: "radial".to_string(),
            columnas: 0,
            filas: 2,
            comportamiento: "toggle".to_string(),
            ubicacion: "persistente".to_string(),
            tamano_boton: "mediano".to_string(),
            tamano_texto: "mediano".to_string(),
        }
    }
}
