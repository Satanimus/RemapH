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
// AbrirAccionJson / AbrirExtraJson
//     Datos del tipo "abrir" (Abrir Archivo/App) — ruta elegida y
//     personalización (inicio de ventana, instancias, programa
//     alternativo o argumento). Ver definición más abajo.
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
    pub filas: Vec<ItemFilaJson>,
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
    // Solo relevantes cuando tipo == "portapapeles". El id de esta
    // misma fila (RemapeoJson::id) ES el id del Portapapeles — mismo
    // criterio que menu_accion/menu_extra. portapapeles_accion es
    // solo el nombre de la ventana (la fila no es dueña de ningún
    // contenido propio, ver PortapapelesAccionJson); portapapeles_extra
    // es comportamiento/ubicación/tamaños/límite pedido. #[serde(default)]
    // para que perfiles guardados antes de esta feature sigan cargando
    // sin romper. Ver PortapapelesAccionJson / PortapapelesExtraJson
    // más abajo.
    #[serde(default)]
    pub portapapeles_accion: PortapapelesAccionJson,
    #[serde(default)]
    pub portapapeles_extra: PortapapelesExtraJson,
    // Solo relevantes cuando tipo == "abrir" (Abrir Archivo/App).
    // abrir_accion es la columna Acción (ruta absoluta elegida —
    // archivo, carpeta, .exe o .lnk); abrir_extra es la columna
    // Extra (modo de inicio de ventana, instancias, y programa
    // alternativo o argumento personalizado según corresponda).
    // #[serde(default)] mismo criterio que menu_accion/menu_extra,
    // para que perfiles guardados antes de esta feature sigan
    // cargando sin romper. Ver AbrirAccionJson / AbrirExtraJson más
    // abajo.
    #[serde(default)]
    pub abrir_accion: AbrirAccionJson,
    #[serde(default)]
    pub abrir_extra: AbrirExtraJson,
    // Solo relevante cuando tipo == "macro". A diferencia de
    // abrir_accion/menu_accion no hay struct propia para la columna
    // Acción: el nombre de la macro asignada sigue viajando en
    // accion_referencia (mismo campo genérico que ya usa Multimedia),
    // sin duplicarlo acá. macro_extra es la columna Extra — desde la
    // Etapa 8A deja de ser la puerta al editor y pasa a guardar
    // únicamente el Comportamiento de disparo (Una ejecución/Toggle/
    // Tecla mantenida). #[serde(default)] mismo criterio que
    // abrir_accion/abrir_extra, para que perfiles guardados antes de
    // esta feature sigan cargando sin romper. Ver MacroExtraJson más
    // abajo.
    #[serde(default)]
    pub macro_extra: MacroExtraJson,
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
        Self { filas: Vec::new() }
    }
}

// ======================================================
// 🗂️ SEPARADOR JSON
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeparadorJson {
    pub id: String,
    pub estado: String,
    pub nota: String,
    pub color: String,
    pub expandido: bool,
}

// ======================================================
// 📦 ITEM DE FILA JSON (fila normal o separador)
// ------------------------------------------------------
// Tag "tipoItem" a nivel del mismo objeto, coincidiendo con
// el discriminante que usa el modelo TS (FilaPerfil.tipoItem
// / SeparadorPerfil.tipoItem). Con #[serde(tag = "tipoItem")]
// sobre un enum de variantes-newtype, serde aplana los campos
// del struct interno al mismo nivel que el tag — no anida un
// objeto extra, que es el formato que espera el frontend.
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "tipoItem", rename_all = "lowercase")]
pub enum ItemFilaJson {
    Fila(RemapeoJson),
    Separador(SeparadorJson),
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

    // Nueva variable global de popup Extra (pulido): "color" |
    // "monocromo". Monocromo (default) deja los botones como
    // estaban antes de este campo — heredan el color de la ventana
    // menú (menuExtra ya no tiene voz acá, el color de fondo sigue
    // siendo el de la FILA MenuExpress, ver AccionCache::
    // MenuExpress::color). Color le da a cada botón el borde del
    // color de SU PROPIA fila referenciada (fila_id) — ver
    // compilador.rs::convertir_menu_express, que resuelve ese color
    // por botón al compilar. #[serde(default)] para que perfiles
    // guardados antes de este campo sigan cargando sin romper.
    #[serde(default = "MenuExpressExtraJson::color_boton_default")]
    pub color_boton: String,
}

impl MenuExpressExtraJson {
    fn color_boton_default() -> String {
        "monocromo".to_string()
    }
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
            color_boton: Self::color_boton_default(),
        }
    }
}

// ======================================================
// 📋 PORTAPAPELES — ACCIÓN / EXTRA
// ------------------------------------------------------
// Datos del tipo "portapapeles". El id del Portapapeles es el mismo
// RemapeoJson::id de la fila (no hay id propio acá) — mismo criterio
// que MenuExpress. A diferencia de MenuExpress, la fila NO es dueña
// de ningún contenido propio: es solo un VISUALIZADOR de un pool de
// elementos rotatorios compartido por todo RemapH (ver
// back_portapapeles.rs, etapas E/F). Los fijados sí son exclusivos
// de cada fila (prefijo {id}_ en el nombre de archivo), pero no
// viajan acá — viven directamente en la carpeta del pool.
//
// PortapapelesAccionJson (columna Acción):
//   nombre: título de la ventana Portapapeles, mostrado en el botón
//     de la columna Acción ("📋 nombre") y en la barra superior de
//     la ventana. Único campo — a diferencia de MenuAccionJson no
//     hay lista de botones que armar acá.
//
// PortapapelesExtraJson (columna Extra):
//   comportamiento: "toggle" | "efimero"
//   ubicacion: "persistente" | "cursor"
//   tamano_boton: "pequeno" | "mediano" | "grande" — tamaño propio
//     (botones alargados, no cuadrados como MenuExpress).
//   tamano_texto: "pequeno" | "mediano" | "grande" — mismo
//     vocabulario/valores que ya usa MenuExpress (tamano_texto).
//   limite: máximo de elementos ROTATORIOS que ESTA fila pide
//     mantener en modo Registro (los fijados no cuentan). El límite
//     REAL que aplica el pool compartido es el mayor límite
//     configurado entre todos los Portapapeles actualmente en modo
//     Registro (ver back_portapapeles.rs, etapa F) — este campo es
//     solo lo que la fila "pide", no lo que termina rigiendo.
// ======================================================

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortapapelesAccionJson {
    pub nombre: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortapapelesExtraJson {
    pub comportamiento: String,

    pub ubicacion: String,

    pub tamano_boton: String,

    pub tamano_texto: String,

    pub limite: u32,
}

impl Default for PortapapelesExtraJson {
    fn default() -> Self {
        Self {
            comportamiento: "toggle".to_string(),
            ubicacion: "persistente".to_string(),
            tamano_boton: "mediano".to_string(),
            tamano_texto: "mediano".to_string(),
            limite: 10,
        }
    }
}

// ======================================================
// 📂 ABRIR ARCHIVO/APP — ACCIÓN / EXTRA
// ------------------------------------------------------
// Datos del tipo "abrir". A diferencia de MenuExpress/Portapapeles
// no hay id propio ni pool compartido: cada fila es totalmente
// independiente, dueña de su propia ruta.
//
// AbrirAccionJson (columna Acción):
//   ruta: ruta absoluta del archivo/carpeta/programa elegido con
//     "Seleccionar...". None hasta que se elige algo — mismo
//     criterio de "dato faltante" que el resto del compilador (la
//     fila se descarta en silencio mientras no haya ruta, ver
//     compilador.rs).
//
// AbrirExtraJson (columna Extra):
//   iniciar: "ventana" | "minimizado" | "maximizado" — modo de
//     ventana al lanzar (pasa directo a ShellExecuteW).
//   instancias: "unica" | "multiple" — en "unica", si el programa
//     objetivo ya está corriendo, se enfoca en vez de abrir otro.
//   abrir_con: ruta absoluta de un programa alternativo elegido
//     para abrir el archivo (en vez del asociado por Windows).
//     Solo tiene sentido cuando ruta NO es un .exe/.lnk — None si
//     no se personalizó (se usa el programa por defecto del
//     sistema). Mutuamente excluyente con `argumento` en la UI
//     (uno u otro según la extensión de `ruta`), pero ambos
//     campos viajan siempre presentes acá.
//   argumento: texto libre agregado a la ejecución cuando ruta ES
//     un .exe (ej. "--config"). "" si no se personalizó.
// ======================================================

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AbrirAccionJson {
    pub ruta: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbrirExtraJson {
    pub iniciar: String,

    pub instancias: String,

    pub abrir_con: Option<String>,

    pub argumento: String,
}

impl Default for AbrirExtraJson {
    fn default() -> Self {
        Self {
            iniciar: "ventana".to_string(),
            instancias: "multiple".to_string(),
            abrir_con: None,
            argumento: String::new(),
        }
    }
}

// ======================================================
// 🧩 MACRO — EXTRA
// ------------------------------------------------------
// Datos del tipo "macro" (columna Extra únicamente — ver nota en
// RemapeoJson.macro_extra sobre por qué no hay MacroAccionJson).
//
// comportamiento: "una_ejecucion" (default) | "toggle" |
//   "tecla_mantenida". Decide en Runtime cómo arranca/corta la
//   ejecución de la macro (ver runt_macro.rs, Etapa 8B):
//   • "una_ejecucion" y "toggle" comparten mecanismo (registro
//     fila → ejecución activa) — la diferencia entre ambos es solo
//     de etiqueta/UX, no de código.
//   • "tecla_mantenida" es mecánicamente distinta: depende de
//     Down/Up físico real (cache.rs::resolver_match la trata como
//     "diferida", igual que Mantener/ClickSostenido).
//
// indicador_ejecucion: muestra el overlay Indicador_Macro (🟢
//   paso/total) mientras esta macro corre — ver runt_macro.rs,
//   ejecutar_macro_completa. Default false (apagado).
// ======================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroExtraJson {
    pub comportamiento: String,
    pub indicador_ejecucion: bool,
}

impl Default for MacroExtraJson {
    fn default() -> Self {
        Self {
            comportamiento: "una_ejecucion".to_string(),
            indicador_ejecucion: false,
        }
    }
}

