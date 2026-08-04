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
// activar() / desactivar()
//     Se llaman al abrir/cerrar la ventana de captura.
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
// ======================================================

use crate::config;
use crate::eventos::{InputEvent, InputId, InputState};

static ACTIVA: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
static GUARDADO_SOLICITADO: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
static RESULTADO: std::sync::Mutex<Option<(f64, f64)>> = std::sync::Mutex::new(None);

/// Llamada al abrir la ventana de captura.
pub fn activar() {
    *ACTIVA.lock().unwrap() = true;
    *GUARDADO_SOLICITADO.lock().unwrap() = false;
    *RESULTADO.lock().unwrap() = None;
}

/// Llamada al cerrar la ventana de captura (Cancelar, guardado, o
/// cierre externo) — deja todo limpio para la próxima apertura.
pub fn desactivar() {
    *ACTIVA.lock().unwrap() = false;
    *GUARDADO_SOLICITADO.lock().unwrap() = false;
}

/// Tap pasivo llamado por entrada.rs en CADA evento físico, antes de
/// cualquier otra cosa. Nunca retorna nada que cambie el flujo de
/// entrada.rs — solo observa. Si no hay captura activa, es un chequeo
/// de un bool y listo (costo despreciable).
pub fn observar_evento(evento: &InputEvent) {
    if !*ACTIVA.lock().unwrap() {
        return;
    }

    if evento.state != InputState::Down {
        return;
    }

    let tecla = InputId::new("keyboard", &config::tecla_guardar_coordenada());

    if evento.input == tecla {
        *GUARDADO_SOLICITADO.lock().unwrap() = true;
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
