// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ======================================================
// 1. ¿Qué hace este archivo?
//
// Motor de bajo nivel que redirige cada Down/Up físico
// hacia quien corresponda (Cache en modo Runtime,
// perfil_ui en modo Captura), y resuelve por su cuenta
// la condición del gatillo (Simple / Doble / Mantenido)
// mediante un timer real.
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
// Durante la espera de un Mantenido confirmado (ver Cache,
// estado "esperando finalizar"), este archivo reenvía a
// Cache TODOS los Up que ocurran (de cualquier tecla, no
// solo del gatillo) — es la única forma de que Cache se
// entere en el momento en que se suelta cualquier pieza
// involucrada, para poder finalizar la instancia. Fuera de
// esa ventana, ningún Up se reenvía nunca a Cache.
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
// iniciar_timer() — orden de Cache (solo modo Runtime,
//     solo cuando hay ambigüedad real).
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
//     (Simple / Doble / Mantenido) en un mensaje aparte.
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
// Reglas del timer (las 3 salidas posibles — igual en los
// dos modos; lo que cambia es quién lo arranca y a quién
// se le avisa el resultado)
//
// El timer analiza una tecla puntual (la última que iba
// a intentar convertirse en el timer se le pide para
// ella). Usa tiempo_mantenido y tiempo_doble (config
// global, ver config.rs).
//
// A) MANTENIDO — pasa tiempo_mantenido desde el Down real
//    de esa tecla, sin que haya llegado su Up ni el Down
//    de otra tecla → se envía "Mantenido". Se destruye el
//    timer.
// B) SIMPLE — llega el Up de esa tecla antes de cumplirse
//    tiempo_mantenido → arranca la cuenta de tiempo_doble
//    desde ese Up; si pasa tiempo_doble sin un nuevo Down
//    de la misma tecla → se envía "Simple". Se destruye
//    el timer.
// C) DOBLE — llega el Up y, antes de cumplirse
//    tiempo_doble desde ese Up, un nuevo Down de la misma
//    tecla → se envía "Doble" en el acto. Se destruye el
//    timer.
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
// iniciar_timer(objetivo: InputId)
//     (Solo llamado por Cache, modo Runtime.) Arranca el
//     timer sobre esa tecla puntual.
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

pub fn iniciar_timer(objetivo: InputId) {
    instancia_runtime().iniciar_timer(objetivo)
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

struct Timer {
    generacion: u64,
    objetivo: InputId,
}

struct Grupo {
    presionados: Vec<InputId>,
    timer: Option<Timer>,
    reenviando_ups: bool,

    // Solo lo usa la rueda (InputState::Pulse): cuántos pulsos lleva la
    // ráfaga actual. Nunca se toca para teclas/clics normales.
    pulsos: u64,
}

impl Grupo {
    fn nuevo() -> Self {
        Self {
            presionados: Vec::new(),
            timer: None,
            reenviando_ups: false,
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

                // ¿Es el segundo Down de un objetivo que está en fase
                // "esperando doble"? -> Doble, resuelto en el acto.
                if let Some((clave, generacion)) = self.buscar_espera_doble(&grupos, &evento.input)
                {
                    if let Some(grupo) = grupos.get_mut(&clave) {
                        grupo.timer = None;
                        grupo.presionados.push(evento.input.clone());
                    }
                    let _ = generacion;
                    drop(grupos);

                    // OJO: acá NO se reenvía este segundo Down (a
                    // diferencia del primero, más abajo). Cache/perfil_ui
                    // ya se enteraron de esta tecla con el primer Down —
                    // este segundo Down es la MISMA tecla confirmando el
                    // doble, no una tecla nueva que agregar a la
                    // secuencia. Reenviarlo de nuevo duplicaba la tecla
                    // en Captura (quedaba como "modificador + gatillo x2")
                    // y en Cache generaba una lista extra de la nada —
                    // ver el punto de índices inestables en cache.rs.
                    Self::enviar_condicion(self.modo, CondicionTrigger::Doble);
                    return Some(());
                }

                let clave = evento.input.clone();
                let grupo = grupos.entry(clave).or_insert_with(Grupo::nuevo);
                grupo.presionados.push(evento.input.clone());
                drop(grupos);

                self.reenviar_down(evento.input.clone());
                Some(())
            }

            InputState::Up => {
                let clave = Self::clave_grupo(&grupos, &evento.input)?;

                let reenviar_up = grupos
                    .get(&clave)
                    .map(|g| g.reenviando_ups)
                    .unwrap_or(false);

                let era_objetivo_mantenido_pendiente = grupos
                    .get(&clave)
                    .and_then(|g| g.timer.as_ref())
                    .map(|t| t.objetivo == evento.input)
                    .unwrap_or(false);

                if let Some(grupo) = grupos.get_mut(&clave) {
                    grupo.presionados.retain(|i| i != &evento.input);
                }

                if reenviar_up && self.modo == ModoAnalizador::Runtime {
                    cache::recibir_up(evento.input.clone());
                }

                if era_objetivo_mantenido_pendiente {
                    // Se soltó antes de cumplirse tiempo_mantenido.
                    // Arranca la Fase B (esperar tiempo_doble) desde ahora.
                    let generacion = grupos
                        .get(&clave)
                        .and_then(|g| g.timer.as_ref())
                        .map(|t| t.generacion + 1)
                        .unwrap_or(0);

                    if let Some(grupo) = grupos.get_mut(&clave) {
                        grupo.timer = Some(Timer {
                            generacion,
                            objetivo: evento.input.clone(),
                        });
                    }

                    let grupos_arc = Arc::clone(&self.grupos);
                    let modo = self.modo;
                    let objetivo = evento.input.clone();
                    let clave2 = clave.clone();

                    std::thread::spawn(move || {
                        Self::esperar_doble(grupos_arc, clave2, objetivo, generacion, modo);
                    });
                }

                let vacio = grupos
                    .get(&clave)
                    .map(|g| g.presionados.is_empty())
                    .unwrap_or(false);

                if vacio && !era_objetivo_mantenido_pendiente {
                    grupos.remove(&clave);
                }

                Some(())
            }

            InputState::Pulse => {
                let clave = evento.input.clone();
                let es_primero = !grupos.contains_key(&clave);

                let generacion = {
                    let grupo = grupos.entry(clave.clone()).or_insert_with(Grupo::nuevo);
                    grupo.pulsos += 1;

                    let generacion = grupo.timer.as_ref().map(|t| t.generacion + 1).unwrap_or(0);

                    grupo.timer = Some(Timer {
                        generacion,
                        objetivo: clave.clone(),
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
    /// en Fase B (esperando doble) en algún grupo.
    fn buscar_espera_doble(
        &self,
        grupos: &HashMap<InputId, Grupo>,
        input: &InputId,
    ) -> Option<(InputId, u64)> {
        grupos.iter().find_map(|(clave, g)| {
            g.timer
                .as_ref()
                .filter(|t| &t.objetivo == input)
                .map(|t| (clave.clone(), t.generacion))
        })
    }

    fn reenviar_down(&self, input: InputId) {
        match self.modo {
            ModoAnalizador::Runtime => cache::recibir_down(input.clone()),
            ModoAnalizador::Captura => perfil_ui::recibir_down(input.clone()),
        }

        if self.modo == ModoAnalizador::Captura {
            self.iniciar_timer(input);
        }
    }

    /// En Runtime, Cache lo pide explícitamente al detectar ambigüedad.
    /// En Captura, se llama solo con cada Down nuevo.
    pub fn iniciar_timer(&self, objetivo: InputId) {
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
            g.reenviando_ups = modo == ModoAnalizador::Runtime;
        }
        drop(grupos_g);

        Self::enviar_condicion(modo, CondicionTrigger::Mantenido);
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
        drop(grupos_g);

        Self::enviar_condicion(modo, CondicionTrigger::Simple);
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

    /// Orden de Cache: descarta timer en curso y sale de la fase de
    /// reenvío de Ups. No toca "presionados ahora".
    pub fn limpiar(&self) {
        let mut grupos = self.grupos.lock().unwrap();
        for grupo in grupos.values_mut() {
            grupo.timer = None;
            grupo.reenviando_ups = false;
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
