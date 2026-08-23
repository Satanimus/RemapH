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
    AccionCache, AppCache, ComportamientoMacro, CondicionTrigger, CoordenadaCache, ExtraCache,
    RemapeoCache,
};
use crate::{config, entrada, perfil_ui, runtime};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

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
    detener_repeticiones_rueda();
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
                // [FIX] Antes la sesión nueva arrancaba SOLO con `input`
                // — sin importar qué otras teclas (Shift/Ctrl/Alt u
                // otras) ya estuvieran físicamente abajo en este
                // momento. Eso las trataba como invisibles para el
                // matching: una tecla como "Q" (compilada sola, ej.
                // "Q"=1) hacía match aunque Shift estuviera físicamente
                // presionado a la vez (bug: Shift+Q terminaba en "!" —
                // Windows aplicaba su propio Shift al "1" simulado que
                // Runtime tipeaba, porque el Shift físico real sí se
                // había dejado pasar crudo a Windows, al no matchear
                // nada él solo). Y del otro lado, "Ctrl+Q"=1 con Ctrl
                // sostenido y Q tocada repetidas veces solo funcionaba
                // la PRIMERA vez: al resolverse el match, la sesión
                // [Ctrl,Q] se elimina entera (eliminar_sesion) — nadie
                // recordaba que Ctrl seguía físicamente abajo — así que
                // la siguiente vez que tocás Q, arrancaba una sesión
                // nueva con solo [Q], sin Ctrl, y no matcheaba "Ctrl+Q"
                // (bug: "1qqqqq" en vez de "111111").
                //
                // Ahora la sesión nueva arranca sembrada con TODO lo
                // que está físicamente presionado ahora mismo
                // (`runtime.presionadas`, que ya incluye a `input` — se
                // agrega en procesar_down_runtime antes de llegar acá,
                // y siempre queda último por orden de inserción, igual
                // que el gatillo en un trigger compilado). Esto hace
                // que Shift/Ctrl/Alt sean "teclas como cualquier otra"
                // para el matching (si no hay ningún trigger compilado
                // que empiece con la combinación real sostenida,
                // posibles da 0 y se aborta/pasa crudo — sin disparar
                // por error un trigger de una sola tecla), y que soltar
                // y volver a tocar el gatillo de un combo con
                // modificador sostenido (Ctrl+Q) vuelva a matchear sin
                // necesidad de soltar el modificador.
                let id = nuevo_id_sesion(&mut runtime);
                // Clonado a una variable aparte ANTES del push: pedir
                // `runtime.sesiones` (mutable) y `runtime.presionadas`
                // (inmutable) en la misma expresión no compila —
                // `runtime` es un MutexGuard, así que el acceso a sus
                // campos pasa por Deref/DerefMut (llamadas a método,
                // no proyección directa de campo), y el borrow checker
                // no puede probar que son disjuntos a través de eso.
                let presionadas_actuales = runtime.presionadas.clone();
                runtime
                    .sesiones
                    .push(Sesion::nueva(id, presionadas_actuales, false));
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

    // [FIX] Antes esto exigía posibles == exactas para arrancar el
    // timer de Mantenido con los necesita_doble/necesita_triple
    // reales, y para el caso posibles > exactas solo lo arrancaba
    // (con necesita_doble/triple hardcodeado en false) si HABÍA UNA
    // CANDIDATA SIMPLE en el medio (`hay_simple`). Eso dejaba sin
    // ningún timer — y por lo tanto sin ninguna forma de resolver —
    // una candidata Doble/Triple/Mantenido que compartiera prefijo
    // con un trigger más largo (ej.: "1"x2=A compilado junto a
    // "1+2"x2=B: al presionar 1 solo, posibles=2 pero exactas=1 y la
    // única candidata es Doble, no Simple, así que hay_simple daba
    // false y nunca arrancaba nada; la sesión quedaba en
    // Construyendo esperando un Up que el Up-handler solo sabe
    // resolver retroactivo a Simple — nunca a Doble/Triple/
    // Mantenido — así que terminaba abortando y reenviando la tecla
    // cruda). La condición correcta es "hay al menos una candidata
    // EXACTA acá" (exactas >= 1), sin importar si además hay un
    // prefijo más largo también posible: si lo hay, cualquier Down
    // adicional que lo complete va a invalidar este timer por
    // generación de todos modos (ver el bloque de arriba, "¿alguna
    // sesión existente acepta esto como continuación"). Con solo
    // candidatas Simple en el medio, necesita_doble/triple dan
    // false/false de por sí — mismo comportamiento que antes para
    // ese caso, sin necesidad del caso especial `hay_simple`.
    if exactas >= 1 {
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

    // exactas == 0 (y posibles > 0, porque si fuera 0 ya se abortó
    // arriba): ninguna candidata en este punto exacto todavía, solo
    // prefijos más largos posibles — seguir esperando sin timer.
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
        let runtime = RUNTIME.lock().unwrap();
        if !vigente(&runtime, id, generacion) {
            return;
        }
        let idx = runtime.sesiones.iter().position(|s| s.id == id).unwrap();

        // [FIX] Antes esto siempre intentaba resolver como
        // CondicionTrigger::Mantenido al vencer, sin importar si la
        // entrada exacta en este punto tenía o no compilado un
        // trigger con esa condición. Cuando la única candidata
        // exacta era Simple (caso típico: "Q"=1 con Extra Normal/
        // Turbo/Mantenido/Simple, ambiguo con un prefijo más largo
        // como "Q+W", que impide la resolución rápida en
        // recibir_down_rt), resolver_condicion_por_id(Mantenido) no
        // encontraba match_final (ningún trigger Mantenido compilado
        // para esta entrada) y terminaba abortando todo con
        // entrada::pasar() — eso volcaba a Windows el buffer crudo
        // de Down repetidos acumulados mientras la tecla seguía
        // abajo (la "q" repetida del bug reportado), y la acción
        // remapeada ("1") nunca llegaba a dispararse, ni en modo
        // diferido ni instantáneo. Mismo bug, mismo síntoma, que ya
        // se había resuelto una vez en la versión vieja de
        // analizador_trigger.rs (timer de desambiguación
        // Simple+prefijos-largos vencía como Mantenido y nunca
        // matcheaba) — se reintrodujo en esta reescritura.
        //
        // Ahora se decide primero si existe una candidata exacta
        // Mantenido para la entrada actual (cubre triggers realmente
        // definidos como Mantenido, sin cambios ahí); si no existe,
        // se cae a Simple: la tecla sigue físicamente abajo en este
        // punto (todavía no llegó ningún Up), así que commitear a
        // Simple acá adentro es lo correcto — resolver_match() de
        // ahí en más decide solo, con requiere_up_real() más el
        // estado real de RUNTIME.presionadas, si corresponde modo
        // diferido (Normal/
        // Turbo/Mantenido: repite u ocupa hasta el Up real) o
        // instantáneo (Extra Simple: dispara una vez y listo).
        let entrada_actual = runtime.sesiones[idx].entrada.clone();
        let compilado = compilado_actual();
        let (_posibles, _exactas, candidatas) = contar(&compilado, &entrada_actual);
        let condicion = if candidatas
            .iter()
            .any(|c| c.trigger.condicion == CondicionTrigger::Mantenido)
        {
            CondicionTrigger::Mantenido
        } else {
            CondicionTrigger::Simple
        };

        resolver_condicion_por_id(runtime, idx, condicion);
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
    let (_posibles, _exactas, candidatas) = contar(&compilado, &entrada_actual);

    let match_final = candidatas
        .iter()
        .find(|c| c.trigger.condicion == condicion && c.trigger.entrada == entrada_actual)
        .cloned();

    // [FIX] Antes acá exigía `condicion == Simple || posibles ==
    // exactas` para ejecutar — pensado para no disparar de más
    // mientras todavía hubiera un prefijo más largo por completar.
    // Pero esta función SOLO se llama desde lugares que ya
    // "cerraron" la decisión (un timer que expiró sin que llegara
    // más Down, o un segundo/tercer toque físico real que confirmó
    // Doble/Triple) — si hubiera existido un prefijo más largo
    // todavía viable, un Down nuevo lo habría extendido/invalidado
    // ANTES de llegar hasta acá (ver "¿alguna sesión existente
    // acepta esto como continuación" en recibir_down_rt, que bump-ea
    // la generación). O sea que en este punto posibles > exactas ya
    // no significa "todavía ambiguo", solo significa "también había
    // un trigger más largo compilado, que el usuario terminó no
    // completando". Exigir posibles == exactas acá bloqueaba
    // injustamente Doble/Triple/Mantenido cuando compartían prefijo
    // con un trigger más largo (mismo bug reportado que el de
    // recibir_down_rt, más arriba) — se ejecuta siempre que haya
    // match_final, igual que ya hacía Simple.
    if let Some(remapeo) = match_final {
        eliminar_sesion(&mut runtime, id);
        drop(candidatas);
        drop(runtime); // ⚠️ soltar ANTES de resolver_match — ver doc de arriba
        resolver_match(remapeo, entrada_actual, id, Vec::new());
        return;
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
    // [FIX] Bug: "Q"=1 (Turbo/Mantenido, en bucle) + "Shift+Q"=2
    // compilados juntos. Al mantener Q presionado (bucle emitiendo
    // "1"), tocar Shift generaba una sesión nueva sembrada con
    // [Q, Shift] (ver nota en recibir_down_rt) que matcheaba
    // "Shift+Q"=2 como un match SEPARADO — sin que nadie se
    // enterara de que la InstanciaActiva de "Q"=1 seguía viva. El
    // bucle de "1" no se detenía y encima arrancaba el de "2" en
    // paralelo (la salida se veía como "111112222..." intercalado,
    // según el error de arriba: "shift lo convierte a ! en bucle").
    //
    // Windows nunca dispara dos triggers de teclado a la vez sobre
    // el mismo gatillo: si mantenés "1" y tocás "2", el bucle de
    // "1" se corta y sale "2" — porque el sistema operativo entrega
    // el evento a un único destinatario activo por vez. Acá el
    // "destinatario activo" es la InstanciaActiva, así que hay que
    // replicar ese corte manualmente: cualquier InstanciaActiva
    // cuya `entrada` sea un subconjunto de la del match que se va a
    // iniciar ahora (mismo gatillo, sin nada que el nuevo match no
    // tenga) se detiene ANTES de arrancar el nuevo — igual que
    // Windows corta el auto-repeat de "1" al recibir "2". El Down
    // de Shift en sí ya fue "consumido" por haber completado este
    // match nuevo, así que no hace falta devolverlo a Windows aparte.
    {
        let mut runtime = RUNTIME.lock().unwrap();
        let a_detener: Vec<String> = runtime
            .activas
            .iter()
            .filter(|a| a.entrada.iter().all(|i| entrada.contains(i)))
            .map(|a| a.id.clone())
            .collect();
        runtime.activas.retain(|a| !a_detener.contains(&a.id));
        drop(runtime);
        for id_activa in a_detener {
            runtime::ejecutar(OrdenRuntime::Detener { id: id_activa });
        }
    }

    // Determinar si el gatillo (último elemento de entrada) sigue presionado
    let gatillo = entrada.last();
    let gatillo_presionado = gatillo.map_or(false, |g| {
        let runtime = RUNTIME.lock().unwrap();
        runtime.presionadas.contains(g)
    });

    // Etapa 8B: una fila Macro nunca tiene ExtraCache propio (Extra
    // ahí es Comportamiento, no un molde de Turbo/Mantener), así que
    // requiere_up_real() de arriba nunca la detecta. Comportamiento
    // "Tecla mantenida" necesita el mismo tratamiento diferido
    // (Iniciar en el Down real, Detener en el Up real) — se suma acá
    // como segunda condición, mirando accion directo en vez de
    // extra. "Una ejecución"/"Toggle" NO son diferidas por este
    // mecanismo: su propio registro de arranque/parada vive en
    // runt_macro.rs, independiente de este camino (ver comentario
    // ahí) — el Iniciar+Detener inmediato que sigue yendo por la
    // rama de abajo es, para esos dos, un no-op en el lado de
    // Runtime (runt_macro::detener() ignora un Detener sin registro
    // propio previo).
    let macro_tecla_mantenida = matches!(
        &remapeo.accion,
        AccionCache::Macro {
            comportamiento: ComportamientoMacro::TeclaMantenida,
            ..
        }
    );

    let diferido = (remapeo
        .extra
        .as_ref()
        .is_some_and(|extra| extra.requiere_up_real())
        || macro_tecla_mantenida)
        && gatillo_presionado;

    if diferido {
        let mut runtime = RUNTIME.lock().unwrap();
        runtime.activas.push(InstanciaActiva {
            id: remapeo.id.clone(),
            entrada: entrada.clone(),
        });
        drop(runtime);
        runtime::ejecutar(OrdenRuntime::Iniciar {
            id: remapeo.id,
            accion: remapeo.accion,
            extra: remapeo.extra,
            coordenada: remapeo.coordenada,
        });

        // Para el grupo DEVOLVIENDO, necesitamos pasar las teclas que están presionadas
        // y que son parte de la entrada (para bloquear sus repeticiones). En este caso,
        // solo el gatillo (si está presionado) debería ser bloqueado, porque los modificadores
        // no deben ser bloqueados (ya que no son el gatillo y no deben afectar la ejecución).
        // Sin embargo, para que el up del gatillo cierre la instancia, solo necesitamos que
        // el gatillo esté en el grupo DEVOLVIENDO. Si además hay modificadores, no deben
        // estar en el grupo porque no queremos que su up cierre la instancia.
        let vivas = if gatillo_presionado {
            vec![gatillo.unwrap().clone()]
        } else {
            Vec::new()
        };
        resembrar_fantasma(restantes);
        entrada::consumir(&vivas);
        return;
    }

    // No diferido: ejecutar Iniciar + Detener inmediato
    runtime::ejecutar(OrdenRuntime::Iniciar {
        id: remapeo.id.clone(),
        accion: remapeo.accion,
        extra: remapeo.extra,
        coordenada: remapeo.coordenada,
    });
    runtime::ejecutar(OrdenRuntime::Detener { id: remapeo.id });

    resembrar_fantasma(restantes);
    entrada::consumir(&[]);
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
        let runtime = RUNTIME.lock().unwrap();
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
// (sin cambios respecto a la entrega anterior — ver historial)
// ======================================================

struct EstadoCaptura {
    activa: bool,
    sesion: Option<Sesion>,
    presionadas: Vec<InputId>,
    vigia_generacion: u64,
}

static CAPTURA: Mutex<EstadoCaptura> = Mutex::new(EstadoCaptura {
    activa: false,
    sesion: None,
    presionadas: Vec::new(),
    vigia_generacion: 0,
});

fn activar_captura_interna() {
    let mut captura = CAPTURA.lock().unwrap();
    captura.activa = true;
    captura.sesion = None;
    captura.presionadas.clear();
    captura.vigia_generacion += 1;
    let generacion = captura.vigia_generacion;
    drop(captura);
    iniciar_vigia_inactividad_captura(generacion);
}

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

// ======================================================
// 🚪 CANCELACIÓN FORZADA (Esc / vigía de inactividad)
// ------------------------------------------------------
// Punto único usado por las dos vías que pueden cortar una
// captura sin que haya un resultado real: la tecla Esc
// (procesar_down_captura) y el vigía de inactividad (más abajo).
// Desactiva la captura interna de inmediato (libera el bloqueo
// de entrada.rs) y recién después avisa a perfil_ui con la misma
// señal que ya existe para "se descartó" — cero plomería nueva
// del lado del frontend, el botón ya sabe resetearse ante ese
// resultado.
// ======================================================

fn cancelar_captura_forzada() {
    let mut captura = CAPTURA.lock().unwrap();
    if !captura.activa {
        return;
    }
    desactivar_captura_interna(&mut captura);
    drop(captura);
    perfil_ui::cancelar_captura();
}

pub(crate) fn recibir_down_captura(input: InputId) {
    let mut captura = CAPTURA.lock().unwrap();
    if !captura.activa {
        return;
    }

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

pub(crate) fn recibir_up_captura(input: InputId) {
    let mut captura = CAPTURA.lock().unwrap();
    if !captura.activa {
        return;
    }

    captura.presionadas.retain(|i| i != &input);

    let interrumpe_mantenido = captura.sesion.as_ref().is_some_and(|s| {
        s.objetivo() == Some(&input) && matches!(s.fase, FaseSesion::EsperandoMantenido { .. })
    });

    if interrumpe_mantenido {
        let sesion = captura.sesion.as_mut().unwrap();
        sesion.generacion += 1;
        sesion.fase = FaseSesion::EsperandoTriple { toques: 1 };
        let generacion = sesion.generacion;
        drop(captura);
        iniciar_timer_triple_captura(generacion);
        return;
    }

    if let Some(condicion) = flush_si_vacio(&mut captura) {
        drop(captura);
        perfil_ui::recibir_condicion(condicion);
    }
}

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

fn iniciar_timer_mantenido_captura(generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_mantenido()));
        let mut captura = CAPTURA.lock().unwrap();
        if !vigente_captura(&captura, generacion) {
            return;
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
            return;
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
// 🛟 VIGÍA DE INACTIVIDAD (red de seguridad)
// ------------------------------------------------------
// Mismo patrón que iniciar_timer_mantenido_captura/
// iniciar_timer_triple_captura (thread + sleep + chequeo por
// generación, para descartar vigías viejos de una captura
// anterior) pero con su propia generación (vigia_generacion),
// independiente de la de la sesión. A propósito NO se reinicia
// con la actividad (Down/Up/Pulse): son 10s absolutos desde que
// se activó la captura hasta que se fuerza su cierre si no
// terminó sola. Motivo: si una tecla se traba físicamente y deja
// de mandar Up pero sigue repitiendo Down, un vigía "de silencio"
// nunca vería silencio y jamás cortaría — 10s son de sobra para
// terminar cualquier captura real.
// ======================================================

fn vigia_captura_vigente(captura: &EstadoCaptura, generacion: u64) -> bool {
    captura.activa && captura.vigia_generacion == generacion
}

fn iniciar_vigia_inactividad_captura(generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(
            config::tiempo_inactividad_captura(),
        ));
        let captura = CAPTURA.lock().unwrap();
        if !vigia_captura_vigente(&captura, generacion) {
            return;
        }
        drop(captura);
        cancelar_captura_forzada();
    });
}

// ======================================================
// 🖱️ RUEDA EN CAPTURA
// ------------------------------------------------------
// [FIX] La rueda llega SIEMPRE como InputState::Pulse — nunca tiene
// Down/Up propios (cada muesca es un pulso suelto, no un
// "sostener"). Antes, procesar_evento_captura() la enrutaba igual
// que una tecla (procesar_down_captura/recibir_down_captura), que
// la empuja a `captura.presionadas` esperando un Up que la saque de
// ahí más adelante — ese Up nunca llega, así que la rueda quedaba
// en `presionadas` PARA SIEMPRE. Como flush_si_vacio() exige
// `presionadas` vacío para poder cerrar CUALQUIER captura (no solo
// la de la rueda), un solo giro dejaba la ventana de Captura
// trabada para siempre — y como Modo Captura consume todo
// incondicionalmente, el teclado quedaba mudo.
//
// Ahora la rueda arma su propia ráfaga, mismo criterio que
// recibir_pulse_rt en Runtime (ver FaseSesion::CerrandoRueda):
// cuenta pulsos consecutivos SIN tocar `presionadas`, y al pasar
// config::tiempo_doble() sin un pulso nuevo, resuelve Simple (menos
// de sensibilidad_rueda() pulsos) o Mantenido (esa cantidad o más)
// a través del mismo resolver_condicion_captura() que ya usa
// Doble/Triple/Mantenido de teclado — así una combinación como
// Ctrl+Rueda también funciona: Ctrl sí queda en `presionadas` (tiene
// Down/Up reales), así que la condición de la rueda queda
// "pendiente" hasta que Ctrl se suelte, igual que ya pasa hoy con
// cualquier otra combinación modificador+condición.
// ======================================================

fn recibir_pulse_captura(input: InputId) {
    let mut captura = CAPTURA.lock().unwrap();
    if !captura.activa {
        return;
    }

    let continua_rueda = captura.sesion.as_ref().is_some_and(|s| {
        s.objetivo() == Some(&input) && matches!(s.fase, FaseSesion::CerrandoRueda { .. })
    });

    if continua_rueda {
        let sesion = captura.sesion.as_mut().unwrap();
        sesion.generacion += 1;
        if let FaseSesion::CerrandoRueda { pulsos } = sesion.fase {
            sesion.fase = FaseSesion::CerrandoRueda { pulsos: pulsos + 1 };
        }
        let generacion = sesion.generacion;
        drop(captura);
        iniciar_timer_rueda_captura(generacion);
        return;
    }

    let sesion = captura
        .sesion
        .get_or_insert_with(|| Sesion::nueva(0, Vec::new(), false));
    sesion.entrada.push(input.clone());
    sesion.generacion += 1;
    sesion.fase = FaseSesion::CerrandoRueda { pulsos: 1 };
    let generacion = sesion.generacion;
    drop(captura);

    perfil_ui::recibir_down(input);
    iniciar_timer_rueda_captura(generacion);
}

fn iniciar_timer_rueda_captura(generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_doble()));
        let mut captura = CAPTURA.lock().unwrap();
        if !vigente_captura(&captura, generacion) {
            return;
        }
        let pulsos = match captura.sesion.as_ref().map(|s| s.fase) {
            Some(FaseSesion::CerrandoRueda { pulsos }) => pulsos,
            _ => 1,
        };
        let condicion = if pulsos >= config::sensibilidad_rueda() {
            CondicionTrigger::Mantenido
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
// ===== ETAPA 4 — PUNTO DE ENTRADA ÚNICO ===============
// ======================================================

pub fn captura_activa() -> bool {
    CAPTURA.lock().unwrap().activa
}

pub fn activar_captura() {
    activar_captura_interna();
}

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
    recibir_up_rt(input);
}

fn procesar_pulse_runtime(input: InputId) {
    // Repetición de Rueda se resuelve por fuera del mecanismo de
    // sesión/timer de más abajo: no espera ningún silencio, dispara
    // (encola) con cada pulso. Si esta entrada tiene una fila así,
    // el pulso físico queda descartado simplemente por no
    // reenviarlo a Windows — no hace falta entrada::retener()/
    // pasar()/consumir() para esto, mismo criterio que ya usa el
    // match inmediato de teclado en recibir_down_rt (posibles ==
    // exactas == 1 && Simple): un match sin ninguna ambigüedad no
    // necesita buffer, solo hay que no reenviar el evento crudo.
    if let Some(remapeo) = candidata_repeticion_rueda(&input) {
        encolar_repeticion_rueda(remapeo);
        // [FIX] Bug reportado: "Q+Rueda abajo"=1 Extra Repetición
        // imprimía "qq" al soltar Q. Ver doc completo en
        // silenciar_modificadores_repeticion_rueda() — en resumen,
        // este camino nunca le avisaba a entrada.rs que el
        // modificador sostenido (Q) ya había sido "usado" en un
        // match, así que su RETENIDO quedaba huérfano y se volcaba
        // crudo a Windows al soltar. Esta llamada cierra ese
        // RETENIDO en silencio, sin generar ninguna salida.
        silenciar_modificadores_repeticion_rueda();
        return;
    }

    if !hay_candidata_para(&input) {
        // [FIX] Antes esto llamaba a recibir_down_rt(input) directo,
        // salteando procesar_down_runtime() — que es el único lugar
        // que agrega `input` a `runtime.presionadas` ANTES de llamar.
        // Como recibir_down_rt() siembra toda sesión NUEVA con
        // `runtime.presionadas.clone()` (ver su comentario "[FIX]"),
        // la sesión nacía con entrada VACÍA (sin el propio pulso).
        // contar() contra un slice vacío considera "posible" a
        // CUALQUIER trigger compilado (el prefijo vacío matchea
        // todo) pero "exacta" a ninguno — cae en la rama "seguir
        // esperando sin timer", y entrada::retener() bloqueaba el
        // pulso PARA SIEMPRE (nunca hay Up real ni timer que lo
        // resuelva para un input que nunca vuelve a llegar tal
        // cual). Como RETENIDO es un buffer único global, todo
        // evento físico posterior (cualquier tecla, cualquier rueda)
        // quedaba atrapado ahí también, hasta que la red de
        // seguridad de config::tiempo_maximo_retenido() (5s) lo
        // soltara de golpe — de ahí "ninguna rueda funciona en
        // Windows" en cuanto había algún trigger compilado (con
        // cache vacía esto ni se alcanzaba a ejecutar, ver
        // cache::esta_vacia() en entrada.rs, que corta antes).
        //
        // Fix: agregar el pulso a `presionadas` antes de llamar,
        // igual que procesar_down_runtime, y sacarlo enseguida
        // después — a diferencia de una tecla real, la rueda no
        // tiene un Up físico que lo haga por nosotros, así que no
        // puede quedar marcada como "sostenida" para siempre.
        {
            let mut runtime = RUNTIME.lock().unwrap();
            if !runtime.presionadas.contains(&input) {
                runtime.presionadas.push(input.clone());
            }
        }
        recibir_down_rt(input.clone());
        {
            let mut runtime = RUNTIME.lock().unwrap();
            runtime.presionadas.retain(|i| i != &input);
        }
        return;
    }
    recibir_pulse_rt(input);
}

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
            // [FIX] Antes la sesión nueva de rueda arrancaba SIEMPRE
            // con `vec![input]` — solo la rueda, sin importar si ya
            // había una sesión en curso con un modificador sostenido
            // (ej. "Q", esperando en Construyendo porque "Q+Rueda
            // abajo" es un prefijo válido) ni qué otras teclas
            // estuvieran físicamente abajo. Eso hacía dos cosas mal
            // a la vez: 1) un trigger compilado como "Q+Rueda
            // abajo"=2 nunca podía matchear — la entrada evaluada
            // siempre era `[Rueda]` sola, nunca `[Q, Rueda]` — así
            // que si además existía "Rueda abajo"=1 sin modificador,
            // esa ganaba siempre (bug: "en ambos casos solo genera
            // 1"); y 2) si SOLO existía el trigger con modificador
            // (sin el trigger de rueda sola), la entrada `[Rueda]`
            // no matcheaba nada tampoco, y al no haber ninguna
            // candidata exacta el timer resolvía a Simple contra una
            // entrada que ya no le servía a nadie (bug: "no hace
            // match").
            //
            // Mismo criterio que recibir_down_rt (ver su comentario
            // "[FIX]" en el bloque None de más arriba): primero se
            // busca si alguna sesión EXISTENTE acepta este pulso como
            // continuación válida (mismo chequeo que hay_candidata_
            // para) y, si la hay, se la extiende en vez de crear una
            // nueva — así "Q" sigue siendo parte de la misma sesión
            // que ahora también tiene la rueda. Si no hay ninguna
            // sesión que extender, la sesión nueva se siembra con
            // TODO lo que está físicamente presionado ahora mismo
            // (`runtime.presionadas`) más el propio pulso, en vez de
            // solo el pulso — cubre el caso de un modificador
            // sostenido que él solo no alcanzó a crear una sesión
            // (no es prefijo de nada por sí solo).
            let compilado = compilado_actual();
            let extendible = runtime.sesiones.iter().position(|s| {
                let mut probable = s.entrada.clone();
                probable.push(input.clone());
                contar(&compilado, &probable).0 > 0
            });

            let id = match extendible {
                Some(idx) => {
                    let s = &mut runtime.sesiones[idx];
                    s.entrada.push(input.clone());
                    s.fantasma = false;
                    s.fase = FaseSesion::CerrandoRueda { pulsos: 1 };
                    s.generacion += 1;
                    s.id
                }
                None => {
                    let id = nuevo_id_sesion(&mut runtime);
                    let mut entrada = runtime.presionadas.clone();
                    if !entrada.contains(&input) {
                        entrada.push(input.clone());
                    }
                    let mut s = Sesion::nueva(id, entrada, false);
                    s.fase = FaseSesion::CerrandoRueda { pulsos: 1 };
                    runtime.sesiones.push(s);
                    id
                }
            };
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
        entrada::retener();
    }

    iniciar_timer_rueda(id, generacion);
}

fn iniciar_timer_rueda(id: u64, generacion: u64) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(config::tiempo_doble()));
        let runtime = RUNTIME.lock().unwrap();
        if !vigente(&runtime, id, generacion) {
            return;
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

// ======================================================
// 🔁 REPETICIÓN DE RUEDA (Extra RepeticionRueda)
// ------------------------------------------------------
// Cola-por-contador: cada pulso de rueda que matchea una fila con
// Extra RepeticionRueda suma 1 a `pendientes` de esa fila. Un único
// hilo por fila (`corriendo`) va ejecutando una repetición, resta 1,
// espera config::delay_rueda_repeticion(), y repite hasta que
// `pendientes` llegue a 0 — momento en que se apaga solo
// (`corriendo = false`). Si llegan más pulsos mientras el hilo ya
// está corriendo, no se arranca un segundo hilo: solo se suma al
// contador que el hilo existente ya está leyendo.
//
// `detener_repeticiones_rueda()` (llamada desde borrar_cache(), ver
// ahí) vacía el mapa entero — el hilo, la próxima vez que chequee
// (antes de ejecutar la siguiente repetición o al despertar del
// sleep), no encuentra su entrada y corta solo sin ejecutar nada
// más. Es un corte best-effort: si el hilo ya estaba a mitad de
// ejecutar runtime::ejecutar() cuando se vacía el mapa, esa
// repetición puntual sí termina de salir — aceptable (mismo
// criterio que la red de seguridad de tiempo_maximo_retenido en
// entrada.rs: preferir un corte simple y best-effort a una
// sincronización perfecta).
// ======================================================

struct EstadoRepeticion {
    pendientes: u32,
    corriendo: bool,
}

static REPETICION_RUEDA: LazyLock<Mutex<HashMap<String, EstadoRepeticion>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Corta en seco cualquier bucle de Repetición de Rueda en curso.
/// Llamada desde cache::borrar_cache() — cubre tanto "desactivar
/// perfil" como cualquier recompilación (cambiar/guardar/clonar/
/// renombrar/eliminar/crear/seleccionar perfil), porque TODOS esos
/// caminos en perfil.rs pasan por borrar_cache() antes de compilar
/// de nuevo (o directamente, en el caso de desactivar).
fn detener_repeticiones_rueda() {
    REPETICION_RUEDA.lock().unwrap().clear();
}

/// Busca una fila cuyo trigger sea EXACTAMENTE `presionadas + input`
/// (mismo criterio de armado de entrada que el resto del archivo:
/// modificadores físicamente sostenidos + el pulso actual) y cuyo
/// Extra sea RepeticionRueda. Ignora la Condición del trigger a
/// propósito (ver regla de negocio en sección 1 de este plan).
fn candidata_repeticion_rueda(input: &InputId) -> Option<RemapeoCache> {
    let runtime = RUNTIME.lock().unwrap();
    let mut entrada = runtime.presionadas.clone();
    drop(runtime);
    if !entrada.contains(input) {
        entrada.push(input.clone());
    }

    let compilado = compilado_actual();
    compilado
        .remapeos
        .iter()
        .find(|fila| {
            app_habilitada(&fila.trigger.app, &compilado.apps)
                && fila.trigger.entrada == entrada
                && matches!(fila.extra, Some(ExtraCache::RepeticionRueda))
        })
        .cloned()
}

/// Suma 1 al contador de pendientes de `remapeo.id` y arranca un
/// hilo consumidor si no había uno ya corriendo para esa fila.
fn encolar_repeticion_rueda(remapeo: RemapeoCache) {
    let mut mapa = REPETICION_RUEDA.lock().unwrap();
    let estado = mapa.entry(remapeo.id.clone()).or_insert(EstadoRepeticion {
        pendientes: 0,
        corriendo: false,
    });
    estado.pendientes += 1;
    let debo_arrancar = !estado.corriendo;
    if debo_arrancar {
        estado.corriendo = true;
    }
    drop(mapa);

    if debo_arrancar {
        std::thread::spawn(move || worker_repeticion_rueda(remapeo));
    }
}

/// [FIX] Bug reportado: "Q+Rueda abajo"=1 Extra Repetición imprimía
/// "qq" al soltar Q. Causa: cuando Q baja sola (antes de que llegue
/// ningún pulso de rueda), recibir_down_rt() no encuentra ningún
/// trigger con entrada EXACTA `[Q]` (solo `[Q, Rueda]`), así que cae
/// en la rama "exactas==0, seguir esperando sin timer" -> abre un
/// RETENIDO en entrada.rs y deja una Sesion `[Q]` viva en
/// Construyendo. El pulso de rueda que sí completa el trigger nunca
/// pasa por recibir_down_rt()/recibir_pulse_rt() — candidata_
/// repeticion_rueda() lo intercepta antes, en procesar_pulse_runtime
/// — así que esa Sesion y ese RETENIDO quedaban huérfanos para
/// siempre. Al soltar Q, recibir_up_rt() encontraba esa sesión, no
/// hallaba match posible para la entrada `[Q]` sola (el único
/// trigger compilado para Q es `[Q,Rueda]`, no `[Q]`) y abortaba con
/// entrada::pasar() — volcando crudo a Windows el Down (y luego el
/// Up) de Q que habían quedado retenidos. De ahí las "qq".
///
/// Fix: al confirmarse un match de Repetición de Rueda, se busca la
/// sesión cuya entrada sea EXACTAMENTE lo que sigue físicamente
/// presionado en este momento (los modificadores del combo, sin la
/// rueda) y, si existe, se la elimina y se cierra su RETENIDO con
/// entrada::consumir(vivas) — mismo patrón que resolver_match() usa
/// en el camino normal. Esos modificadores quedan marcados "vivos"
/// del lado de entrada.rs: de ahí en más sus repeats/Up reales se
/// enrutan solos a cache::soltar_fisico() (ver su comentario más
/// abajo) en vez de volver a pasar por acá, y no generan ninguna
/// salida por sí solos — pero siguen en RUNTIME.presionadas, así que
/// un próximo trigger que los necesite (otra tecla, u otro giro de
/// rueda) los sigue viendo sostenidos.
///
/// Si no hay ninguna sesión así (ya se resolvió en un pulso anterior
/// de la misma ráfaga, o el trigger no tiene modificador — solo
/// "Rueda abajo"=X suelta), no hace nada: es idempotente, se puede
/// llamar en cada pulso de la ráfaga sin costo ni efecto secundario.
fn silenciar_modificadores_repeticion_rueda() {
    let mut runtime = RUNTIME.lock().unwrap();
    let vivas = runtime.presionadas.clone();
    if vivas.is_empty() {
        return;
    }
    let Some(idx) = runtime
        .sesiones
        .iter()
        .position(|s| s.fase == FaseSesion::Construyendo && !s.fantasma && s.entrada == vivas)
    else {
        return;
    };
    let id = runtime.sesiones[idx].id;
    eliminar_sesion(&mut runtime, id);
    drop(runtime);
    entrada::consumir(&vivas);
}

/// Hilo consumidor de la cola de una fila. Ejecuta una repetición,
/// resta 1, y si todavía queda algo pendiente espera
/// config::delay_rueda_repeticion() y vuelve a empezar — hasta
/// vaciarse solo o hasta que detener_repeticiones_rueda() borre su
/// entrada del mapa (ver chequeos `let Some(estado) = ... else {
/// return; }` en cada vuelta).
fn worker_repeticion_rueda(remapeo: RemapeoCache) {
    loop {
        // ¿Sigue existiendo esta fila en el mapa? (si
        // detener_repeticiones_rueda() la borró, cortar ya, sin
        // ejecutar ni una repetición más).
        {
            let mapa = REPETICION_RUEDA.lock().unwrap();
            if !mapa.contains_key(&remapeo.id) {
                return;
            }
        }

        // Opción B (ver PLAN_RUEDA_REPETICION.md, sección de fix
        // post-implementación): se manda `extra: None` a propósito.
        // La cola/orden/timing de las repeticiones ya los maneja
        // este worker por completo — Runtime no necesita saber que
        // el Extra de esta fila es RepeticionRueda, y así cada
        // pulso se resuelve por el camino simple de "Emitir"
        // (COLA_SALIDA), sin pasar por runt_extra::obtener() ni por
        // ejecutar_extra_en_hilo() (que registraría un hilo/entrada
        // en INSTANCIAS innecesarios para algo que este worker ya
        // resuelve solo).
        runtime::ejecutar(OrdenRuntime::Iniciar {
            id: remapeo.id.clone(),
            accion: remapeo.accion.clone(),
            extra: None,
            coordenada: remapeo.coordenada.clone(),
        });
        runtime::ejecutar(OrdenRuntime::Detener {
            id: remapeo.id.clone(),
        });

        let debo_seguir = {
            let mut mapa = REPETICION_RUEDA.lock().unwrap();
            let Some(estado) = mapa.get_mut(&remapeo.id) else {
                return; // se canceló mientras se ejecutaba esta repetición
            };
            estado.pendientes = estado.pendientes.saturating_sub(1);
            if estado.pendientes == 0 {
                estado.corriendo = false;
                false
            } else {
                true
            }
        };

        if !debo_seguir {
            return;
        }

        std::thread::sleep(std::time::Duration::from_millis(
            config::delay_rueda_repeticion(),
        ));
    }
}

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

    // [FIX] Antes esta función solo actualizaba `presionadas` y el
    // historial de la sesión — nunca miraba `runtime.activas`. Pero
    // este es exactamente el único lugar al que llega el Up real de
    // una tecla cuyo match fue diferido (Normal/Turbo/Mantenido/
    // ClickSostenido): entrada.rs, al ver que la tecla ya está en un
    // grupo DEVOLVIENDO con bloquear=true (abierto por consumir()),
    // resuelve el Up ahí mismo (rama a) y llama a soltar_fisico() en
    // vez de reenviar el evento a cache::procesar_evento_runtime() —
    // por diseño, para no volver a analizar un Up que ya sabemos que
    // solo debe cerrar el grupo. Eso significa que recibir_up_rt()
    // (que sí revisa `activas` al principio) NUNCA se entera de este
    // Up. Sin este chequeo acá, la InstanciaActiva quedaba viva para
    // siempre: el bucle de Normal/Turbo nunca recibía su Detener al
    // soltar la tecla (bug reportado como "genera 1 en bucle y no se
    // detiene"), y Mantenido nunca emitía el Up de la acción
    // simulada (bug reportado como "genera 1 down pero no el up").
    // Mismo bug de fondo, mismo síntoma, que ya se había resuelto
    // una vez en la versión vieja (analizador_trigger.rs: "entrada
    // ::pasar() en vez de consumir() cuando SÍ había match, repeat
    // quedaba huérfano") — ahí el fix fue evitar abrir el grupo
    // DEVOLVIENDO; acá no es opción (sí lo necesitamos, para no
    // filtrar los repeats crudos), así que el fix correcto es que
    // soltar_fisico() cierre la instancia activa correspondiente.
    //
    // Este mismo camino (routing directo a soltar_fisico, sin pasar
    // por recibir_up_rt) es también el que cierra en silencio el Up
    // real de un modificador que quedó "vivo" tras un match de
    // Repetición de Rueda (ver silenciar_modificadores_repeticion_
    // rueda()): como no hay InstanciaActiva ni Sesion asociada a ese
    // modificador, el bloque de abajo simplemente no encuentra nada
    // que detener — soltar_fisico() no genera ninguna salida por sí
    // solo, solo actualiza presionadas.
    if let Some(pos) = runtime
        .activas
        .iter()
        .position(|a| a.entrada.contains(&input))
    {
        let instancia = runtime.activas.remove(pos);
        drop(runtime);
        runtime::ejecutar(OrdenRuntime::Detener { id: instancia.id });
        resembrar_fantasma(Vec::new());
    }
}

pub fn procesar_evento_captura(evento: InputEvent) {
    match evento.state {
        InputState::Down => procesar_down_captura(evento.input),
        InputState::Up => procesar_up_captura(evento.input),
        InputState::Pulse => recibir_pulse_captura(evento.input),
    }
}

fn presionada_en_captura(captura: &EstadoCaptura, input: &InputId) -> bool {
    captura.presionadas.contains(input)
}

fn procesar_down_captura(input: InputId) {
    // Esc es la salida manual del modo Captura: cancela en vez de
    // capturarse a sí misma. Mismo camino que el vigía de
    // inactividad (cancelar_captura_forzada) — la UI ya sabe
    // resetear el botón al valor guardado (o a como fue creado, si
    // no había ninguno) ante ese resultado.
    if input == InputId::new("keyboard", "Escape") {
        cancelar_captura_forzada();
        return;
    }

    {
        let captura = CAPTURA.lock().unwrap();
        if !captura.activa {
            return;
        }
        if presionada_en_captura(&captura, &input) {
            return;
        }
    }
    recibir_down_captura(input);
}

fn procesar_up_captura(input: InputId) {
    recibir_up_captura(input);
}
