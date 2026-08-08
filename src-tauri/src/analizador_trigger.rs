// ======================================================
// 🧠 Analizador Trigger
// ======================================================
// 1. ¿Qué hace este archivo?
//
// Motor de bajo nivel que redirige cada Down/Up físico
// hacia quien corresponda (Cache en modo Runtime,
// perfil_ui en modo Captura), y resuelve por su cuenta
// la condición del gatillo (Simple / Doble / Triple /
// Mantenido) mediante un timer real.
//
// NO acumula secuencia. No sabe qué es "modificador" ni
// "gatillo" — cada Down se reenvía tal como llega, en el
// momento en que llega. Esa memoria (si hace falta) vive
// en quien lo recibe (Cache o perfil_ui), no acá.
//
// SÍ mantiene dos cosas propias:
//   a) qué está físicamente presionado ahora mismo
//      (se agrega en cada Down real, se saca en cada Up
//      real — sobrevive a cualquier resolución de Cache).
//   b) el timer en curso, si hay uno.
//
// Tiene dos modos, fijados al crearlo: Runtime y Captura.
// El comportamiento de fondo (filtro de repeats, timer)
// es el mismo; cambia a quién le reenvía los Down y
// cuándo arranca el timer.
//
// ⚠️ DECISIÓN RECONCILIADA (revisar si algo no cuadra):
// Este archivo reenvía a Cache TODOS los Up reales, siempre
// (no solo durante la espera de un Mantenido confirmado).
// Cache decide qué hacer con cada uno: si hay una instancia
// Mantenido esperando justo ese Up, lo usa para finalizarla;
// si no, lo usa para mantener sincronizadas sus listas de
// comparación en reposo (sacar una tecla que ya se soltó,
// para que no quede como fantasma bloqueando el próximo
// match real de esa misma tecla). Antes, fuera de la ventana
// de Mantenido, ningún Up llegaba nunca a Cache — eso dejaba
// listas fantasma con teclas ya sueltas (ver cache.rs,
// recibir_up).
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// entrada.rs (modo Runtime): le entrega cada InputEvent
//     físico a medida que llega, vía
//     procesar_evento_runtime().
// entrada.rs (modo Captura): mientras hay una captura
//     activa (ver captura_activa()), le entrega cada
//     InputEvent físico vía procesar_evento_captura(), en
//     vez de procesar_evento_runtime(). entrada.rs no
//     emite esos eventos a Windows — este archivo decide
//     el consumo simplemente por estar en esa rama, sin
//     que nadie se lo pida explícitamente (no hay
//     protocolo retener/pasar/consumir en Captura: acá
//     SIEMPRE se consume, no hay ambigüedad que resolver).
// perfil_ui (modo Captura): llama activar_captura() al
//     arrancar una captura y desactivar_captura() al
//     cancelarla. Este archivo también se
//     autodesactiva solo, apenas termina de resolver y
//     avisar el resultado final (ver enviar_condicion).
// Cache: le pide el timer cuando detecta ambigüedad, y le
//     pide el conjunto de "presionados ahora" cuando
//     necesita reiniciar su lista de comparación.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// procesar(evento: InputEvent) — cada Down/Up/Pulse real.
// iniciar_timer(objetivo, necesita_doble, necesita_triple) —
//     orden de Cache (solo modo Runtime, solo cuando hay
//     ambigüedad real). Cache decide ambos flags mirando las
//     candidatas de la entrada actual: necesita_doble, true si
//     hay al menos una con condición Doble; necesita_triple,
//     true si hay al menos una con condición Triple (manda
//     sobre necesita_doble — ver Up-handler de procesar()). Si
//     los dos son false, el Up resuelve Simple en el acto — no
//     tiene sentido esperar para descartar algo que nunca fue
//     una posibilidad real.
// obtener_presionados() — consulta puntual de Cache, para
//     reiniciar su lista tras resolver.
// limpiar() — orden de Cache: descarta timer en curso y
//     sale de la fase de reenvío de Ups. NO borra el
//     conjunto de "presionados ahora" (ese siempre debe
//     reflejar la realidad física, más allá de lo que
//     Cache decida internamente).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// A Cache (modo Runtime): cada Down nuevo (no repeats).
//     Durante espera de Mantenido: además, cada Up.
//     Cuando el timer resuelve: CondicionTrigger
//     (Simple / Doble / Triple / Mantenido) en un mensaje
//     aparte.
// A perfil_ui (modo Captura): cada Down nuevo (no
//     repeats). Al final del gesto completo: el
//     CondicionTrigger resuelto, una sola vez.
// A quien llama procesar(): None si el evento fue un
//     repeat filtrado (sin novedad, nada que hacer más
//     allá de lo que ya estaba en curso).
// ------------------------------------------------------
// Reglas del filtro de repeats (aplica a los dos modos)
//
// El auto-repeat de Windows genera Downs repetidos de una
// tecla mientras sigue abajo, sin Up de por medio. Solo el
// PRIMER Down de una tecla (tras un Up, o el primerísimo)
// se reenvía a destino. Los repeats siguientes:
//   - Actualizan el conjunto de "presionados ahora" (ya
//     estaba presionada, no cambia nada ahí).
//   - Sirven de "tick" para que el timer, si hay uno
//     corriendo sobre esa tecla, revise si ya se cumplió
//     tiempo_mantenido.
//   - No se reenvían a Cache/perfil_ui.
// ------------------------------------------------------
// Reglas del timer (las salidas posibles — igual en los
// dos modos; lo que cambia es quién lo arranca y a quién
// se le avisa el resultado)
//
// El timer analiza una tecla puntual (la última que iba
// a intentar convertirse en el timer se le pide para
// ella). Usa tiempo_mantenido, tiempo_doble y tiempo_triple
// (config global, ver config.rs). Tiene 3 fases posibles,
// guardadas explícitas en Timer.fase — nunca anidadas, la
// fase se decide UNA sola vez, al terminar la fase Mantenido:
//
// FASE MANTENIDO — pasa tiempo_mantenido desde el Down real
//    de esa tecla, sin que haya llegado su Up ni el Down
//    de otra tecla → se envía "Mantenido". Se destruye el
//    timer.
//
//    Si en cambio llega el Up antes de cumplirse
//    tiempo_mantenido, se decide EN ESE INSTANTE (Up-handler
//    de procesar()) a cuál de las otras dos fases pasar, mirando
//    las candidatas de esta entrada (necesita_doble/necesita_triple,
//    que llegaron desde Cache en iniciar_timer y viajan colgando
//    del Timer durante toda la fase Mantenido):
//    - Si hay algún Triple candidato (necesita_triple=true) → FASE
//      TRIPLE (gana sobre Doble, ver abajo).
//    - Si no hay Triple pero sí algún Doble candidato
//      (necesita_doble=true) → FASE DOBLE.
//    - Si no hay ninguno de los dos → se envía "Simple" en el acto,
//      sin pasar por ninguna fase de espera adicional.
//
// FASE DOBLE — arranca al terminar Mantenido con Up y
//    necesita_doble=true (y necesita_triple=false). Cuenta
//    tiempo_doble desde ese Up:
//    - Si llega un nuevo Down de la misma tecla antes de que
//      expire → se envía "Doble" en el acto. Se destruye el timer.
//    - Si expira sin ese Down → se envía "Simple". Se destruye
//      el timer.
//
// FASE TRIPLE — arranca al terminar Mantenido con Up y
//    necesita_triple=true. A diferencia de Doble, esta fase NO
//    resuelve nada apenas llega el segundo Down — solo lo cuenta
//    (Timer.toques, arranca en 1). Cuenta tiempo_triple completo
//    desde el Up del primer toque:
//    - Si llega un tercer Down (toques pasa de 2 a 3) antes de que
//      expire → se envía "Triple" en el acto. Se destruye el timer.
//    - Si expira sin ese tercer Down → se envía "Doble" si llegó a
//      registrarse el segundo toque (toques == 2), o "Simple" si no
//      llegó ninguno (toques == 1). Se destruye el timer.
//    - El Up del segundo toque, si llega mientras se cuenta, pasa
//      de largo sin efecto (ver Up-handler: solo actúa sobre un
//      timer en fase Mantenido, una fase Doble/Triple ya en curso
//      no se toca hasta que resuelva).
//
// El timer corre en un único hilo (no uno por Down). Usa
// un número de generación: si mientras corre aparece una
// situación que lo vuelve irrelevante (otra tecla nueva
// que reemplaza a la que estaba siendo analizada), se
// incrementa la generación — el timer, al despertarse, se
// revisa contra la generación vigente: si ya no coincide,
// no manda nada y se descarta solo.
// ------------------------------------------------------
// Reglas específicas — Modo Runtime
//
// - Reenvía a Cache solo el primer Down de cada tecla.
// - NUNCA reenvía Up a Cache, EXCEPTO durante la ventana
//   posterior a un "Mantenido" ya confirmado y avisado —
//   ahí reenvía cada Up (ver nota reconciliada al inicio).
// - El timer solo lo arranca Cache (pidiéndoselo) — el
//   analizador no decide por su cuenta cuándo hay
//   ambigüedad, eso es trabajo de Cache.
// - Cache puede pedir en cualquier momento el conjunto de
//   "presionados ahora" (para reiniciar su lista tras
//   resolver algo — soporta el caso Ctrl+C → Ctrl+V, donde
//   Ctrl sigue abajo y debe heredarse a la siguiente
//   comparación).
// - Debe soportar más de un "grupo" en análisis al mismo
//   tiempo (ej: una mano en un atajo de teclado, la otra
//   en un gesto de mouse). Una tecla físicamente nueva,
//   que no tiene relación con lo que ya está en curso,
//   dispara su propio seguimiento independiente (timer
//   propio si hace falta), sin bloquearse por lo que ya
//   esté pendiente en otro grupo.
// ------------------------------------------------------
// Reglas específicas — Modo Captura
//
// - Reenvía a perfil_ui solo el primer Down de cada tecla.
// - El timer se arranca solo, automáticamente, con cada
//   Down nuevo — acá no hay Cache pidiéndolo.
// - Si llega un Down nuevo antes de que el timer termine,
//   se reinicia sobre esa tecla nueva (la anterior queda
//   como modificador implícito — perfil_ui ya la recibió
//   como Down separado, no hace falta avisar nada más
//   sobre ella).
// - Mientras siga habiendo algo presionado, el resultado
//   que el timer vaya determinando se guarda como dato
//   interno — NO se envía todavía.
// - Recién cuando ya no queda nada presionado Y el timer
//   termina su ciclo sin que llegue nada nuevo, se envía a
//   perfil_ui el resultado final, una sola vez.
// - Implementación: en Captura, TODA la secuencia comparte un
//   único grupo (a diferencia de Runtime, donde cada tecla sin
//   relación abre su propio grupo) — así solo hay un timer
//   vigente a la vez, y una tecla nueva cancela por generación
//   el de la anterior (ver reenviar_down / iniciar_timer). El
//   resultado resuelto pero con algo aún presionado se guarda
//   en Grupo.pendiente (ver resolver_o_posponer) y se manda
//   recién cuando el Up-handler detecta el grupo vacío. La
//   condición de "grupo vacío" se revisa contra presionados,
//   pero un grupo con presionados=[] puede seguir teniendo un
//   timer async vivo (esperar_doble/esperar_mantenido de una
//   tecla distinta a la que se acaba de soltar) — mientras ese
//   timer no haya terminado, el grupo NO se borra, aunque esté
//   vacío (ver el Up-handler: el borrado se decide mirando si
//   queda timer, no si ESTE Up puntual arrancó uno).
// - No existe la fase de reenvío de Ups post-Mantenido —
//   eso es exclusivo de Runtime (ahí no hay instancia de
//   Runtime que finalizar).
// ------------------------------------------------------
// 5. Funciones del archivo
//
// nuevo(modo: ModoAnalizador)
//     Crea el analizador en modo Runtime o Captura.
// procesar(evento: InputEvent) -> Option<()>
//     Punto de entrada único para cada evento físico.
//     Aplica el filtro de repeats, actualiza "presionados
//     ahora", reenvía a destino según el modo, y alimenta
//     el timer si hay uno corriendo. None si fue un
//     repeat filtrado sin novedad.
// iniciar_timer(objetivo: InputId, necesita_doble: bool, necesita_triple: bool)
//     (Solo llamado por Cache, modo Runtime.) Arranca el
//     timer sobre esa tecla puntual (fase Mantenido).
// obtener_presionados() -> Vec<InputId>
//     Devuelve el conjunto de "presionados ahora". Lo
//     llama Cache tras resolver, para reiniciar su lista.
// limpiar()
//     Orden de Cache: descarta el timer en curso y sale
//     de la fase de reenvío de Ups. No toca "presionados
//     ahora".
// limpiar_grupos()
//     Vacía por completo "grupos" (a diferencia de
//     limpiar(), acá no queda ni "presionados ahora").
//     Se usa solo para dejar la instancia de Captura en
//     cero antes de una captura nueva.
// ------------------------------------------------------
// Funciones libres — instancia de modo Captura
//
// captura_activa() -> bool
//     True mientras haya una captura en curso. Lo consulta
//     entrada.rs antes que cualquier otra cosa.
// activar_captura()
//     Llamada por perfil_ui al arrancar una captura: limpia
//     la instancia de Captura y marca captura_activa() en
//     true.
// desactivar_captura()
//     Marca captura_activa() en false y limpia la
//     instancia. Se llama sola apenas se resuelve y avisa
//     el resultado final (ver enviar_condicion), y también
//     la llama perfil_ui al cancelar una captura a mitad
//     de camino.
// procesar_evento_captura(evento: InputEvent) -> Option<()>
//     Entrega el evento a la instancia de modo Captura.
//     Llamada por entrada.rs mientras captura_activa().
// ------------------------------------------------------
// Transformación:
//
// InputEvent físico
//     ↓
// ¿Es repeat? → sí: actualiza timer, no reenvía, None
//     ↓ no
// Actualiza "presionados ahora"
//     ↓
// Reenvía Down a destino (Cache o perfil_ui)
//     ↓
// (si hay timer corriendo) Alimenta el timer
//     ↓
// Timer resuelve → envía CondicionTrigger a destino
// ======================================================
use crate::eventos::{InputEvent, InputId, InputState};
use crate::perfil_cache::CondicionTrigger;
use crate::{cache, config, perfil_ui};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

// ======================================================
// 🌐 INSTANCIA GLOBAL DE MODO RUNTIME
// ------------------------------------------------------
// Cache y entrada.rs no tienen (ni deben tener) una
// referencia propia al analizador — lo alcanzan a través
// de estas funciones libres, que delegan al único
// AnalizadorTrigger en modo Runtime que existe por
// proceso. El modo Captura sigue siendo una instancia
// aparte, creada por perfil_ui cuando arranca una captura.
// ======================================================

static INSTANCIA_RUNTIME: OnceLock<AnalizadorTrigger> = OnceLock::new();

fn instancia_runtime() -> &'static AnalizadorTrigger {
    INSTANCIA_RUNTIME.get_or_init(|| AnalizadorTrigger::nuevo(ModoAnalizador::Runtime))
}

pub fn procesar_evento_runtime(evento: InputEvent) -> Option<()> {
    instancia_runtime().procesar(evento)
}

pub fn iniciar_timer(objetivo: InputId, necesita_doble: bool, necesita_triple: bool) {
    instancia_runtime().iniciar_timer(objetivo, necesita_doble, necesita_triple)
}

pub fn limpiar() {
    instancia_runtime().limpiar()
}

pub fn obtener_presionados() -> Vec<InputId> {
    instancia_runtime().obtener_presionados()
}

/// Llamada por entrada.rs cuando un Up se resuelve por el atajo de
/// grupos DEVOLVIENDO (pasa derecho, sin pasar por procesar_evento_runtime).
/// Sin esto, el conjunto interno de "presionados ahora" nunca se entera
/// de ese Up, y el filtro de repeats trata la próxima Down de esa misma
/// tecla como si siguiera abajo — la descarta en vez de reenviarla.
pub fn soltar_fisico(input: InputId) {
    instancia_runtime().soltar(input);
}

// ======================================================
// 🎬 INSTANCIA GLOBAL DE MODO CAPTURA
// ------------------------------------------------------
// Una única instancia por proceso, igual que Runtime — pero
// a diferencia de Runtime, esta se activa y desactiva una y
// otra vez (una vez por cada captura que se hace). Mientras
// está inactiva, entrada.rs ni la toca: sigue todo por el
// camino normal de Runtime.
// ======================================================

static INSTANCIA_CAPTURA: OnceLock<AnalizadorTrigger> = OnceLock::new();
static CAPTURA_ACTIVA: Mutex<bool> = Mutex::new(false);

fn instancia_captura() -> &'static AnalizadorTrigger {
    INSTANCIA_CAPTURA.get_or_init(|| AnalizadorTrigger::nuevo(ModoAnalizador::Captura))
}

/// Consultada por entrada.rs antes que cualquier otra cosa: mientras
/// esté en true, TODO evento físico se consume y se reenvía acá,
/// nunca a Windows ni a Cache.
pub fn captura_activa() -> bool {
    *CAPTURA_ACTIVA.lock().unwrap()
}

/// Llamada por perfil_ui al arrancar una captura nueva.
pub fn activar_captura() {
    instancia_captura().limpiar_grupos();
    *CAPTURA_ACTIVA.lock().unwrap() = true;
}

/// Se llama sola al terminar (ver enviar_condicion) y también la
/// llama perfil_ui si el usuario cancela la captura a mitad de camino.
pub fn desactivar_captura() {
    *CAPTURA_ACTIVA.lock().unwrap() = false;
    instancia_captura().limpiar_grupos();
}

/// Llamada por entrada.rs, evento por evento, mientras captura_activa().
pub fn procesar_evento_captura(evento: InputEvent) -> Option<()> {
    instancia_captura().procesar(evento)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModoAnalizador {
    Runtime,
    Captura,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FaseTimer {
    // Esperando tiempo_mantenido desde el Down real, sin Up todavía.
    // Es la ÚNICA fase que el Up-handler de procesar() transforma en
    // otra cosa — Doble/Triple, una vez arrancadas, resuelven solas
    // (por Down o por expirar), nunca se re-derivan desde el Up-handler.
    Mantenido,
    // Esperando tiempo_doble desde el Up del primer toque. Un segundo
    // Down de la misma tecla resuelve Doble en el acto.
    Doble,
    // Esperando tiempo_triple desde el Up del primer toque. A
    // diferencia de Doble, el segundo Down NO resuelve nada por sí
    // solo — solo incrementa Timer.toques y la fase sigue corriendo
    // hasta el tercer Down (resuelve Triple) o hasta que expire
    // (resuelve Doble o Simple según Timer.toques).
    Triple,
}

struct Timer {
    generacion: u64,
    objetivo: InputId,
    fase: FaseTimer,
    // Cuántos toques (Downs) de esta tecla se contaron desde que
    // arrancó la fase Triple (arranca en 1, representando el primer
    // toque que ya ocurrió antes de esta fase). Solo lo usa/actualiza
    // la fase Triple — en Mantenido/Doble no tiene sentido y queda en 1.
    toques: u8,
    // Solo relevantes mientras fase == Mantenido: qué haría falta
    // esperar si el Up llega antes de tiempo_mantenido. Cache los
    // calcula en iniciar_timer() (o, en modo Captura, siempre true
    // los dos) y quedan colgando del timer hasta que el Up-handler
    // los lee, una única vez, para decidir a qué fase pasar (ver
    // Reglas del timer, arriba). Sin uso fuera de la fase Mantenido.
    necesita_doble: bool,
    necesita_triple: bool,
}

struct Grupo {
    presionados: Vec<InputId>,
    timer: Option<Timer>,

    // Solo lo usa Captura (ver resolver_o_posponer): una condición ya
    // resuelta (Simple/Doble/Mantenido) pero que todavía no se manda
    // porque, en el instante de resolverse, seguía quedando algo
    // físicamente presionado. Se manda recién cuando el Up-handler
    // detecta que ya no queda nada (ver header, Reglas — Modo Captura).
    pendiente: Option<CondicionTrigger>,

    // Solo lo usa la rueda (InputState::Pulse): cuántos pulsos lleva la
    // ráfaga actual. Nunca se toca para teclas/clics normales.
    pulsos: u64,
}

impl Grupo {
    fn nuevo() -> Self {
        Self {
            presionados: Vec::new(),
            timer: None,
            pendiente: None,
            pulsos: 0,
        }
    }
}

pub struct AnalizadorTrigger {
    modo: ModoAnalizador,
    grupos: Arc<Mutex<HashMap<InputId, Grupo>>>,
}

impl AnalizadorTrigger {
    pub fn nuevo(modo: ModoAnalizador) -> Self {
        Self {
            modo,
            grupos: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn clave_grupo(grupos: &HashMap<InputId, Grupo>, input: &InputId) -> Option<InputId> {
        grupos
            .iter()
            .find(|(_, g)| g.presionados.contains(input))
            .map(|(clave, _)| clave.clone())
    }

    pub fn procesar(&self, evento: InputEvent) -> Option<()> {
        let mut grupos = self.grupos.lock().unwrap();

        match evento.state {
            InputState::Down => {
                if Self::clave_grupo(&grupos, &evento.input).is_some() {
                    // Repeat: ya estaba presionado. No se reenvía.
                    return None;
                }

                // ¿Es un nuevo Down de un objetivo que está en fase
                // Doble o Triple (esperando ambigüedad)?
                if let Some((clave, _generacion, fase, toques_actual)) =
                    self.buscar_espera_ambiguedad(&grupos, &evento.input)
                {
                    // Fase Triple con toques_actual == 1: este es el
                    // SEGUNDO Down físico. A diferencia de Doble, acá no
                    // se resuelve nada todavía — solo se cuenta y la
                    // fase sigue corriendo hasta el tercer Down o hasta
                    // que expire tiempo_triple (ver esperar_triple).
                    if fase == FaseTimer::Triple && toques_actual == 1 {
                        if let Some(grupo) = grupos.get_mut(&clave) {
                            if let Some(timer) = grupo.timer.as_mut() {
                                timer.toques = 2;
                            }
                            grupo.presionados.push(evento.input.clone());
                        }
                        // Mismo motivo que el comentario de más abajo:
                        // no se reenvía, es la MISMA tecla repitiendo.
                        return Some(());
                    }

                    // Cualquier otro caso resuelve en el acto:
                    // - Fase Doble -> Doble (segundo Down).
                    // - Fase Triple con toques_actual == 2 -> Triple
                    //   (tercer Down).
                    let condicion = if fase == FaseTimer::Triple {
                        CondicionTrigger::Triple
                    } else {
                        CondicionTrigger::Doble
                    };

                    if let Some(grupo) = grupos.get_mut(&clave) {
                        grupo.timer = None;
                        grupo.presionados.push(evento.input.clone());
                    }

                    let a_enviar =
                        Self::resolver_o_posponer(&mut grupos, &clave, self.modo, condicion);
                    drop(grupos);

                    // OJO: acá NO se reenvía este Down (a diferencia del
                    // primero, más abajo). Cache/perfil_ui ya se
                    // enteraron de esta tecla con el primer Down — este
                    // Down es la MISMA tecla confirmando el gesto, no
                    // una tecla nueva que agregar a la secuencia.
                    // Reenviarlo de nuevo duplicaba la tecla en Captura
                    // (quedaba como "modificador + gatillo x2") y en
                    // Cache generaba una lista extra de la nada — ver el
                    // punto de índices inestables en cache.rs.
                    if let Some(condicion) = a_enviar {
                        Self::enviar_condicion(self.modo, condicion);
                    }
                    return Some(());
                }

                // En Captura, TODA tecla de la secuencia comparte un único
                // grupo (a diferencia de Runtime, donde cada tecla
                // realmente nueva abre su propio grupo independiente) —
                // así solo puede haber un timer vigente a la vez para
                // toda la captura, y una tecla nueva cancela (por
                // generación) el timer de la anterior, que pasa a quedar
                // como modificador implícito. Sin esto, cada tecla
                // resolvía su propio Simple/Doble/Mantenido en paralelo
                // e independiente, cerrando la captura antes de tiempo.
                let clave = if self.modo == ModoAnalizador::Captura {
                    grupos
                        .keys()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| evento.input.clone())
                } else {
                    evento.input.clone()
                };
                let grupo = grupos.entry(clave).or_insert_with(Grupo::nuevo);
                grupo.presionados.push(evento.input.clone());
                drop(grupos);

                self.reenviar_down(evento.input.clone());
                Some(())
            }

            InputState::Up => {
                let clave = Self::clave_grupo(&grupos, &evento.input)?;

                // Si hay timer corriendo sobre ESTA tecla puntual Y
                // sigue en fase Mantenido, nos interesa qué pedía
                // disambiguar (necesita_doble/necesita_triple) — de eso
                // depende a qué fase pasar, o si se puede resolver
                // Simple ya mismo. Un timer que YA pasó a fase Doble o
                // Triple (un Up de un toque intermedio, ej. el Up2 de un
                // Triple en curso) no se toca acá: sigue corriendo solo,
                // ver Reglas del timer más arriba.
                let timer_objetivo = grupos
                    .get(&clave)
                    .and_then(|g| g.timer.as_ref())
                    .filter(|t| t.objetivo == evento.input && t.fase == FaseTimer::Mantenido)
                    .map(|t| (t.necesita_doble, t.necesita_triple));

                if let Some(grupo) = grupos.get_mut(&clave) {
                    grupo.presionados.retain(|i| i != &evento.input);
                }

                // OJO: hay que soltar el lock de `grupos` ANTES de avisarle a
                // Cache. cache::recibir_up() puede terminar llamando de vuelta
                // a este mismo AnalizadorTrigger (limpiar() / obtener_presionados()),
                // que también piden self.grupos.lock() — con el lock todavía
                // tomado acá, ese re-lock en el mismo hilo es un deadlock
                // (Mutex de std no es reentrante), y deja el hilo que procesa
                // cada InputEvent físico trabado para siempre.
                if self.modo == ModoAnalizador::Runtime {
                    drop(grupos);
                    cache::recibir_up(evento.input.clone());
                    grupos = self.grupos.lock().unwrap();
                }

                if let Some((necesita_doble, necesita_triple)) = timer_objetivo {
                    if necesita_triple {
                        // Hay (al menos) un binding Triple en juego para
                        // esta entrada: pasa a fase Triple — un timer
                        // ÚNICO con tiempo_triple completo, que no
                        // resuelve nada apenas llegue el segundo Down
                        // (ver Down-handler más arriba y esperar_triple
                        // más abajo).
                        let generacion = grupos
                            .get(&clave)
                            .and_then(|g| g.timer.as_ref())
                            .map(|t| t.generacion + 1)
                            .unwrap_or(0);

                        if let Some(grupo) = grupos.get_mut(&clave) {
                            grupo.timer = Some(Timer {
                                generacion,
                                objetivo: evento.input.clone(),
                                fase: FaseTimer::Triple,
                                toques: 1,
                                necesita_doble: false,
                                necesita_triple: false,
                            });
                        }

                        let grupos_arc = Arc::clone(&self.grupos);
                        let modo = self.modo;
                        let objetivo = evento.input.clone();
                        let clave2 = clave.clone();

                        std::thread::spawn(move || {
                            Self::esperar_triple(grupos_arc, clave2, objetivo, generacion, modo);
                        });
                    } else if necesita_doble {
                        // Ningún binding Triple es posible, pero SÍ hay
                        // (al menos) un binding Doble: pasa a fase Doble,
                        // sin cambios respecto al comportamiento de
                        // siempre.
                        let generacion = grupos
                            .get(&clave)
                            .and_then(|g| g.timer.as_ref())
                            .map(|t| t.generacion + 1)
                            .unwrap_or(0);

                        if let Some(grupo) = grupos.get_mut(&clave) {
                            grupo.timer = Some(Timer {
                                generacion,
                                objetivo: evento.input.clone(),
                                fase: FaseTimer::Doble,
                                toques: 1,
                                necesita_doble: false,
                                necesita_triple: false,
                            });
                        }

                        let grupos_arc = Arc::clone(&self.grupos);
                        let modo = self.modo;
                        let objetivo = evento.input.clone();
                        let clave2 = clave.clone();

                        std::thread::spawn(move || {
                            Self::esperar_doble(grupos_arc, clave2, objetivo, generacion, modo);
                        });
                    } else {
                        // Ni Doble ni Triple son posibles para esta
                        // entrada: esperar acá no descarta nada real, es
                        // demora pura. Se resuelve Simple ya mismo, sin
                        // pasar por ningún hilo ni sleep.
                        if let Some(grupo) = grupos.get_mut(&clave) {
                            grupo.timer = None;
                        }

                        let vacio_ahora = grupos
                            .get(&clave)
                            .map(|g| g.presionados.is_empty())
                            .unwrap_or(false);
                        if vacio_ahora {
                            grupos.remove(&clave);
                        }

                        let modo = self.modo;
                        drop(grupos);
                        Self::enviar_condicion(modo, CondicionTrigger::Simple);
                        return Some(());
                    }
                }

                let vacio = grupos
                    .get(&clave)
                    .map(|g| g.presionados.is_empty())
                    .unwrap_or(false);

                // Si el grupo TODAVÍA tiene un timer vivo (puede ser el
                // que se acaba de arrancar arriba, o uno de OTRA tecla
                // del mismo grupo que sigue corriendo — ej: en Captura,
                // Ctrl y A comparten grupo, y acá se está soltando Ctrl
                // mientras el esperar_doble de A sigue en curso), el
                // grupo no puede borrarse aunque quede vacío de
                // presionados: ese hilo lo va a necesitar cuando
                // despierte (ver vigente()). Si se borra antes, el hilo
                // no encuentra su grupo, no manda nada, y la captura
                // queda trabada en "esperando...".
                let tiene_timer_vivo = grupos
                    .get(&clave)
                    .map(|g| g.timer.is_some())
                    .unwrap_or(false);

                // Captura: si ya no queda nada presionado, esta es la
                // señal de que cualquier condición resuelta pero
                // pospuesta (ver resolver_o_posponer) ya puede mandarse.
                if vacio && self.modo == ModoAnalizador::Captura {
                    let pendiente = grupos.get_mut(&clave).and_then(|g| g.pendiente.take());
                    if let Some(condicion) = pendiente {
                        if !tiene_timer_vivo {
                            grupos.remove(&clave);
                        }
                        drop(grupos);
                        Self::enviar_condicion(self.modo, condicion);
                        return Some(());
                    }
                }

                if vacio && !tiene_timer_vivo {
                    grupos.remove(&clave);
                }

                Some(())
            }

            InputState::Pulse => {
                let clave = evento.input.clone();

                // Si ningún candidato posible espera esta rueda como
                // próximo paso AHORA MISMO (ni sola, ni como
                // continuación de modificadores ya presionados),
                // agruparla en ráfagas no sirve para nada (no hay
                // condición Simple/Mantenido que resolver) y solo logra
                // tragarse pulsos físicos que deberían volver tal cual a
                // Windows. En ese caso cada pulso se trata como un
                // evento suelto e independiente — mismo camino que ya
                // usa el teclado para un input sin ningún candidato
                // (ver cache::recibir_down → posibles == 0 → pasar()).
                // Solo aplica a Runtime: Captura sigue necesitando ver
                // cada pulso agrupado para poder armar el trigger nuevo
                // que se está grabando.
                if self.modo == ModoAnalizador::Runtime && !cache::hay_candidata_para(&clave) {
                    drop(grupos);
                    cache::recibir_down(clave);
                    return Some(());
                }

                let es_primero = !grupos.contains_key(&clave);

                let generacion = {
                    let grupo = grupos.entry(clave.clone()).or_insert_with(Grupo::nuevo);
                    grupo.pulsos += 1;

                    let generacion = grupo.timer.as_ref().map(|t| t.generacion + 1).unwrap_or(0);

                    grupo.timer = Some(Timer {
                        generacion,
                        objetivo: clave.clone(),
                        // Ninguno de estos campos los usa cerrar_rueda
                        // (no pasa por esperar_doble/esperar_triple/
                        // esperar_mantenido); presentes solo porque el
                        // struct los pide.
                        fase: FaseTimer::Mantenido,
                        toques: 1,
                        necesita_doble: false,
                        necesita_triple: false,
                    });

                    generacion
                };

                drop(grupos);

                if es_primero {
                    // Solo el primer pulso de la ráfaga se reenvía —
                    // Cache/perfil_ui ya se enteran de la rueda con este.
                    // Los pulsos siguientes de la MISMA ráfaga solo suman
                    // al conteo (ver cerrar_rueda) para decidir si termina
                    // siendo Simple o Mantenida — reenviarlos todos
                    // generaría el mismo problema que el Doble duplicado
                    // de más arriba. Nunca se agrega al grupo.presionados
                    // (a propósito: así iniciar_timer(), que busca por
                    // presionados, no encuentra nada y no arranca -de
                    // pedo- su propio timer de Mantenido de tecla — el
                    // cierre de la rueda lo maneja únicamente
                    // cerrar_rueda, más abajo).
                    match self.modo {
                        ModoAnalizador::Runtime => cache::recibir_down(clave.clone()),
                        ModoAnalizador::Captura => perfil_ui::recibir_down(clave.clone()),
                    }
                }

                let grupos_arc = Arc::clone(&self.grupos);
                let modo = self.modo;
                let objetivo = clave.clone();

                std::thread::spawn(move || {
                    Self::cerrar_rueda(grupos_arc, objetivo, generacion, modo);
                });

                Some(())
            }
        }
    }

    /// Busca si `input` es exactamente el objetivo de un timer que está
    /// en fase Doble o Triple (esperando ambigüedad) en algún grupo —
    /// NUNCA en fase Mantenido: mientras esa fase corre, la tecla sigue
    /// en "presionados" y el filtro de repeats de más arriba ya la
    /// descarta antes de llegar acá, así que no hace falta excluirla
    /// explícitamente. Devuelve también la fase y los toques actuales,
    /// para que el Down-handler sepa si este Down resuelve algo ya o
    /// solo suma un toque más (ver fase Triple).
    fn buscar_espera_ambiguedad(
        &self,
        grupos: &HashMap<InputId, Grupo>,
        input: &InputId,
    ) -> Option<(InputId, u64, FaseTimer, u8)> {
        grupos.iter().find_map(|(clave, g)| {
            g.timer
                .as_ref()
                .filter(|t| &t.objetivo == input)
                .map(|t| (clave.clone(), t.generacion, t.fase, t.toques))
        })
    }

    fn reenviar_down(&self, input: InputId) {
        match self.modo {
            ModoAnalizador::Runtime => cache::recibir_down(input.clone()),
            ModoAnalizador::Captura => perfil_ui::recibir_down(input.clone()),
        }

        if self.modo == ModoAnalizador::Captura {
            // Captura siempre corre las fases completas: todavía no
            // sabemos qué gesto va a terminar siendo (esto es
            // justamente lo que el usuario está definiendo), así que
            // necesita_doble y necesita_triple siempre son true acá —
            // a diferencia de Runtime, donde Cache ya sabe de antemano
            // qué condiciones son candidatas reales.
            self.iniciar_timer(input, true, true);
        }
    }

    /// En Runtime, Cache lo pide explícitamente al detectar ambigüedad,
    /// pasando si hace falta o no disambiguar Doble y/o Triple para esta
    /// entrada puntual (ver cache.rs::recibir_down). En Captura, se
    /// llama solo con cada Down nuevo, siempre con ambos flags en true.
    pub fn iniciar_timer(&self, objetivo: InputId, necesita_doble: bool, necesita_triple: bool) {
        let mut grupos = self.grupos.lock().unwrap();
        let clave = match Self::clave_grupo(&grupos, &objetivo) {
            Some(c) => c,
            None => return,
        };

        let generacion = grupos
            .get(&clave)
            .and_then(|g| g.timer.as_ref())
            .map(|t| t.generacion + 1)
            .unwrap_or(0);

        if let Some(grupo) = grupos.get_mut(&clave) {
            grupo.timer = Some(Timer {
                generacion,
                objetivo: objetivo.clone(),
                fase: FaseTimer::Mantenido,
                toques: 1,
                necesita_doble,
                necesita_triple,
            });
        }
        drop(grupos);

        let grupos_arc = Arc::clone(&self.grupos);
        let modo = self.modo;

        std::thread::spawn(move || {
            Self::esperar_mantenido(grupos_arc, clave, objetivo, generacion, modo);
        });
    }

    fn vigente(grupos: &HashMap<InputId, Grupo>, clave: &InputId, generacion: u64) -> bool {
        grupos
            .get(clave)
            .and_then(|g| g.timer.as_ref())
            .map(|t| t.generacion == generacion)
            .unwrap_or(false)
    }

    /// Fase A: ¿sigue presionado tras tiempo_mantenido, sin Up de por medio?
    fn esperar_mantenido(
        grupos: Arc<Mutex<HashMap<InputId, Grupo>>>,
        clave: InputId,
        objetivo: InputId,
        generacion: u64,
        modo: ModoAnalizador,
    ) {
        std::thread::sleep(Duration::from_millis(config::tiempo_mantenido()));

        let mut grupos_g = grupos.lock().unwrap();

        if !Self::vigente(&grupos_g, &clave, generacion) {
            return; // se soltó antes (ver Up) o llegó algo que lo invalidó
        }

        if let Some(g) = grupos_g.get_mut(&clave) {
            g.timer = None;
        }

        let a_enviar =
            Self::resolver_o_posponer(&mut grupos_g, &clave, modo, CondicionTrigger::Mantenido);
        drop(grupos_g);

        if let Some(condicion) = a_enviar {
            Self::enviar_condicion(modo, condicion);
        }
    }

    /// Fase B: si no llega un segundo Down antes de tiempo_doble -> Simple.
    fn esperar_doble(
        grupos: Arc<Mutex<HashMap<InputId, Grupo>>>,
        clave: InputId,
        _objetivo: InputId,
        generacion: u64,
        modo: ModoAnalizador,
    ) {
        std::thread::sleep(Duration::from_millis(config::tiempo_doble()));

        let mut grupos_g = grupos.lock().unwrap();

        if !Self::vigente(&grupos_g, &clave, generacion) {
            return; // llegó el segundo Down -> ya se mandó Doble en procesar()
        }

        if let Some(g) = grupos_g.get_mut(&clave) {
            g.timer = None;
        }

        let a_enviar =
            Self::resolver_o_posponer(&mut grupos_g, &clave, modo, CondicionTrigger::Simple);
        drop(grupos_g);

        if let Some(condicion) = a_enviar {
            Self::enviar_condicion(modo, condicion);
        }
    }

    /// Fase Triple: timer único con tiempo_triple completo. Si expira
    /// sin que haya llegado el tercer Down (eso lo resuelve directo el
    /// Down-handler de procesar(), que invalida este timer por
    /// generación), se decide acá según cuántos toques se llegaron a
    /// contar: 2 -> Doble, 1 -> Simple.
    fn esperar_triple(
        grupos: Arc<Mutex<HashMap<InputId, Grupo>>>,
        clave: InputId,
        _objetivo: InputId,
        generacion: u64,
        modo: ModoAnalizador,
    ) {
        std::thread::sleep(Duration::from_millis(config::tiempo_triple()));

        let mut grupos_g = grupos.lock().unwrap();

        if !Self::vigente(&grupos_g, &clave, generacion) {
            return; // llegó el tercer Down -> ya se mandó Triple en procesar()
        }

        let toques = grupos_g
            .get(&clave)
            .and_then(|g| g.timer.as_ref())
            .map(|t| t.toques)
            .unwrap_or(1);

        if let Some(g) = grupos_g.get_mut(&clave) {
            g.timer = None;
        }

        let condicion = if toques >= 2 {
            CondicionTrigger::Doble
        } else {
            CondicionTrigger::Simple
        };

        let a_enviar = Self::resolver_o_posponer(&mut grupos_g, &clave, modo, condicion);
        drop(grupos_g);

        if let Some(condicion) = a_enviar {
            Self::enviar_condicion(modo, condicion);
        }
    }

    /// Cierre de una ráfaga de rueda (InputState::Pulse). Cada pulso
    /// nuevo reprograma este cierre (por generación, igual que
    /// esperar_mantenido/esperar_doble). Si pasa config::tiempo_doble()
    /// sin que llegue un pulso nuevo, se cuenta cuántos hubo en total y
    /// se decide: menos de config::sensibilidad_rueda() -> Simple, esa
    /// cantidad o más -> Mantenido. La rueda no tiene Doble — no hay
    /// forma física de "soltarla y volver a apretarla" como una tecla.
    fn cerrar_rueda(
        grupos: Arc<Mutex<HashMap<InputId, Grupo>>>,
        clave: InputId,
        generacion: u64,
        modo: ModoAnalizador,
    ) {
        std::thread::sleep(Duration::from_millis(config::tiempo_doble()));

        let mut grupos_g = grupos.lock().unwrap();

        if !Self::vigente(&grupos_g, &clave, generacion) {
            return; // llegó un pulso nuevo antes de terminar de esperar
        }

        let pulsos = grupos_g.get(&clave).map(|g| g.pulsos).unwrap_or(0);
        grupos_g.remove(&clave);
        drop(grupos_g);

        let condicion = if pulsos >= config::sensibilidad_rueda() {
            CondicionTrigger::Mantenido
        } else {
            CondicionTrigger::Simple
        };

        Self::enviar_condicion(modo, condicion);
    }

    /// En Captura, una condición recién resuelta solo se manda si además
    /// ya no queda nada físicamente presionado en el grupo (ver header,
    /// Reglas — Modo Captura). Si todavía hay algo presionado, se guarda
    /// como `pendiente` y se manda más adelante, cuando el Up que suelta
    /// lo último detecte que ya no queda nada (ver Up-handler). En
    /// Runtime nunca se pospone: se manda siempre de inmediato.
    fn resolver_o_posponer(
        grupos_g: &mut HashMap<InputId, Grupo>,
        clave: &InputId,
        modo: ModoAnalizador,
        condicion: CondicionTrigger,
    ) -> Option<CondicionTrigger> {
        if modo == ModoAnalizador::Captura {
            let sigue_presionado = grupos_g
                .get(clave)
                .map(|g| !g.presionados.is_empty())
                .unwrap_or(false);

            if sigue_presionado {
                if let Some(g) = grupos_g.get_mut(clave) {
                    g.pendiente = Some(condicion);
                }
                return None;
            }
        }
        Some(condicion)
    }

    fn enviar_condicion(modo: ModoAnalizador, condicion: CondicionTrigger) {
        match modo {
            ModoAnalizador::Runtime => cache::recibir_condicion(condicion),
            ModoAnalizador::Captura => {
                perfil_ui::recibir_condicion(condicion);

                // La captura ya se resolvió y perfil_ui tiene el
                // resultado final: a partir de acá, cualquier evento
                // físico nuevo debe volver a fluir normal (Runtime),
                // no seguir siendo tragado por Captura.
                desactivar_captura();
            }
        }
    }

    /// Orden de Cache: descarta timer en curso. No toca "presionados
    /// ahora" (ver header: los Up siempre se reenvían a Cache, ya no
    /// depende de una fase especial).
    pub fn limpiar(&self) {
        let mut grupos = self.grupos.lock().unwrap();
        for grupo in grupos.values_mut() {
            grupo.timer = None;
        }
    }

    /// A diferencia de limpiar(), acá no queda nada: ni timers ni
    /// "presionados ahora". Se usa para dejar la instancia de Captura
    /// en cero antes de (o después de) cada captura.
    pub fn limpiar_grupos(&self) {
        let mut grupos = self.grupos.lock().unwrap();
        grupos.clear();
    }

    /// Consulta puntual de Cache tras resolver, para reiniciar su lista.
    pub fn obtener_presionados(&self) -> Vec<InputId> {
        let grupos = self.grupos.lock().unwrap();
        grupos
            .values()
            .flat_map(|g| g.presionados.clone())
            .collect()
    }

    /// Saca `input` de "presionados ahora" sin pasar por el pipeline
    /// completo de Up (sin timer, sin avisar a Cache/perfil_ui). Lo
    /// usa entrada.rs cuando un Up nunca llega hasta acá (atajo de
    /// grupos DEVOLVIENDO) — mantiene el conjunto interno sincronizado
    /// con la realidad física para que el filtro de repeats no se
    /// desincronice.
    pub fn soltar(&self, input: InputId) {
        let mut grupos = self.grupos.lock().unwrap();

        let Some(clave) = Self::clave_grupo(&grupos, &input) else {
            return;
        };

        if let Some(grupo) = grupos.get_mut(&clave) {
            grupo.presionados.retain(|i| i != &input);
        }

        let vacio = grupos
            .get(&clave)
            .map(|g| g.presionados.is_empty())
            .unwrap_or(false);

        if vacio {
            grupos.remove(&clave);
        }
    }
}
