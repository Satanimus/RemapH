// ======================================================
// 🗃️ Cache
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
//    confirmar un match vía condición). Cada lista tiene un
//    ID propio (contador incremental, no su posición en el
//    Vec) — importante: si se identificara por posición y
//    otra lista (de otro grupo físico) se eliminara del medio
//    mientras esta seguía esperando su timer, la posición de
//    ESTA cambiaría por debajo suyo sin que nadie se entere,
//    y el timer terminaría resolviendo (o no encontrando) la
//    lista equivocada — dejando el RETENIDO de entrada.rs
//    abierto para siempre (ver su red de seguridad). Con ID
//    fijo, ninguna remoción ajena puede afectar a esta lista.
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
// recibir_up(input: InputId) — llega con CADA Up real (el
//     analizador ya no filtra por ventana de Mantenido). Si
//     hay una instancia Mantenido esperando ese Up, se usa
//     para detectar "se perdió el match"; si no, para
//     mantener sincronizadas las listas de comparación en
//     reposo (sacar una tecla que ya se soltó, ver la
//     función).
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
//        → Resuelve sin pedir condición (no hace falta
//          desambiguar). Igual que en recibir_condicion: si
//          el Extra de la fila requiere_up_real() (Turbo,
//          Mantener, Click Sostenido) → Iniciar sin
//          Finalizar, queda esperando recibir_up(); si no →
//          Iniciar + Finalizar juntos. Vaciar lista y
//          reiniciarla con obtener_presionados().
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
//     - Si el Extra de esa fila requiere Up real
//       (ExtraCache::requiere_up_real — Turbo, Mantener,
//       Click Sostenido) → Iniciar (sin Finalizar) a
//       Runtime. Queda "esperando finalizar" para ese id —
//       a partir de acá empieza a recibir recibir_up() del
//       analizador. Esto NO depende de qué Condición lo
//       disparó (Simple/Doble/Mantenido): un Mantenido sin
//       Extra de este tipo igual se Finaliza de una — lo que
//       importa es si el Extra necesita que alguien le avise
//       cuándo soltar, no cómo se armó el trigger.
//     - Si no, Iniciar + Finalizar juntos a Runtime. Vaciar
//       lista, reiniciar con obtener_presionados().
// - Si no coincide con ninguna candidata → Pasar. Vaciar
//   lista. limpiar() al analizador.
// ------------------------------------------------------
// 7. Cada Up real que llega (recibir_up)
//
// - Si formaba parte de la entrada de un Mantenido activo
//   esperando finalizar → se perdió el match. Se manda
//   Finalizar a Runtime, se ordena limpiar() al analizador,
//   y se reinicia la lista (fantasma) con obtener_presionados().
// - Si no, y esa tecla está en una lista FANTASMA (sembrada por
//   reiniciar_desde_presionados(), sin nadie esperando en
//   entrada.rs) → se descarta la lista ENTERA, no solo la
//   tecla. Un resto recortado no representa ninguna decisión
//   real (ver Lista::fantasma).
// - Si no, y esa tecla está en una lista EN CONSTRUCCIÓN (no
//   fantasma, no esperando_condicion; entrada.rs tiene un
//   RETENIDO abierto sobre ella) → se resuelve SIEMPRE algo:
//     - Si la entrada de antes de soltar esta tecla ya matcheaba
//       exacto una candidata Simple → se resuelve retroactivamente
//       como ese match (Iniciar+Finalizar), aunque el Extra pida
//       up real (ese Up ya pasó).
//     - Si no, se aborta todo (Pasar) y lo que siga físicamente
//       presionado se resiembra como lista fantasma nueva.
// - Las listas esperando_condicion nunca se tocan acá — se
//   resuelven solas por recibir_condicion().
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
// obtener_remapeo(id) -> Option<RemapeoCache>
//     Busca una fila ya compilada por id (clon, de solo
//     lectura). Etapa 7 de MenuExpress: back_menu_express.rs
//     la usa para resolver qué Acción/Extra ejecutar cuando
//     se hace clic en un botón de un menú.
// esta_vacia()
//     true si no quedó ningún remapeo compilado (perfil
//     vacío o todo OFF). La consulta perfil.rs para
//     informar cache_activo a la UI.
// hay_candidata_para()
//     true si un InputId podría continuar ahora mismo algún
//     trigger posible (de solo lectura, no toca LISTAS). La
//     consulta AnalizadorTrigger para la rueda del mouse:
//     solo agrupa sus pulsos en ráfagas cuando hace falta
//     resolver Simple/Mantenido para un candidato real.
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
use crate::perfil_cache::{
    AccionCache, AppCache, CondicionTrigger, CoordenadaCache, ExtraCache, RemapeoCache,
};
use crate::{analizador_trigger, entrada, runtime};
use std::sync::Mutex;

#[derive(Clone)]
struct Lista {
    // Identidad estable de esta lista — nunca cambia con el tiempo,
    // a diferencia de su posición en LISTAS (ver header, punto 1a).
    id: u64,

    entrada: Vec<InputId>,

    esperando_condicion: bool,

    // true = "fantasma": sembrada por reiniciar_desde_presionados()
    // después de que un match (o un Pasar) YA se resolvió, puramente
    // especulativa (soporta encadenar modificadores, ej. Ctrl+C ->
    // Ctrl+V). entrada.rs NO tiene ningún RETENIDO abierto por esta
    // lista — ya recibió su pasar()/consumir() antes de que esta
    // lista naciera.
    //
    // false = "en construcción": nació (o fue extendida) por un Down
    // real llegando ahora mismo vía recibir_down(), y entrada.rs SÍ
    // tiene un RETENIDO abierto esperando que esta lista se resuelva
    // (ver recibir_down(), siempre termina en entrada::retener() para
    // este caso).
    //
    // Una fantasma pasa a false en cuanto un Down real la extiende
    // (recibir_down encuentra que puede continuarla) — a partir de
    // ahí entrada.rs vuelve a tener un RETENIDO sobre ella. Ver
    // recibir_up() para por qué esta distinción es necesaria (bugs 1
    // y 3): a una lista en construcción hay que avisarle SIEMPRE algo
    // a entrada.rs cuando se suelta una de sus teclas (o resuelve, o
    // pasa); a una fantasma no hay que avisarle nada (nadie espera),
    // pero tampoco hay que dejarle un resto recortado dando vueltas.
    fantasma: bool,
}

struct InstanciaActiva {
    id: String,
    entrada: Vec<InputId>,
}

/// Orden que Cache le manda a Runtime para iniciar o detener la
/// ejecución de una acción.
pub enum OrdenRuntime {
    Iniciar {
        id: String,
        accion: AccionCache,
        extra: Option<ExtraCache>,
        coordenada: Option<CoordenadaCache>,
    },
    Detener {
        id: String,
    },
}

#[derive(Clone, PartialEq)]
pub struct AppEstadoCache {
    pub app: AppCache,
    pub activa: bool,
}

static CACHE: Mutex<Vec<RemapeoCache>> = Mutex::new(Vec::new());
static APPS: Mutex<Vec<AppEstadoCache>> = Mutex::new(Vec::new());
static LISTAS: Mutex<Vec<Lista>> = Mutex::new(Vec::new());
static SIGUIENTE_ID_LISTA: Mutex<u64> = Mutex::new(0);
static ACTIVAS: Mutex<Vec<InstanciaActiva>> = Mutex::new(Vec::new());
static PREGUNTA_PENDIENTE: Mutex<Option<u64>> = Mutex::new(None);

/// Da de alta un ID nuevo, único y estable, para una lista. Nunca se
/// reutiliza ni se reordena — es lo que reemplaza a la posición en el
/// Vec como identidad (ver header, punto 1a).
fn nuevo_id_lista() -> u64 {
    let mut id = SIGUIENTE_ID_LISTA.lock().unwrap();
    *id += 1;
    *id
}

pub fn escribir_cache(remapeos: Vec<RemapeoCache>) {
    *CACHE.lock().unwrap() = remapeos;
}

pub fn escribir_fila(remapeo: RemapeoCache) {
    CACHE.lock().unwrap().push(remapeo);
}

pub fn borrar_cache() {
    CACHE.lock().unwrap().clear();
}

/// true si el perfil compilado no dejó ningún remapeo activo (perfil
/// vacío, o todas sus filas en estado != "ON"). Lo consulta perfil.rs
/// justo después de compilar, para informarle a la UI si el perfil
/// actual tiene algo funcionando o no (cache_activo).
pub fn esta_vacia() -> bool {
    CACHE.lock().unwrap().is_empty()
}

pub fn borrar_fila(id: &str) {
    CACHE.lock().unwrap().retain(|r| r.id != id);
}

/// Busca una fila ya compilada por su id — usado por
/// back_menu_express.rs (etapa 7) para resolver qué Acción/Extra
/// ejecutar cuando se hace clic en un botón DE ADENTRO de un menú
/// (el fila_id guardado en cada MenuBotonCache, ver perfil_cache.rs).
/// Clone porque OrdenRuntime::Iniciar necesita quedarse con su propia
/// copia de accion/extra/coordenada — el lock no puede quedar tomado
/// mientras Runtime hace su trabajo.
pub fn obtener_remapeo(id: &str) -> Option<RemapeoCache> {
    CACHE.lock().unwrap().iter().find(|r| r.id == id).cloned()
}

/// true si `input` podría continuar AHORA MISMO algún trigger posible
/// — considerando las listas en curso (modificadores ya presionados en
/// una secuencia sin resolver todavía) igual que recibir_down(), pero
/// de solo lectura: no crea ni modifica ninguna lista, no llama a
/// entrada.rs. Lo usa AnalizadorTrigger para la rueda del mouse (ver
/// analizador_trigger.rs, procesar() rama Pulse): solo agrupa la rueda
/// en ráfagas (para poder resolver Simple/Mantenido) cuando existe un
/// candidato real cuyo PRÓXIMO input sea esta rueda — no simplemente
/// porque la rueda aparezca en algún remapeo sin relación con lo que
/// está pasando ahora.
pub fn hay_candidata_para(input: &InputId) -> bool {
    let listas = LISTAS.lock().unwrap();

    for lista in listas.iter() {
        let mut probable = lista.entrada.clone();
        probable.push(input.clone());
        let (posibles, _, _) = contar(&probable);
        if posibles > 0 {
            return true;
        }
    }

    drop(listas);

    let (posibles, _, _) = contar(std::slice::from_ref(input));
    posibles > 0
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

/// Apps distintas (sin contar Global) que aparecen en algún remapeo
/// cargado — es lo que back_app::revisar_apps() necesita para saber
/// qué vigilar.
pub fn apps_a_vigilar() -> Vec<AppCache> {
    let cache = CACHE.lock().unwrap();
    let mut vistas = Vec::new();

    for fila in cache.iter() {
        if fila.trigger.app == AppCache::Global {
            continue;
        }
        if !vistas.contains(&fila.trigger.app) {
            vistas.push(fila.trigger.app.clone());
        }
    }

    vistas
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
    let mut id_encontrado = None;
    for lista in listas.iter() {
        let mut probable = lista.entrada.clone();
        probable.push(input.clone());
        let (posibles, _, _) = contar(&probable);
        if posibles > 0 {
            id_encontrado = Some(lista.id);
            break;
        }
    }

    let id = match id_encontrado {
        Some(id) => {
            if let Some(lista) = listas.iter_mut().find(|l| l.id == id) {
                lista.entrada.push(input.clone());
                // Un Down real la extiende: si era fantasma, deja de
                // serlo — a partir de acá entrada.rs vuelve a tener
                // (o abre) un RETENIDO sobre esta lista (ver
                // recibir_down más abajo, siempre termina en
                // entrada::retener() para este camino).
                lista.fantasma = false;
            }
            id
        }
        None => {
            let id = nuevo_id_lista();
            listas.push(Lista {
                id,
                entrada: vec![input.clone()],
                esperando_condicion: false,
                fantasma: false,
            });
            id
        }
    };

    let entrada_actual = listas
        .iter()
        .find(|l| l.id == id)
        .map(|l| l.entrada.clone())
        .unwrap_or_default();
    drop(listas);

    let (posibles, exactas, candidatas) = contar(&entrada_actual);

    if posibles == 0 {
        limpiar_lista(id);
        analizador_trigger::limpiar();
        entrada::pasar();
        return;
    }

    if posibles == exactas
        && exactas == 1
        && candidatas[0].trigger.condicion == CondicionTrigger::Simple
    {
        let remapeo = candidatas[0].clone();
        let diferido = remapeo
            .extra
            .as_ref()
            .is_some_and(|extra| extra.requiere_up_real());

        if diferido {
            iniciar_solamente(remapeo, entrada_actual);
        } else {
            iniciar_y_finalizar(remapeo);
        }
        limpiar_lista(id);
        reiniciar_desde_presionados();
        entrada::consumir();
        return;
    }

    if posibles == exactas && exactas >= 1 {
        marcar_esperando_condicion(id);
        let gatillo = entrada_actual.last().cloned().unwrap();
        // Solo hace falta salir de la Fase Mantenido hacia una espera
        // de ambigüedad (Doble o Triple) si entre las candidatas reales
        // de esta entrada hay al menos un binding que lo pida — si no,
        // esperar ese tiempo no descarta nada real, es demora pura (ver
        // analizador_trigger::procesar, Up-handler).
        let necesita_doble = candidatas
            .iter()
            .any(|c| c.trigger.condicion == CondicionTrigger::Doble);
        // Triple manda sobre Doble: si hay al menos un binding Triple
        // candidato, la espera post-Up1 usa la ventana tiempo_triple
        // completa (ver analizador_trigger.rs, fase Triple) en vez de
        // resolver Doble apenas llega el segundo Down.
        let necesita_triple = candidatas
            .iter()
            .any(|c| c.trigger.condicion == CondicionTrigger::Triple);
        analizador_trigger::iniciar_timer(gatillo, necesita_doble, necesita_triple);
        entrada::retener();
        return;
    }

    entrada::retener();
}

/// Llamada por el timer del analizador (hilo aparte). No hay nadie
/// esperando un valor de retorno: avisa directo a quien corresponda.
pub fn recibir_condicion(condicion: CondicionTrigger) {
    let id = match *PREGUNTA_PENDIENTE.lock().unwrap() {
        Some(id) => id,
        None => return,
    };

    let entrada_actual = {
        let listas = LISTAS.lock().unwrap();
        match listas.iter().find(|l| l.id == id) {
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
            let diferido = remapeo
                .extra
                .as_ref()
                .is_some_and(|extra| extra.requiere_up_real());

            if diferido {
                iniciar_solamente(remapeo, entrada_actual);
            } else {
                iniciar_y_finalizar(remapeo);
            }
            limpiar_lista(id);
            reiniciar_desde_presionados();
            entrada::consumir();
        }
        _ => {
            limpiar_lista(id);
            analizador_trigger::limpiar();
            entrada::pasar();
        }
    }
}

/// Llega con CADA Up real (el analizador ya no filtra por ventana de
/// Mantenido, ver analizador_trigger.rs). Tres casos:
///
/// 1. Hay una instancia Mantenido activa esperando justo este Up ->
///    se perdió el match: finalizar esa instancia (como antes).
///
/// 2. Si no, y la tecla está en una lista FANTASMA (ver
///    Lista::fantasma) -> se descarta la lista ENTERA, no solo la
///    tecla. Una fantasma no tiene a nadie esperando en entrada.rs
///    (ya recibió su pasar()/consumir()), así que no hace falta
///    avisarle nada — pero el resto que quedaría recortando tecla por
///    tecla tampoco representa ninguna decisión real, es solo un
///    accidente de qué tecla se soltó primero, y dejarlo dando vueltas
///    es lo que causaba el huérfano del bug 3 (ej. [Ctrl] solo,
///    dispuesto a "completarse" con una Q físicamente ajena). El caso
///    de uso real (Ctrl+C -> Ctrl+V) sigue cubierto: en cuanto la
///    instancia activa se finalice de verdad, se vuelve a sembrar una
///    fantasma nueva y precisa (ver reiniciar_desde_presionados()).
///
/// 3. Si no, y la tecla está en una lista EN CONSTRUCCIÓN (no
///    fantasma, no esperando_condicion) -> entrada.rs tiene un
///    RETENIDO abierto esperando que esto se resuelva, así que hay
///    que avisarle SIEMPRE algo (bug 1: antes no se avisaba nada y
///    quedaba colgado hasta la red de seguridad). Se resuelve así
///    (ver conversación con el usuario sobre Tipo A):
///      a) Si la entrada tal cual estaba ANTES de sacar esta tecla
///         (o sea, incluyéndola) matchea EXACTO una única candidata
///         Simple -> ya no hay forma de que la tecla vuelva a
///         "bajar" sola para completar un prefijo más largo, así que
///         se resuelve ahí mismo como si esa hubiese sido la entrada
///         final desde el principio (ej. bug 1: 3>A con [3]+1>B,
///         soltás "3" sin tocar "1" -> genera A).
///      b) Si no, no hubo match posible: se aborta TODO lo retenido
///         (Opción "Abortar", no "Reintentar con lo que queda" — más
///         simple y predecible, y evita reabrir la puerta a otro
///         huérfano tipo bug 3). entrada::pasar() reinyecta el buffer
///         retenido tal cual a Windows. Lo que siga físicamente
///         presionado (ej. el "1" de un [1,2] sin match, soltando
///         "2") no se pierde: se resiembra como lista fantasma nueva
///         vía reiniciar_desde_presionados(), para que un trigger
///         futuro que lo use (ej. 1+5) lo siga reconociendo.
///
/// Las listas esperando_condicion NO se tocan acá en ningún caso —
/// todavía necesitan ver esa tecla en su entrada para que
/// recibir_condicion() las compare bien; se resuelven solas por su
/// propio camino.
pub fn recibir_up(input: InputId) {
    let mut activas = ACTIVAS.lock().unwrap();
    if let Some(pos) = activas.iter().position(|a| a.entrada.contains(&input)) {
        let instancia = activas.remove(pos);
        drop(activas);

        runtime::ejecutar(runtime::OrdenRuntime::Detener { id: instancia.id });
        analizador_trigger::limpiar();
        reiniciar_desde_presionados();
        return;
    }
    drop(activas);

    let mut listas = LISTAS.lock().unwrap();

    let Some(idx) = listas
        .iter()
        .position(|l| !l.esperando_condicion && l.entrada.contains(&input))
    else {
        return;
    };

    if listas[idx].fantasma {
        // Caso 2: fantasma — se descarta entera, nadie la espera.
        listas.remove(idx);
        return;
    }

    // Caso 3: en construcción — entrada.rs SIEMPRE recibe algo abajo.
    let id = listas[idx].id;
    let entrada_antes = listas[idx].entrada.clone();

    let (posibles, exactas, candidatas) = contar(&entrada_antes);
    let match_simple = (posibles == exactas && exactas == 1)
        .then(|| {
            candidatas
                .into_iter()
                .find(|c| c.trigger.condicion == CondicionTrigger::Simple)
        })
        .flatten();

    if let Some(remapeo) = match_simple {
        // 3a) La entrada, tal cual estaba con la tecla recién soltada
        // adentro, ya era un match Simple exacto — se resuelve ahora,
        // retroactivamente (la tecla soltada ya no puede completar
        // ningún prefijo más largo).
        //
        // Siempre Iniciar+Finalizar juntos acá, aunque el Extra de la
        // fila requiera_up_real() (Turbo/Mantener/Click Sostenido): el
        // Up real que ese Extra necesita para saber cuándo soltar YA
        // pasó (es justo el que estamos procesando) — dejarlo
        // "diferido" esperando un Up que no va a volver a llegar sin
        // que la tecla se presione de nuevo lo dejaría colgado.
        drop(listas);
        iniciar_y_finalizar(remapeo);
        limpiar_lista(id);
        reiniciar_desde_presionados();
        entrada::consumir();
        return;
    }

    // 3b) No hubo match posible: se aborta todo lo retenido.
    listas.remove(idx);
    drop(listas);
    limpiar_lista(id);
    analizador_trigger::limpiar();
    entrada::pasar();
    // Lo que siga físicamente presionado no se pierde — se resiembra
    // como fantasma nueva (mismo camino que tras un match real).
    reiniciar_desde_presionados();
}

fn iniciar_y_finalizar(remapeo: RemapeoCache) {
    runtime::ejecutar(runtime::OrdenRuntime::Iniciar {
        id: remapeo.id.clone(),
        accion: remapeo.accion,
        extra: remapeo.extra,
        coordenada: remapeo.coordenada,
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
        coordenada: remapeo.coordenada,
    });
}

fn marcar_esperando_condicion(id: u64) {
    if let Some(l) = LISTAS.lock().unwrap().iter_mut().find(|l| l.id == id) {
        l.esperando_condicion = true;
    }
    *PREGUNTA_PENDIENTE.lock().unwrap() = Some(id);
}

fn limpiar_lista(id: u64) {
    LISTAS.lock().unwrap().retain(|l| l.id != id);

    let mut pregunta = PREGUNTA_PENDIENTE.lock().unwrap();
    if *pregunta == Some(id) {
        *pregunta = None;
    }
}

/// Hereda lo que sigue físicamente presionado (soporta Ctrl+C -> Ctrl+V).
/// Siembra una lista FANTASMA (ver Lista::fantasma) — puramente
/// especulativa, nadie en entrada.rs está esperando su resolución.
fn reiniciar_desde_presionados() {
    let presionados = analizador_trigger::obtener_presionados();
    if !presionados.is_empty() {
        let id = nuevo_id_lista();
        LISTAS.lock().unwrap().push(Lista {
            id,
            entrada: presionados,
            esperando_condicion: false,
            fantasma: true,
        });
    }
}
