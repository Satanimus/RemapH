// ======================================================
// 🔴 Grabacion_Macro
// ------------------------------------------------------
// Captura cruda de la Etapa D del Grabador de Macro. Tap
// pasivo sobre entrada.rs (mismo patrón que
// captura_coordenada.rs): mientras está activa, registra
// cada evento físico tal cual llega, sin retenerlo ni
// bloquearlo — Windows sigue funcionando normal. Registro
// propio de teclas abajo, independiente del de entrada.rs
// y del de captura_coordenada.rs.
// ======================================================

use crate::back_coordenada;
use crate::config;
use crate::eventos::{InputEvent, InputId, InputState};

use serde::Serialize;
use std::sync::Mutex;
use std::time::Instant;

// ======================================================
// 📦 EVENTO GRABADO
// ======================================================

#[derive(Clone, Serialize)]
pub struct EventoGrabado {
    pub input: InputId,

    pub state: InputState,

    pub magnitud: Option<i16>,

    pub momento_ms: u64,

    /// Cursor absoluto de pantalla en el momento del evento. Se
    /// completa en Down/Pulse/Up (Up incluido: es lo que permite
    /// detectar arrastre — Down en un punto, Up en otro distinto).
    pub posicion: Option<(i32, i32)>,

    /// Rect de la ventana activa (x, y, ancho, alto) en el momento
    /// del evento. Se completa en Down/Pulse/Up, igual que posicion.
    pub ventana: Option<(i32, i32, i32, i32)>,
}

static ACTIVA: Mutex<bool> = Mutex::new(false);
static ARMADA: Mutex<bool> = Mutex::new(false);
static INICIO: Mutex<Option<Instant>> = Mutex::new(None);
static EVENTOS: Mutex<Vec<EventoGrabado>> = Mutex::new(Vec::new());
static TECLAS_ABAJO: Mutex<Vec<InputId>> = Mutex::new(Vec::new());

// ======================================================
// 🟡🔴 ESTADO (Etapa G, revisado — botón "Grabar Macro" ya
// no arranca la captura directo: solo arma la escucha de la
// tecla toggle. La propia tecla física decide cuándo pasa de
// Armada a Activa, y de Activa de vuelta a Inactiva).
// ======================================================

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EstadoGrabacion {
    /// Panel de inicio abierto, ventana overlay visible (🟡),
    /// esperando que se presione la tecla toggle para arrancar
    /// de verdad. Los eventos NO se registran todavía.
    Armada,
    /// Tecla toggle presionada estando Armada: grabando de
    /// verdad (🔴), eventos yendo a EVENTOS.
    Activa,
    /// Ni armada ni grabando.
    Inactiva,
}

pub fn estado_grabacion() -> EstadoGrabacion {
    if *ACTIVA.lock().unwrap() {
        EstadoGrabacion::Activa
    } else if *ARMADA.lock().unwrap() {
        EstadoGrabacion::Armada
    } else {
        EstadoGrabacion::Inactiva
    }
}

/// Llamada al abrir el panel de inicio (botón "Grabar Macro"):
/// deja la escucha de la tecla toggle lista, pero sin arrancar
/// la captura todavía — eso lo dispara observar_evento() cuando
/// detecta la tecla mientras está Armada.
pub fn armar_grabacion() {
    *ARMADA.lock().unwrap() = true;
    TECLAS_ABAJO.lock().unwrap().clear();
}

/// Arranca la grabación de verdad (Armada → Activa, disparado
/// desde observar_evento() al detectar la tecla toggle): limpia
/// buffer y registro de teclas abajo, reinicia el reloj de
/// momento_ms.
fn activar_grabacion() {
    *ARMADA.lock().unwrap() = false;
    *ACTIVA.lock().unwrap() = true;
    *INICIO.lock().unwrap() = Some(Instant::now());
    EVENTOS.lock().unwrap().clear();
    TECLAS_ABAJO.lock().unwrap().clear();
}

fn desactivar_interna() {
    *ACTIVA.lock().unwrap() = false;
}

/// Corte forzado desde la UI (Etapa G): el editor lo llama si se
/// cierra (Cancelar/Guardar) mientras el panel de inicio seguía
/// Armada o ya estaba Activa — sin esto, el hook (observar_evento)
/// seguiría escuchando la tecla toggle indefinidamente sin que
/// nadie vaya a leer tomar_eventos(), y la ventana overlay del
/// indicador quedaría huérfana. Cubre ambos estados de una vez
/// (no distingue Armada/Activa porque para la UI el efecto que
/// necesita es el mismo: "cortar todo, ya").
pub fn detener_grabacion() {
    *ARMADA.lock().unwrap() = false;
    desactivar_interna();
}

/// Tap pasivo llamado por entrada.rs en CADA evento físico. Si no
/// hay grabación armada ni activa, es un chequeo de dos bools y
/// listo. Nunca retorna nada que cambie el flujo de entrada.rs.
pub fn observar_evento(evento: &InputEvent) {
    let armada = *ARMADA.lock().unwrap();
    let activa = *ACTIVA.lock().unwrap();

    if !armada && !activa {
        return;
    }

    let mut abajo = TECLAS_ABAJO.lock().unwrap();

    let es_toggle = match evento.state {
        InputState::Up => {
            abajo.retain(|i| i != &evento.input);
            false
        }
        InputState::Down => {
            if abajo.contains(&evento.input) {
                false
            } else {
                abajo.push(evento.input.clone());

                let atajo = config::tecla_grabar_macro();

                atajo.gatillo == evento.input
                    && atajo.modificadores.len() == abajo.len().saturating_sub(1)
                    && atajo
                        .modificadores
                        .iter()
                        .all(|modificador| abajo.contains(modificador))
            }
        }
        InputState::Pulse => false,
    };

    drop(abajo);

    if es_toggle {
        if activa {
            desactivar_interna();
        } else {
            // Armada → Activa: la propia tecla toggle es la que
            // arranca la captura de verdad.
            activar_grabacion();
        }
        return;
    }

    // Armada pero todavía no llegó la tecla toggle: el evento solo
    // sirvió para el bookkeeping de arriba (abajo), no se registra
    // como parte de la grabación.
    if !activa {
        return;
    }

    let inicio_opt: Option<Instant> = *INICIO.lock().unwrap();
    let momento_ms = inicio_opt
        .map(|inicio| inicio.elapsed().as_millis() as u64)
        .unwrap_or(0);

    // Cursor/ventana activa se leen para TODO evento (Down/Pulse/Up)
    // — el Up necesita su propia posición para que el análisis
    // (core_analisis_grabacion.ts) pueda comparar contra la de
    // apertura del grupo y detectar arrastre (Down en un punto, Up
    // en otro).
    let cursor = back_coordenada::obtener_cursor();
    let ventana_activa =
        back_coordenada::obtener_ventana_activa().map(|v| (v.x, v.y, v.ancho, v.alto));

    let (posicion, ventana) = (Some(cursor), ventana_activa);

    EVENTOS.lock().unwrap().push(EventoGrabado {
        input: evento.input.clone(),
        state: evento.state.clone(),
        magnitud: evento.magnitud,
        momento_ms,
        posicion,
        ventana,
    });
}

/// Consumida una sola vez: devuelve el buffer acumulado y lo vacía.
pub fn tomar_eventos() -> Vec<EventoGrabado> {
    std::mem::take(&mut *EVENTOS.lock().unwrap())
}
