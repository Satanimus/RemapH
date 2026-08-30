// ======================================================
// 🖱️📌 Captura_Coordenada
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Estado transitorio de la ventana de captura de "Click en
// coordenada". A propósito NO reutiliza el "modo Captura"
// de analizador_trigger/perfil_ui (ese consume TODO el
// input físico) — acá el pedido es lo opuesto: mientras la
// ventana de captura está abierta, Windows debe seguir
// funcionando normal (mouse y teclado). Solo se necesita
// saber, sin bloquear nada, si la tecla configurada
// (config::tecla_guardar_coordenada()) se apretó.
//
// Por eso esto es un TAP pasivo sobre entrada.rs: se llama
// en cada evento físico, pero nunca decide nada sobre él
// (nunca retiene/consume) — solo mira, y si corresponde,
// deja una marca para que la UI de captura la retire por
// polling. El resultado final (coordenada calculada) NO
// vive acá — lo arma el JS de la ventana de captura (que ya
// sabe la ubicación/modo elegidos y ya está sondeando cursor
// y ventana activa) y se guarda acá solo para que el popup
// de la fila del perfil lo retire.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
//
// entrada.rs (observar_evento(), en cada evento físico —
//     nunca cambia su resultado).
// comandos.rs (activar/desactivar al abrir/cerrar la
//     ventana; consultar_guardado/guardar_resultado/
//     obtener_resultado, todo por polling desde la UI).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
//
// observar_evento(): cada InputEvent físico tal cual llega.
// guardar_resultado(x, y): coordenada ya calculada por la
//     ventana de captura (según la ubicación/modo activos).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// consultar_guardado() -> bool (una sola vez por pulsación).
// obtener_resultado() -> Option<(f64, f64)> (una sola vez).
// ------------------------------------------------------
// 5. Funciones del archivo
//
// activar(ubicacion, modo_ventana, punto_referencia) / desactivar()
//     Se llaman al abrir/cerrar la ventana de captura. activar()
//     recibe también la configuración activa de la fila que abrió
//     la ventana (mismo vocabulario que core_coordenada.ts:
//     "absoluta"/"relativa_cursor"/"relativa_ventana",
//     "porcentaje"/"pixeles", "sup_izq"/etc.) — se guarda tal cual,
//     sin interpretarla acá, para que captura.html la lea una sola
//     vez al abrir vía obtener_config_activa().
// observar_evento(evento)
//     Tap pasivo: si está activo y el evento es Down de la
//     tecla configurada, marca "se pidió guardar". Nunca
//     consume ni retiene nada — entrada.rs sigue su curso
//     normal después de llamar esto, siempre.
// consultar_guardado()
//     Polling desde la ventana de captura: ¿se pidió
//     guardar desde la última consulta? Se limpia al leerla.
// guardar_resultado(x, y)
//     La ventana de captura ya calculó la coordenada final
//     (absoluta/offset/porcentaje según corresponda) y la
//     deja acá.
// obtener_resultado()
//     Polling desde el popup de la fila del perfil: retira
//     el resultado cuando está listo. Se limpia al leerla.
// guardar_seleccion_banco(coordenada) / obtener_seleccion_banco()
//     Mismo mecanismo que guardar_resultado()/obtener_resultado(),
//     pero para la ventana "Coordenadas guardadas" (Etapa C/D):
//     deja/retira la CoordenadaBanco elegida (existente o recién
//     creada) para que el popup de la fila del perfil la aplique.
//     Independiente de ACTIVA/desactivar() — no depende de que la
//     ventana overlay de captura esté abierta.
// activar_preview(id, ubicacion, modo_ventana, punto_referencia, x, y) /
// desactivar_preview(id) / actualizar_xy_preview(id, x, y) /
// obtener_config_preview(id) / desactivar_todas_las_previews()
//     Previsualización por fila (Etapa F): cada fila con el toggle
//     ⊙️ encendido tiene su propia entrada en CONFIG_PREVIEWS, key =
//     id de CoordenadaBanco — puede haber cualquier cantidad activas
//     a la vez, cada una en su propia ventana overlay. Mutuamente
//     excluyente con el modo captura de arriba: activar() llama
//     desactivar_todas_las_previews() y activar_preview() llama
//     desactivar(). obtener_config_preview(id) es polling desde
//     comandos.rs::obtener_destino_preview_coordenada, sin consumir
//     el valor (a diferencia de obtener_resultado()).
//     actualizar_xy_preview(id, x, y) la llama comandos.rs::
//     guardar_posicion_preview_coordenada tras persistir el arrastre
//     del marcador (Regla 17) en Coordenadas.tsv, para que el x/y en
//     memoria no quede desincronizado del disco mientras la
//     previsualización sigue abierta.
// ======================================================

use std::collections::HashMap;

use crate::banco_coordenadas::CoordenadaBanco;
use crate::config;
use crate::eventos::{InputEvent, InputId, InputState};

/// Config de la fila que abrió la ventana — mismo vocabulario de
/// strings que core_coordenada.ts, sin interpretar acá.
#[derive(Clone)]
pub struct ConfigCaptura {
    pub ubicacion: String,

    pub modo_ventana: String,

    pub punto_referencia: String,
}

static ACTIVA: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
static GUARDADO_SOLICITADO: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
static RESULTADO: std::sync::Mutex<Option<(f64, f64)>> = std::sync::Mutex::new(None);
static CONFIG_ACTIVA: std::sync::Mutex<Option<ConfigCaptura>> = std::sync::Mutex::new(None);
static SELECCION_BANCO: std::sync::Mutex<Option<CoordenadaBanco>> = std::sync::Mutex::new(None);

/// Config del modo previsualización, mismo vocabulario de strings
/// que ConfigCaptura.
#[derive(Clone)]
pub struct ConfigPreview {
    pub ubicacion: String,

    pub modo_ventana: String,

    pub punto_referencia: String,

    pub x: f64,

    pub y: f64,
}

/// Previsualizaciones activas, una entrada por fila con el toggle
/// ⊙️ encendido (Etapa F) — key = id de CoordenadaBanco. Reemplaza
/// el par CONFIG_PREVIEW (individual, Etapa E) + CONFIG_PREVIEW_GRUPO
/// (indexado por posición, Etapa G) — ya no hace falta distinguir
/// "una" de "un grupo fijo": cada fila abre/cierra su propia entrada
/// de forma independiente y puede haber cualquier cantidad activas
/// a la vez.
static CONFIG_PREVIEWS: std::sync::LazyLock<std::sync::Mutex<HashMap<String, ConfigPreview>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

// Teclas físicamente abajo relevantes al atajo de guardar coordenada
// (ahora AtajoSimple: modificadores + gatillo, no una tecla suelta).
// Registro propio, independiente del de entrada.rs — observar_evento()
// es un tap pasivo y no tiene acceso al estado interno de otros
// archivos.
static TECLAS_ABAJO: std::sync::Mutex<Vec<InputId>> = std::sync::Mutex::new(Vec::new());

/// Llamada al abrir la ventana de captura, con la config de la fila
/// que la abrió.
pub fn activar(ubicacion: String, modo_ventana: String, punto_referencia: String) {
    desactivar_todas_las_previews();

    *ACTIVA.lock().unwrap() = true;
    *GUARDADO_SOLICITADO.lock().unwrap() = false;
    *RESULTADO.lock().unwrap() = None;
    *CONFIG_ACTIVA.lock().unwrap() = Some(ConfigCaptura {
        ubicacion,
        modo_ventana,
        punto_referencia,
    });
}

/// Llamada al cerrar la ventana de captura (Cancelar, guardado, o
/// cierre externo/auto-cancelación por cambio de opción) — deja todo
/// limpio para la próxima apertura.
pub fn desactivar() {
    *ACTIVA.lock().unwrap() = false;
    *GUARDADO_SOLICITADO.lock().unwrap() = false;
    *CONFIG_ACTIVA.lock().unwrap() = None;
}

/// Llamada al abrir la ventana overlay de previsualización de una
/// fila (Etapa F) — mutuamente excluyente con el modo captura de
/// arriba, no con las demás previsualizaciones activas.
pub fn activar_preview(
    id: String,
    ubicacion: String,
    modo_ventana: String,
    punto_referencia: String,
    x: f64,
    y: f64,
) {
    desactivar();

    CONFIG_PREVIEWS.lock().unwrap().insert(
        id,
        ConfigPreview {
            ubicacion,
            modo_ventana,
            punto_referencia,
            x,
            y,
        },
    );
}

/// Llamada al cerrar la ventana overlay de previsualización de una
/// fila puntual.
pub fn desactivar_preview(id: &str) {
    CONFIG_PREVIEWS.lock().unwrap().remove(id);
}

/// Llamada desde comandos.rs::guardar_posicion_preview_coordenada
/// (Regla 17, arrastre del marcador) justo después de persistir en
/// Coordenadas.tsv — sin esto, obtener_config_preview(id) seguiría
/// devolviendo el x/y ANTERIOR al arrastre (CONFIG_PREVIEWS es
/// memoria, independiente del archivo en disco) y el siguiente tick
/// de polling de captura.html haría que el marcador saltara de
/// vuelta a la posición vieja. No-op si la fila ya no tiene una
/// previsualización activa (se cerró la ventana en el ínterin).
pub fn actualizar_xy_preview(id: &str, x: f64, y: f64) {
    if let Some(config) = CONFIG_PREVIEWS.lock().unwrap().get_mut(id) {
        config.x = x;
        config.y = y;
    }
}

/// Llamada al activar el modo captura real (mutuamente excluyente
/// con cualquier previsualización activa).
pub fn desactivar_todas_las_previews() {
    CONFIG_PREVIEWS.lock().unwrap().clear();
}

/// Polling desde comandos.rs: config de previsualización activa para
/// ese id, si hay una. No se consume al leerse — el polling de
/// captura.html la necesita en cada tick.
pub fn obtener_config_preview(id: &str) -> Option<ConfigPreview> {
    CONFIG_PREVIEWS.lock().unwrap().get(id).cloned()
}

/// Consultada una sola vez por captura.html al cargar.
pub fn obtener_config_activa() -> Option<ConfigCaptura> {
    CONFIG_ACTIVA.lock().unwrap().clone()
}

/// Tap pasivo llamado por entrada.rs en CADA evento físico, antes de
/// cualquier otra cosa. Nunca retorna nada que cambie el flujo de
/// entrada.rs — solo observa. Si no hay captura activa, es un chequeo
/// de un bool y listo (costo despreciable).
pub fn observar_evento(evento: &InputEvent) {
    if !*ACTIVA.lock().unwrap() {
        return;
    }

    let mut abajo = TECLAS_ABAJO.lock().unwrap();

    match evento.state {
        InputState::Up => {
            abajo.retain(|i| i != &evento.input);
        }
        InputState::Down => {
            if !abajo.contains(&evento.input) {
                abajo.push(evento.input.clone());
            }

            let atajo = config::tecla_guardar_coordenada();

            let coincide = atajo.gatillo == evento.input
                && atajo.modificadores.len() == abajo.len().saturating_sub(1)
                && atajo
                    .modificadores
                    .iter()
                    .all(|modificador| abajo.contains(modificador));

            if coincide {
                *GUARDADO_SOLICITADO.lock().unwrap() = true;
            }
        }
        InputState::Pulse => {}
    }
}

/// Polling desde la ventana de captura: ¿se pidió guardar? Se
/// consume al leerse (una sola notificación por pulsación real).
pub fn consultar_guardado() -> bool {
    let mut guardado = GUARDADO_SOLICITADO.lock().unwrap();

    let valor = *guardado;
    *guardado = false;
    valor
}

/// La ventana de captura ya resolvió qué significa "guardar ahora"
/// (coordenada absoluta, offset, o porcentaje) y deja el resultado acá.
pub fn guardar_resultado(x: f64, y: f64) {
    *RESULTADO.lock().unwrap() = Some((x, y));
}

/// Polling desde el popup de la fila del perfil: retira el resultado
/// cuando esté listo. Se consume al leerse.
pub fn obtener_resultado() -> Option<(f64, f64)> {
    RESULTADO.lock().unwrap().take()
}

/// La ventana "Coordenadas guardadas" deja acá la coordenada elegida
/// (existente o recién creada) para que el popup de la fila del
/// perfil que abrió esa ventana la retire.
pub fn guardar_seleccion_banco(coordenada: CoordenadaBanco) {
    *SELECCION_BANCO.lock().unwrap() = Some(coordenada);
}

/// Polling desde el popup de la fila del perfil: retira la selección
/// cuando esté lista. Se consume al leerse.
pub fn obtener_seleccion_banco() -> Option<CoordenadaBanco> {
    SELECCION_BANCO.lock().unwrap().take()
}
