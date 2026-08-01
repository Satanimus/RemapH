// ======================================================
// ⚙️ Runtime RemapH V3
// ======================================================
// 1. ¿Qué hace este archivo?
//
// Motor de ejecución. Recibe una orden ya resuelta desde
// Cache (qué id, qué acción, qué extra) y la convierte en
// pasos físicos reales: teclas/botones emitidos, archivos
// abiertos, elementos de UI mostrados.
//
// No decide SI algo debe ejecutarse ni CUÁNDO — eso ya lo
// resolvió Cache. Runtime solo sabe CÓMO ejecutarlo.
//
// Runtime NO conoce: UI, perfil.json, Captura, Cache más
// allá de recibir sus órdenes, ni cómo se arma un trigger.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// Cache — únicamente a través de ejecutar(orden), con un
//     OrdenRuntime::Iniciar{id, accion, extra} o
//     OrdenRuntime::Detener{id}.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// ejecutar(orden: OrdenRuntime) — punto de entrada único.
// La acción (AccionCache) y el extra (Option<ExtraCache>)
// ya vienen completos, resueltos por Compilador — Runtime
// no interpreta perfiles ni conoce pulsadores.tsv más allá
// de resolver_input() (traducción inversa: nombre interno
// → InputId, para las líneas DOWN/UP/pulse).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Nada por retorno — actúa directo contra
// back_interception::emitir_evento() para cada paso físico.
// ------------------------------------------------------
// 5. Decisiones de diseño (no perder esto de vista)
//
// A) Idioma Runtime, en español, único vocabulario válido:
//    ESPERAR <ms> · DOWN <control> · UP <control> ·
//    REPETIR · DETENER · <control> (a secas = pulse).
//    Los <control> son el nombre "interno" de
//    pulsadores.tsv, sin prefijo de dispositivo — Runtime
//    nunca deduce el dispositivo, pulsadores.tsv ya lo
//    sabe.
//
// B) AccionCache — las 4 variantes son tupla, no struct
//    (Emitir(InputId), Macro(String), AbrirArchivo(String),
//    Ui(String)). Si algún día se reescribe perfil_cache.rs,
//    no volver a variantes struct sin revisar compilador.rs
//    y este archivo — ya se rompió una vez por esto.
//
// C) Extra (Turbo/Mantener/Toggle/etc.) SIEMPRE pasa por
//    runt_extra::obtener(&extra), que devuelve un molde en
//    Idioma Runtime con placeholders [ACCION] /
//    [ACCION_DOWN] / [ACCION_UP]. Runtime sustituye esos
//    placeholders por el control real de la Acción de esa
//    fila (sustituir_accion()) y lo corre como una macro
//    más. La sustitución SOLO tiene efecto si la Acción es
//    Emitir — para Macro/AbrirArchivo/Ui no hay down/up
//    físico que poner ahí, los placeholders quedan sin
//    reemplazar (línea inofensiva, no rompe nada, pero
//    tampoco hace lo que probablemente se esperaba: esa
//    combinación no está pensada para usarse — la UI es
//    quien debe evitar ofrecerla).
//
// D) Un trigger Simple no puede tener Turbo — Turbo exige
//    mantener presionado. El equivalente para un Simple es
//    Toggle (switch: un trigger lo prende, el mismo trigger
//    repetido lo apaga). Runtime NO valida estas
//    combinaciones — cualquier restricción de qué Extra
//    tiene sentido con qué Condición se resuelve en la UI,
//    filtrando qué opciones se ofrecen ahí.
//
// E) Hilos: cualquier ejecución que pueda quedar esperando
//    o repitiéndose (todo lo que tiene Extra, y también
//    Macro de archivo — puede traer su propio ESPERAR/
//    REPETIR largo) corre en su PROPIO hilo, uno por id,
//    arrancado con thread::spawn. Nunca en el hilo que
//    llamó a ejecutar() — ese es el mismo hilo que escucha
//    el teclado/mouse (viene encadenado desde entrada.rs);
//    si algo bloqueara ahí, se congelaría la escucha entera
//    mientras dura la espera/repetición. Una Acción simple
//    sin Extra (Emitir/AbrirArchivo/Ui solos) no necesita
//    nada de esto — se ejecuta directo, sin hilo.
//
// F) Estado compartido entre hilos: INSTANCIAS
//    (Mutex<HashMap<id, bool>>, true = "debe detenerse").
//    Se consulta SIEMPRE antes de cada REPETIR, nunca se
//    entra a una vuelta nueva sin chequear primero. Si el
//    id no está en el mapa (no existe o ya se limpió), se
//    asume que debe detenerse — nunca al revés, por
//    seguridad. detener(id) no mata el hilo a la fuerza,
//    solo marca la bandera; el propio hilo se entera y
//    corta solo la próxima vez que llega a un REPETIR (o
//    termina naturalmente si el script no tiene ninguno).
//    "DETENER" como línea dentro de un script usa el mismo
//    mecanismo (llama a la misma detener()) — no hay dos
//    caminos distintos para lo mismo.
//
// G) Backend de salida: emitir_evento() usa
//    back_interception exclusivamente. Ya no existe el
//    modo dual con back_windows (descartado para la 1.0,
//    ver decisiones de backend/).
// ------------------------------------------------------
// 6. Funciones del archivo
//
// ejecutar(orden)
//     Punto de entrada único. Iniciar → ejecutar_accion().
//     Detener → detener().
// ejecutar_accion(id, accion, extra)
//     Si hay extra, va por el camino con hilo
//     (ejecutar_extra_en_hilo). Si no, ejecuta la Acción
//     directo (Emitir/AbrirArchivo/Ui) o en hilo si es
//     Macro de archivo.
// ejecutar_extra_en_hilo(id, extra, accion)
//     Pide el molde a runt_extra, sustituye los
//     placeholders, corre el resultado como macro en un
//     hilo nuevo.
// sustituir_accion(lineas, accion)
//     Reemplaza [ACCION]/[ACCION_DOWN]/[ACCION_UP] por el
//     control real, solo si accion es Emitir.
// ejecutar_macro_en_hilo(id, ruta)
//     Lee un archivo de macro de usuario, lo corre en un
//     hilo nuevo (mismo intérprete que un Extra).
// ejecutar_lineas(id, lineas)
//     El loop real: recorre las líneas, vuelve al inicio
//     en cada REPETIR salvo que debe_detenerse() diga que
//     no siga. Registra y limpia la instancia en
//     INSTANCIAS.
// ejecutar_linea(id, linea)
//     Interpreta una línea suelta (ESPERAR/DOWN/UP/
//     DETENER/pulse). REPETIR NO se maneja acá — lo
//     intercepta ejecutar_lineas() antes de llegar aquí,
//     porque necesita controlar el loop completo, no solo
//     un paso.
// detener(id) / debe_detenerse(id) / limpiar_instancia(id)
//     Manejo de la bandera compartida en INSTANCIAS.
// resolver_input(interno) / ejecutar_down() /
// ejecutar_up() / ejecutar_pulse() / emitir() / esperar()
//     Los pasos físicos individuales del Idioma Runtime.
// abrir_archivo() / mostrar_ui()
//     Sin implementar todavía (fuera de esta sesión de
//     trabajo) — quedan como punto de entrada ya conectado.
// emitir_evento()
//     Único punto de salida real hacia back_interception.
// ------------------------------------------------------
// Idioma Runtime:
//
// Una línea = un paso. Vocabulario fijo, en español:
//
// ESPERAR 50
// DOWN A
// UP A
// LeftButton          (sin DOWN/UP = pulse)
// REPETIR
// DETENER
//
// Los identificadores son el nombre "interno" de
// pulsadores.tsv — Runtime nunca deduce el dispositivo.
// ------------------------------------------------------
// Transformación:
//
// OrdenRuntime (Cache)
//     ↓
// ejecutar_accion()
//     ├─ sin Extra, sin Macro → ejecución directa
//     └─ con Extra o Macro de archivo → hilo propio (id)
//               ↓
//         ejecutar_lineas() [loop, revisa INSTANCIAS
//         antes de cada REPETIR]
//               ↓
//         ejecutar_linea() por cada paso
//               ↓
//         back_interception::emitir_evento()
//               ↓
//         Dispositivo físico
// ======================================================
use crate::back_interception;

use crate::cache::OrdenRuntime;

use crate::eventos::InputId;

use crate::perfil_cache::{AccionCache, ExtraCache};

use crate::runt_extra;

use std::collections::HashMap;

use std::fs::File;

use std::io::{BufRead, BufReader};

use std::sync::Mutex;

use std::thread;

use std::time::Duration;

// ======================================================
// 🗂️ INSTANCIAS ACTIVAS
// ------------------------------------------------------
// Un id -> true si le llegó orden de detener. Cada hilo
// de ejecución consulta esto antes de cada REPETIR.
// ======================================================

static INSTANCIAS: Mutex<HashMap<String, bool>> = Mutex::new(HashMap::new());

// ======================================================
// 🚀 EJECUTAR ORDEN
// ======================================================

pub fn ejecutar(orden: OrdenRuntime) {
    match orden {
        OrdenRuntime::Iniciar { id, accion, extra } => {
            ejecutar_accion(id, accion, extra);
        }

        OrdenRuntime::Detener { id } => {
            detener(id);
        }
    }
}

// ======================================================
// ⚡ EJECUTAR ACCIÓN
// ======================================================

fn ejecutar_accion(id: String, accion: AccionCache, extra: Option<ExtraCache>) {
    if let Some(extra) = extra {
        ejecutar_extra_en_hilo(id, extra, accion);

        return;
    }

    match accion {
        AccionCache::Emitir(input) => {
            emitir(input);
        }

        AccionCache::Macro(ruta) => {
            ejecutar_macro_en_hilo(id, ruta);
        }

        AccionCache::AbrirArchivo(ruta) => {
            abrir_archivo(ruta);
        }

        AccionCache::Ui(valor) => {
            mostrar_ui(valor);
        }
    }
}

// ======================================================
// 📂 ABRIR ARCHIVO
// ======================================================

fn abrir_archivo(ruta: String) {
    // El backend decide cómo abrir la ruta
    // según corresponda (programa, documento,
    // carpeta, URL, etc.).

    let _ = ruta;
}

// ======================================================
// 🪟 MOSTRAR UI
// ======================================================

fn mostrar_ui(valor: String) {
    // Enviar al Back_UI.
    // El backend interpreta la orden y
    // construye el elemento visual solicitado.

    let _ = valor;
}

// ======================================================
// 📤 EMITIR
// ======================================================

fn emitir(input: InputId) {
    let evento = crate::eventos::InputEvent::pulse(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⏱️ ESPERAR
// ======================================================

fn esperar(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

// ======================================================
// 🧩 EJECUTAR EXTRA (en su propio hilo)
// ------------------------------------------------------
// Pide el molde a runt_extra, sustituye [ACCION] /
// [ACCION_DOWN] / [ACCION_UP] por la acción real de esta
// fila, y lo corre como cualquier macro.
// ======================================================

fn ejecutar_extra_en_hilo(id: String, extra: ExtraCache, accion: AccionCache) {
    thread::spawn(move || {
        let lineas = runt_extra::obtener(&extra);

        let lineas = sustituir_accion(lineas, &accion);

        ejecutar_lineas(id, lineas);
    });
}

// ======================================================
// 🔤 SUSTITUIR [ACCION] / [ACCION_DOWN] / [ACCION_UP]
// ------------------------------------------------------
// Solo tiene sentido cuando la acción es Emitir (una
// tecla/botón física). Los demás tipos de acción no
// tienen un down/up que sustituir — la línea queda como
// está (no hace nada al ejecutarse, no rompe nada).
// ======================================================

fn sustituir_accion(lineas: Vec<String>, accion: &AccionCache) -> Vec<String> {
    let AccionCache::Emitir(input) = accion else {
        return lineas;
    };

    let Some(control) = input.control() else {
        return lineas;
    };

    lineas
        .into_iter()
        .map(|linea| {
            linea
                .replace("[ACCION_DOWN]", &format!("DOWN {control}"))
                .replace("[ACCION_UP]", &format!("UP {control}"))
                .replace("[ACCION]", control)
        })
        .collect()
}

// ======================================================
// ⏹️ DETENER
// ======================================================

fn detener(id: String) {
    if let Some(detenido) = INSTANCIAS.lock().unwrap().get_mut(&id) {
        *detenido = true;
    }
}

// ======================================================
// ❓ DEBE DETENERSE
// ------------------------------------------------------
// Si el id ya no está registrado (nunca existió, o ya se
// limpió), se considera que debe detenerse — nunca se
// entra a un ciclo sin garantía de poder pararlo.
// ======================================================

fn debe_detenerse(id: &str) -> bool {
    INSTANCIAS.lock().unwrap().get(id).copied().unwrap_or(true)
}

// ======================================================
// 📜 EJECUTAR MACRO DE ARCHIVO (en su propio hilo)
// ======================================================

fn ejecutar_macro_en_hilo(id: String, ruta: String) {
    thread::spawn(move || {
        let archivo = match File::open(&ruta) {
            Ok(valor) => valor,

            Err(_) => {
                // FALTA CREAR:
                // sistema global de notificaciones y registro.

                limpiar_instancia(id);

                return;
            }
        };

        let lector = BufReader::new(archivo);

        let lineas: Vec<String> = lector
            .lines()
            .filter_map(|linea| linea.ok())
            .map(|linea| linea.trim().to_string())
            .filter(|linea| !linea.is_empty())
            .collect();

        ejecutar_lineas(id, lineas);
    });
}

// ======================================================
// 🔁 EJECUTAR LÍNEAS
// ------------------------------------------------------
// El loop real de ejecución. Corre siempre dentro de un
// hilo propio de la instancia (lo arma quien la llama).
// Antes de cada REPETIR, revisa si llegó orden de
// detener — nunca entra a una vuelta sin chequear antes.
// ======================================================

fn ejecutar_lineas(id: String, lineas: Vec<String>) {
    INSTANCIAS.lock().unwrap().insert(id.clone(), false);

    'ciclo: loop {
        for linea in &lineas {
            if linea.is_empty() {
                continue;
            }

            if linea == "REPETIR" {
                if debe_detenerse(&id) {
                    break 'ciclo;
                }

                continue 'ciclo;
            }

            ejecutar_linea(&id, linea);
        }

        break;
    }

    limpiar_instancia(id);
}

// ======================================================
// 🧠 INTERPRETAR LÍNEA
// ======================================================

fn ejecutar_linea(id: &str, linea: &str) {
    let partes: Vec<&str> = linea.split_whitespace().collect();

    if partes.is_empty() {
        return;
    }

    match partes[0] {
        // ==============================================
        // ⏱️ ESPERAR
        // ==============================================
        "ESPERAR" => {
            if let Some(valor) = partes.get(1) {
                if let Ok(ms) = valor.parse::<u64>() {
                    esperar(ms);
                }
            }
        }

        // ==============================================
        // ⬇️ DOWN
        // ==============================================
        "DOWN" => {
            if let Some(control) = partes.get(1) {
                ejecutar_down(control);
            }
        }

        // ==============================================
        // ⬆️ UP
        // ==============================================
        "UP" => {
            if let Some(control) = partes.get(1) {
                ejecutar_up(control);
            }
        }

        // ==============================================
        // ⏹️ DETENER
        // ==============================================
        "DETENER" => {
            detener(id.to_string());
        }

        // ==============================================
        // ⚡ PULSE
        // ==============================================
        _ => {
            ejecutar_pulse(partes[0]);
        }
    }
}

// ======================================================
// 🧹 LIMPIAR INSTANCIA
// ======================================================

fn limpiar_instancia(id: String) {
    INSTANCIAS.lock().unwrap().remove(&id);
}

// ======================================================
// 🔎 RESOLVER INPUT
// ======================================================

fn resolver_input(interno: &str) -> Option<InputId> {
    let pulsador = crate::pulsadores::por_interno(interno)?;

    Some(InputId::new(&pulsador.fuente, &pulsador.interception))
}

// ======================================================
// ⬇️ DOWN
// ======================================================

fn ejecutar_down(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    let evento = crate::eventos::InputEvent::down(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⬆️ UP
// ======================================================

fn ejecutar_up(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    let evento = crate::eventos::InputEvent::up(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⚡ PULSE
// ======================================================

fn ejecutar_pulse(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    emitir(input);
}

// ======================================================
// 📤 EMITIR EVENTO
// ======================================================

fn emitir_evento(evento: crate::eventos::InputEvent) {
    back_interception::emitir_evento(evento);
}
