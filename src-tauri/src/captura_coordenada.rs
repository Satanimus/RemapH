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
// activar_preview(ubicacion, modo_ventana, punto_referencia, x, y) /
// desactivar_preview() / obtener_config_preview()
//     Modo previsualización individual (Etapa E): independiente del
//     modo captura de arriba, pero mutuamente excluyente con él y
//     con el modo grupo de abajo en la misma ventana overlay —
//     activar_preview()/activar()/activar_preview_grupo() se
//     desactivan entre sí, así nunca coexisten dos modos a la vez.
//     obtener_config_preview() es polling desde comandos.rs::
//     obtener_destino_preview_coordenada, sin consumir el valor (a
//     diferencia de obtener_resultado()).
// activar_preview_grupo(configs) / desactivar_preview_grupo() /
// obtener_config_preview_grupo(indice)
//     Modo previsualización de GRUPO (Etapa G): mismo criterio que
//     el individual de arriba, pero guarda un Vec<ConfigPreview> (una
//     entrada por ventana overlay abierta) en vez de una sola config.
//     obtener_config_preview_grupo(indice) es polling desde
//     comandos.rs::obtener_destino_preview_grupo, una consulta por
//     ventana overlay (cada una con su propio índice).
// ======================================================

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

/// Config del modo previsualización (Etapa E) — independiente del
/// resto del estado de arriba (modo captura), mismo vocabulario de
/// strings que ConfigCaptura.
#[derive(Clone)]
pub struct ConfigPreview {
    pub ubicacion: String,

    pub modo_ventana: String,

    pub punto_referencia: String,

    pub x: f64,

    pub y: f64,
}

static CONFIG_PREVIEW: std::sync::Mutex<Option<ConfigPreview>> = std::sync::Mutex::new(None);

/// Estado del modo previsualización de GRUPO (Etapa G) — independiente
/// del CONFIG_PREVIEW individual de arriba (Etapa E), que sigue
/// intacto para la previsualización de una sola fila.
static CONFIG_PREVIEW_GRUPO: std::sync::Mutex<Vec<ConfigPreview>> = std::sync::Mutex::new(Vec::new());

// Teclas físicamente abajo relevantes al atajo de guardar coordenada
// (ahora AtajoSimple: modificadores + gatillo, no una tecla suelta).
// Registro propio, independiente del de entrada.rs — observar_evento()
// es un tap pasivo y no tiene acceso al estado interno de otros
// archivos.
static TECLAS_ABAJO: std::sync::Mutex<Vec<InputId>> = std::sync::Mutex::new(Vec::new());

/// Llamada al abrir la ventana de captura, con la config de la fila
/// que la abrió.
pub fn activar(ubicacion: String, modo_ventana: String, punto_referencia: String) {
    desactivar_preview();
    desactivar_preview_grupo();

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

/// Llamada al abrir la ventana overlay en modo previsualización
/// (Etapa E) — mutuamente excluyente con el modo captura de arriba.
pub fn activar_preview(
    ubicacion: String,
    modo_ventana: String,
    punto_referencia: String,
    x: f64,
    y: f64,
) {
    desactivar();
    desactivar_preview_grupo();

    *CONFIG_PREVIEW.lock().unwrap() = Some(ConfigPreview {
        ubicacion,
        modo_ventana,
        punto_referencia,
        x,
        y,
    });
}

/// Llamada al cerrar la ventana overlay en modo previsualización.
pub fn desactivar_preview() {
    *CONFIG_PREVIEW.lock().unwrap() = None;
}

/// Polling desde comandos.rs: config de previsualización activa, si
/// hay una. No se consume al leerse (a diferencia de obtener_resultado()) —
/// el polling de captura.html la necesita en cada tick.
pub fn obtener_config_preview() -> Option<ConfigPreview> {
    CONFIG_PREVIEW.lock().unwrap().clone()
}

/// Llamada al abrir las ventanas overlay en modo previsualización de
/// GRUPO (Etapa G) — mutuamente excluyente con el modo captura y con
/// la previsualización individual de arriba.
pub fn activar_preview_grupo(configs: Vec<ConfigPreview>) {
    desactivar();
    desactivar_preview();

    *CONFIG_PREVIEW_GRUPO.lock().unwrap() = configs;
}

/// Llamada al cerrar las ventanas overlay en modo previsualización de
/// grupo.
pub fn desactivar_preview_grupo() {
    CONFIG_PREVIEW_GRUPO.lock().unwrap().clear();
}

/// Polling desde comandos.rs: config de previsualización de grupo en
/// la posición `indice`, si existe. No se consume al leerse — mismo
/// criterio que obtener_config_preview().
pub fn obtener_config_preview_grupo(indice: usize) -> Option<ConfigPreview> {
    CONFIG_PREVIEW_GRUPO.lock().unwrap().get(indice).cloned()
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
