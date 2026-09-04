// ======================================================
// 🧩 runt_macro
// ======================================================
// 1. ¿Qué hace este archivo?
//
// Ejecutor real de una fila tipo "macro". Lee el JSON de
// /Macros/<nombre>.json (vía macros::abrir_macro) e
// interpreta sus pasos directo, llamando a las mismas
// funciones de ejecución que ya usa el resto del motor
// (runtime.rs, back_coordenada, back_portapapeles,
// back_multimedia) — nada de bajo nivel se reimplementa acá.
//
// Reemplaza al viejo ejecutor de macro de texto plano
// (ejecutar_macro_en_hilo, eliminado de runtime.rs en esta
// misma etapa). No usa ejecutar_lineas/ejecutar_linea — esas
// siguen siendo el intérprete del Idioma Runtime de
// runt_extra (Turbo/Normal/Mantener), un camino paralelo que
// esta etapa no toca.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// runtime.rs — únicamente desde ejecutar_accion(), rama
//     AccionCache::Macro{..}, vía iniciar(). Ningún otro
//     archivo llama directo a este módulo.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// iniciar(id_fila, nombre, programa, comportamiento) — todo
// ya resuelto por compilador.rs (Etapa 8A): id de la fila,
// nombre de la macro, programa del Filtro de App de la fila
// (para el paso Multimedia "En App") y el Comportamiento ya
// convertido a enum.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Nada por retorno — actúa contra las mismas funciones de
// salida física que ya usa runtime.rs.
// ------------------------------------------------------
// 5. Decisiones de diseño
//
// A) Comportamiento — dos mecanismos, no tres (ver
//    ComportamientoMacro en perfil_cache.rs):
//    • Una ejecución / Toggle comparten mecanismo: un
//      registro propio (ACTIVAS) de fila → bandera de
//      detener. Disparar el trigger de una fila SIN
//      ejecución activa arranca una nueva; disparar el
//      trigger de una fila QUE YA tiene una activa la para.
//      Este registro es independiente de
//      runtime::INSTANCIAS/GENERACIONES — el Detener
//      inmediato que Cache manda siempre después de un
//      Iniciar (cache.rs, rama "no diferido") no pasa por
//      acá: como iniciar_una_ejecucion_o_toggle() nunca
//      registra nada en GENERACIONES, ese Detener llega a
//      runtime::detener() y no encuentra cola que traducir —
//      no-op automático, sin tocar runtime.rs.
//    • Tecla mantenida SÍ pasa por el mecanismo general:
//      cache.rs::resolver_match ya la trata como diferida
//      (Iniciar en el Down real, Detener en el Up real), así
//      que acá se usa runtime::nueva_id_ejecucion() +
//      runtime::registrar_instancia()/debe_detenerse()/
//      limpiar_instancia() — mismo patrón que
//      ejecutar_lineas(), para que el Up físico real (vía
//      runtime::detener_ejecucion()) la encuentre y la corte.
//
// B) Espera interrumpible propia (reemplaza el sondeo de
//    15ms SOLO para Macro — el de esperar_detener() en
//    runtime.rs, usado por Mantener/ClickSostenido, no se
//    toca): un único Mutex<()>+Condvar compartido por TODAS
//    las ejecuciones de Macro activas. Cada punto de espera
//    (paso "Tiempo de espera", Mantenido con Extra Ninguno,
//    bucle de repetición Normal/Turbo) despierta apenas
//    alguna ejecución cambia su bandera de detener — el hilo,
//    al despertar, vuelve a consultar SU PROPIA función
//    `debe_detenerse` (closure distinta según Comportamiento,
//    ver A) antes de seguir, filtrando así si la notificación
//    era para él.
//
// C) Bucle/Marcador: el propio valor de `marcador` guardado
//    en cada paso es la referencia estable (no un índice) —
//    se busca por valor en el Vec de pasos en cada visita al
//    Bucle, así drag&drop en el editor (que no corre durante
//    una ejecución) nunca puede desincronizar nada. Un solo
//    algoritmo (ver macro_json.rs): contador por índice de
//    paso Bucle (HashMap local a ESTA ejecución, se pierde al
//    terminar) — resta 1 y vuelve al paso marcado mientras
//    contador > 0; al llegar a 0, resetea al valor programado
//    y sigue de largo (listo para una próxima visita si está
//    anidado dentro de otro bucle).
//
// D) "Simular teclas": condicion (Simple/Doble/Triple/
//    Mantenido, ya tipada en tecla_accion.condicion) y
//    tecla_extra (""/"normal"/"turbo") son dimensiones
//    independientes. Con Extra Ninguno, la condición decide
//    el patrón de toque directo (reusa runtime::
//    emitir_un_toque/emitir_multiples_toques para Simple/
//    Doble/Triple; Mantenido arma su propio Down/espera
//    interrumpible/Up, ya que no hay Up físico real que
//    esperar). Con Extra Normal/Turbo, se ignora el matiz de
//    la condición y se repite un toque simple en bucle
//    (simula una tecla física sostenida, mismo concepto que
//    runt_extra::Normal/Turbo) hasta agotar tecla_duracion_ms
//    o hasta debe_detenerse — Normal agrega la espera inicial
//    config::tiempo_espera_normal() antes de empezar a
//    repetir (mismo valor que ya usa runt_extra), Turbo repite
//    directo. El tiempo se mide siempre contra un `deadline`
//    fijo calculado una vez al arrancar el paso (Instant::now
//    + duración total), nunca contra una cuenta regresiva
//    acumulada a mano.
//
// E) Coordenada: "Posición inicial" se resuelve UNA vez por
//    ejecución completa de macro (se captura el cursor real
//    apenas arranca ejecutar_pasos(), antes del primer paso) —
//    cualquier paso Coordenada con coord_posicion_inicial en
//    true mueve ahí, sin volver a mirar back_coordenada::
//    obtener_cursor() en ese momento (that ya sería la
//    posición post-movimiento, no la original).
// ------------------------------------------------------
// 6. Funciones del archivo
//
// iniciar(id_fila, nombre, programa, comportamiento)
//     Punto de entrada único (llamado por runtime.rs).
//     Despacha a iniciar_una_ejecucion_o_toggle() o
//     iniciar_tecla_mantenida() según Comportamiento.
// iniciar_una_ejecucion_o_toggle(id_fila, nombre, programa)
//     Registro propio ACTIVAS — arranca o para (ver A).
// iniciar_tecla_mantenida(id_fila, nombre, programa)
//     Vía runtime::nueva_id_ejecucion()/INSTANCIAS (ver A).
// ejecutar_pasos(pasos, programa, debe_detenerse)
//     Loop principal: recorre el Vec<PasoMacroJson>,
//     resolviendo Bucle/Marcador (ver C), despachando cada
//     paso a su ejecutor propio.
// ejecutar_paso_tecla_mouse / ejecutar_paso_coordenada /
// ejecutar_paso_pegar / ejecutar_paso_abrir /
// ejecutar_paso_multimedia
//     Un ejecutor por tipo de paso, todos reusando funciones
//     ya existentes en runtime.rs/back_*.rs.
// esperar_interrumpible(ms, debe_detenerse)
// notificar_todas()
//     Mecanismo de espera/notificación compartido (ver B). Desde la
//     Etapa 8C, notificar_todas() también es pub(crate): runtime::
//     detener_todo() la llama para la red de seguridad global.
// detener_todas_las_activas()
//     Etapa 8C: llamada por runtime::detener_todo() — marca detenida
//     y vacía el registro ACTIVAS (Una ejecución/Toggle).
// convertir_* (varias)
//     Equivalentes locales, paralelos a los de compilador.rs
//     (privados ahí) — traducen los campos String sueltos de
//     PasoMacroJson a los enums de perfil_cache, en tiempo de
//     EJECUCIÓN (compilador.rs solo resuelve esto para
//     RemapeoJson, nunca para el contenido de una macro, ver
//     decisión de la Etapa 7/8A).
// ======================================================

use crate::back_coordenada;
use crate::back_multimedia;
use crate::back_portapapeles;
use crate::eventos::InputId;
use crate::macro_json::PasoMacroJson;
use crate::macros;
use crate::perfil_cache::{
    AlcanceMultimedia, ComandoMultimedia, ComportamientoMacro, CondicionTrigger, CoordenadaCache,
    IniciarVentana, InstanciasAbrir, PostAccionCache, PuntoReferenciaCache, UbicacionCache,
};
use crate::perfil_json::Input;
use crate::runtime;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use tauri::AppHandle;

// ======================================================
// 🗂️ ACTIVAS — registro propio de Una ejecución/Toggle
// ------------------------------------------------------
// id de FILA -> bandera compartida con el hilo en curso.
// Ver decisión A) en el header.
// ======================================================

static ACTIVAS: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ======================================================
// 🟢 PROGRESO — Indicador de ejecución (overlay play)
// ------------------------------------------------------
// Estado en memoria de la ejecución EN CURSO, mismo patrón que
// ACTIVA/ARMADA de grabacion_macro.rs. total_pasos es la
// cantidad de pasos NO-bucle del array (calculada una sola vez
// al arrancar, ver iniciar_progreso_indicador_macro) —
// paso_actual cuenta ejecuciones reales de esos pasos, así que
// con un bucle activo puede seguir subiendo más allá de
// total_pasos (el contador no se "resetea" ni se recorta en
// cada vuelta; el indicador visual, Etapa E/F, decide cómo
// mostrar eso si ocurre). Solo una ejecución de macro puede
// estar "en curso" para el overlay a la vez — coincide con que
// solo puede haber una ventana Indicador_Macro abierta (mismo
// label, ver comandos.rs).
// ======================================================

static PASO_ACTUAL_INDICADOR: Mutex<u32> = Mutex::new(0);
static TOTAL_PASOS_INDICADOR: Mutex<u32> = Mutex::new(0);

pub fn progreso_indicador_macro() -> (u32, u32) {
    (
        *PASO_ACTUAL_INDICADOR.lock().unwrap(),
        *TOTAL_PASOS_INDICADOR.lock().unwrap(),
    )
}

pub(crate) fn iniciar_progreso_indicador_macro(total: u32) {
    *TOTAL_PASOS_INDICADOR.lock().unwrap() = total;
    *PASO_ACTUAL_INDICADOR.lock().unwrap() = 0;
}

// ======================================================
// 🟢 APPHANDLE GLOBAL — overlay del indicador (modo play)
// ------------------------------------------------------
// Mismo patrón que back_menu_express::inicializar/app_handle: el
// trigger que ejecuta una macro llega desde el hilo de entrada
// física (ver runtime.rs), no desde un comando Tauri, así que no
// hay forma de recibirlo como parámetro en ese momento. Se fija
// una única vez, apenas Tauri termina de inicializar (lib.rs).
// ======================================================

static APP: OnceLock<AppHandle> = OnceLock::new();

pub fn inicializar(app: AppHandle) {
    let _ = APP.set(app);
}

fn app_handle() -> Option<&'static AppHandle> {
    APP.get()
}

// ======================================================
// 🔔 NOTIFICADOR — espera interrumpible compartida
// ------------------------------------------------------
// Ver decisión B) en el header.
// ======================================================

static NOTIFICADOR: LazyLock<(Mutex<()>, Condvar)> =
    LazyLock::new(|| (Mutex::new(()), Condvar::new()));

// pub(crate) desde la Etapa 8C: runtime::detener_todo() la llama
// para despertar cualquier hilo de Macro dormido (cualquiera de los
// tres Comportamientos) apenas les toca revisar su propia bandera de
// detener — ver runtime.rs.
pub(crate) fn notificar_todas() {
    NOTIFICADOR.1.notify_all();
}

// ======================================================
// 🛑 DETENER TODAS LAS ACTIVAS (Etapa 8C)
// ------------------------------------------------------
// Llamada por runtime::detener_todo(). Marca la bandera de cada
// ejecución Una ejecución/Toggle en curso (el hilo la nota en su
// próxima consulta a debe_detenerse, acelerada por notificar_todas())
// y vacía el registro — mismo criterio que detener_todas_las_
// instancias() en runtime.rs para INSTANCIAS, pero sobre ACTIVAS.
// ======================================================

pub(crate) fn detener_todas_las_activas() {
    let mut activas = ACTIVAS.lock().unwrap();

    for bandera in activas.values() {
        bandera.store(true, Ordering::SeqCst);
    }

    activas.clear();
}

/// Duerme hasta `ms` (contados desde AHORA, ver decisión D), o hasta
/// que `debe_detenerse()` de esta ejecución puntual se vuelva true,
/// lo que ocurra primero. Se despierta antes del plazo si
/// notificar_todas() se llama desde cualquier ejecución de Macro
/// (propia u otra) — al despertar vuelve a chequear su propia
/// `debe_detenerse` antes de decidir si sigue esperando.
fn esperar_interrumpible(ms: u64, debe_detenerse: &dyn Fn() -> bool) {
    let objetivo = Instant::now() + Duration::from_millis(ms);
    let (mutex, condvar) = &*NOTIFICADOR;
    let mut guard = mutex.lock().unwrap();

    loop {
        if debe_detenerse() {
            return;
        }

        let ahora = Instant::now();

        if ahora >= objetivo {
            return;
        }

        let (nuevo_guard, _resultado) = condvar.wait_timeout(guard, objetivo - ahora).unwrap();
        guard = nuevo_guard;
    }
}

// ======================================================
// 🚀 INICIAR (punto de entrada único, llamado por runtime.rs)
// ======================================================

pub fn iniciar(
    id_fila: String,
    nombre: String,
    programa: Option<String>,
    comportamiento: ComportamientoMacro,
    indicador_ejecucion: bool,
) {
    match comportamiento {
        ComportamientoMacro::UnaEjecucion | ComportamientoMacro::Toggle => {
            iniciar_una_ejecucion_o_toggle(id_fila, nombre, programa, indicador_ejecucion);
        }

        ComportamientoMacro::TeclaMantenida => {
            iniciar_tecla_mantenida(id_fila, nombre, programa, indicador_ejecucion);
        }
    }
}

// ======================================================
// 🔁 UNA EJECUCIÓN / TOGGLE
// ======================================================

fn iniciar_una_ejecucion_o_toggle(
    id_fila: String,
    nombre: String,
    programa: Option<String>,
    indicador_ejecucion: bool,
) {
    let mut activas = ACTIVAS.lock().unwrap();

    if let Some(bandera) = activas.remove(&id_fila) {
        // Ya había una ejecución en curso para esta fila: pararla es
        // la acción de este trigger (para Una ejecución y para
        // Toggle es literalmente el mismo código, ver decisión A).
        drop(activas);
        bandera.store(true, Ordering::SeqCst);
        notificar_todas();
        return;
    }

    let bandera = Arc::new(AtomicBool::new(false));
    activas.insert(id_fila.clone(), bandera.clone());
    drop(activas);

    thread::spawn(move || {
        let bandera_hilo = bandera.clone();
        let debe_detenerse = move || bandera_hilo.load(Ordering::SeqCst);

        ejecutar_macro_completa(&nombre, &programa, &debe_detenerse, indicador_ejecucion);

        // Limpieza al terminar naturalmente — solo si nadie volvió a
        // arrancar/reemplazar esta fila mientras corría (Arc::ptr_eq
        // evita pisar una ejecución nueva más reciente).
        let mut activas = ACTIVAS.lock().unwrap();

        if let Some(actual) = activas.get(&id_fila) {
            if Arc::ptr_eq(actual, &bandera) {
                activas.remove(&id_fila);
            }
        }
    });
}

// ======================================================
// ⌨️ TECLA MANTENIDA
// ------------------------------------------------------
// cache.rs ya la trató como diferida (Iniciar en el Down real) — acá
// solo falta anotarla en el mecanismo general para que el Up físico
// real (runtime::detener_ejecucion, vía runtime::detener) la corte.
// ======================================================

fn iniciar_tecla_mantenida(
    id_fila: String,
    nombre: String,
    programa: Option<String>,
    indicador_ejecucion: bool,
) {
    let id_ejecucion = runtime::nueva_id_ejecucion(&id_fila);

    thread::spawn(move || {
        runtime::registrar_instancia(&id_ejecucion);

        // Clon propio para el closure (move, no una referencia
        // compartida) — evita cualquier ambigüedad de lifetime entre
        // este closure y el limpiar_instancia(id_ejecucion) de más
        // abajo, que necesita moverlo.
        let id_para_chequeo = id_ejecucion.clone();
        let debe_detenerse = move || runtime::debe_detenerse(&id_para_chequeo);

        ejecutar_macro_completa(&nombre, &programa, &debe_detenerse, indicador_ejecucion);

        runtime::limpiar_instancia(id_ejecucion);
    });
}

// ======================================================
// 📖 EJECUTAR MACRO COMPLETA
// ------------------------------------------------------
// Lee el archivo (macros::abrir_macro) y corre sus pasos. Si el
// archivo ya no existe (macro renombrada/eliminada mientras estaba
// referenciada — ver decisión ya documentada fuera de este archivo:
// mismo tratamiento que "Abrir Archivo" con ruta inválida, la fila
// queda advertida pero no bloqueada), simplemente no hay nada que
// ejecutar.
// ======================================================

fn ejecutar_macro_completa(
    nombre: &str,
    programa: &Option<String>,
    debe_detenerse: &dyn Fn() -> bool,
    indicador_ejecucion: bool,
) {
    let Ok(macro_archivo) = macros::abrir_macro(nombre.to_string()) else {
        return;
    };

    if debe_detenerse() {
        return;
    }

    // Ver decisión E) — posición capturada UNA vez, antes del primer paso.
    let posicion_inicial = back_coordenada::obtener_cursor();

    // Etapa F: teclas retenidas por un paso "Solo Down" que todavía
    // no recibieron su "Solo Up" — se libera cualquier resto al
    // terminar la macro (red de seguridad si se detiene a mitad de
    // un Down sin Up, ver Regla 16 para la validación que evita esto
    // en el editor).
    let mut retenidos: Vec<InputId> = Vec::new();

    // Overlay del indicador de ejecución (modo play): solo si
    // indicadorEjecucion está activo (Regla 11); sin AppHandle
    // disponible se omite igual y la macro corre normalmente (E5).
    let overlay_abierto = indicador_ejecucion && abrir_overlay_indicador_play(&macro_archivo.pasos);

    ejecutar_pasos(
        &macro_archivo.pasos,
        programa,
        posicion_inicial,
        debe_detenerse,
        &mut retenidos,
    );

    for input in retenidos.into_iter().rev() {
        runtime::emitir_up_input(input);
    }

    if overlay_abierto {
        cerrar_overlay_indicador_play();
    }
}

// ======================================================
// 🟢 OVERLAY INDICADOR (modo play) — abrir/cerrar
// ------------------------------------------------------
// abrir_overlay_indicador_play calcula total_pasos (pasos NO-bucle
// del array), arranca el contador en 0 (iniciar_progreso_indicador_
// macro, Etapa D) y abre la ventana Indicador_Macro en modo play
// vía comandos::abrir_ventana_indicador_macro_interno (misma función
// interna que ya usa el modo grabación, Etapa B) — corriendo en el
// hilo principal de la UI (AppHandle::run_on_main_thread), porque
// WebviewWindowBuilder::build() lo exige en Windows y esta función
// se llama desde el hilo propio de la macro (thread::spawn en
// iniciar_una_ejecucion_o_toggle/iniciar_tecla_mantenida), nunca
// desde el hilo principal directamente.
// ======================================================

fn abrir_overlay_indicador_play(pasos: &[PasoMacroJson]) -> bool {
    let Some(app) = app_handle() else {
        return false;
    };

    let total_pasos = pasos.iter().filter(|paso| paso.tipo != "bucle").count() as u32;

    iniciar_progreso_indicador_macro(total_pasos);

    let app = app.clone();

    // run_on_main_thread encola el closure y devuelve de inmediato —
    // no bloquea este hilo esperando a que la ventana termine de
    // crearse. Igual que back_menu_express::crear_ventana, se acepta
    // esa carrera: la ventana puede tardar unos ms en aparecer tras
    // el primer paso, no es crítico para un indicador visual.
    let resultado = app.clone().run_on_main_thread(move || {
        let url = "indicador_macro.html?modo=play".to_string();

        if let Err(error) = crate::comandos::abrir_ventana_indicador_macro_interno(&app, url) {
            eprintln!("⚠️ No se pudo abrir el indicador de ejecución: {}", error);
        }
    });

    resultado.is_ok()
}

fn cerrar_overlay_indicador_play() {
    let Some(app) = app_handle() else {
        return;
    };

    let app = app.clone();

    let _ = app.clone().run_on_main_thread(move || {
        crate::comandos::cerrar_ventana_indicador_macro(app);
    });
}

// ======================================================
// 🔂 EJECUTAR PASOS (loop principal + Bucle/Marcador)
// ======================================================

fn ejecutar_pasos(
    pasos: &[PasoMacroJson],
    programa: &Option<String>,
    posicion_inicial: (i32, i32),
    debe_detenerse: &dyn Fn() -> bool,
    retenidos: &mut Vec<InputId>,
) {
    let mut contadores: HashMap<usize, u32> = HashMap::new();
    let mut i = 0;

    while i < pasos.len() {
        if debe_detenerse() {
            return;
        }

        let paso = &pasos[i];

        if paso.tipo == "bucle" {
            let Some(destino_letra) = &paso.bucle_marcador_destino else {
                i += 1;
                continue;
            };

            let Some(destino_idx) = pasos
                .iter()
                .position(|p| p.marcador.as_deref() == Some(destino_letra.as_str()))
            else {
                i += 1;
                continue;
            };

            let contador = contadores.entry(i).or_insert(paso.bucle_veces);

            if *contador > 0 {
                *contador -= 1;
                i = destino_idx;
                continue;
            }

            *contador = paso.bucle_veces;
            i += 1;
            continue;
        }

        ejecutar_paso(paso, programa, posicion_inicial, debe_detenerse, retenidos);

        *PASO_ACTUAL_INDICADOR.lock().unwrap() += 1;

        i += 1;
    }
}

fn ejecutar_paso(
    paso: &PasoMacroJson,
    programa: &Option<String>,
    posicion_inicial: (i32, i32),
    debe_detenerse: &dyn Fn() -> bool,
    retenidos: &mut Vec<InputId>,
) {
    match paso.tipo.as_str() {
        "tecla_mouse" => ejecutar_paso_tecla_mouse(paso, debe_detenerse, retenidos),

        "espera" => esperar_interrumpible(paso.espera_ms, debe_detenerse),

        "coordenada" => ejecutar_paso_coordenada(paso, posicion_inicial, debe_detenerse),

        "pegar" => ejecutar_paso_pegar(paso),

        "abrir" => ejecutar_paso_abrir(paso),

        "multimedia" => ejecutar_paso_multimedia(paso, programa),

        // "bucle" se maneja en ejecutar_pasos() antes de llegar acá.
        _ => {}
    }
}

// ======================================================
// ⌨️ PASO: SIMULAR TECLAS
// ------------------------------------------------------
// Ver decisión D) en el header.
// ======================================================

fn ejecutar_paso_tecla_mouse(
    paso: &PasoMacroJson,
    debe_detenerse: &dyn Fn() -> bool,
    retenidos: &mut Vec<InputId>,
) {
    let mods = convertir_inputs(&paso.tecla_accion.modificadores);

    let Some(gatillo_json) = &paso.tecla_accion.gatillo else {
        return;
    };

    let gatillo = convertir_input(gatillo_json);

    // Etapa F: arrastre diferido. "down" retiene mods+gatillo abajo
    // (mismo orden que el tramo Down de Mantenido) hasta que un paso
    // "up" posterior los libere (mismo orden que el tramo Up de
    // Mantenido: gatillo primero, mods en reversa) — sin pasar por
    // tecla_extra/condicion normales.
    match paso.tecla_retencion.as_deref() {
        Some("down") => {
            runtime::emitir_mods_abajo(&mods);
            retenidos.extend(mods.iter().cloned());

            runtime::emitir_down_input(gatillo.clone());
            retenidos.push(gatillo);

            return;
        }

        Some("up") => {
            runtime::emitir_up_input(gatillo.clone());
            retenidos.retain(|input| input != &gatillo);

            for modificador in mods.iter().rev() {
                runtime::emitir_up_input(modificador.clone());
                retenidos.retain(|input| input != modificador);
            }

            return;
        }

        _ => {}
    }

    match paso.tecla_extra.as_str() {
        "normal" | "turbo" => {
            let duracion = paso.tecla_duracion_ms.unwrap_or(0);
            let es_normal = paso.tecla_extra == "normal";

            ejecutar_repeticion(&mods, &gatillo, es_normal, duracion, debe_detenerse);
        }

        _ => match paso.tecla_accion.condicion {
            CondicionTrigger::Simple => runtime::emitir_un_toque(&mods, &gatillo),

            CondicionTrigger::Doble => runtime::emitir_multiples_toques(&mods, &gatillo, 2),

            CondicionTrigger::Triple => runtime::emitir_multiples_toques(&mods, &gatillo, 3),

            CondicionTrigger::Mantenido => {
                runtime::emitir_mods_abajo(&mods);

                runtime::emitir_down_input(gatillo.clone());

                esperar_interrumpible(paso.tecla_duracion_ms.unwrap_or(0), debe_detenerse);

                runtime::emitir_up_input(gatillo.clone());

                for modificador in mods.iter().rev() {
                    runtime::emitir_up_input(modificador.clone());
                }
            }
        },
    }
}

/// Extra Normal/Turbo dentro de una Macro: no hay Up físico real que
/// marque el final (a diferencia del Extra homónimo de un remapeo
/// normal), así que `duracion_ms` es el presupuesto TOTAL de tiempo
/// del bucle — se repite un toque simple hasta agotarlo o hasta
/// debe_detenerse. Normal agrega la espera inicial de
/// config::tiempo_espera_normal() antes de la primera repetición
/// (mismo valor que runt_extra::obtener(ExtraCache::Normal)), Turbo
/// repite directo. Deadline fijo (ver decisión D) — nunca cuenta
/// regresiva acumulada a mano.
fn ejecutar_repeticion(
    mods: &[InputId],
    gatillo: &InputId,
    es_normal: bool,
    duracion_ms: u64,
    debe_detenerse: &dyn Fn() -> bool,
) {
    let deadline = Instant::now() + Duration::from_millis(duracion_ms);

    if debe_detenerse() || Instant::now() >= deadline {
        return;
    }

    runtime::emitir_un_toque(mods, gatillo);

    if es_normal {
        if debe_detenerse() {
            return;
        }

        let restante = deadline.saturating_duration_since(Instant::now());
        let espera = restante.min(Duration::from_millis(crate::config::tiempo_espera_normal()));

        esperar_interrumpible(espera.as_millis() as u64, debe_detenerse);
    }

    loop {
        if debe_detenerse() || Instant::now() >= deadline {
            return;
        }

        runtime::emitir_un_toque(mods, gatillo);

        if debe_detenerse() || Instant::now() >= deadline {
            return;
        }

        let restante = deadline.saturating_duration_since(Instant::now());
        let espera = restante.min(Duration::from_millis(crate::config::tiempo_repeticion()));

        esperar_interrumpible(espera.as_millis() as u64, debe_detenerse);
    }
}

// ======================================================
// 🖱️ PASO: COORDENADA
// ------------------------------------------------------
// Solo mueve el mouse, sin click (ver macro_json.rs). Sin
// post_accion propio — "Posición inicial" es la única opción, ya
// resuelta antes de entrar acá (ver decisión E).
// ======================================================

fn ejecutar_paso_coordenada(
    paso: &PasoMacroJson,
    posicion_inicial: (i32, i32),
    debe_detenerse: &dyn Fn() -> bool,
) {
    if paso.coord_posicion_inicial {
        back_coordenada::mover_cursor(posicion_inicial.0, posicion_inicial.1, debe_detenerse);
        return;
    }

    let Some(ubicacion) = convertir_ubicacion_paso(paso) else {
        return;
    };

    let coordenada = CoordenadaCache {
        ubicacion,
        // Sin uso acá (ejecutar_paso_coordenada no lo consulta) —
        // valor cualquiera para poder reusar el tipo CoordenadaCache
        // tal cual, sin duplicarlo.
        post_accion: PostAccionCache::Final,
    };

    let destino = back_coordenada::calcular_destino(&coordenada.ubicacion);

    back_coordenada::mover_cursor(destino.0, destino.1, debe_detenerse);
}

fn convertir_ubicacion_paso(paso: &PasoMacroJson) -> Option<UbicacionCache> {
    let x = paso.coord_x?;
    let y = paso.coord_y?;

    Some(match paso.coord_ubicacion.as_str() {
        "relativa_cursor" => UbicacionCache::RelativaCursor {
            offset_x: x,
            offset_y: y,
        },

        "relativa_ventana" => match paso.coord_modo_ventana.as_str() {
            "porcentaje" => UbicacionCache::RelativaVentanaPorcentaje { h: x, v: y },

            _ => UbicacionCache::RelativaVentanaPixeles {
                offset_x: x,
                offset_y: y,
                referencia: convertir_punto_referencia(&paso.coord_punto_referencia),
            },
        },

        _ => UbicacionCache::Absoluta { x, y },
    })
}

fn convertir_punto_referencia(valor: &str) -> PuntoReferenciaCache {
    match valor {
        "sup_der" => PuntoReferenciaCache::SupDer,
        "centro" => PuntoReferenciaCache::Centro,
        "inf_izq" => PuntoReferenciaCache::InfIzq,
        "inf_der" => PuntoReferenciaCache::InfDer,
        _ => PuntoReferenciaCache::SupIzq,
    }
}

// ======================================================
// 📋 PASO: PEGAR RUTA/TEXTO
// ======================================================

fn ejecutar_paso_pegar(paso: &PasoMacroJson) {
    let Some(ruta) = &paso.pegar_ruta else {
        return;
    };

    // true: bloquea hasta que el Ctrl+V realmente se emitió — sin
    // esto, dos pasos "Pegar" seguidos en la misma Macro podían
    // sobreescribir el portapapeles antes de que el primer Ctrl+V
    // encolado llegara a procesarse (ver comentario largo en
    // runtime::emitir_ctrl_v_bloqueante).
    let _ = back_portapapeles::pegar(ruta, true);
}

// ======================================================
// 📂 PASO: ABRIR ARCHIVO/PROGRAMA
// ------------------------------------------------------
// Mismos 5 campos que AccionCache::AbrirArchivo, reusando
// runtime::abrir_archivo() sin reimplementar nada.
// ======================================================

fn ejecutar_paso_abrir(paso: &PasoMacroJson) {
    let Some(ruta) = &paso.abrir_ruta else {
        return;
    };

    runtime::abrir_archivo(
        ruta.clone(),
        convertir_iniciar_ventana(&paso.abrir_iniciar),
        convertir_instancias_abrir(&paso.abrir_instancias),
        paso.abrir_con.clone(),
        paso.abrir_argumento.clone(),
    );
}

fn convertir_iniciar_ventana(valor: &str) -> IniciarVentana {
    match valor {
        "minimizado" => IniciarVentana::Minimizado,
        "maximizado" => IniciarVentana::Maximizado,
        _ => IniciarVentana::Ventana,
    }
}

fn convertir_instancias_abrir(valor: &str) -> InstanciasAbrir {
    match valor {
        "unica" => InstanciasAbrir::Unica,
        _ => InstanciasAbrir::Multiple,
    }
}

// ======================================================
// 🎚️ PASO: MULTIMEDIA
// ------------------------------------------------------
// "En App" reusa el programa del Filtro de App de la FILA MACRO
// contenedora (ya resuelto por compilador.rs::convertir_macro, ver
// perfil_cache.rs::AccionCache::Macro.programa) — un paso de Macro
// nunca tiene Filtro de App propio.
// ======================================================

fn ejecutar_paso_multimedia(paso: &PasoMacroJson, programa: &Option<String>) {
    let Some(comando_str) = &paso.multimedia_comando else {
        return;
    };

    let Some(comando) = convertir_comando_multimedia(comando_str) else {
        return;
    };

    let alcance = if paso.multimedia_alcance == "en_app" && comando.es_de_volumen() {
        match programa {
            Some(programa) => AlcanceMultimedia::EnApp {
                programa: programa.clone(),
            },
            None => AlcanceMultimedia::Global,
        }
    } else {
        AlcanceMultimedia::Global
    };

    back_multimedia::ejecutar(&comando, &alcance);
}

fn convertir_comando_multimedia(valor: &str) -> Option<ComandoMultimedia> {
    match valor {
        "volumen_subir" => Some(ComandoMultimedia::VolumenSubir),
        "volumen_bajar" => Some(ComandoMultimedia::VolumenBajar),
        "silenciar" => Some(ComandoMultimedia::Silenciar),
        "play_pausa" => Some(ComandoMultimedia::PlayPausa),
        "detener" => Some(ComandoMultimedia::Detener),
        "siguiente" => Some(ComandoMultimedia::Siguiente),
        "anterior" => Some(ComandoMultimedia::Anterior),
        _ => None,
    }
}

// ======================================================
// 🔎 CONVERTIR INPUT / ENTRADA
// ------------------------------------------------------
// Equivalentes locales de compilador.rs::convertir_input (privada
// ahí) — mismo criterio, columna "interno" de pulsadores.tsv vía
// perfil_json::Input.
// ======================================================

fn convertir_input(input: &Input) -> InputId {
    InputId::new(&input.fuente, &input.control)
}

fn convertir_inputs(inputs: &[Input]) -> Vec<InputId> {
    inputs.iter().map(convertir_input).collect()
}
