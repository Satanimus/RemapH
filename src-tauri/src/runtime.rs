// ======================================================
// ⚙️ Runtime
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
//    (Emitir(Vec<InputId>), Macro(String), AbrirArchivo(String),
//    Ui(String)). Si algún día se reescribe perfil_cache.rs,
//    no volver a variantes struct sin revisar compilador.rs
//    y este archivo — ya se rompió una vez por esto.
//    Emitir guarda modificadores + gatillo en ese orden
//    (último elemento = gatillo, ver perfil_cache.rs) — nunca
//    un InputId suelto, para no perder el Mod de un combo
//    A+B en la salida (bug 3-A).
//
// C) Extra (Turbo/Mantener/Toggle/etc.) SIEMPRE pasa por
//    runt_extra::obtener(&extra), que devuelve un molde en
//    Idioma Runtime con placeholders [ACCION] /
//    [ACCION_DOWN] / [ACCION_UP]. Runtime sustituye esos
//    placeholders por los pasos reales de la Acción de esa
//    fila (sustituir_accion()) y lo corre como una macro
//    más. Con un combo como Acción, cada placeholder puede
//    expandirse a VARIAS líneas (no es un simple reemplazo
//    de texto): [ACCION] → DOWN de cada mod (en orden) → DOWN
//    gatillo → UP gatillo → UP de cada mod (orden inverso).
//    [ACCION_DOWN] → solo la mitad de abajo (DOWN mods + DOWN
//    gatillo); [ACCION_UP] → solo la mitad de arriba (UP
//    gatillo + UP mods en reversa). Así Turbo repite el combo
//    completo en cada vuelta, y Mantener no suelta nada hasta
//    el final. La sustitución SOLO tiene efecto si la Acción
//    es Emitir — para Macro/AbrirArchivo/Ui no hay down/up
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
//    mientras dura la espera/repetición.
//
//    Emitir tampoco corre ahí — aunque no tiene ESPERAR ni
//    REPETIR, un combo (Vec<InputId> de varios pasos DOWN/
//    UP reales) sí toma un tiempo real de más de un send, y
//    bloquear ahí retrasaba la lectura del próximo evento
//    físico (goteo con triggers seguidos rápido — bug
//    encontrado después de introducir combos en Emitir). Por
//    eso Emitir va a COLA_SALIDA: un canal + un único hilo
//    de salida de por vida, no uno por id — no hace falta
//    Detener un Emitir a mitad de camino, así que no necesita
//    la maquinaria de INSTANCIAS. Solo Ui/AbrirArchivo siguen
//    sin hilo — son de verdad instantáneas, no producen
//    eventos físicos encadenados.
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
//    caminos distintos para lo mismo. Emitir no usa esto —
//    ver COLA_SALIDA en E).
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
//     (ejecutar_extra_en_hilo). Si no: Emitir encola en
//     COLA_SALIDA (no bloquea), AbrirArchivo/Ui corren
//     directo, y Macro va a su propio hilo.
// ejecutar_extra_en_hilo(id, extra, accion)
//     Pide el molde a runt_extra, sustituye los
//     placeholders, corre el resultado como macro en un
//     hilo nuevo.
// sustituir_accion(lineas, accion)
//     Expande [ACCION]/[ACCION_DOWN]/[ACCION_UP] a los pasos
//     DOWN/UP reales del combo (mods + gatillo), solo si
//     accion es Emitir. Una línea con placeholder puede
//     convertirse en varias líneas.
// ejecutar_emitir(inputs, condicion)
//     Despacha un Emitir directo (sin Extra) según su
//     condición: Simple → emitir_combo() una vez. Doble →
//     emitir_combo() dos veces, separadas por
//     config::delay_entre_salida_doble(). Mantenido →
//     emitir_combo_abajo(), espera
//     config::tiempo_salida_mantenido(), emitir_combo_arriba().
// emitir_combo(inputs) / emitir_combo_abajo(inputs) /
// emitir_combo_arriba(inputs)
//     Ejecuta (o solo la mitad de) un combo: DOWN de los
//     mods en orden, DOWN+UP del gatillo, UP de los mods en
//     orden inverso.
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
// esperar_detener(id)
//     Caso especial de ESPERAR: en vez de dormir un tiempo
//     fijo, bloquea sondeando debe_detenerse() hasta que
//     llegue la orden de detener. Usado por Mantener/Click
//     Sostenido para no soltar la acción hasta que Cache
//     avise que el físico se soltó.
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
// Una línea = un paso. Vocabulario fijo, en español
// (excepto DOWN/UP, que se mantienen en inglés):
//
// ESPERAR 50
// ESPERAR DETENER     (bloquea hasta la orden de detener)
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
//     ├─ Emitir → COLA_SALIDA (encola, no bloquea) → hilo
//     │           de salida dedicado → emitir_combo()
//     ├─ AbrirArchivo / Ui → ejecución directa
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
use crate::back_coordenada;

use crate::back_interception;

pub use crate::cache::OrdenRuntime;

use crate::config;

use crate::eventos::InputId;

use crate::perfil_cache::{
    AccionCache, CondicionTrigger, CoordenadaCache, ExtraCache, PostAccionCache,
};

use crate::runt_extra;

use std::collections::HashMap;

use std::fs::File;

use std::io::{BufRead, BufReader};

use std::sync::mpsc::Sender;

use std::sync::Mutex;

use std::thread;

use std::time::Duration;

// ======================================================
// 🗂️ INSTANCIAS ACTIVAS
// ------------------------------------------------------
// Un id -> true si le llegó orden de detener. Cada hilo
// de ejecución consulta esto antes de cada REPETIR.
// ======================================================

static INSTANCIAS: std::sync::LazyLock<Mutex<HashMap<String, bool>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// ======================================================
// 📤 COLA DE SALIDA (Emitir, sin Extra)
// ------------------------------------------------------
// Un Emitir (con o sin combo) YA no se ejecuta en el hilo
// que llama a ejecutar() — ese hilo es, en la práctica,
// siempre el mismo que lee el driver de captura
// (back_interception::iniciar(), un loop estrictamente
// serial: no vuelve a leer la próxima tecla física hasta
// que ejecutar() vuelve). Antes de la Vec<InputId> de
// combos esto no importaba (Emitir era un único pulse,
// prácticamente instantáneo) — pero un combo de varias
// teclas (varios DOWN/UP reales, uno por uno) sí toma un
// tiempo real, y ejecutarlo ahí bloqueaba la lectura del
// próximo evento físico: con teclas encadenadas rápido, la
// salida se iba atrasando cada vez más (goteo), porque el
// hilo de captura solo avanzaba al siguiente evento cuando
// terminaba de mandar el combo completo del anterior.
//
// Solución: un canal FIFO + UN solo hilo de salida
// dedicado. ejecutar_accion() ya no llama a emitir_combo()
// directo — encola el Vec<InputId> (operación no bloqueante,
// sobre un channel sin límite) y vuelve enseguida a leer
// el próximo evento físico. El hilo de salida drena la cola
// por su cuenta, en orden, a su propio ritmo — no necesita
// que llegue un evento físico nuevo para seguir procesando
// lo que ya tiene encolado.
//
// El orden de salida queda garantizado por el canal en sí:
// hay un solo productor real (el hilo de captura, que es
// intrínsecamente serial) y un solo consumidor (este hilo),
// así que el orden de llegada es siempre el mismo orden en
// que se dispararon los triggers. El Mutex de abajo no es
// para eso — es porque mpsc::Sender<T> no es Sync (no se
// puede compartir por referencia entre hilos, cada hilo
// necesita su propio clone), y un `static` necesita que su
// contenido sea Sync. Envolverlo en Mutex<Sender<..>> lo
// resuelve sin agregar contención real: send() es rápido y,
// hoy, el único que lo llama es el hilo de captura.
//
// No hace falta registrar esto en INSTANCIAS: un Emitir
// nunca recibe Detener real (Cache lo manda con
// Iniciar+Detener juntos, ver iniciar_y_finalizar en
// cache.rs) — no hay nada que cancelar a mitad de camino.
// ======================================================

static COLA_SALIDA: std::sync::LazyLock<Mutex<Sender<(Vec<InputId>, CondicionTrigger)>>> =
    std::sync::LazyLock::new(|| {
        let (tx, rx) = std::sync::mpsc::channel::<(Vec<InputId>, CondicionTrigger)>();

        thread::spawn(move || {
            for (inputs, condicion) in rx {
                ejecutar_emitir(&inputs, &condicion);
            }
        });

        Mutex::new(tx)
    });

// ======================================================
// 🚀 EJECUTAR ORDEN
// ======================================================

pub fn ejecutar(orden: OrdenRuntime) {
    match orden {
        OrdenRuntime::Iniciar {
            id,
            accion,
            extra,
            coordenada,
        } => {
            ejecutar_accion(id, accion, extra, coordenada);
        }

        OrdenRuntime::Detener { id } => {
            detener(id);
        }
    }
}

// ======================================================
// ⚡ EJECUTAR ACCIÓN
// ======================================================

fn ejecutar_accion(
    id: String,
    accion: AccionCache,
    extra: Option<ExtraCache>,
    coordenada: Option<CoordenadaCache>,
) {
    if let Some(coordenada) = coordenada {
        ejecutar_click_coordenada(id, accion, extra, coordenada);

        return;
    }

    if let Some(extra) = extra {
        ejecutar_extra_en_hilo(id, extra, accion);

        return;
    }

    match accion {
        AccionCache::Emitir(inputs, condicion) => {
            // No se ejecuta acá — se encola para el hilo de salida
            // dedicado (ver COLA_SALIDA). El lock es sobre el
            // Sender, no sobre la cola en sí — send() es rápido,
            // así que la contención es irrelevante en la práctica
            // (además, hoy solo hay un llamador: el hilo de
            // captura). Si el receiver ya no existiera (no debería
            // pasar, el hilo es de por vida), se ignora en
            // silencio: no hay nada más que hacer con un envío
            // fallido acá.
            let _ = COLA_SALIDA.lock().unwrap().send((inputs, condicion));
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

        AccionCache::Multimedia(comando, alcance) => {
            crate::back_multimedia::ejecutar(&comando, &alcance);
        }
    }
}

// ======================================================
// 🖱️ EJECUTAR CLICK EN COORDENADA
// ------------------------------------------------------
// El destino se calcula UNA sola vez acá, antes de arrancar
// — nunca se recalcula durante Turbo/Mantener (ver
// perfil_cache.rs, CoordenadaCache). Corre siempre en su
// propio hilo (igual que Extra/Macro): mover el cursor +
// ejecutar un combo ya es más de un pulse instantáneo, así
// que no debe bloquear el hilo que lee el input físico.
//
// Post-acción "Inicial": se guarda la posición real del
// cursor ANTES de moverlo, y se restaura recién cuando la
// ejecución completa termina — para Mantener/Turbo, eso es
// cuando ejecutar_lineas() vuelve (o sea, cuando llegó el
// Detener real). Post-acción "Final": no se guarda nada, el
// cursor queda donde el click lo dejó.
// ======================================================

fn ejecutar_click_coordenada(
    id: String,
    accion: AccionCache,
    extra: Option<ExtraCache>,
    coordenada: CoordenadaCache,
) {
    thread::spawn(move || {
        let origen = match coordenada.post_accion {
            PostAccionCache::Inicial => Some(back_coordenada::obtener_cursor()),
            PostAccionCache::Final => None,
        };

        let destino = back_coordenada::calcular_destino(&coordenada.ubicacion);
        back_coordenada::mover_cursor(destino.0, destino.1);

        match extra {
            Some(extra) => {
                let lineas = runt_extra::obtener(&extra);
                let lineas = sustituir_accion(lineas, &accion);
                ejecutar_lineas(id, lineas);
            }

            None => {
                if let AccionCache::Emitir(inputs, condicion) = &accion {
                    ejecutar_emitir(inputs, condicion);
                }
            }
        }

        if let Some((x, y)) = origen {
            back_coordenada::mover_cursor(x, y);
        }
    });
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
// ⚡ EJECUTAR EMITIR (según condición: Simple/Doble/Mantenido)
// ------------------------------------------------------
// Único lugar que decide, para un Emitir sin Extra, cómo
// ejecutar el combo según la condición capturada en la
// Acción (ver perfil_cache.rs / compilador.rs — antes esto
// se descartaba y todo se ejecutaba como Simple):
// • Simple    → un combo completo (down+up).
// • Doble     → un combo completo, espera
//               config::delay_entre_salida_doble(), y
//               repite el combo completo.
// • Mantenido → solo el DOWN del combo, espera
//               config::tiempo_salida_mantenido(), y recién
//               ahí manda el UP.
// Llamado tanto desde el hilo de salida dedicado
// (COLA_SALIDA) como desde Click en coordenada sin Extra.
// ======================================================

fn ejecutar_emitir(inputs: &[InputId], condicion: &CondicionTrigger) {
    match condicion {
        CondicionTrigger::Simple => emitir_combo(inputs),

        CondicionTrigger::Doble => {
            emitir_combo(inputs);

            thread::sleep(Duration::from_millis(config::delay_entre_salida_doble()));

            emitir_combo(inputs);
        }

        CondicionTrigger::Mantenido => {
            emitir_combo_abajo(inputs);

            thread::sleep(Duration::from_millis(config::tiempo_salida_mantenido()));

            emitir_combo_arriba(inputs);
        }
    }
}

// ======================================================
// 📤 EMITIR COMBO (Emitir directo, sin Extra)
// ------------------------------------------------------
// inputs = [mod1, mod2, ..., gatillo] (último = gatillo,
// ver perfil_cache.rs). DOWN de los mods en orden → DOWN+UP
// del gatillo → UP de los mods en orden inverso. Con un solo
// elemento (sin modificadores) es equivalente a un pulse.
// ======================================================

fn emitir_combo(inputs: &[InputId]) {
    emitir_combo_abajo(inputs);
    emitir_combo_arriba(inputs);
}

// ======================================================
// ⬇️ EMITIR COMBO ABAJO (solo la mitad DOWN)
// ------------------------------------------------------
// DOWN de los mods en orden → DOWN del gatillo. Usado solo
// (sin la mitad de arriba) por Mantenido, para dejar la
// tecla de salida apretada hasta el UP.
// ======================================================

fn emitir_combo_abajo(inputs: &[InputId]) {
    let Some((gatillo, mods)) = inputs.split_last() else {
        return;
    };

    for modificador in mods {
        emitir_down_input(modificador.clone());
    }

    emitir_down_input(gatillo.clone());
}

// ======================================================
// ⬆️ EMITIR COMBO ARRIBA (solo la mitad UP)
// ------------------------------------------------------
// UP del gatillo → UP de los mods en orden inverso.
// ======================================================

fn emitir_combo_arriba(inputs: &[InputId]) {
    let Some((gatillo, mods)) = inputs.split_last() else {
        return;
    };

    emitir_up_input(gatillo.clone());

    for modificador in mods.iter().rev() {
        emitir_up_input(modificador.clone());
    }
}

fn emitir_down_input(input: InputId) {
    let evento = crate::eventos::InputEvent::down(input, crate::instante::ahora());

    emitir_evento(evento);
}

fn emitir_up_input(input: InputId) {
    let evento = crate::eventos::InputEvent::up(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⏱️ ESPERAR
// ======================================================

fn esperar(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

// ======================================================
// ⏸️ ESPERAR DETENER
// ------------------------------------------------------
// Bloquea el hilo de la instancia hasta que llegue la
// orden de detener (detener(id)) — usado por Mantener y
// Click Sostenido para no soltar la acción hasta que el
// físico se suelte (Cache es quien manda el Detener en
// ese momento). Sondea debe_detenerse() en vez de dormir
// una sola vez porque no hay forma de despertar el hilo
// desde afuera — el intervalo es corto para que la
// liberación se sienta instantánea.
// ======================================================

fn esperar_detener(id: &str) {
    while !debe_detenerse(id) {
        thread::sleep(Duration::from_millis(15));
    }
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
// tecla/botón física, o un combo mod+gatillo). Los demás
// tipos de acción no tienen un down/up que sustituir — la
// línea queda como está (no hace nada al ejecutarse, no
// rompe nada).
//
// Con un combo, un placeholder que ocupa toda la línea se
// expande a VARIAS líneas (no un simple reemplazo de texto):
// • [ACCION_DOWN] → DOWN de cada mod (orden) + DOWN gatillo
// • [ACCION_UP]   → UP gatillo + UP de cada mod (orden inverso)
// • [ACCION]      → la secuencia [ACCION_DOWN] + [ACCION_UP]
//   seguida, para que un solo pulso incluya el combo entero
//   (así Turbo repite mod+gatillo completos en cada vuelta).
//
// Un placeholder que viene mezclado con más texto en la
// misma línea (ej. "TOGGLE [ACCION]") no se puede expandir a
// varias líneas sin romper esa línea, así que ahí se sustituye
// solo por el control del gatillo — igual que antes de este
// cambio. Toggle hoy no distingue combos con modificador.
// ======================================================

fn sustituir_accion(lineas: Vec<String>, accion: &AccionCache) -> Vec<String> {
    // La condición (Simple/Doble/Mantenido) de la Acción no aplica
    // acá — un Extra (Turbo/Mantener/Toggle) ya define su propio
    // ritmo de repetición/sostenido, así que el placeholder siempre
    // se expande a un combo simple down/up.
    let AccionCache::Emitir(inputs, _condicion) = accion else {
        return lineas;
    };

    let controles: Vec<&str> = inputs.iter().filter_map(InputId::control).collect();

    let (gatillo, mods) = match controles.split_last() {
        Some((gatillo, mods)) => (*gatillo, mods),

        None => return lineas,
    };

    lineas
        .into_iter()
        .flat_map(|linea| expandir_placeholder(linea, mods, gatillo))
        .collect()
}

fn expandir_placeholder(linea: String, mods: &[&str], gatillo: &str) -> Vec<String> {
    match linea.as_str() {
        "[ACCION_DOWN]" => lineas_abajo(mods, gatillo),

        "[ACCION_UP]" => lineas_arriba(mods, gatillo),

        "[ACCION]" => {
            let mut pasos = lineas_abajo(mods, gatillo);

            pasos.extend(lineas_arriba(mods, gatillo));

            pasos
        }

        _ => vec![linea
            .replace("[ACCION_DOWN]", &format!("DOWN {gatillo}"))
            .replace("[ACCION_UP]", &format!("UP {gatillo}"))
            .replace("[ACCION]", gatillo)],
    }
}

fn lineas_abajo(mods: &[&str], gatillo: &str) -> Vec<String> {
    mods.iter()
        .map(|modificador| format!("DOWN {modificador}"))
        .chain(std::iter::once(format!("DOWN {gatillo}")))
        .collect()
}

fn lineas_arriba(mods: &[&str], gatillo: &str) -> Vec<String> {
    std::iter::once(format!("UP {gatillo}"))
        .chain(
            mods.iter()
                .rev()
                .map(|modificador| format!("UP {modificador}")),
        )
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
                if *valor == "DETENER" {
                    esperar_detener(id);
                } else if let Ok(ms) = valor.parse::<u64>() {
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
