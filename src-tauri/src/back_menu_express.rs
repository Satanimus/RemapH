// ======================================================
// ⚡🪟 Back_Menu_Express
// ======================================================
// ETAPAS 5, 6 y 7 DEL FLUJO MenuExpress
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
// Este archivo NO dibuja el layout en sí (radial/cuadrícula es
// 100% TS dentro de la ventana, ver menu_express_main.ts). Sí
// calcula el TAMAÑO de ventana según forma/cantidad de botones/
// tamaño de botón (calcular_tamano_ventana, etapa 6) — con el
// mismo criterio geométrico que el TS usa para posicionar cada
// botón adentro, para que la ventana entre justo sin recorte ni
// scroll. También resuelve la EJECUCIÓN de un botón de adentro
// (etapa 7: boton_down/boton_up) — busca la fila referenciada
// en la caché ya compilada (cache::obtener_remapeo) y llama a
// runtime::ejecutar directo, EXACTAMENTE la misma función que ya
// usa cache.rs (iniciar_solamente/iniciar_y_finalizar) para un
// trigger físico normal — el motor de Mantenido/Turbo/etc. ya
// estabilizado queda intacto, esto solo lo dispara desde un
// clic de UI en vez de un Down/Up físico. El resto es ciclo de
// vida: abrir, cerrar, alternar, y entregarle sus datos (nombre/
// botones/forma/etc.) una sola vez al cargar — mismo patrón que
// captura_coordenada.rs + comandos.rs::
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
// comandos.rs — expone cerrar_menu_express(),
//     obtener_datos_menu_express(), menu_express_boton_down() y
//     menu_express_boton_up() como comandos Tauri finos que
//     delegan acá (mismo criterio que el resto del archivo:
//     comandos.rs nunca tiene lógica propia — acá vive la
//     lógica real, incluida la llamada a runtime::ejecutar, para
//     no romper esa regla en comandos.rs).
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
// boton_down(fila_id) / boton_up(id_menu, fila_id) — la propia
//     ventana, en cada mousedown/mouseup sobre un botón de
//     adentro (ver menu_express_main.ts, etapa 7).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// obtener_datos(id) -> Option<MenuExpressDatosUI>: paquete ya
//     convertido a vocabulario string (mismo que
//     core_menu_express.ts) para que la ventana no tenga que
//     conocer los enums de Rust.
// boton_up(...) -> bool: true si el Comportamiento de este menú
//     es Efímero (debe cerrarse tras este clic). La ventana NO
//     se cierra sola acá — es responsabilidad del TS reproducir
//     el fade-out y recién ahí invocar cerrar_menu_express (ver
//     menu_express_main.ts, etapa 8).
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
//     define qué pasa al hacer clic en un botón DE ADENTRO, ver
//     boton_up() más abajo). Si no, la crea.
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
// boton_down(fila_id)
//     Busca fila_id en la caché compilada (cache::
//     obtener_remapeo) y manda OrdenRuntime::Iniciar — mismo
//     down que un trigger físico. Si la fila ya no existe
//     (referenciaba algo borrado — no debería pasar, compilar()
//     ya cierra el menú entero, ver cerrar_todas() — pero por
//     si acaso), no hace nada.
// boton_up(id_menu, fila_id)
//     Manda OrdenRuntime::Detener (mismo up físico) y después
//     resuelve Comportamiento: si el menú es Efímero, devuelve
//     true para que la propia ventana (menu_express_main.ts)
//     juegue su animación de fade-out y recién después invoque
//     cerrar_menu_express (etapa 8 — el cierre real NO ocurre
//     acá, para darle tiempo a la animación antes de que la
//     ventana se destruya). Si es Toggle, no hace nada más — el
//     menú se queda abierto hasta el [x] o el mismo trigger de
//     nuevo (ver abrir_o_alternar).
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
// • Persistente → última posición real en memoria para ese id
//   (ULTIMA_POSICION, etapa 8), guardada al cerrar la ventana
//   anterior (CloseRequested, ver crear_ventana). Si nunca se
//   cerró una en esta sesión, no hay "última" — se deja sin
//   posición explícita y Tauri usa su default.
//
// La ventana NUNCA debe robarle el foco a la app activa (ni al
// abrirse ni al clickear un botón de adentro) — el usuario tiene
// que poder, por ejemplo, seguir pegando texto en la app que
// estaba usando. .focused(true) en el builder ya no se usa (se
// dejó en false); y evento.preventDefault() en el mousedown del
// TS NO alcanza para el clic, porque Windows decide la activación
// a nivel de sistema antes de que el evento llegue a la webview.
// La única forma real es pedirle a Windows el estilo extendido
// WS_EX_NOACTIVATE sobre el HWND nativo — ver desactivar_activacion()
// más abajo, llamada apenas se crea la ventana (crear_ventana).
//
// boton_down/boton_up SIEMPRE mandan Iniciar+Detener por
// separado (nunca iniciar_y_finalizar de cache.rs) — eso es
// justamente lo que necesita Mantenido/Turbo: el down real
// ocurre al mousedown, el up real (el que decide cuánto duró el
// Mantenido, o si hubo tiempo para otra vuelta de Turbo) ocurre
// al mouseup, exactamente como con un trigger físico sostenido.
// Runtime ya sabe distinguir solo: si el Extra de esa fila no
// pide down/up diferido (requiere_up_real() == false — ej. un
// Emitir Simple sin Extra), el Detener no tiene nada que hacer
// porque la ejecución ya terminó sola con el Iniciar — no hace
// falta que boton_down/boton_up lo verifiquen ellos mismos.
// ======================================================

use crate::perfil_cache::{
    ColorBotonMenu, ComportamientoMenu, FormaMenu, MenuBotonCache, TamanoMenu, UbicacionMenu,
};

use crate::runtime::OrdenRuntime;

use serde::Serialize;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

// Para desactivar_activacion() — ver esa función más abajo.
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
};

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
    pub color_boton: ColorBotonMenu,
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

    // Color de la fila REFERENCIADA (ver MenuBotonCache::color en
    // perfil_cache.rs) — "" si esa fila no tiene color asignado. La
    // ventana solo lo usa como borde cuando datos.colorBoton ==
    // "color" (ver menu_express_main.ts).
    pub color: String,
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

    // Nueva variable global (pulido, punto "Color botón"): "color" |
    // "monocromo" — ver ColorBotonMenu en perfil_cache.rs.
    pub color_boton: String,
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
                color: boton.color.clone(),
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
        color_boton: paquete.color_boton.como_str().to_string(),
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
// 📍 ÚLTIMA POSICIÓN (ubicacion = Persistente) — ETAPA 8
// ------------------------------------------------------
// id de la fila -> última posición (x, y) en la que se cerró esa
// ventana. Solo en memoria (vive y muere con el proceso, como el
// resto de config.rs) — "recordar la última posición" no implica
// sobrevivir a un reinicio de la app, solo a abrir/cerrar el
// menú varias veces en la misma sesión. Se escribe en
// on_window_event (CloseRequested, ver crear_ventana) y se lee
// acá en abrir_o_alternar() para el próximo open.
// ======================================================

static ULTIMA_POSICION: Mutex<Option<HashMap<String, (i32, i32)>>> = Mutex::new(None);

fn recordar_posicion(id: &str, x: i32, y: i32) {
    let mut guardia = ULTIMA_POSICION.lock().unwrap();
    guardia
        .get_or_insert_with(HashMap::new)
        .insert(id.to_string(), (x, y));
}

fn ultima_posicion(id: &str) -> Option<(i32, i32)> {
    ULTIMA_POSICION
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|mapa| mapa.get(id).copied())
}

// ======================================================
// ⏱️ IGNORAR EL "Moved" FANTASMA DE LA CREACIÓN
// ------------------------------------------------------
// Windows manda WM_MOVE (→ WindowEvent::Moved) también cuando el
// SO termina de posicionar la ventana recién creada — no solo
// cuando el usuario la arrastra. Con builder.position(x, y) la
// posición real de creación puede además diferir en unos pocos px
// del valor pedido (redondeo de escalado DPI), así que ese Moved
// "de fábrica" no es basura inofensiva: si se guardara como si
// fuera un arrastre real, cada apertura en modo Cursor
// contaminaría ULTIMA_POSICION con la posición del cursor de ESA
// apertura, y alternar Cursor/Persistente (o incluso Persistente
// solo, por el drift de redondeo) hacía que el menú "caminara"
// solo por la pantalla de apertura en apertura.
//
// Por eso cada ventana registra el instante en que terminó
// build() (VENTANA_LISTA_DESDE) y on_window_event descarta
// cualquier Moved que llegue dentro de los primeros
// IGNORAR_MOVED_MS — tiempo de sobra para el WM_MOVE de creación
// (llega en el mismo tick de mensajes, prácticamente inmediato) y
// corto de sobra para no comerse un arrastre real del usuario
// (nadie arrastra una ventana recién aparecida en menos de
// IGNORAR_MOVED_MS).
// ======================================================

const IGNORAR_MOVED_MS: u128 = 250;

static VENTANA_LISTA_DESDE: Mutex<Option<HashMap<String, std::time::Instant>>> = Mutex::new(None);

fn marcar_ventana_lista(id: &str) {
    let mut guardia = VENTANA_LISTA_DESDE.lock().unwrap();
    guardia
        .get_or_insert_with(HashMap::new)
        .insert(id.to_string(), std::time::Instant::now());
}

fn moved_es_de_creacion(id: &str) -> bool {
    VENTANA_LISTA_DESDE
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|mapa| mapa.get(id))
        .map(|listo_desde| listo_desde.elapsed().as_millis() < IGNORAR_MOVED_MS)
        .unwrap_or(false)
}

fn olvidar_ventana_lista(id: &str) {
    if let Some(mapa) = VENTANA_LISTA_DESDE.lock().unwrap().as_mut() {
        mapa.remove(id);
    }
}

// ======================================================
// 📐 UBICAR VENTANA DENTRO DEL MONITOR (puntos 4 y 5 del pulido)
// ------------------------------------------------------
// Antes, `posicion` (cursor o última guardada) se usaba tal cual
// como esquina superior-izquierda de la ventana — eso hace que:
// • Cursor: el menú se dibuje "pegado hacia abajo-derecha" del
//   cursor y pueda salirse de pantalla si el cursor está cerca
//   de un borde (spec 5).
// • Persistente: si la última posición quedó cerca de un borde
//   (o la ventana se movió de tamaño entre aperturas), puede
//   recortarse igual (mismo bug, ver spec 5 aplicado también a
//   Persistente).
//
// `ubicar_en_monitor` resuelve las DOS reglas de la spec:
// • Cuadrícula: se ancla por ESQUINA — se elige cuál de las 4
//   esquinas de la ventana coincide con el punto de referencia
//   según de qué lado del monitor esté ese punto (si está en la
//   mitad derecha, la ventana crece hacia la izquierda, etc.),
//   así el rectángulo completo siempre entra en el monitor que
//   contiene al punto.
// • Radial: el punto de referencia es el CENTRO del círculo (no
//   una esquina) — se resta la mitad del diámetro en cada eje, y
//   después el resultado se clampea a los bordes del monitor. En
//   Radial no hay "cambio de esquina": el círculo se centra en el
//   cursor y si eso no entra íntegro se recorta el clamp mínimo
//   posible (no puede "cambiar de esquina" un círculo).
//
// Se usa para Cursor SIEMPRE con es_radial según la forma real del
// menú (spec 5). Para Persistente se usa SIEMPRE con es_radial =
// false (ver crear_ventana) — lo guardado en ULTIMA_POSICION es la
// esquina superior-izquierda real de la ventana (outer_position),
// nunca un centro, sea cual sea la forma.
// ======================================================

fn monitor_para_punto(app: &AppHandle, x: i32, y: i32) -> Option<(i32, i32, i32, i32, f64)> {
    let monitores = app.available_monitors().ok()?;

    let contenedor = monitores.iter().find(|m| {
        let pos = m.position();
        let size = m.size();
        x >= pos.x && x < pos.x + size.width as i32 && y >= pos.y && y < pos.y + size.height as i32
    });

    let elegido = contenedor.or_else(|| monitores.first())?;

    let pos = elegido.position();
    let size = elegido.size();

    Some((
        pos.x,
        pos.y,
        size.width as i32,
        size.height as i32,
        elegido.scale_factor(),
    ))
}

// `punto` (cursor u última posición) y el resultado devuelto están
// en coordenadas FÍSICAS (mismo sistema que GetCursorPos y
// Monitor::position/size — ver GetCursorPos en back_coordenada.rs).
// `tamano_ventana_logico` en cambio es el tamaño que arma
// calcular_tamano_ventana(), en unidades LÓGICAS (mismo "px CSS" que
// usa menu_express_main.ts) — WebviewWindowBuilder::inner_size()
// también espera lógico, así que ese valor no se toca.
//
// Mezclar físico y lógico sin convertir fue el bug real de la
// primera versión de esto: con escalado de Windows > 100% (125%,
// 150%...) 1 unidad lógica son N unidades físicas, así que restar
// el ancho lógico de la ventana directo de una coordenada física de
// cursor/monitor la desplazaba de más — y como el resultado
// entraba en el clamp, terminaba pegado al lado OPUESTO del
// monitor, exactamente el síntoma reportado. Acá todo el cálculo de
// geometría ocurre en físico (para poder compararlo con
// Monitor::position/size, que son físicos), y solo el resultado
// final se convierte a lógico — que es lo que
// WebviewWindowBuilder::position() espera.

// ======================================================
// 📐 CLAMPEAR ESQUINA A MONITOR (para Persistente)
// ------------------------------------------------------
// Recibe un punto que es la esquina superior-izquierda REAL
// de la ventana (lo que guarda ULTIMA_POSICION). Solo lo
// clampea para que no quede fuera del monitor.
// NO recalcula la esquina de anclaje (a diferencia de
// ubicar_en_monitor con es_radial = false).
// ======================================================

fn clamp_esquina_a_monitor(
    app: &AppHandle,
    punto: (i32, i32),
    tamano_ventana_logico: (f64, f64),
) -> (f64, f64) {
    let (px, py) = punto;
    let (ancho_log, alto_log) = tamano_ventana_logico;

    let Some((mon_x, mon_y, mon_ancho, mon_alto, escala)) = monitor_para_punto(app, px, py) else {
        return (px as f64, py as f64);
    };

    let ancho = (ancho_log * escala).round() as i32;
    let alto = (alto_log * escala).round() as i32;

    let mon_derecha = mon_x + mon_ancho;
    let mon_abajo = mon_y + mon_alto;

    let x = px.clamp(mon_x, (mon_derecha - ancho).max(mon_x));
    let y = py.clamp(mon_y, (mon_abajo - alto).max(mon_y));

    (x as f64 / escala, y as f64 / escala)
}

fn ubicar_en_monitor(
    app: &AppHandle,
    punto: (i32, i32),
    tamano_ventana_logico: (f64, f64),
    es_radial: bool,
) -> (f64, f64) {
    let (px, py) = punto;

    let Some((mon_x, mon_y, mon_ancho, mon_alto, escala)) = monitor_para_punto(app, px, py) else {
        // Sin info de monitor (no debería pasar) — se devuelve el
        // punto tal cual convertido a lógico con escala 1, mejor
        // que nada.
        return (px as f64, py as f64);
    };

    // Tamaño de la ventana en físico, para poder operar en el mismo
    // sistema que mon_x/mon_y/mon_ancho/mon_alto y el cursor.
    let ancho = (tamano_ventana_logico.0 * escala).round() as i32;
    let alto = (tamano_ventana_logico.1 * escala).round() as i32;

    let mon_derecha = mon_x + mon_ancho;
    let mon_abajo = mon_y + mon_alto;

    let (x_fisico, y_fisico) = if es_radial {
        // El punto de referencia es el CENTRO del círculo — se
        // centra ahí y se clampea (no hay esquina que cambiar).
        let x = px - ancho / 2;
        let y = py - alto / 2;

        let x = x.clamp(mon_x, (mon_derecha - ancho).max(mon_x));
        let y = y.clamp(mon_y, (mon_abajo - alto).max(mon_y));

        (x, y)
    } else {
        // Cuadrícula: se elige la esquina de la ventana que coincide
        // con el punto según en qué mitad del monitor cae ese punto —
        // si el punto está en la mitad derecha, la ventana se dibuja
        // hacia la IZQUIERDA del punto (esquina superior-derecha en el
        // punto), y análogo en vertical.
        let hacia_izquierda = px > mon_x + mon_ancho / 2;
        let hacia_arriba = py > mon_y + mon_alto / 2;

        let x = if hacia_izquierda { px - ancho } else { px };
        let y = if hacia_arriba { py - alto } else { py };

        let x = x.clamp(mon_x, (mon_derecha - ancho).max(mon_x));
        let y = y.clamp(mon_y, (mon_abajo - alto).max(mon_y));

        (x, y)
    };

    // Conversión final físico → lógico, que es lo que
    // WebviewWindowBuilder::position() espera.
    (x_fisico as f64 / escala, y_fisico as f64 / escala)
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
// 📐 TAMAÑO DE VENTANA SEGÚN FORMA
// ------------------------------------------------------
// Mismo criterio geométrico que menu_express_main.ts
// (leerTamanosMenuExpress / renderizarRadial / calcularGrid) —
// si uno cambia, cambiar el otro. Acá solo hace falta el tamaño
// final de VENTANA (para que la ventana nativa entre justo, sin
// scroll ni recorte); el posicionamiento de cada botón adentro
// lo resuelve el TS con ese mismo espacio disponible.
//
// Etapa 8: los px de cada tamaño ya no están fijos acá — se leen
// de config.rs (única fuente de verdad real, configurable), lo
// mismo que hace menu_express_main.ts vía el comando
// obtener_tamanos_menu_express (ver comandos.rs).
// ======================================================

fn tamano_boton_px(tamano: &TamanoMenu) -> (f64, f64) {
    let (ancho, alto) = match tamano {
        TamanoMenu::Pequeno => crate::config::menu_boton_pequeno(),
        TamanoMenu::Mediano => crate::config::menu_boton_mediano(),
        TamanoMenu::Grande => crate::config::menu_boton_grande(),
    };

    (ancho as f64, alto as f64)
}

// Alto fijo del header (título + [x], ver menu_express.css) más
// el padding del cuerpo — se suma al alto de contenido para el
// alto total de ventana en los tres modos.
const ALTO_HEADER: f64 = 32.0;
const PADDING_CUERPO: f64 = 12.0;

fn calcular_tamano_ventana(paquete: &MenuExpressPaquete) -> (f64, f64) {
    let n = paquete.botones.len().max(1);
    let (ancho_boton, alto_boton) = tamano_boton_px(&paquete.tamano_boton);

    match paquete.forma {
        FormaMenu::Radial => {
            // Espejo exacto de calcularRadiosRadial() en
            // menu_express_main.ts — si uno cambia, cambiar el otro.
            // Etapa 9: anillo continuo de gajos, sin header (Radial ya
            // no lo tiene). El radio NO depende de `n`: los gajos
            // cubren el anillo completo y nunca se solapan entre sí,
            // más botones solo angostan cada gajo.
            let grosor = alto_boton;
            let hueco_radio = (alto_boton * 1.3).round();
            let radio_exterior = hueco_radio + grosor;
            let diametro = radio_exterior * 2.0 + 16.0;

            (diametro, diametro)
        }

        FormaMenu::Cuadricula => {
            let (columnas, filas) = calcular_grid_cuadricula(n, paquete.columnas, paquete.filas);

            let ancho = columnas as f64 * ancho_boton
                + (columnas as f64 - 1.0).max(0.0) * 4.0
                + PADDING_CUERPO;

            let alto_cuerpo =
                filas as f64 * alto_boton + (filas as f64 - 1.0).max(0.0) * 4.0 + PADDING_CUERPO;

            (
                ancho.clamp(140.0, 900.0),
                (alto_cuerpo + ALTO_HEADER).clamp(90.0, 700.0),
            )
        }
    }
}

// Espejo de calcularGrid() en menu_express_main.ts — misma regla
// "0 = auto, se rellenan filas/columnas fijas primero, valor no
// válido cae a 1" (ver spec).
fn calcular_grid_cuadricula(n: usize, columnas: u32, filas: u32) -> (u32, u32) {
    if filas > 0 {
        return ((n as f64 / filas as f64).ceil().max(1.0) as u32, filas);
    }

    if columnas > 0 {
        return (
            columnas,
            (n as f64 / columnas as f64).ceil().max(1.0) as u32,
        );
    }

    (1, n.max(1) as u32)
}

// ======================================================
// 🚫 DESACTIVAR ACTIVACIÓN (WS_EX_NOACTIVATE)
// ------------------------------------------------------
// .focused(false) en el builder solo evita el robo de foco AL
// ABRIR — no alcanza para los clics de después, porque Windows
// activa cualquier ventana con la que se interactúa por mouse,
// sin importar cómo se creó. Para que un clic sobre un botón de
// adentro tampoco la active, hay que pedirle a Windows el estilo
// extendido WS_EX_NOACTIVATE sobre el HWND real — con eso la
// ventana sigue recibiendo los clics normalmente (los botones
// funcionan igual), pero deja de pasar a primer plano al
// clickearla, así que la app que el usuario tenía activa
// conserva el foco (puede seguir pegando texto ahí, etc.).
//
// Se obtiene el HWND vía raw_window_handle::HasWindowHandle
// (que WebviewWindow ya implementa) en vez de un método directo
// de Tauri, porque la versión de Tauri usada acá no expone uno.
// Si por lo que sea no se puede obtener el handle (no debería
// pasar en Windows), simplemente no se aplica el estilo — la
// ventana sigue funcionando, solo que sin este pulido.
// ======================================================

fn desactivar_activacion(ventana: &WebviewWindow) {
    let Ok(handle) = ventana.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return;
    };

    let hwnd: HWND = win32.hwnd.get() as *mut core::ffi::c_void;

    unsafe {
        let estilo_actual = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);

        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            estilo_actual | (WS_EX_NOACTIVATE as isize),
        );
    }
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

    let (ancho, alto) = calcular_tamano_ventana(&paquete);

    let es_radial = matches!(paquete.forma, FormaMenu::Radial);
    let tamano_ventana = (ancho, alto);

    let posicion = match paquete.ubicacion {
        UbicacionMenu::Cursor => {
            let cursor = crate::back_coordenada::obtener_cursor();
            Some(ubicar_en_monitor(&app, cursor, tamano_ventana, es_radial))
        }
        // Etapa 8 (+ pulido): recordar la última posición real en
        // memoria (por id, ver ULTIMA_POSICION arriba) — ahora se
        // guarda tanto al mover la ventana como al cerrarla (ver
        // on_window_event más abajo), no solo al cerrar, así que
        // "Persistente" refleja la última ubicación real aunque la
        // ventana no se haya cerrado nunca en la sesión. Primera vez
        // que se abre este id en la sesión → None, y se deja que
        // Tauri elija la posición por defecto (no hay "última"
        // todavía).
        //
        // A diferencia de Cursor, lo guardado en ULTIMA_POSICION
        // siempre es la esquina superior-izquierda REAL de la
        // ventana (eso es lo que entregan Moved/outer_position), sin
        // importar la forma — así que acá se reencuadra siempre como
        // esquina (es_radial: false), nunca como centro: solo hace
        // falta evitar que quede cortada si cambió de monitor/
        // resolución entre sesiones, no recalcular el anclaje.
        UbicacionMenu::Persistente => {
            ultima_posicion(&id).map(|punto| {
                // Solo clampea para que no salga del monitor
                // NO recalcula la esquina de anclaje
                clamp_esquina_a_monitor(&app, punto, tamano_ventana)
            })
        }
    };

    // Copias para el closure movido a run_on_main_thread — se
    // conservan `id`/`label` originales acá afuera para poder
    // limpiar el registro si run_on_main_thread mismo falla en
    // programarse (no llegó a correr el closure de adentro).
    let id_interno = id.clone();
    let label_interno = label.clone();

    // AppHandle propio para el closure: run_on_main_thread ya toma
    // `app` prestado para la llamada en sí (&self), así que el
    // closure `move` no puede mover ese mismo `app` adentro — se
    // clona acá afuera (AppHandle::clone() es barato, es un Arc por
    // dentro) y se mueve la copia.
    let app_interno = app.clone();

    let resultado = app.run_on_main_thread(move || {
        let mut builder = WebviewWindowBuilder::new(
            &app_interno,
            &label_interno,
            WebviewUrl::App(format!("menu_express.html?id={id_interno}").into()),
        )
        .title("RemapH — Menú")
        .inner_size(ancho, alto)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .devtools(true);

        if let Some((x, y)) = posicion {
            builder = builder.position(x as f64, y as f64);
        }

        match builder.build() {
            Ok(ventana) => {
                desactivar_activacion(&ventana);

                let id_cierre = id_interno.clone();

                marcar_ventana_lista(&id_cierre);

                // Clon aparte para leer la posición desde dentro del
                // closure de eventos (on_window_event no entrega la
                // ventana como argumento) — WebviewWindow::clone() es
                // barato (wrapper sobre Arc), mismo criterio que
                // AppHandle::clone() más arriba.
                let ventana_para_evento = ventana.clone();

                // Si la ventana se cierra por cualquier otra vía (Alt+F4,
                // cerrar_todas, el propio botón [x] ya invocó el comando
                // pero por si el usuario la cierra "a lo nativo") — se
                // limpia el registro para que abrir_o_alternar() la trate
                // como cerrada la próxima vez.
                //
                // La posición se guarda en DOS momentos (pulido: antes
                // solo se guardaba al cerrar, y por eso "Persistente" no
                // andaba si la ventana se dejaba abierta o se mataba el
                // proceso sin pasar por CloseRequested):
                // • Moved: cada vez que el usuario arrastra la ventana
                //   (esto es lo que hace que "arrastrar y volver a abrir
                //   después" funcione de verdad) — se ignora el/los
                //   Moved que Windows manda al recién crearse la
                //   ventana (ver moved_es_de_creacion), que NO son un
                //   arrastre real y, si se guardaran, hacían que el
                //   menú "caminara" solo de apertura en apertura.
                // • CloseRequested: por las dudas Moved no haya llegado a
                //   disparar (ej. drag terminado justo al cerrar).
                // El guardado es liviano (un HashMap en memoria), así que
                // no hace falta throttlear los eventos Moved reales.
                ventana.on_window_event(move |evento| {
                    if let tauri::WindowEvent::Moved(posicion) = evento {
                        if !moved_es_de_creacion(&id_cierre) {
                            recordar_posicion(&id_cierre, posicion.x, posicion.y);
                        }
                    }

                    if let tauri::WindowEvent::CloseRequested { .. } = evento {
                        if let Ok(posicion) = ventana_para_evento.outer_position() {
                            recordar_posicion(&id_cierre, posicion.x, posicion.y);
                        }
                    }

                    if let tauri::WindowEvent::CloseRequested { .. }
                    | tauri::WindowEvent::Destroyed = evento
                    {
                        olvidar_ventana_lista(&id_cierre);

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

// ======================================================
// ⬇️ BOTÓN — DOWN
// ------------------------------------------------------
// La propia ventana lo llama en cada mousedown sobre un botón
// de adentro (ver menu_express_main.ts). Busca fila_id en la
// caché ya compilada (misma caché que usa el motor físico
// normal, ver cache.rs) y manda el mismo Iniciar que mandaría
// cache.rs para un trigger real — Runtime no distingue si vino
// de acá o de un Down físico (ver header, decisión de diseño).
// Si la fila ya no existe (no debería pasar, ver cerrar_todas
// en compilador.rs), no hace nada — no hay nada que iniciar.
// ======================================================

pub fn boton_down(fila_id: &str) {
    let Some(remapeo) = crate::cache::obtener_remapeo(fila_id) else {
        return;
    };

    crate::runtime::ejecutar(OrdenRuntime::Iniciar {
        id: remapeo.id,
        accion: remapeo.accion,
        extra: remapeo.extra,
        coordenada: remapeo.coordenada,
    });
}

// ======================================================
// ⬆️ BOTÓN — UP
// ------------------------------------------------------
// La propia ventana lo llama en cada mouseup sobre un botón de
// adentro — SIEMPRE, incluso si el mouse ya no está sobre el
// botón al soltar (mismo criterio que un Up físico: el up de
// Mantenido/Turbo no depende de seguir con el cursor encima).
// Manda el Detener real (mismo que un Up físico) y, recién
// después, resuelve Comportamiento (Toggle/Efímero) — la spec
// aclara que eso se decide "tras el up", nunca antes: si fuera
// antes, un Efímero cerraría la ventana ANTES de que Turbo/
// Mantenido llegaran a soltar de verdad.
//
// Devuelve true si el menú es Efímero (debe cerrarse tras este
// clic) — a propósito NO cierra la ventana acá (etapa 8): eso
// dejaría cero tiempo para la animación de fade-out, porque la
// ventana se destruiría antes de que el TS pudiera reproducirla.
// El cierre real queda en manos del TS (ver menu_express_main.ts,
// cerrarConFade), que anima y recién ahí invoca
// cerrar_menu_express — mismo comando que usa el botón [x].
// ======================================================

pub fn boton_up(id_menu: &str, fila_id: &str) -> bool {
    crate::runtime::ejecutar(OrdenRuntime::Detener {
        id: fila_id.to_string(),
    });

    con_registro(|mapa| {
        mapa.get(id_menu)
            .map(|datos| datos.comportamiento == "efimero")
    })
    .unwrap_or(false)
}
