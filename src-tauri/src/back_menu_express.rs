// ======================================================
// ⚡🪟 Back_Menu_Express
// ======================================================
// ETAPA 5 DEL FLUJO MenuExpress
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Dueño de las ventanas flotantes nativas de MenuExpress.
// Cada fila con tipo == "menu_express" que dispara su trigger
// abre (o alterna) SU PROPIA ventana — label
// "menu_express_<id>", donde <id> es el mismo RemapeoJson::id
// de la fila (ver perfil_json.rs). Pueden existir varias
// ventanas MenuExpress abiertas a la vez, cada una
// independiente (confirmado por el usuario, ver plan por
// etapas).
//
// Este archivo NO decide layout (radial/cuadrícula — eso es
// 100% TS en la ventana, etapa 6) ni ejecuta los botones
// (eso es runtime::ejecutar vía comandos nuevos, etapa 7).
// Su responsabilidad es solo el ciclo de vida de la ventana
// nativa: abrir, cerrar, alternar, y entregarle sus datos
// (nombre/botones/forma/etc.) una sola vez al cargar — mismo
// patrón que captura_coordenada.rs + comandos.rs::
// abrir_ventana_captura_coordenada.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// runtime.rs (ejecutar_accion, brazo AccionCache::MenuExpress)
//     — llama abrir_o_alternar() directo, NUNCA a través de un
//     comando Tauri: el trigger llega desde el hilo físico de
//     entrada (back_interception → entrada → cache → runtime),
//     no desde una invocación JS. Por eso necesita el
//     AppHandle global (inicializar(), fijado en setup() de
//     Tauri) en vez de recibirlo como parámetro de comando.
// comandos.rs — expone cerrar_menu_express() y
//     obtener_datos_menu_express() como comandos Tauri finos
//     que delegan acá (mismo criterio que el resto del
//     archivo: comandos.rs nunca tiene lógica propia).
// compilador.rs (compilar()) — llama cerrar_todas() en cada
//     recompilación, para nunca dejar una ventana abierta con
//     botones que referencian filas que ya no existen (ver
//     nota en compilar()).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// abrir_o_alternar(id, paquete) — id de la fila que disparó
//     el trigger + el AccionCache::MenuExpress ya resuelto
//     (nombre/botones/forma/... — ver perfil_cache.rs).
// cerrar(id) — cierre puntual (botón [x] de la ventana, o
//     alternar() cuando ya estaba abierta).
// cerrar_todas() — cierre masivo (recompilación).
// obtener_datos(id) — la propia ventana, una sola vez al
//     cargar (ver menu_express_main.ts).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// obtener_datos(id) -> Option<MenuExpressDatosUI>: paquete ya
//     convertido a vocabulario string (mismo que
//     core_menu_express.ts) para que la ventana no tenga que
//     conocer los enums de Rust.
// ------------------------------------------------------
// 5. Funciones del archivo
//
// inicializar(app)
//     Guarda el AppHandle global — se llama una sola vez, en
//     el setup() de tauri::Builder (ver lib.rs). Sin esto,
//     abrir_o_alternar() no tiene forma de crear ventanas
//     desde el hilo de entrada física.
// abrir_o_alternar(id, paquete)
//     Si ya hay una ventana abierta para ese id → la cierra
//     (toggle a NIVEL DE TRIGGER: volver a presionar el mismo
//     trigger cierra su menú, sea Toggle o Efímero — eso solo
//     define qué pasa al hacer clic en un botón DE ADENTRO,
//     ver runtime.rs/comandos.rs etapa 7). Si no, la crea.
// crear_ventana(app, id, paquete)
//     Arma y muestra la ventana nueva. Corre en el hilo
//     principal vía AppHandle::run_on_main_thread — este
//     trigger NUNCA llega desde un comando Tauri (a diferencia
//     de abrir_ventana_captura_coordenada, que sí es un
//     comando async y por eso ya corre en un contexto seguro),
//     así que hay que marshalling explícito al hilo principal
//     para WebviewWindowBuilder::build() (WebView2 en Windows
//     exige crearse ahí).
// cerrar(id)
//     Cierra la ventana de ese id si existe, y limpia su
//     entrada del registro.
// cerrar_todas()
//     cerrar() de cada id actualmente registrado.
// obtener_datos(id)
//     Consulta de sólo lectura del registro — no lo modifica.
// label_de(id)
//     "menu_express_<id>" — único lugar que arma el label,
//     para no repetir el formato en cada función.
// ------------------------------------------------------
// 6. Decisiones de diseño
//
// Posición al abrir, según menu_extra.ubicacion:
// • Cursor      → posición actual del cursor
//   (back_coordenada::obtener_cursor()), igual que "Relativa a
//   cursor" en Click en coordenada.
// • Persistente → por ahora (etapa 5) usa un punto por
//   defecto fijo; recordar la ÚLTIMA posición real en memoria
//   (por id) es una mejora de etapa 8 (pulido) — no bloquea
//   que el menú funcione mientras tanto.
// ======================================================

use crate::perfil_cache::{
    ComportamientoMenu, FormaMenu, MenuBotonCache, TamanoMenu, UbicacionMenu,
};

use serde::Serialize;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

// ======================================================
// 📦 PAQUETE RECIBIDO DESDE RUNTIME
// ------------------------------------------------------
// Espejo exacto de los campos de AccionCache::MenuExpress
// (ver perfil_cache.rs) — separado en su propio struct acá
// para no atar este archivo a la forma exacta del enum (y
// para que runtime.rs arme uno solo, sin repetir 9 argumentos
// sueltos en la llamada).
// ======================================================

pub struct MenuExpressPaquete {
    pub nombre: String,
    pub botones: Vec<MenuBotonCache>,
    pub forma: FormaMenu,
    pub columnas: u32,
    pub filas: u32,
    pub comportamiento: ComportamientoMenu,
    pub ubicacion: UbicacionMenu,
    pub tamano_boton: TamanoMenu,
    pub tamano_texto: TamanoMenu,
    pub color: String,
}

// ======================================================
// 🖥️ DATOS SERIALIZABLES PARA LA VENTANA (TS)
// ------------------------------------------------------
// Mismo vocabulario string que core_menu_express.ts — la
// ventana no conoce (ni necesita conocer) los enums de Rust.
// ======================================================

#[derive(Clone, Serialize)]
pub struct MenuBotonUI {
    #[serde(rename = "filaId")]
    pub fila_id: String,

    pub renombrar: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuExpressDatosUI {
    pub nombre: String,

    pub botones: Vec<MenuBotonUI>,

    pub forma: String,

    pub columnas: u32,

    pub filas: u32,

    pub comportamiento: String,

    pub ubicacion: String,

    pub tamano_boton: String,

    pub tamano_texto: String,

    pub color: String,
}

fn convertir_datos_ui(paquete: &MenuExpressPaquete) -> MenuExpressDatosUI {
    MenuExpressDatosUI {
        nombre: paquete.nombre.clone(),

        botones: paquete
            .botones
            .iter()
            .map(|boton| MenuBotonUI {
                fila_id: boton.fila_id.clone(),
                renombrar: boton.renombrar.clone(),
            })
            .collect(),

        forma: paquete.forma.como_str().to_string(),
        columnas: paquete.columnas,
        filas: paquete.filas,
        comportamiento: paquete.comportamiento.como_str().to_string(),
        ubicacion: paquete.ubicacion.como_str().to_string(),
        tamano_boton: paquete.tamano_boton.como_str().to_string(),
        tamano_texto: paquete.tamano_texto.como_str().to_string(),
        color: paquete.color.clone(),
    }
}

// ======================================================
// 🌐 APPHANDLE GLOBAL
// ------------------------------------------------------
// OnceLock: se fija una sola vez (setup() de Tauri, ver
// lib.rs) y no vuelve a cambiar en toda la vida del proceso —
// exactamente lo que necesita abrir_o_alternar() para crear
// ventanas desde el hilo de entrada física, que no tiene
// forma de recibir un AppHandle como parámetro de comando.
// ======================================================

static APP: OnceLock<AppHandle> = OnceLock::new();

/// Llamado una sola vez desde el setup() de tauri::Builder.
pub fn inicializar(app: AppHandle) {
    let _ = APP.set(app);
}

fn app_handle() -> Option<&'static AppHandle> {
    APP.get()
}

// ======================================================
// 🗃️ REGISTRO DE VENTANAS ABIERTAS
// ------------------------------------------------------
// id de la fila -> datos ya convertidos a vocabulario UI. La
// existencia de una entrada acá ES la fuente de verdad de "hay
// una ventana abierta para este id" (más simple que preguntarle
// a Tauri get_webview_window() en cada punto de decisión, y
// además es lo que obtener_datos() necesita servir).
// ======================================================

static ABIERTOS: Mutex<Option<HashMap<String, MenuExpressDatosUI>>> = Mutex::new(None);

fn con_registro<R>(f: impl FnOnce(&mut HashMap<String, MenuExpressDatosUI>) -> R) -> R {
    let mut guardia = ABIERTOS.lock().unwrap();
    let mapa = guardia.get_or_insert_with(HashMap::new);
    f(mapa)
}

fn label_de(id: &str) -> String {
    format!("menu_express_{id}")
}

// ======================================================
// ⚡🪟 ABRIR O ALTERNAR
// ------------------------------------------------------
// Único punto de entrada llamado desde runtime.rs. Alternar es
// A NIVEL DE TRIGGER (ver header) — independiente del
// Comportamiento Toggle/Efímero de la fila, que solo aplica al
// hacer clic en un botón de adentro (etapa 7).
// ======================================================

pub fn abrir_o_alternar(id: String, paquete: MenuExpressPaquete) {
    let ya_abierto = con_registro(|mapa| mapa.contains_key(&id));

    if ya_abierto {
        cerrar(&id);
        return;
    }

    let Some(app) = app_handle() else {
        // No debería pasar: inicializar() corre en setup(), antes de
        // que cualquier trigger físico pueda llegar a compilarse y
        // dispararse. Si pasa, no hay ventana que crear — se ignora
        // en silencio (mismo criterio que el resto de Runtime ante
        // datos/estado incompleto, ver compilador.rs).
        return;
    };

    con_registro(|mapa| {
        mapa.insert(id.clone(), convertir_datos_ui(&paquete));
    });

    crear_ventana(app.clone(), id, paquete);
}

// ======================================================
// 🏗️ CREAR VENTANA
// ------------------------------------------------------
// Corre en el hilo principal (run_on_main_thread) — este
// disparo nunca llega desde un comando Tauri (a diferencia de
// abrir_ventana_captura_coordenada en comandos.rs, que sí es
// `async fn` y por eso ya corre en un contexto seguro para
// WebView2 en Windows). Acá el llamador es el hilo de entrada
// física, así que hay que marshalling explícito.
// ======================================================

fn crear_ventana(app: AppHandle, id: String, paquete: MenuExpressPaquete) {
    let label = label_de(&id);

    // Cantidad de botones define un alto aproximado para la lista
    // simple de esta etapa — el cálculo geométrico real
    // (radial/cuadrícula) llega en la etapa 6, en TS.
    let alto_estimado = (60.0 + paquete.botones.len() as f64 * 36.0).clamp(90.0, 480.0);

    let posicion = match paquete.ubicacion {
        UbicacionMenu::Cursor => Some(crate::back_coordenada::obtener_cursor()),
        // TODO(etapa 8): recordar la última posición real en memoria
        // por id en vez de este punto fijo.
        UbicacionMenu::Persistente => None,
    };

    // Copias para el closure movido a run_on_main_thread — se
    // conservan `id`/`label` originales acá afuera para poder
    // limpiar el registro si run_on_main_thread mismo falla en
    // programarse (no llegó a correr el closure de adentro).
    let id_interno = id.clone();
    let label_interno = label.clone();

    let resultado = app.run_on_main_thread(move || {
        let mut builder = WebviewWindowBuilder::new(
            &app,
            &label_interno,
            WebviewUrl::App(format!("menu_express.html?id={id_interno}").into()),
        )
        .title("RemapH — Menú")
        .inner_size(220.0, alto_estimado)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(true)
        .devtools(true);

        if let Some((x, y)) = posicion {
            builder = builder.position(x as f64, y as f64);
        }

        match builder.build() {
            Ok(ventana) => {
                let id_cierre = id_interno.clone();

                // Si la ventana se cierra por cualquier otra vía (Alt+F4,
                // cerrar_todas, el propio botón [x] ya invocó el comando
                // pero por si el usuario la cierra "a lo nativo") — se
                // limpia el registro para que abrir_o_alternar() la trate
                // como cerrada la próxima vez.
                ventana.on_window_event(move |evento| {
                    if let tauri::WindowEvent::CloseRequested { .. }
                    | tauri::WindowEvent::Destroyed = evento
                    {
                        con_registro(|mapa| {
                            mapa.remove(&id_cierre);
                        });
                    }
                });
            }

            Err(error) => {
                eprintln!("[MenuExpress] no se pudo crear la ventana: {error}");

                con_registro(|mapa| {
                    mapa.remove(&id_interno);
                });
            }
        }
    });

    if let Err(error) = resultado {
        eprintln!("[MenuExpress] run_on_main_thread falló para {label}: {error}");

        con_registro(|mapa| {
            mapa.remove(&id);
        });
    }
}

// ======================================================
// 🚪 CERRAR
// ======================================================

pub fn cerrar(id: &str) {
    if let Some(app) = app_handle() {
        if let Some(ventana) = app.get_webview_window(&label_de(id)) {
            let _ = ventana.close();
        }
    }

    con_registro(|mapa| {
        mapa.remove(id);
    });
}

// ======================================================
// 🚪🚪 CERRAR TODAS
// ------------------------------------------------------
// Llamado desde compilador::compilar() en cada recompilación
// (decisión del usuario) — evita estados donde un botón del
// menú referencia una fila que el perfil recién editado ya no
// tiene.
// ======================================================

pub fn cerrar_todas() {
    let ids: Vec<String> = con_registro(|mapa| mapa.keys().cloned().collect());

    for id in ids {
        cerrar(&id);
    }
}

// ======================================================
// 📤 OBTENER DATOS
// ------------------------------------------------------
// Consulta de sólo lectura — la propia ventana la llama una
// sola vez al cargar (ver menu_express_main.ts), con el id que
// vino en la URL (?id=...).
// ======================================================

pub fn obtener_datos(id: &str) -> Option<MenuExpressDatosUI> {
    con_registro(|mapa| mapa.get(id).cloned())
}
