// ======================================================
// 🚪 Entrada RemapH V3
// ======================================================
// 1. ¿Qué hace este archivo?
//
// El portero: recibe cada InputEvent físico del backend
// (back_interception), se lo entrega al AnalizadorTrigger,
// y según la ResolucionEntrada que termine llegando de
// Cache, decide si el input vuelve a Windows, se bloquea,
// o queda pendiente.
//
// Mantiene, por cada "grupo" físico en curso (ver
// analizador_trigger.rs), un estado de 3 posibles valores
// y un buffer de eventos retenidos:
//
// NORMAL — sin nada pendiente. Comportamiento de siempre.
// RETENIENDO — hay un input bloqueado esperando a ver si
//     termina siendo un match o no.
// DEVOLVIENDO_REPEATS — ya se determinó que NO hubo match;
//     se están reinyectando los eventos retenidos, y
//     además hay que seguir dejando pasar (sin analizar)
//     los repeats/Ups de esas mismas teclas hasta que se
//     suelten todas.
//
// Una tecla físicamente nueva, sin relación con el grupo
// que está en curso, no espera a que ese grupo termine:
// arranca su propio seguimiento en paralelo (su propio
// hilo, su propio estado y buffer).
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// back_interception::iniciar() — le entrega cada
//     InputEvent físico capturado.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Cada InputEvent (Down/Up/Pulse) tal como lo entrega el
// backend.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// No retorna nada — actúa directo llamando a
// back_interception::emitir_evento() cuando corresponde
// dejar pasar algo.
// ------------------------------------------------------
// 5. Comportamiento por estado
//
// NORMAL (por grupo)
//   Se manda el evento al analizador.
//   - None (repeat filtrado) → nada que hacer, seguir
//     NORMAL.
//   - Some(Pasar) → emitir el evento tal cual a Windows.
//     Seguir NORMAL.
//   - Some(Retener) → bloquear (no emitir). Guardar en el
//     buffer del grupo (simplificado: solo identidad y
//     orden — Down/Up y qué tecla, SIN el instante real,
//     para no reproducir después la espera de análisis).
//     Pasar a RETENIENDO.
//   - Some(Consumir) → hubo match. No se emite nada por
//     este camino (la salida remapeada la maneja Runtime
//     aparte). Nada que retener. Seguir NORMAL.
//
// RETENIENDO (por grupo)
//   Se sigue mandando cada evento nuevo al analizador.
//   - None o Some(Retener) → bloquear y agregar al buffer
//     (sin duplicar si ya es un repeat reflejado). Seguir
//     RETENIENDO.
//   - Some(Consumir) → hubo match real. Se DESCARTA el
//     buffer entero sin reinyectar nada (esos eventos ya
//     fueron el match). Volver a NORMAL.
//   - Some(Pasar) → vaciar el buffer completo,
//     reinyectando cada evento guardado en el mismo orden
//     en que ocurrieron, sin ningún delay artificial entre
//     ellos (vía back_interception::emitir_evento). Pasar
//     a DEVOLVIENDO_REPEATS.
//
// DEVOLVIENDO_REPEATS (por grupo)
//   Ya NO se manda nada al analizador para las teclas de
//   este grupo. Cada Down repetido o Up de esas teclas
//   puntuales se deja pasar directo a Windows, sin
//   análisis. Se lleva la cuenta de qué teclas del grupo
//   siguen sin soltarse; cuando llega el Up de la última
//   que faltaba, el grupo se da por terminado y se
//   descarta (vuelve a no existir — la próxima vez que
//   aparezca cualquiera de esas teclas, es un grupo nuevo,
//   en NORMAL).
// ------------------------------------------------------
// 6. Funciones del archivo
//
// procesar_evento(evento: InputEvent)
//     Punto de entrada único. Identifica a qué grupo
//     pertenece el evento (o crea uno nuevo), y aplica el
//     comportamiento según el estado de ese grupo (ver
//     punto 5).
// consumir()
//     No hace nada — el input físico simplemente no se
//     emite. (Puede quedar vacía, es intencional.)
// devolver(evento: InputEvent)
//     Emite el evento tal cual a Windows vía
//     back_interception::emitir_evento().
// reinyectar_buffer(buffer: Vec<InputEvent>)
//     Emite, en orden y sin delay, todos los eventos
//     retenidos de un grupo. Deja ese grupo en
//     DEVOLVIENDO_REPEATS.
// ------------------------------------------------------
// Transformación:
//
// InputEvent físico
//     ↓
// AnalizadorTrigger.procesar()
//     ↓
// (implícito, vía Cache) ResolucionEntrada
//     ↓
// NORMAL / RETENIENDO / DEVOLVIENDO_REPEATS (por grupo)
//     ↓
// devolver() | consumir() | reinyectar_buffer()
// ======================================================

use crate::analizador_trigger;
use crate::back_interception;
use crate::eventos::{InputEvent, InputId, InputState};
use std::cell::RefCell;
use std::sync::Mutex;

enum Fase {
    Reteniendo,
    Devolviendo,
}

struct GrupoPendiente {
    fase: Fase,
    buffer: Vec<InputEvent>,
    faltan_soltar: Vec<InputId>,
}

static PENDIENTE: Mutex<Option<GrupoPendiente>> = Mutex::new(None);

thread_local! {
    // El evento que está siendo procesado ahora mismo en este hilo.
    // Cache lo necesita indirectamente: cuando llama a retener() (sin
    // pasarle el evento, por diseño), acá es donde lo recuperamos para
    // sembrar el buffer del grupo recién creado.
    static EVENTO_EN_CURSO: RefCell<Option<InputEvent>> = RefCell::new(None);
}

pub fn procesar_evento(evento: InputEvent) {
    let mut pendiente = PENDIENTE.lock().unwrap();

    if let Some(grupo) = pendiente.as_mut() {
        match grupo.fase {
            Fase::Devolviendo => {
                // Ya no hay análisis para estas teclas: pasa directo.
                back_interception::emitir_evento(evento.clone());

                if evento.state == InputState::Up {
                    grupo.faltan_soltar.retain(|i| i != &evento.input);
                }

                if grupo.faltan_soltar.is_empty() {
                    *pendiente = None;
                }
                return;
            }
            Fase::Reteniendo => {
                grupo.buffer.push(evento.clone());
                drop(pendiente);
                analizador_trigger::procesar_evento_runtime(evento);
                return;
            }
        }
    }

    drop(pendiente);

    EVENTO_EN_CURSO.with(|c| *c.borrow_mut() = Some(evento.clone()));
    analizador_trigger::procesar_evento_runtime(evento);
    EVENTO_EN_CURSO.with(|c| *c.borrow_mut() = None);
}

/// Llamada por Cache (síncrona o desde el timer): no hay match posible.
/// Si había un buffer retenido, se reinyecta todo en orden, sin delay,
/// y queda pendiente de que se suelten esas teclas antes de volver a
/// analizar con normalidad.
pub fn pasar() {
    let mut pendiente = PENDIENTE.lock().unwrap();

    let Some(grupo) = pendiente.take() else {
        // Nada retenido: es el evento que está en curso ahora mismo.
        drop(pendiente);
        EVENTO_EN_CURSO.with(|c| {
            if let Some(evento) = c.borrow().clone() {
                back_interception::emitir_evento(evento);
            }
        });
        return;
    };

    let mut faltan_soltar: Vec<InputId> = Vec::new();

    for evento in grupo.buffer {
        back_interception::emitir_evento(evento.clone());

        match evento.state {
            InputState::Down => {
                if !faltan_soltar.contains(&evento.input) {
                    faltan_soltar.push(evento.input);
                }
            }
            InputState::Up => faltan_soltar.retain(|i| i != &evento.input),
            InputState::Pulse => {}
        }
    }

    if faltan_soltar.is_empty() {
        return;
    }

    *pendiente = Some(GrupoPendiente {
        fase: Fase::Devolviendo,
        buffer: Vec::new(),
        faltan_soltar,
    });
}

/// Llamada por Cache: todavía puede llegar a ser un match. Bloquea el
/// evento en curso y abre (si no existía) el grupo retenido.
pub fn retener() {
    let mut pendiente = PENDIENTE.lock().unwrap();

    if pendiente.is_none() {
        let evento_inicial = EVENTO_EN_CURSO.with(|c| c.borrow().clone());

        *pendiente = Some(GrupoPendiente {
            fase: Fase::Reteniendo,
            buffer: evento_inicial.into_iter().collect(),
            faltan_soltar: Vec::new(),
        });
    }
}

/// Llamada por Cache: hubo match real y ya se avisó a Runtime. Lo
/// retenido (si algo había) YA fue el match — se descarta sin
/// reinyectar nada.
pub fn consumir() {
    *PENDIENTE.lock().unwrap() = None;
}
