// ======================================================
// 🗃️ Cache — ETAPAS 1 a 5 / 6 (ver DISENO_CACHE_V2.md)
// ======================================================
// Este archivo fusiona lo que antes eran cache.rs + analizador_
// trigger.rs (BORRADO, ver Etapa 5): Etapa 1 (datos compilados),
// Etapa 2 (motor de sesiones Runtime), Etapa 3 (motor de Captura) y
// Etapa 4 (filtro de repeats + ruteo). Desde la Etapa 5,
// lib.rs/entrada.rs/perfil_ui.rs ya apuntan acá (cache::) en vez de
// al archivo viejo — el proyecto compila de punta a punta.
// Queda pendiente la Etapa 6 (verificación funcional en la app real,
// checklist maestro).
// ======================================================

use crate::eventos::{InputEvent, InputId, InputState};
use crate::perfil_cache::{
    AccionCache, AppCache, CondicionTrigger, CoordenadaCache, ExtraCache, RemapeoCache,
};
use crate::{config, entrada, perfil_ui, runtime};
use std::sync::Mutex;

// ======================================================
// ============ ETAPA 1 — DATOS COMPILADOS ==============
// ======================================================
// (sin cambios respecto a la entrega anterior)

#[derive(Clone, PartialEq)]
pub struct AppEstadoCache {
    pub app: AppCache,
    pub activa: bool,
}

struct EstadoCompilado {
    remapeos: Vec<RemapeoCache>,
    apps: Vec<AppEstadoCache>,
}

static COMPILADO: Mutex<EstadoCompilado> = Mutex::new(EstadoCompilado {
    remapeos: Vec::new(),
    apps: Vec::new(),
});

pub fn escribir_cache(remapeos: Vec<RemapeoCache>) {
    COMPILADO.lock().unwrap().remapeos = remapeos;
}

pub fn borrar_cache() {
    COMPILADO.lock().unwrap().remapeos.clear();
}

pub fn esta_vacia() -> bool {
    COMPILADO.lock().unwrap().remapeos.is_empty()
}

pub fn obtener_remapeo(id: &str) -> Option<RemapeoCache> {
    COMPILADO
        .lock()
        .unwrap()
        .remapeos
        .iter()
        .find(|r| r.id == id)
        .cloned()
}

pub fn apps_a_vigilar() -> Vec<AppCache> {
    let compilado = COMPILADO.lock().unwrap();
    let mut vistas = Vec::new();

    for fila in compilado.remapeos.iter() {
        if fila.trigger.app == AppCache::Global {
            continue;
        }
        if !vistas.contains(&fila.trigger.app) {
            vistas.push(fila.trigger.app.clone());
        }
    }

    vistas
}

pub fn actualizar_estado_app(app: AppCache, activa: bool) {
    let mut compilado = COMPILADO.lock().unwrap();
    if let Some(e) = compilado.apps.iter_mut().find(|e| e.app == app) {
        e.activa = activa;
    } else {
        compilado.apps.push(AppEstadoCache { app, activa });
    }
}

fn app_habilitada(app: &AppCache, apps: &[AppEstadoCache]) -> bool {
    apps.iter()
        .find(|e| &e.app == app)
        .map(|e| e.activa)
        .unwrap_or(true)
}

/// Cuenta posibles/exactas de `entrada` contra los remapeos
/// compilados, filtrando por app habilitada. Recibe el guard de
/// COMPILADO ya abierto — nunca vuelve a pedir el lock.
fn contar(compilado: &EstadoCompilado, entrada: &[InputId]) -> (usize, usize, Vec<RemapeoCache>) {
    let mut posibles = 0;
    let mut candidatas = Vec::new();

    for fila in compilado.remapeos.iter() {
        if !app_habilitada(&fila.trigger.app, &compilado.apps) {
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

/// Snapshot clonado de COMPILADO — lo usa recibir_down_rt/recibir_up_rt
/// para poder llamar contar() varias veces sin mantener el lock de
/// COMPILADO tomado mientras trabajan con RUNTIME (regla de oro:
/// nunca dos Mutex tomados a la vez).
fn compilado_actual() -> EstadoCompilado {
    let g = COMPILADO.lock().unwrap();
    EstadoCompilado {
        remapeos: g.remapeos.clone(),
        apps: g.apps.clone(),
    }
}

// ======================================================
// ======= ETAPA 2 — MOTOR DE SESIONES RUNTIME ==========
// ======================================================
// 1. ¿Qué hace esta parte?
//
// Matching de cada Down/Up real contra los remapeos
// compilados (Etapa 1), con timers de 3 fases + rueda, e
// instancias activas esperando su Up real (Turbo/Mantener/
// Click Sostenido/Normal).
//
// Reemplaza a Lista (cache.rs viejo) + Grupo (archivo aparte viejo,
// ya borrado) por un único concepto: Sesion. Su `entrada` es la ÚNICA
// fuente de "qué hay en este grupo" — ya no hay una copia acá y
// otra en un archivo aparte (esa duplicación era la sospechosa
// número uno del bug que motivó esta reescritura).
// ------------------------------------------------------
// 2. ¿Quién llama esta parte?
// La Etapa 4 (filtro de repeats + ruteo, se agrega después a
//     este mismo archivo) — único punto de entrada real:
//     recibir_down_rt() / recibir_up_rt(). Hasta que esa etapa
//     no esté, estas dos funciones no las llama nadie más.
// runtime.rs — recibe OrdenRuntime::Iniciar / Detener.
// entrada.rs — recibe retener() / pasar() / consumir(), SIN
//     cambios de contrato respecto a hoy.
// ------------------------------------------------------
// 3. Decisión de diseño que se apartó del documento original
// (marcarlo para que quede asentado, ver regla "no improvisar"):
//
// El documento DISENO_CACHE_V2.md describía reiniciar_desde_
// presionados() como una consulta global (heredada del viejo
// obtener_presionados() del analizador). Al implementar, esa
// consulta global ya no hace falta: como Sesion.entrada es la
// ÚNICA fuente de verdad (ya no hay un tracker físico aparte),
// "lo que sigue presionado" después de resolver un match SIEMPRE
// es un subconjunto de la ENTRADA de la sesión que se acaba de
// resolver — nunca de otra sesión (las demás sesiones siguen su
// vida aparte, sin tocarse). Por eso reembrar_fantasma() recibe
// directo la lista de lo que quedó sin soltar, calculada en el
// mismo lugar donde se resuelve el match, en vez de recorrer
// todo el estado de nuevo. Mismo resultado, menos código y sin
// la dependencia cruzada que causaba el bug original.
// ======================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum FaseSesion {
    /// Acumulando Downs, sin timer corriendo.
    Construyendo,
    /// Timer con tiempo_mantenido corriendo sobre el último Down
    /// agregado (el "objetivo" es siempre entrada.last()).
    EsperandoMantenido {
        necesita_doble: bool,
        necesita_triple: bool,
    },
    /// Timer con tiempo_doble corriendo, desde el Up del primer toque.
    EsperandoDoble,
    /// Timer con tiempo_triple corriendo. `toques` arranca en 1
    /// (representa el primer toque, ya ocurrido).
    EsperandoTriple { toques: u8 },
    /// Exclusivo de la rueda del mouse — se completa en la Etapa 4
    /// (ahí es donde llegan los InputState::Pulse). Queda declarado
    /// acá porque es parte del mismo enum de fases y del mismo
    /// mecanismo de timers.
    CerrandoRueda { pulsos: u64 },
}

struct Sesion {
    /// Identidad estable — nunca cambia, no depende de la posición
    /// en el Vec.
    id: u64,
    entrada: Vec<InputId>,
    fase: FaseSesion,
    /// Invalida timers viejos que ya no aplican.
    generacion: u64,
    /// Solo Runtime. true = sembrada especulativamente tras resolver
    /// algo (ver resembrar_fantasma). Nadie en entrada.rs tiene un
    /// RETENIDO esperándola. Las sesiones de Captura NUNCA son
    /// fantasma (siempre queda en false).
    fantasma: bool,
    /// Solo Captura (ver Etapa 3, más abajo). Una condición ya
    /// resuelta pero que no se manda todavía porque sigue quedando
    /// algo físicamente presionado. Runtime nunca la toca — sus
    /// resoluciones siempre se mandan en el acto.
    pendiente: Option<CondicionTrigger>,
}

impl Sesion {
    fn nueva(id: u64, entrada: Vec<InputId>, fantasma: bool) -> Self {
        Self {
            id,
            entrada,
            fase: FaseSesion::Construyendo,
            generacion: 0,
            fantasma,
            pendiente: None,
        }
    }

    /// El "objetivo" de un timer siempre es el último elemento
    /// agregado a la entrada — la tecla que se está disambiguando.
    fn objetivo(&self) -> Option<&InputId> {
        self.entrada.last()
    }
}

struct InstanciaActiva {
    id: String,
    entrada: Vec<InputId>,
}

/// Orden que Cache le manda a Runtime para iniciar o detener la
/// ejecución de una acción. Sin cambios respecto al diseño viejo —
/// runtime.rs no se toca.
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

struct EstadoRuntime {
    sesiones: Vec<Sesion>,
    siguiente_id: u64,
    activas: Vec<InstanciaActiva>,
    /// Lo que está físicamente presionado ahora mismo (Runtime). NO es
    /// lo mismo que la unión de Sesion.entrada: entrada es historial
    /// acumulado de una sesión y a propósito NO se achica al pasar de
    /// EsperandoMantenido a EsperandoDoble/EsperandoTriple (ver
    /// objetivo()), porque el timer necesita seguir sabiendo cuál fue
    /// el último Down agregado incluso después de soltarlo. Usar
    /// `entrada` como filtro de repeats (Etapa 4) confundía un
    /// segundo/tercer toque real (Doble/Triple, con Up de por medio)
    /// con un auto-repeat de Windows — el bug quedó documentado en la
    /// nota (b) de la Etapa 4, que resultó incorrecta al confrontarla
    /// con la implementación real. Este campo es la única fuente de
    /// verdad para "¿está la tecla abajo ahora mismo?".
    presionadas: Vec<InputId>,
}

static RUNTIME: Mutex<EstadoRuntime> = Mutex::new(EstadoRuntime {
    sesiones: Vec::new(),
    siguiente_id: 0,
    activas: Vec::new(),
    presionadas: Vec::new(),
});

fn nuevo_id_sesion(runtime: &mut EstadoRuntime) -> u64 {
    runtime.siguiente_id += 1;
    runtime.siguiente_id
}

// ------------------------------------------------------
// 🔽 DOWN
// ------------------------------------------------------

/// Punto de entrada para cada Down REAL (ya filtrado de repeats —
/// ver Etapa 4). Nunca devuelve nada: avisa directo a entrada.rs
/// (pasar/retener/consumir), igual que el diseño viejo.
pub(crate) fn recibir_down_rt(input: InputId) {
    let mut runtime = RUNTIME.lock().unwrap();

    // --- Down interrumpe timer: ¿es un segundo/tercer toque de la
    // MISMA tecla que ya está en fase Doble o Triple? (ver
    // DISENO_CACHE_V2.md, "Down interrumpe timer"). No se confunde
    // con un repeat (Etapa 4 ya lo dejó pasar como Down real: hubo
    // un Up de por medio entre toques). ---
    if let Some(idx) = runtime.sesiones.iter().position(|s| {
        matches!(
            s.fase,
            FaseSesion::EsperandoDoble | FaseSesion::EsperandoTriple { .. }
        ) && s.objetivo() == Some(&input)
    }) {
        match runtime.sesiones[idx].fase {
            FaseSesion::EsperandoTriple { toques: 1 } => {
                // Segundo Down físico: solo cuenta, sigue esperando
                // (el timer esperar_triple sigue corriendo, va a leer
                // `toques` al despertar — NO se toca la generación).
                runtime.sesiones[idx].fase = FaseSesion::EsperandoTriple { toques: 2 };
                return;
            }
            FaseSesion::EsperandoDoble => {
                resolver_condicion_por_id(runtime, idx, CondicionTrigger::Doble);
                return;
            }
            FaseSesion::EsperandoTriple { toques: 2 } => {
                resolver_condicion_por_id(runtime, idx, CondicionTrigger::Triple);
                return;
            }
            _ => unreachable!(),
        }
    }

    // --- ¿Alguna sesión existente acepta esto como continuación
    // válida? (extiende una en construcción, o "despierta" una
    // fantasma — deja de serlo). ---
    let compilado = compilado_actual();

    let id = {
        let existente = runtime.sesiones.iter().position(|s| {
            let mut probable = s.entrada.clone();
            probable.push(input.clone());
            contar(&compilado, &probable).0 > 0
        });

        match existente {
            Some(idx) => {
                let s = &mut runtime.sesiones[idx];
                s.entrada.push(input.clone());
                s.fantasma = false;
                // Si estaba a mitad de un timer de otra tecla del
                // mismo grupo (caso raro: tercera tecla ajena
                // mientras se disambigua una Doble/Triple previa),
                // ese timer queda invalidado por generación y la
                // sesión vuelve a Construyendo — se recalcula todo
                // de cero más abajo.
                s.fase = FaseSesion::Construyendo;
                s.generacion += 1;
                s.id
            }
            None => {
                let id = nuevo_id_sesion(&mut runtime);
                runtime
                    .sesiones
                    .push(Sesion::nueva(id, vec![input.clone()], false));
                id
            }
        }
    };

    let entrada_actual = runtime
        .sesiones
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.entrada.clone())
        .unwrap_or_default();

    let (posibles, exactas, candidatas) = contar(&compilado, &entrada_actual);

    if posibles == 0 {
        eliminar_sesion(&mut runtime, id);
        drop(runtime);
        entrada::pasar();
        return;
    }

    if posibles == exactas
        && exactas == 1
        && candidatas[0].trigger.condicion == CondicionTrigger::Simple
    {
        let remapeo = candidatas[0].clone();
        eliminar_sesion(&mut runtime, id);
        drop(runtime);
        resolver_match(remapeo, entrada_actual, id, Vec::new());
        return;
    }

    if posibles == exactas && exactas >= 1 {
        let necesita_doble = candidatas
            .iter()
            .any(|c| c.trigger.condicion == CondicionTrigger::Doble);
        let necesita_triple = candidatas
            .iter()
            .any(|c| c.trigger.condicion == CondicionTrigger::Triple);
        iniciar_espera_mantenido(&mut runtime, id, necesita_doble, necesita_triple);
        drop(runtime);
        entrada::retener();
        return;
    }

    // posibles > exactas: sigue habiendo prefijos más largos. Si hay
    // un Simple candidato en el medio, igual conviene arrancar el
    // timer de Mantenido (sin exigir doble/triple) para poder
    // resolverlo si el Up llega antes de que se complete un prefijo
    // más largo (ver DISENO_CACHE_V2.md, punto 6 de recibir_down_rt).
    let hay_simple = candidatas
        .iter()
        .any(|c| c.trigger.condicion == CondicionTrigger::Simple);
    if hay_simple {
        iniciar_espera_mantenido(&mut runtime, id, false, false);
    }
    drop(runtime);
    entrada::retener();
}

fn iniciar_espera_mantenido(
    runtime: &mut EstadoRuntime,
    id: u64,
    necesita_doble: bool,
    necesita_triple: bool,
) {
    let Some(s) = runtime.sesiones.iter_mut().find(|s| s.id == id) else {
        return;
    };
    s.generacion += 1;
    s.fase = FaseSesion::EsperandoMantenido {
        necesita_doble,
        necesita_triple,
    };
    let generacion = s.generacion;

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_mantenido()));
        let mut runtime = RUNTIME.lock().unwrap();
        if !vigente(&runtime, id, generacion) {
            return;
        }
        let idx = runtime.sesiones.iter().position(|s| s.id == id).unwrap();
        resolver_condicion_por_id(runtime, idx, CondicionTrigger::Mantenido);
    });
}

fn vigente(runtime: &EstadoRuntime, id: u64, generacion: u64) -> bool {
    runtime
        .sesiones
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.generacion == generacion)
        .unwrap_or(false)
}

fn eliminar_sesion(runtime: &mut EstadoRuntime, id: u64) {
    runtime.sesiones.retain(|s| s.id != id);
}

// ------------------------------------------------------
// ⏱️ RESOLUCIÓN DE CONDICIÓN (llamada por Down interrumpe timer,
// por los timers al expirar, y por el Up-handler)
// ------------------------------------------------------

/// Resuelve la condición `condicion` para la sesión en `runtime.sesiones[idx]`.
/// Si hay match exacto contra los remapeos compilados, ejecuta. Si no,
/// aborta (Pasar). Toma posesión del MutexGuard (en vez de un `&mut`
/// prestado) para poder soltarlo explícitamente ANTES de llamar a
/// resolver_match()/entrada::pasar() — llamarlas con el lock de RUNTIME
/// todavía tomado es exactamente el deadlock que la "regla de oro"
/// del diseño prohíbe (resolver_match, para un Extra diferido como
/// Normal/Turbo/Mantener/ClickSostenido, vuelve a pedir RUNTIME.lock()
/// para registrar la InstanciaActiva). Todos los llamadores actuales
/// ya tratan esta llamada como el último uso de `runtime` en su scope
/// (return inmediato después, o fin del closure del timer), así que
/// mover el `drop` para acá no les cambia el comportamiento.
fn resolver_condicion_por_id(
    runtime: std::sync::MutexGuard<'static, EstadoRuntime>,
    idx: usize,
    condicion: CondicionTrigger,
) {
    let mut runtime = runtime;
    let id = runtime.sesiones[idx].id;
    let entrada_actual = runtime.sesiones[idx].entrada.clone();

    let compilado = compilado_actual();
    let (posibles, exactas, candidatas) = contar(&compilado, &entrada_actual);

    let match_final = candidatas
        .iter()
        .find(|c| c.trigger.condicion == condicion && c.trigger.entrada == entrada_actual)
        .cloned();

    // Para Simple, se permite ejecutar aunque haya prefijos más
    // largos sin completar (ver DISENO_CACHE_V2.md / comentario
    // histórico "3>A" en cache.rs viejo).
    if let Some(remapeo) = match_final {
        if condicion == CondicionTrigger::Simple || posibles == exactas {
            eliminar_sesion(&mut runtime, id);
            drop(candidatas);
            drop(runtime); // ⚠️ soltar ANTES de resolver_match — ver doc de arriba
            resolver_match(remapeo, entrada_actual, id, Vec::new());
            return;
        }
    }

    eliminar_sesion(&mut runtime, id);
    drop(runtime); // ⚠️ soltar ANTES de entrada::pasar()
    entrada::pasar();
}

/// Ejecuta el match: decide Iniciar-solo vs Iniciar+Finalizar según
/// `ExtraCache::requiere_up_real()`, avisa a Runtime, resiembra lo
/// que haya quedado físicamente presionado como sesión fantasma, y
/// cierra el RETENIDO de entrada.rs con consumir(). Se llama SIEMPRE
/// con el lock de RUNTIME ya liberado (evita el deadlock del diseño
/// viejo: runtime::ejecutar()/entrada::consumir() nunca deben
/// llamarse con RUNTIME tomado).
///
/// `restantes`: lo que sigue físicamente presionado de ESTA sesión
/// después del match (ver nota de diseño más arriba, punto 3) — para
/// un match resuelto por Down, es la entrada completa salvo que se
/// indique lo contrario; para un match retroactivo por Up, es
/// `entrada_antes` menos la tecla que se soltó.
fn resolver_match(remapeo: RemapeoCache, entrada: Vec<InputId>, _id: u64, restantes: Vec<InputId>) {
    let diferido = remapeo
        .extra
        .as_ref()
        .is_some_and(|extra| extra.requiere_up_real());

    if diferido {
        let mut runtime = RUNTIME.lock().unwrap();
        runtime.activas.push(InstanciaActiva {
            id: remapeo.id.clone(),
            entrada,
        });
        drop(runtime);
        runtime::ejecutar(OrdenRuntime::Iniciar {
            id: remapeo.id,
            accion: remapeo.accion,
            extra: remapeo.extra,
            coordenada: remapeo.coordenada,
        });
    } else {
        runtime::ejecutar(OrdenRuntime::Iniciar {
            id: remapeo.id.clone(),
            accion: remapeo.accion,
            extra: remapeo.extra,
            coordenada: remapeo.coordenada,
        });
        runtime::ejecutar(OrdenRuntime::Detener { id: remapeo.id });
    }

    resembrar_fantasma(restantes);
    entrada::consumir();
}

/// Reemplaza cualquier sesión fantasma existente por una nueva con
/// `restantes` (o por ninguna, si `restantes` está vacío). Ver nota
/// de diseño al principio de la Etapa 2: ya no consulta un tracker
/// físico global, recibe directo lo que quedó sin soltar de la
/// sesión que se acaba de resolver.
fn resembrar_fantasma(restantes: Vec<InputId>) {
    let mut runtime = RUNTIME.lock().unwrap();
    runtime.sesiones.retain(|s| !s.fantasma);

    if !restantes.is_empty() {
        let id = nuevo_id_sesion(&mut runtime);
        runtime.sesiones.push(Sesion::nueva(id, restantes, true));
    }
}

// ------------------------------------------------------
// 🔼 UP
// ------------------------------------------------------

/// Punto de entrada para cada Up REAL. Tres casos, en este orden
/// (ver DISENO_CACHE_V2.md, recibir_up_rt):
/// 1. Instancia activa (Mantenido/Turbo/etc.) esperando este Up ->
///    se perdió el match, Finalizar.
/// 2. Sesión con timer corriendo (fase != Construyendo):
///    - Si `input` es el objetivo Y la fase es EsperandoMantenido ->
///      este Up interrumpe Mantenido, decide a qué fase pasar.
///    - Cualquier otro caso (Up de un modificador, o Up intermedio
///      durante Doble/Triple) -> no-op, se ignora por completo.
/// 3. Sesión en Construyendo:
///    - Fantasma -> se descarta la sesión ENTERA.
///    - No fantasma -> resolución retroactiva a Simple, o abortar
///      todo lo retenido.
pub(crate) fn recibir_up_rt(input: InputId) {
    let mut runtime = RUNTIME.lock().unwrap();

    if let Some(pos) = runtime
        .activas
        .iter()
        .position(|a| a.entrada.contains(&input))
    {
        let instancia = runtime.activas.remove(pos);
        drop(runtime);
        runtime::ejecutar(OrdenRuntime::Detener { id: instancia.id });
        resembrar_fantasma(Vec::new());
        return;
    }

    let Some(idx) = runtime
        .sesiones
        .iter()
        .position(|s| s.entrada.contains(&input))
    else {
        return; // no pertenece a ninguna sesión (ya se sacó antes, o nunca estuvo)
    };

    if runtime.sesiones[idx].fase != FaseSesion::Construyendo {
        let es_objetivo_mantenido = matches!(
            runtime.sesiones[idx].fase,
            FaseSesion::EsperandoMantenido { .. }
        ) && runtime.sesiones[idx].objetivo() == Some(&input);

        if !es_objetivo_mantenido {
            return; // no-op: Up de modificador, o toque intermedio de Doble/Triple
        }

        let (necesita_doble, necesita_triple) = match runtime.sesiones[idx].fase {
            FaseSesion::EsperandoMantenido {
                necesita_doble,
                necesita_triple,
            } => (necesita_doble, necesita_triple),
            _ => unreachable!(),
        };

        if necesita_triple {
            runtime.sesiones[idx].generacion += 1;
            runtime.sesiones[idx].fase = FaseSesion::EsperandoTriple { toques: 1 };
            iniciar_timer_generico(
                &mut runtime,
                idx,
                config::tiempo_triple(),
                CondicionTrigger::Doble, // fallback si expira con 2 toques (ver timer)
            );
        } else if necesita_doble {
            runtime.sesiones[idx].generacion += 1;
            runtime.sesiones[idx].fase = FaseSesion::EsperandoDoble;
            iniciar_timer_generico(
                &mut runtime,
                idx,
                config::tiempo_doble(),
                CondicionTrigger::Simple,
            );
        } else {
            resolver_condicion_por_id(runtime, idx, CondicionTrigger::Simple);
        }
        return;
    }

    // Fase Construyendo:
    if runtime.sesiones[idx].fantasma {
        let id = runtime.sesiones[idx].id;
        eliminar_sesion(&mut runtime, id);
        return;
    }

    let id = runtime.sesiones[idx].id;
    let entrada_antes = runtime.sesiones[idx].entrada.clone();
    let mut entrada_sin_tecla = entrada_antes.clone();
    entrada_sin_tecla.retain(|i| i != &input);

    let compilado = compilado_actual();

    let buscar_simple = |entrada: &[InputId]| -> Option<RemapeoCache> {
        let (posibles, exactas, candidatas) = contar(&compilado, entrada);
        if posibles == exactas && exactas == 1 {
            candidatas
                .into_iter()
                .find(|c| c.trigger.condicion == CondicionTrigger::Simple)
        } else {
            None
        }
    };

    if let Some(remapeo) = buscar_simple(&entrada_sin_tecla) {
        eliminar_sesion(&mut runtime, id);
        drop(runtime);
        resolver_match(remapeo, entrada_sin_tecla, id, Vec::new());
        return;
    }

    if let Some(remapeo) = buscar_simple(&entrada_antes) {
        eliminar_sesion(&mut runtime, id);
        drop(runtime);
        // El Up ya pasó — nada de esta sesión sigue físicamente
        // presionado salvo lo que no sea `input` (que fue lo que
        // recién se soltó y disparó esta resolución retroactiva).
        resolver_match(remapeo, entrada_antes.clone(), id, entrada_sin_tecla);
        return;
    }

    // No hubo match posible: abortar todo lo retenido. Lo que siga
    // presionado de esta sesión (entrada_sin_tecla) se resiembra
    // como fantasma nueva.
    eliminar_sesion(&mut runtime, id);
    drop(runtime);
    entrada::pasar();
    resembrar_fantasma(entrada_sin_tecla);
}

// ------------------------------------------------------
// ⏱️ TIMERS (un hilo por timer)
// ------------------------------------------------------

/// Timer genérico para las fases Doble/Triple: duerme `espera_ms`, y
/// si sigue vigente al despertar, resuelve `condicion_si_expira` —
/// EXCEPTO en fase Triple, donde la condición real depende de
/// `toques` (se recalcula al despertar, `condicion_si_expira` ahí
/// solo es un valor de referencia sin uso, ver más abajo).
fn iniciar_timer_generico(
    runtime: &mut EstadoRuntime,
    idx: usize,
    espera_ms: u64,
    condicion_si_expira: CondicionTrigger,
) {
    let id = runtime.sesiones[idx].id;
    let generacion = runtime.sesiones[idx].generacion;
    let fase_iniciada = runtime.sesiones[idx].fase;

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(espera_ms));
        let mut runtime = RUNTIME.lock().unwrap();
        if !vigente(&runtime, id, generacion) {
            return; // se resolvió por Down antes de expirar
        }
        let idx = runtime.sesiones.iter().position(|s| s.id == id).unwrap();

        let condicion = match fase_iniciada {
            FaseSesion::EsperandoTriple { .. } => {
                let toques = match runtime.sesiones[idx].fase {
                    FaseSesion::EsperandoTriple { toques } => toques,
                    _ => 1,
                };
                if toques >= 2 {
                    CondicionTrigger::Doble
                } else {
                    CondicionTrigger::Simple
                }
            }
            _ => condicion_si_expira,
        };

        resolver_condicion_por_id(runtime, idx, condicion);
    });
}

// ======================================================
// ============ ETAPA 3 — MOTOR DE CAPTURA ==============
// ======================================================
// 1. ¿Qué hace esta parte?
//
// La sesión única de Captura: mientras el usuario define un gatillo
// nuevo desde la UI, esto recibe cada Down/Up real (ya filtrado de
// repeats — ver Etapa 4) y arma, con el mismo mecanismo de fases y
// timers de la Etapa 2 (Mantenido → Doble/Triple/Simple), la
// condición final. A diferencia de Runtime, acá no hay candidatas
// compiladas que consultar: todo Down es válido, y necesita_doble /
// necesita_triple llegan SIEMPRE en true (no se sabe todavía qué
// condición va a terminar siendo — eso es justo lo que se está
// grabando). Por eso, en la transición Mantenido→(Doble|Triple) que
// decide el Up-handler, necesita_triple manda siempre (mismo orden
// de prioridad que Runtime) y la fase EsperandoDoble nunca se
// alcanza en la práctica — se deja el camino igual, por si algún día
// deja de estar hardcodeado.
// ------------------------------------------------------
// 2. ¿Quién llama esta parte?
// La Etapa 4 (filtro de repeats + ruteo, se agrega después a este
//     mismo archivo) — único punto de entrada real:
//     recibir_down_captura() / recibir_up_captura(). Hasta que esa
//     etapa no esté, nadie llama estas funciones todavía (mismo
//     estado que recibir_down_rt/recibir_up_rt en la Etapa 2).
// perfil_ui.rs — recibe recibir_down() con cada Down nuevo (ya
//     filtrado, uno por tecla) y recibir_condicion() con el
//     resultado final. Ambas firmas SIN cambios respecto a hoy —
//     perfil_ui ya arma el TriggerCapturaUI completo con su propia
//     secuencia acumulada (via recibir_down), no hace falta
//     mandarle nada más.
// ------------------------------------------------------
// 3. Decisiones de diseño que se apartaron del documento original
// (marcarlas para que quede asentado, ver regla "no improvisar"):
//
// a) `EstadoCaptura` NO guarda fila_id/columna. El documento los
//    incluía en el modelo de datos, pero la propia nota de la Etapa
//    4 (sección "Cambio de firma a marcar") ya adelantaba que
//    convenía dejarlos únicamente en perfil_ui.rs (que ya los
//    guarda en su propio CapturaEnCurso) y que `cache::activar_
//    captura()` quedara sin argumentos — evita datos duplicados sin
//    ningún uso real del lado de Cache. Por eso `activar_captura_
//    interna()` tampoco los recibe.
//
// b) `Sesion` gana un campo nuevo (`pendiente`, ver arriba en la
//    Etapa 2) que la entrega anterior no incluía — el documento ya
//    lo preveía en el modelo de datos unificado ("Solo Captura"),
//    simplemente no hacía falta hasta esta etapa.
//
// c) `EstadoCaptura` agrega `presionadas: Vec<InputId>`, aparte de
//    `Sesion.entrada`. Esto NO es la misma duplicación que motivó la
//    reescritura (dos estructuras representando LO MISMO): acá son
//    dos cosas genuinamente distintas.
//    - `entrada` es la secuencia acumulada del gesto, creciente,
//      nunca se achica — la usa `objetivo()` para saber sobre qué
//      tecla sigue corriendo el timer de Doble/Triple, incluso
//      después de que esa tecla se soltó (igual que en Runtime,
//      donde `entrada` tampoco se achica cuando el Up interrumpe
//      Mantenido).
//    - `presionadas` es el estado físico real (qué sigue abajo
//      AHORA), se achica en cada Up — la usa `resolver_condicion_
//      captura` para decidir si ya puede mandar el resultado o si
//      tiene que posponerlo (`pendiente`), y `flush_si_vacio` para
//      saber cuándo, al fin, mandarlo.
//    Si `entrada` hiciera las dos cosas a la vez (como insinuaba el
//    documento con "Saca input de entrada" en el Up-handler), se
//    perdería el objetivo del timer apenas se soltara la tecla que
//    se está disambiguando — exactamente el tipo de bug de estado
//    cruzado que esta reescritura busca evitar.
// ======================================================

struct EstadoCaptura {
    activa: bool,
    /// Solo existe (Some) mientras `activa`. Reusa el mismo struct
    /// Sesion y la misma máquina de fases/timers de Runtime, aunque
    /// Captura solo tenga una a la vez y nunca sea fantasma.
    sesion: Option<Sesion>,
    /// Lo que sigue físicamente presionado del gesto en curso — ver
    /// nota de diseño (c) más arriba.
    presionadas: Vec<InputId>,
}

static CAPTURA: Mutex<EstadoCaptura> = Mutex::new(EstadoCaptura {
    activa: false,
    sesion: None,
    presionadas: Vec::new(),
});

/// Abre una captura nueva: limpia cualquier resto de una captura
/// anterior y arranca en blanco. Sin argumentos — ver nota de diseño
/// (a) más arriba (fila_id/columna se quedan en perfil_ui.rs).
fn activar_captura_interna() {
    let mut captura = CAPTURA.lock().unwrap();
    captura.activa = true;
    captura.sesion = None;
    captura.presionadas.clear();
}

/// Cierra la captura por completo: ya no hay `desactivar_captura()`
/// como llamada externa aparte (decisión ya confirmada, ver cabecera
/// del documento) — esto queda inline, se llama solo desde donde se
/// termina de resolver y mandar el resultado final.
fn desactivar_captura_interna(captura: &mut EstadoCaptura) {
    captura.activa = false;
    captura.sesion = None;
    captura.presionadas.clear();
}

fn vigente_captura(captura: &EstadoCaptura, generacion: u64) -> bool {
    captura
        .sesion
        .as_ref()
        .map(|s| s.generacion == generacion)
        .unwrap_or(false)
}

// ------------------------------------------------------
// 🔽 DOWN
// ------------------------------------------------------

/// Punto de entrada para cada Down REAL de una captura en curso (ya
/// filtrado de repeats — ver Etapa 4).
pub(crate) fn recibir_down_captura(input: InputId) {
    let mut captura = CAPTURA.lock().unwrap();
    if !captura.activa {
        return; // defensivo — en uso normal Etapa 4 no llega hasta acá si no hay captura activa
    }

    // --- Down interrumpe timer: ¿la sesión de Captura está
    // esperando Doble/Triple sobre esta MISMA tecla? (mismo
    // mecanismo que recibir_down_rt en Runtime — acá solo hay una
    // sesión, así que no hace falta buscarla). ---
    let interrupcion = captura.sesion.as_ref().and_then(|s| {
        if s.objetivo() != Some(&input) {
            return None;
        }
        match s.fase {
            FaseSesion::EsperandoDoble => Some(FaseSesion::EsperandoDoble),
            FaseSesion::EsperandoTriple { toques } => Some(FaseSesion::EsperandoTriple { toques }),
            _ => None,
        }
    });

    if let Some(fase_previa) = interrupcion {
        // Segundo/tercer toque físico de la misma tecla: cuenta,
        // pero NO se reenvía de nuevo a perfil_ui (ya la recibió con
        // el primer Down — ver header, punto 1).
        captura.presionadas.push(input);

        let resultado = match fase_previa {
            FaseSesion::EsperandoDoble => {
                resolver_condicion_captura(&mut captura, CondicionTrigger::Doble)
            }
            FaseSesion::EsperandoTriple { toques: 1 } => {
                if let Some(s) = captura.sesion.as_mut() {
                    s.fase = FaseSesion::EsperandoTriple { toques: 2 };
                }
                None
            }
            FaseSesion::EsperandoTriple { .. } => {
                resolver_condicion_captura(&mut captura, CondicionTrigger::Triple)
            }
            _ => unreachable!(),
        };

        drop(captura);
        if let Some(condicion) = resultado {
            perfil_ui::recibir_condicion(condicion);
        }
        return;
    }

    // --- Down realmente nuevo: se agrega a la sesión (creándola si
    // hace falta), se reenvía a perfil_ui, y se reinicia el timer de
    // Mantenido sobre ESTA tecla — la anterior, si había una a
    // mitad de camino, queda invalidada por generación y pasa a ser
    // modificador implícito (ver header, decisión ya confirmada). ---
    captura.presionadas.push(input.clone());
    let sesion = captura
        .sesion
        .get_or_insert_with(|| Sesion::nueva(0, Vec::new(), false));
    sesion.entrada.push(input.clone());
    sesion.generacion += 1;
    sesion.fase = FaseSesion::EsperandoMantenido {
        necesita_doble: true,
        necesita_triple: true,
    };
    let generacion = sesion.generacion;
    drop(captura);

    perfil_ui::recibir_down(input);
    iniciar_timer_mantenido_captura(generacion);
}

// ------------------------------------------------------
// 🔼 UP
// ------------------------------------------------------

/// Punto de entrada para cada Up REAL de una captura en curso.
pub(crate) fn recibir_up_captura(input: InputId) {
    let mut captura = CAPTURA.lock().unwrap();
    if !captura.activa {
        return;
    }

    captura.presionadas.retain(|i| i != &input);

    // ¿Esta tecla es el objetivo de un timer en EsperandoMantenido?
    // Si es así, este Up lo interrumpe — decide a qué fase pasar.
    let interrumpe_mantenido = captura.sesion.as_ref().is_some_and(|s| {
        s.objetivo() == Some(&input) && matches!(s.fase, FaseSesion::EsperandoMantenido { .. })
    });

    if interrumpe_mantenido {
        // necesita_doble y necesita_triple llegaron siempre en true
        // desde recibir_down_captura → necesita_triple manda siempre
        // (mismo orden de prioridad que Runtime): acá SIEMPRE se pasa
        // a EsperandoTriple, nunca a Doble ni a Simple inmediato (ver
        // nota 1 de la cabecera de esta etapa).
        let sesion = captura.sesion.as_mut().unwrap();
        sesion.generacion += 1;
        sesion.fase = FaseSesion::EsperandoTriple { toques: 1 };
        let generacion = sesion.generacion;
        drop(captura);
        iniciar_timer_triple_captura(generacion);
        return;
    }

    // No interrumpe ningún timer en curso: si esta tecla era la
    // última que seguía físicamente presionada y hay una condición
    // ya resuelta esperando (`pendiente`), este es el momento de
    // mandarla. Si sigue habiendo algo presionado, o si no hay
    // pendiente todavía (el timer en curso sigue su ciclo solo), no
    // hay nada más que hacer por ahora.
    if let Some(condicion) = flush_si_vacio(&mut captura) {
        drop(captura);
        perfil_ui::recibir_condicion(condicion);
    }
}

// ------------------------------------------------------
// ⏱️ RESOLUCIÓN DE CONDICIÓN Y POSPOSICIÓN
// ------------------------------------------------------

/// Resuelve `condicion` para la sesión de Captura en curso. Si
/// todavía queda algo físicamente presionado, la guarda en
/// `pendiente` (pisando cualquier pendiente anterior — la última
/// resolución manda, igual que el diseño viejo) y NO cierra la
/// captura. Si no queda nada presionado, cierra la captura ya mismo
/// y devuelve la condición para que el llamador la mande (siempre
/// con el lock de CAPTURA ya liberado — ver regla de oro
/// anti-deadlock).
fn resolver_condicion_captura(
    captura: &mut EstadoCaptura,
    condicion: CondicionTrigger,
) -> Option<CondicionTrigger> {
    if !captura.presionadas.is_empty() {
        if let Some(sesion) = captura.sesion.as_mut() {
            sesion.pendiente = Some(condicion);
            sesion.fase = FaseSesion::Construyendo;
        }
        return None;
    }

    desactivar_captura_interna(captura);
    Some(condicion)
}

/// Si ya no queda nada físicamente presionado y hay una condición
/// pendiente guardada, la retira y cierra la captura. Devuelve la
/// condición a mandar (con el lock todavía tomado — el llamador debe
/// soltarlo antes de reenviarla a perfil_ui).
fn flush_si_vacio(captura: &mut EstadoCaptura) -> Option<CondicionTrigger> {
    if !captura.presionadas.is_empty() {
        return None;
    }

    let pendiente = captura.sesion.as_mut().and_then(|s| s.pendiente.take());
    if pendiente.is_some() {
        desactivar_captura_interna(captura);
    }
    pendiente
}

// ------------------------------------------------------
// ⏱️ TIMERS (un hilo por timer, igual que en Runtime)
// ------------------------------------------------------
// Solo hacen falta Mantenido y Triple: la fase Doble nunca se
// alcanza en Captura (ver nota 1 de la cabecera de esta etapa), así
// que no tiene sentido un timer propio para ella todavía.

fn iniciar_timer_mantenido_captura(generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_mantenido()));
        let mut captura = CAPTURA.lock().unwrap();
        if !vigente_captura(&captura, generacion) {
            return; // se interrumpió por Up/Down antes de expirar
        }
        let resultado = resolver_condicion_captura(&mut captura, CondicionTrigger::Mantenido);
        drop(captura);
        if let Some(condicion) = resultado {
            perfil_ui::recibir_condicion(condicion);
        }
    });
}

fn iniciar_timer_triple_captura(generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_triple()));
        let mut captura = CAPTURA.lock().unwrap();
        if !vigente_captura(&captura, generacion) {
            return; // llegó el tercer Down -> ya se resolvió en recibir_down_captura
        }
        let toques = match captura.sesion.as_ref().map(|s| s.fase) {
            Some(FaseSesion::EsperandoTriple { toques }) => toques,
            _ => 1,
        };
        let condicion = if toques >= 2 {
            CondicionTrigger::Doble
        } else {
            CondicionTrigger::Simple
        };
        let resultado = resolver_condicion_captura(&mut captura, condicion);
        drop(captura);
        if let Some(condicion) = resultado {
            perfil_ui::recibir_condicion(condicion);
        }
    });
}

// ======================================================
// ===== ETAPA 4 — PUNTO DE ENTRADA ÚNICO (FILTRO DE ====
// ===== REPEATS + RUTEO)                             ===
// ======================================================
// 1. ¿Qué hace esta parte?
//
// La capa que hoy hace AnalizadorTrigger::procesar(): recibe CADA
// InputEvent físico (Down/Up/Pulse), filtra el auto-repeat de
// Windows, y decide si el evento va al motor de Runtime (Etapa 2) o
// al de Captura (Etapa 3). Es la ÚNICA parte de este archivo que
// sabe de InputEvent completo — las etapas 2 y 3 solo trabajan con
// InputId puro (ya filtrado, ya "es un Down/Up real").
//
// El filtro de repeats ya NO necesita "alimentar" ningún timer con
// cada repeat (a diferencia del diseño viejo, que usaba el repeat
// como tick para revisar tiempo_mantenido): acá los timers son
// autónomos por sleep, se resuelven solos al despertar. Un repeat
// simplemente no reenvía nada y no toca nada más.
// ------------------------------------------------------
// 2. ¿Quién llama esta parte?
// entrada.rs — le entrega cada InputEvent físico vía
//     procesar_evento_runtime() (modo normal) o
//     procesar_evento_captura() (mientras hay captura activa, ver
//     captura_activa()). También llama soltar_fisico() en el atajo
//     DEVOLVIENDO (Up que nunca pasa por el pipeline normal).
// lib.rs — usa captura_activa() como el `Fn() -> bool` que le pasa a
//     back_interception::iniciar().
// perfil_ui.rs — llama activar_captura() al arrancar una captura
//     nueva (sin argumentos — ver nota (a) de la Etapa 3: fila_id/
//     columna se quedan en perfil_ui.rs, no hace falta duplicarlos
//     acá).
// ------------------------------------------------------
// 3. Decisiones de diseño que se apartaron del documento original
// (marcarlas para que quede asentado, ver regla "no improvisar"):
//
// a) [CORREGIDO tras pruebas — ver historial] El filtro de repeats
//    para Runtime originalmente consultaba RUNTIME.sesiones (¿`input`
//    está en la `entrada` de alguna sesión?) en vez de un tracker de
//    "presionadas ahora" aparte, razonando que Sesion.entrada era la
//    única fuente de verdad necesaria. Eso estaba MAL: entrada es el
//    historial acumulado de una sesión y a propósito NO se achica al
//    pasar de EsperandoMantenido a EsperandoDoble/EsperandoTriple
//    (ver objetivo()) — así que después de soltar el primer toque de
//    un Doble/Triple, `entrada` seguía conteniendo esa tecla aunque
//    ya no estuviera físicamente presionada. El filtro la trataba
//    como repeat y descartaba el segundo/tercer toque real sin que
//    llegara nunca a "Down interrumpe timer" (bloque (b) más abajo) —
//    la sesión quedaba esperando hasta que el timer expiraba solo, y
//    como no había match para la condición de fallback, terminaba en
//    entrada::pasar() (reenvía los eventos crudos retenidos a
//    Windows). Solución: EstadoRuntime.presionadas, un Vec<InputId>
//    aparte que solo refleja qué está físicamente abajo ahora mismo
//    (se agrega en cada Down real, se saca en cada Up) — mismo patrón
//    que ya usa EstadoCaptura.presionadas desde la Etapa 3. El filtro
//    de repeats consulta ESTO, nunca Sesion.entrada.
//
// b) Down interrumpe timer (Doble/Triple) para Runtime NO se repite
//    acá: ya vive en recibir_down_rt (Etapa 2, primer bloque de la
//    función), porque ese caso necesita revisar el estado completo de
//    las sesiones (fase, objetivo) para decidir a qué condición
//    resolver — el filtro de repeats de más arriba ya garantiza que,
//    para cuando este Down llega hasta acá, es un Down real (hubo Up
//    de por medio), así que este bloque no necesita, ni debe, volver
//    a preguntarse si es repeat.
// ======================================================

/// Consultada por entrada.rs antes que cualquier otra cosa: mientras
/// esté en true, TODO evento físico se consume y se reenvía acá vía
/// procesar_evento_captura(), nunca a Windows ni a Runtime.
pub fn captura_activa() -> bool {
    CAPTURA.lock().unwrap().activa
}

/// Llamada por perfil_ui.rs al arrancar una captura nueva. Sin
/// argumentos — ver nota (a) de la Etapa 3.
pub fn activar_captura() {
    activar_captura_interna();
}

// ------------------------------------------------------
// 🔁 MOTOR RUNTIME — filtro de repeats + ruteo
// ------------------------------------------------------

/// Punto de entrada único para cada InputEvent físico en modo
/// Runtime (ya filtrado de repeats acá mismo). Reemplaza a
/// AnalizadorTrigger::procesar() en modo Runtime.
pub fn procesar_evento_runtime(evento: InputEvent) {
    match evento.state {
        InputState::Down => procesar_down_runtime(evento.input),
        InputState::Up => procesar_up_runtime(evento.input),
        InputState::Pulse => procesar_pulse_runtime(evento.input),
    }
}

fn procesar_down_runtime(input: InputId) {
    let mut runtime = RUNTIME.lock().unwrap();
    if runtime.presionadas.contains(&input) {
        // Repeat: ya estaba físicamente presionada, sin Up de por
        // medio. No se reenvía.
        return;
    }
    runtime.presionadas.push(input.clone());
    drop(runtime);
    recibir_down_rt(input);
}

fn procesar_up_runtime(input: InputId) {
    {
        let mut runtime = RUNTIME.lock().unwrap();
        runtime.presionadas.retain(|i| i != &input);
    }
    // A diferencia del Down, el Up siempre se reenvía — no hay
    // "repeat" de Up (Windows no los genera) y recibir_up_rt ya sabe
    // no-opear si `input` no pertenece a ninguna sesión ni instancia
    // activa (ver Etapa 2).
    recibir_up_rt(input);
}

/// Rueda del mouse: solo aplica a Runtime (ver hay_candidata_para).
/// Si ningún candidato posible espera este pulso ahora mismo, se
/// trata como evento suelto — un Down real por cada pulso, sin
/// agrupar (mismo camino que ya usa el teclado para un input sin
/// ninguna candidata, vía posibles == 0 -> pasar() dentro de
/// recibir_down_rt). Si hay candidata, se agrupa en la sesión
/// (fase CerrandoRueda, ver más abajo) — ahí SÍ hace falta distinguir
/// "primer pulso de la ráfaga" (se reenvía) de "pulso siguiente de la
/// misma ráfaga" (solo suma al conteo), a diferencia del filtro de
/// repeats de Down/Up de más arriba.
fn procesar_pulse_runtime(input: InputId) {
    if !hay_candidata_para(&input) {
        recibir_down_rt(input);
        return;
    }
    recibir_pulse_rt(input);
}

/// ¿Existe alguna sesión de Runtime que, extendida con `input`,
/// siga teniendo `posibles > 0`? Si ninguna sirve, prueba `input`
/// solo (caso "rueda como primer paso de un gesto nuevo"). Ya no es
/// pub — solo la usa procesar_pulse_runtime, dentro de este mismo
/// archivo (reemplaza a la vieja hay_candidata_para, que el
/// el archivo viejo original llamaba desde afuera).
fn hay_candidata_para(input: &InputId) -> bool {
    let runtime = RUNTIME.lock().unwrap();
    let compilado = compilado_actual();

    let alguna_sesion_sirve = runtime.sesiones.iter().any(|s| {
        let mut probable = s.entrada.clone();
        probable.push(input.clone());
        contar(&compilado, &probable).0 > 0
    });

    if alguna_sesion_sirve {
        return true;
    }

    contar(&compilado, std::slice::from_ref(input)).0 > 0
}

/// Punto de entrada para cada Pulse REAL de rueda ya decidido como
/// "hay candidata" (ver procesar_pulse_runtime). Agrupa en ráfagas:
/// solo el primer pulso de la ráfaga se reenvía a recibir_down_rt
/// (Runtime ya se entera de la rueda con ese); los pulsos siguientes
/// de la MISMA ráfaga solo suman al conteo, hasta que
/// iniciar_timer_rueda decide Mantenido o Simple al expirar.
fn recibir_pulse_rt(input: InputId) {
    let mut runtime = RUNTIME.lock().unwrap();

    let existente = runtime.sesiones.iter().position(|s| {
        s.objetivo() == Some(&input) && matches!(s.fase, FaseSesion::CerrandoRueda { .. })
    });

    let (id, es_primero) = match existente {
        Some(idx) => {
            let s = &mut runtime.sesiones[idx];
            s.generacion += 1;
            match s.fase {
                FaseSesion::CerrandoRueda { pulsos } => {
                    s.fase = FaseSesion::CerrandoRueda { pulsos: pulsos + 1 };
                }
                _ => unreachable!(),
            }
            (s.id, false)
        }
        None => {
            let id = nuevo_id_sesion(&mut runtime);
            let mut s = Sesion::nueva(id, vec![input.clone()], false);
            s.fase = FaseSesion::CerrandoRueda { pulsos: 1 };
            runtime.sesiones.push(s);
            (id, true)
        }
    };

    let generacion = runtime
        .sesiones
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.generacion)
        .unwrap_or(0);
    drop(runtime);

    if es_primero {
        // Nunca pasa por recibir_down_rt (a propósito, igual que el
        // diseño viejo): no se agrega a ninguna sesión "normal" de
        // matching, ya se creó explícitamente arriba en fase
        // CerrandoRueda — recibir_down_rt duplicaría la sesión.
        entrada::retener();
    }

    iniciar_timer_rueda(id, generacion);
}

/// Timer de cierre de una ráfaga de rueda. Si pasa tiempo_doble() sin
/// que llegue un pulso nuevo, decide: `pulsos >= sensibilidad_rueda()`
/// -> Mantenido, si no -> Simple. La rueda no tiene fase Doble/Triple
/// propia — no hay forma física de "soltarla y volver a apretarla"
/// como una tecla.
fn iniciar_timer_rueda(id: u64, generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_doble()));
        let mut runtime = RUNTIME.lock().unwrap();
        if !vigente(&runtime, id, generacion) {
            return; // llegó un pulso nuevo antes de terminar de esperar
        }
        let idx = runtime.sesiones.iter().position(|s| s.id == id).unwrap();
        let pulsos = match runtime.sesiones[idx].fase {
            FaseSesion::CerrandoRueda { pulsos } => pulsos,
            _ => 1,
        };
        let condicion = if pulsos >= config::sensibilidad_rueda() {
            CondicionTrigger::Mantenido
        } else {
            CondicionTrigger::Simple
        };
        resolver_condicion_por_id(runtime, idx, condicion);
    });
}

/// Reemplaza al soltar() de hoy: saca `input` de la sesión de
/// Runtime que lo contenga, sin pasar por el resto del pipeline de Up
/// — no avisa a entrada.rs/runtime.rs/perfil_ui.rs, solo mantiene
/// `entrada` sincronizada con la realidad física. Usado por
/// entrada.rs en el atajo DEVOLVIENDO (evento que nunca llega hasta
/// procesar_evento_runtime por el camino normal).
pub fn soltar_fisico(input: InputId) {
    let mut runtime = RUNTIME.lock().unwrap();
    runtime.presionadas.retain(|i| i != &input);
    if let Some(s) = runtime
        .sesiones
        .iter_mut()
        .find(|s| s.entrada.contains(&input))
    {
        s.entrada.retain(|i| i != &input);
    }
}

// ------------------------------------------------------
// 🎬 MOTOR CAPTURA — filtro de repeats + ruteo
// ------------------------------------------------------

/// Punto de entrada único para cada InputEvent físico en modo
/// Captura (ya filtrado de repeats acá mismo). Reemplaza a
/// AnalizadorTrigger::procesar() en modo Captura. Llamada por
/// entrada.rs mientras captura_activa().
pub fn procesar_evento_captura(evento: InputEvent) {
    match evento.state {
        InputState::Down => procesar_down_captura(evento.input),
        InputState::Up => procesar_up_captura(evento.input),
        // La rueda del mouse no se agrupa en Captura: cada pulso es
        // simplemente un Down más de la secuencia que se está
        // grabando (Captura no filtra por candidatas — ver Etapa 3,
        // "acá todo Down es válido"). Se reenvía tal cual a
        // recibir_down_captura, que ya sabe filtrar repeats con
        // `presionadas` igual que cualquier otro input.
        InputState::Pulse => procesar_down_captura(evento.input),
    }
}

fn presionada_en_captura(captura: &EstadoCaptura, input: &InputId) -> bool {
    captura.presionadas.contains(input)
}

fn procesar_down_captura(input: InputId) {
    {
        let captura = CAPTURA.lock().unwrap();
        if !captura.activa {
            return; // defensivo — entrada.rs no debería llegar hasta acá sin captura activa
        }
        if presionada_en_captura(&captura, &input) {
            return; // repeat, no se reenvía
        }
    }
    recibir_down_captura(input);
}

fn procesar_up_captura(input: InputId) {
    recibir_up_captura(input);
}
