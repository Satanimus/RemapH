// ======================================================
// 🗃️ Cache — ETAPAS 1 y 2 / 6 (ver DISENO_CACHE_V2.md)
// ======================================================
// Este archivo acumula lo entregado hasta ahora. Reemplaza
// por completo al "cache_etapa1.rs" anterior (mismo contenido
// de la Etapa 1, sin cambios, más la Etapa 2 agregada abajo).
//
// ⚠️ Sigue sin compilar el proyecto completo — analizador_trigger.rs
// todavía existe y sigue pidiendo funciones viejas de cache.rs que
// ya no están (recibir_down, recibir_up, recibir_condicion,
// hay_candidata_para). Eso se resuelve recién en la Etapa 5. Hasta
// entonces este archivo se revisa por lectura, no se pisa el
// cache.rs real del proyecto.
// ======================================================

use crate::eventos::InputId;
use crate::perfil_cache::{
    AccionCache, AppCache, CondicionTrigger, CoordenadaCache, ExtraCache, RemapeoCache,
};
use crate::{config, entrada, runtime};
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
// Reemplaza a Lista (cache.rs viejo) + Grupo (analizador_trigger.rs
// viejo) por un único concepto: Sesion. Su `entrada` es la ÚNICA
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
    /// true = sembrada especulativamente tras resolver algo (ver
    /// resembrar_fantasma). Nadie en entrada.rs tiene un RETENIDO
    /// esperándola.
    fantasma: bool,
}

impl Sesion {
    fn nueva(id: u64, entrada: Vec<InputId>, fantasma: bool) -> Self {
        Self {
            id,
            entrada,
            fase: FaseSesion::Construyendo,
            generacion: 0,
            fantasma,
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
}

static RUNTIME: Mutex<EstadoRuntime> = Mutex::new(EstadoRuntime {
    sesiones: Vec::new(),
    siguiente_id: 0,
    activas: Vec::new(),
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
                resolver_condicion_por_id(&mut runtime, idx, CondicionTrigger::Doble);
                return;
            }
            FaseSesion::EsperandoTriple { toques: 2 } => {
                resolver_condicion_por_id(&mut runtime, idx, CondicionTrigger::Triple);
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
        resolver_condicion_por_id(&mut runtime, idx, CondicionTrigger::Mantenido);
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
/// aborta (Pasar). Consume el índice — después de llamar esto, `idx`
/// puede ya no ser válido (la sesión pudo haberse eliminado).
fn resolver_condicion_por_id(runtime: &mut EstadoRuntime, idx: usize, condicion: CondicionTrigger) {
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
            eliminar_sesion(runtime, id);
            drop(candidatas);
            resolver_match(remapeo, entrada_actual, id, Vec::new());
            return;
        }
    }

    eliminar_sesion(runtime, id);
    // (drop implícito de runtime al salir de la función — la llamada
    // real toma el lock afuera; ver timers e Up-handler más abajo,
    // todos sueltan el lock ANTES de esta parte final.)
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
            resolver_condicion_por_id(&mut runtime, idx, CondicionTrigger::Simple);
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

        resolver_condicion_por_id(&mut runtime, idx, condicion);
    });
}
