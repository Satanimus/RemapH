// ======================================================
// 🗃️ Cache RemapH V3
// ======================================================
// 1. ¿Qué hace este archivo?
//
// Mantiene en memoria los remapeos compilados y el estado
// de aplicaciones. Su trabajo central es recibir Downs
// desde AnalizadorTrigger (modo Runtime), decidir si eso
// coincide con algún remapeo, y avisarle a entrada.rs qué
// hacer con el input físico (dejarlo pasar, retenerlo, o
// darlo por consumido).
//
// Mantiene DOS memorias separadas y de vida distinta:
//
// a) Lista de comparación — se arma con los Down que le
//    llegan del analizador. Se vacía POR COMPLETO cada vez
//    que Cache resuelve algo (Pasar, Consumir, o al
//    confirmar un match vía condición).
//
// b) Ninguna memoria propia de "qué sigue presionado" —
//    en vez de eso, la CONSULTA puntualmente al analizador
//    (obtener_presionados()) justo cuando necesita
//    reiniciar su lista de comparación tras resolver. Así
//    hereda lo que sigue físicamente abajo (soporta
//    Ctrl+C → Ctrl+V: al resolver Ctrl+C, la nueva lista
//    arranca ya sabiendo que Ctrl sigue presionado).
//
// Debe soportar más de una lista de comparación en
// simultáneo, una por cada "grupo" físico independiente
// que el analizador le esté reportando a la vez (ver
// analizador_trigger.rs).
//
// Cache NO conoce Runtime más allá de mandarle órdenes. No
// conoce Captura ni perfil_ui.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// entrada.rs — le entrega cada Down (a través del
//     analizador) y actúa según la ResolucionEntrada que
//     Cache le devuelve.
// AnalizadorTrigger — le entrega el resultado del timer
//     (CondicionTrigger) cuando lo pidió.
// Compilador — le entrega los remapeos al compilar.
// back_app — le avisa cambios de foco de ventana.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// recibir_down(input: InputId) — un Down nuevo (no
//     repeat) del analizador.
// recibir_condicion(condicion: CondicionTrigger) —
//     resultado del timer que Cache había pedido.
// recibir_up(input: InputId) — SOLO llega durante la
//     ventana de espera de un Mantenido activo (ver
//     analizador_trigger.rs) — se usa para detectar
//     "se perdió el match".
// escribir_cache() / escribir_fila() / borrar_* — desde
//     Compilador.
// actualizar_estado_app() — desde back_app.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// ResolucionEntrada, a entrada.rs, con cada Down:
//   Pasar — no hay ningún match posible con esto. Se
//     vacía la lista de comparación y se le ordena
//     limpiar() al analizador.
//   Retener — todavía puede llegar a ser un match (sigue
//     habiendo posibles, o se está esperando la
//     condición). No se avisa nada a Runtime todavía.
//   Consumir — ya hubo match confirmado y se le avisó a
//     Runtime (Iniciar, y si corresponde, Finalizar).
//
// A Runtime: OrdenRuntime::Iniciar{id, accion, extra} y
//     OrdenRuntime::Detener{id}.
// ------------------------------------------------------
// 5. Reglas de resolución, por cada Down nuevo
//
// 1. Se agrega a la lista de comparación de su grupo.
// 2. Se calculan posibles/exactas contra las filas
//    habilitadas (filtradas por app activa).
// 3. posibles == 0
//        → Pasar. Vaciar lista. limpiar() al analizador.
// 4. posibles == exactas == 1, y la fila candidata es
//    Simple
//        → Iniciar + Finalizar juntos a Runtime (sin
//          pedir condición, no hace falta desambiguar).
//          Vaciar lista y reiniciarla con
//          obtener_presionados().
// 5. posibles == exactas (≥1), y la(s) candidata(s) no
//    son trivialmente Simple (una sola candidata que no es
//    Simple, o varias con distinta condición)
//        → Retener. Pedirle iniciar_timer() al analizador
//          sobre la tecla candidata a gatillo, si no se
//          le pidió ya para esta misma situación.
// 6. posibles > exactas (sigue habiendo prefijos posibles
//    sin match exacto todavía)
//        → Retener. Seguir esperando más Downs.
// ------------------------------------------------------
// 6. Al recibir la condición del timer (recibir_condicion)
//
// - Si la condición recibida coincide con una fila
//   candidata exacta:
//     - Simple o Doble → Iniciar + Finalizar juntos a
//       Runtime. Vaciar lista, reiniciar con
//       obtener_presionados().
//     - Mantenido → Iniciar (sin Finalizar) a Runtime.
//       Queda "esperando finalizar" para ese id — a partir
//       de acá empieza a recibir recibir_up() del
//       analizador.
// - Si no coincide con ninguna candidata → Pasar. Vaciar
//   lista. limpiar() al analizador.
// ------------------------------------------------------
// 7. Mientras está "esperando finalizar" un Mantenido
//
// Cada recibir_up(input) que llegue: si ese input formaba
// parte de la entrada que generó el match activo → se
// perdió el match. Se manda Finalizar a Runtime, se vacía
// la lista, se ordena limpiar() al analizador, y se
// reinicia la lista con obtener_presionados().
// ------------------------------------------------------
// 8. Funciones del archivo
//
// recibir_down(input: InputId) -> ResolucionEntrada
//     Punto de entrada principal, ver reglas 1 a 6.
// recibir_condicion(condicion: CondicionTrigger)
//     -> ResolucionEntrada
//     Ver regla 6.
// recibir_up(input: InputId)
//     Ver regla 7. No devuelve ResolucionEntrada — actúa
//     directo sobre Runtime si corresponde.
// escribir_cache() / escribir_fila() / borrar_cache() /
// borrar_fila()
//     Igual que antes, sin cambios de diseño.
// actualizar_estado_app() / app_habilitada()
//     Igual que antes, sin cambios de diseño.
// ------------------------------------------------------
// Transformación:
//
// Down (AnalizadorTrigger)
//     ↓
// Lista de comparación del grupo
//     ↓
// posibles / exactas contra filas habilitadas
//     ↓
// Pasar | Retener (+ timer si hace falta) | Consumir
//     ↓
// Runtime (Iniciar / Finalizar)
// ======================================================

use crate::eventos::InputId;
use crate::perfil_cache::{AccionCache, AppCache, CondicionTrigger, ExtraCache, RemapeoCache};
use crate::{analizador_trigger, entrada, runtime};
use std::sync::Mutex;

#[derive(Clone)]
struct Lista {
    entrada: Vec<InputId>,
    esperando_condicion: bool,
}

struct InstanciaActiva {
    id: String,
    entrada: Vec<InputId>,
}

#[derive(Clone, PartialEq)]
pub struct AppEstadoCache {
    pub app: AppCache,
    pub activa: bool,
}

static CACHE: Mutex<Vec<RemapeoCache>> = Mutex::new(Vec::new());
static APPS: Mutex<Vec<AppEstadoCache>> = Mutex::new(Vec::new());
static LISTAS: Mutex<Vec<Lista>> = Mutex::new(Vec::new());
static ACTIVAS: Mutex<Vec<InstanciaActiva>> = Mutex::new(Vec::new());
static PREGUNTA_PENDIENTE: Mutex<Option<usize>> = Mutex::new(None);

pub fn escribir_cache(remapeos: Vec<RemapeoCache>) {
    *CACHE.lock().unwrap() = remapeos;
}

pub fn escribir_fila(remapeo: RemapeoCache) {
    CACHE.lock().unwrap().push(remapeo);
}

pub fn borrar_cache() {
    CACHE.lock().unwrap().clear();
}

pub fn borrar_fila(id: &str) {
    CACHE.lock().unwrap().retain(|r| r.id != id);
}

pub fn actualizar_estado_app(app: AppCache, activa: bool) {
    let mut apps = APPS.lock().unwrap();
    if let Some(e) = apps.iter_mut().find(|e| e.app == app) {
        e.activa = activa;
    } else {
        apps.push(AppEstadoCache { app, activa });
    }
}

fn app_habilitada(app: &AppCache, apps: &[AppEstadoCache]) -> bool {
    apps.iter()
        .find(|e| &e.app == app)
        .map(|e| e.activa)
        .unwrap_or(true)
}

/// Cuenta posibles/exactas de `entrada` contra la cache. exactas ignora
/// la condición de la fila (eso se filtra aparte, según necesite).
fn contar(entrada: &[InputId]) -> (usize, usize, Vec<RemapeoCache>) {
    let cache = CACHE.lock().unwrap();
    let apps = APPS.lock().unwrap();

    let mut posibles = 0;
    let mut candidatas = Vec::new();

    for fila in cache.iter() {
        if !app_habilitada(&fila.trigger.app, &apps) {
            continue;
        }
        if fila.trigger.entrada.starts_with(entrada) {
            posibles += 1;
        }
        if fila.trigger.entrada.as_slice() == entrada {
            candidatas.push(fila.clone());
        }
    }

    (posibles, candidatas.len(), candidatas)
}

/// Nunca devuelve nada — avisa directo a entrada.rs (pasar / retener /
/// consumir), igual que recibir_condicion(). Ningún componente pregunta,
/// quien tiene la respuesta avisa.
pub fn recibir_down(input: InputId) {
    let mut listas = LISTAS.lock().unwrap();

    // ¿Alguna lista existente acepta esto como continuación válida?
    let mut indice = None;
    for (i, lista) in listas.iter().enumerate() {
        let mut probable = lista.entrada.clone();
        probable.push(input.clone());
        let (posibles, _, _) = contar(&probable);
        if posibles > 0 {
            indice = Some(i);
            break;
        }
    }

    let indice = match indice {
        Some(i) => {
            listas[i].entrada.push(input.clone());
            i
        }
        None => {
            listas.push(Lista {
                entrada: vec![input.clone()],
                esperando_condicion: false,
            });
            listas.len() - 1
        }
    };

    let entrada_actual = listas[indice].entrada.clone();
    drop(listas);

    let (posibles, exactas, candidatas) = contar(&entrada_actual);

    if posibles == 0 {
        limpiar_lista(indice);
        analizador_trigger::limpiar();
        entrada::pasar();
        return;
    }

    if posibles == exactas
        && exactas == 1
        && candidatas[0].trigger.condicion == CondicionTrigger::Simple
    {
        iniciar_y_finalizar(candidatas[0].clone());
        limpiar_lista(indice);
        reiniciar_desde_presionados();
        entrada::consumir();
        return;
    }

    if posibles == exactas && exactas >= 1 {
        marcar_esperando_condicion(indice);
        let gatillo = entrada_actual.last().cloned().unwrap();
        analizador_trigger::iniciar_timer(gatillo);
        entrada::retener();
        return;
    }

    entrada::retener();
}

/// Llamada por el timer del analizador (hilo aparte). No hay nadie
/// esperando un valor de retorno: avisa directo a quien corresponda.
pub fn recibir_condicion(condicion: CondicionTrigger) {
    let indice = match *PREGUNTA_PENDIENTE.lock().unwrap() {
        Some(i) => i,
        None => return,
    };

    let entrada_actual = {
        let listas = LISTAS.lock().unwrap();
        match listas.get(indice) {
            Some(l) => l.entrada.clone(),
            None => return,
        }
    };

    let (posibles, exactas, candidatas) = contar(&entrada_actual);
    let match_final = candidatas
        .into_iter()
        .find(|c| c.trigger.condicion == condicion);

    match match_final {
        Some(remapeo) if posibles == exactas => {
            if condicion == CondicionTrigger::Mantenido {
                iniciar_solamente(remapeo, entrada_actual);
            } else {
                iniciar_y_finalizar(remapeo);
            }
            limpiar_lista(indice);
            reiniciar_desde_presionados();
            entrada::consumir();
        }
        _ => {
            limpiar_lista(indice);
            analizador_trigger::limpiar();
            entrada::pasar();
        }
    }
}

/// Solo llega mientras hay una instancia Mantenido activa esperando su
/// Up (ver analizador_trigger.rs, reenviando_ups).
pub fn recibir_up(input: InputId) {
    let mut activas = ACTIVAS.lock().unwrap();
    let Some(pos) = activas.iter().position(|a| a.entrada.contains(&input)) else {
        return;
    };

    let instancia = activas.remove(pos);
    drop(activas);

    runtime::ejecutar(runtime::OrdenRuntime::Detener { id: instancia.id });
    analizador_trigger::limpiar();
    reiniciar_desde_presionados();
}

fn iniciar_y_finalizar(remapeo: RemapeoCache) {
    runtime::ejecutar(runtime::OrdenRuntime::Iniciar {
        id: remapeo.id.clone(),
        accion: remapeo.accion,
        extra: remapeo.extra,
    });
    runtime::ejecutar(runtime::OrdenRuntime::Detener { id: remapeo.id });
}

fn iniciar_solamente(remapeo: RemapeoCache, entrada: Vec<InputId>) {
    ACTIVAS.lock().unwrap().push(InstanciaActiva {
        id: remapeo.id.clone(),
        entrada,
    });
    runtime::ejecutar(runtime::OrdenRuntime::Iniciar {
        id: remapeo.id,
        accion: remapeo.accion,
        extra: remapeo.extra,
    });
}

fn marcar_esperando_condicion(indice: usize) {
    if let Some(l) = LISTAS.lock().unwrap().get_mut(indice) {
        l.esperando_condicion = true;
    }
    *PREGUNTA_PENDIENTE.lock().unwrap() = Some(indice);
}

fn limpiar_lista(indice: usize) {
    let mut listas = LISTAS.lock().unwrap();
    if indice < listas.len() {
        listas.remove(indice);
    }
    let mut pregunta = PREGUNTA_PENDIENTE.lock().unwrap();
    if *pregunta == Some(indice) {
        *pregunta = None;
    }
}

/// Hereda lo que sigue físicamente presionado (soporta Ctrl+C -> Ctrl+V).
fn reiniciar_desde_presionados() {
    let presionados = analizador_trigger::obtener_presionados();
    if !presionados.is_empty() {
        LISTAS.lock().unwrap().push(Lista {
            entrada: presionados,
            esperando_condicion: false,
        });
    }
}
