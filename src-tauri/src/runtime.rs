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
//    REPETIR · DETENER · INICIO_BUCLE · <control> (a
//    secas = pulse). Los <control> son el nombre "interno"
//    de pulsadores.tsv, sin prefijo de dispositivo —
//    Runtime nunca deduce el dispositivo, pulsadores.tsv ya
//    lo sabe. INICIO_BUCLE es solo una marca (no hace nada
//    al ejecutarse): REPETIR salta ahí si la receta la
//    tiene, o a la línea 0 si no la tiene — ver
//    ejecutar_lineas().
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
//    de texto), y la CONDICIÓN de la Acción (Simple/Doble/
//    Triple/Mantenido) decide cuántas: [ACCION] → todos los
//    toques completos (DOWN mods → DOWN gatillo → UP gatillo
//    → UP mods en reversa, repetido 1/2/3 veces según Simple/
//    Doble/Triple, o DOWN+espera+UP si es Mantenido).
//    [ACCION_DOWN] → todos los toques salvo el último
//    completos, y el último solo con su DOWN (sostenido);
//    [ACCION_UP] → siempre solo la mitad de arriba (UP
//    gatillo + UP mods en reversa), sin importar la condición
//    — lo único pendiente cuando llega el Up físico real es
//    soltar ese último DOWN. Así Turbo repite el combo
//    (con sus N toques) completo en cada vuelta, y Mantener
//    no suelta nada hasta el final — ver
//    lineas_accion_completa/lineas_accion_down/lineas_accion_up.
//    La sustitución SOLO tiene efecto si la Acción es Emitir —
//    para Macro/AbrirArchivo/Ui no hay down/up físico que
//    poner ahí, los placeholders quedan sin reemplazar (línea
//    inofensiva, no rompe nada, pero tampoco hace lo que
//    probablemente se esperaba: esa combinación no está
//    pensada para usarse — la UI es quien debe evitar
//    ofrecerla).
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
//     DOWN/UP reales del combo (mods + gatillo) según la
//     condición de la Acción (Simple/Doble/Triple/Mantenido),
//     solo si accion es Emitir. Una línea con placeholder
//     puede convertirse en varias líneas.
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
//     El loop real: recorre las líneas, en cada REPETIR
//     vuelve a la línea INICIO_BUCLE si la receta la tiene
//     (o a la línea 0 si no) salvo que debe_detenerse()
//     diga que no siga. Registra y limpia la instancia en
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
// INICIO_BUCLE        (marca; REPETIR salta acá si existe,
//                      si no, salta a la línea 0)
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

use crate::back_menu_express;

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

        // Alternar la ventana flotante del menú (etapa 5) — abrir_o_
        // alternar() decide sola si toca crear la ventana o cerrar la
        // que ya estaba abierta para este id (mismo trigger = toggle,
        // ver back_menu_express.rs). El id de la ORDEN (no de la
        // Acción) es el mismo RemapeoJson::id de la fila MenuExpress.
        AccionCache::MenuExpress {
            nombre,
            botones,
            forma,
            columnas,
            filas,
            comportamiento,
            ubicacion,
            tamano_boton,
            tamano_texto,
            color,
            color_boton,
        } => {
            back_menu_express::abrir_o_alternar(
                id,
                back_menu_express::MenuExpressPaquete {
                    nombre,
                    botones,
                    forma,
                    columnas,
                    filas,
                    comportamiento,
                    ubicacion,
                    tamano_boton,
                    tamano_texto,
                    color,
                    color_boton,
                },
            );
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
// ⚡ EJECUTAR EMITIR (según condición: Simple/Doble/Triple/Mantenido)
// ------------------------------------------------------
// Único lugar que decide, para un Emitir sin Extra, cómo
// ejecutar el combo según la condición capturada en la
// Acción (ver perfil_cache.rs / compilador.rs — antes esto
// se descartaba y todo se ejecutaba como Simple):
// • Simple    → un combo completo (down+up).
// • Doble     → un combo completo, espera
//               config::delay_entre_salida_doble(), y
//               repite el combo completo.
// • Triple    → un combo completo, espera
//               config::delay_entre_salida_doble(), repite,
//               espera de nuevo el mismo delay, y repite una
//               tercera vez (reusa el delay de Doble, sin
//               campo propio en config.rs).
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

        CondicionTrigger::Triple => {
            emitir_combo(inputs);

            thread::sleep(Duration::from_millis(config::delay_entre_salida_doble()));

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
// expande a VARIAS líneas (no un simple reemplazo de texto)
// según la condición de la Acción — ver el bloque de
// comentario sobre lineas_accion_completa/lineas_accion_down/
// lineas_accion_up, más abajo, para el detalle completo.
//
// Un placeholder que viene mezclado con más texto en la
// misma línea (ej. "TOGGLE [ACCION]") no se puede expandir a
// varias líneas sin romper esa línea, así que ahí se sustituye
// solo por el control del gatillo — igual que antes de este
// cambio. Toggle hoy no distingue combos con modificador ni
// condición (queda igual que antes; no está en uso desde la
// UI hoy — ver EXTRA_OPCIONES/EXTRA_TECLA_MOUSE_OPCIONES en
// el frontend, ninguna ofrece "toggle").
// ======================================================

fn sustituir_accion(lineas: Vec<String>, accion: &AccionCache) -> Vec<String> {
    // La condición (Simple/Doble/Triple/Mantenido) de la Acción SÍ
    // aplica acá — cada placeholder se expande según ella (ver
    // lineas_accion_completa/lineas_accion_down/lineas_accion_up),
    // así que un Extra (Turbo/Mantener/etc.) combinado con Doble o
    // Triple dispara los N toques completos, no un down/up simple.
    let AccionCache::Emitir(inputs, condicion) = accion else {
        return lineas;
    };

    let controles: Vec<&str> = inputs.iter().filter_map(InputId::control).collect();

    let (gatillo, mods) = match controles.split_last() {
        Some((gatillo, mods)) => (*gatillo, mods),

        None => return lineas,
    };

    lineas
        .into_iter()
        .flat_map(|linea| expandir_placeholder(linea, mods, gatillo, condicion))
        .collect()
}

fn expandir_placeholder(
    linea: String,
    mods: &[&str],
    gatillo: &str,
    condicion: &CondicionTrigger,
) -> Vec<String> {
    match linea.as_str() {
        "[ACCION_DOWN]" => lineas_accion_down(mods, gatillo, condicion),

        "[ACCION_UP]" => lineas_accion_up(mods, gatillo),

        "[ACCION]" => lineas_accion_completa(mods, gatillo, condicion),

        _ => vec![linea
            .replace("[ACCION_DOWN]", &format!("DOWN {gatillo}"))
            .replace("[ACCION_UP]", &format!("UP {gatillo}"))
            .replace("[ACCION]", gatillo)],
    }
}

// ======================================================
// 🔁 [ACCION] / [ACCION_DOWN] / [ACCION_UP] — condición-aware
// ------------------------------------------------------
// Un "toque" es un DOWN+UP completo del combo. La condición decide
// cuántos toques hacen falta antes de la mitad final:
// • Simple           → 1 toque.
// • Doble            → 2 toques.
// • Triple           → 3 toques.
// • Mantenido        → no tiene "toques": es DOWN, sostenido, UP.
//
// [ACCION] (unidad completa, la usan Normal/Turbo en su bucle):
// todos los toques completos, separados por
// delay_entre_salida_doble; para Mantenido, DOWN + espera
// tiempo_salida_mantenido + UP (sostenido artificial, igual que
// ejecutar_emitir sin Extra).
//
// [ACCION_DOWN]/[ACCION_UP] (mitades separadas, las usan Mantener/
// Click Sostenido para sostener hasta el Up físico real):
// [ACCION_DOWN] manda todos los toques salvo el último completos,
// y deja el último solo con su DOWN (sostenido) — ej. Triple:
// toque, espera, toque, espera, DOWN. [ACCION_UP] siempre es
// solamente la mitad de arriba (UP gatillo + UP mods en reversa):
// no importa la condición, lo único que queda pendiente cuando
// llega el Up físico real es soltar ese último DOWN sostenido.
// Para Mantenido, [ACCION_DOWN] es solo DOWN — su propio
// tiempo_salida_mantenido no aplica acá: el sostenido real que
// pide el Extra (ESPERAR DETENER) ya lo reemplaza.
// ======================================================

fn lineas_un_toque(mods: &[&str], gatillo: &str) -> Vec<String> {
    let mut pasos = lineas_abajo(mods, gatillo);

    pasos.extend(lineas_arriba(mods, gatillo));

    pasos
}

fn lineas_accion_completa(
    mods: &[&str],
    gatillo: &str,
    condicion: &CondicionTrigger,
) -> Vec<String> {
    match condicion {
        CondicionTrigger::Simple => lineas_un_toque(mods, gatillo),

        CondicionTrigger::Doble => lineas_n_toques(mods, gatillo, 2),

        CondicionTrigger::Triple => lineas_n_toques(mods, gatillo, 3),

        CondicionTrigger::Mantenido => {
            let mut pasos = lineas_abajo(mods, gatillo);

            pasos.push(format!("ESPERAR {}", config::tiempo_salida_mantenido()));

            pasos.extend(lineas_arriba(mods, gatillo));

            pasos
        }
    }
}

fn lineas_n_toques(mods: &[&str], gatillo: &str, n: u8) -> Vec<String> {
    let mut pasos = Vec::new();

    for indice in 0..n {
        if indice > 0 {
            pasos.push(format!("ESPERAR {}", config::delay_entre_salida_doble()));
        }

        pasos.extend(lineas_un_toque(mods, gatillo));
    }

    pasos
}

fn lineas_accion_down(mods: &[&str], gatillo: &str, condicion: &CondicionTrigger) -> Vec<String> {
    let toques_previos = match condicion {
        CondicionTrigger::Simple | CondicionTrigger::Mantenido => 0,

        CondicionTrigger::Doble => 1,

        CondicionTrigger::Triple => 2,
    };

    let mut pasos = Vec::new();

    for _ in 0..toques_previos {
        pasos.extend(lineas_un_toque(mods, gatillo));

        pasos.push(format!("ESPERAR {}", config::delay_entre_salida_doble()));
    }

    pasos.extend(lineas_abajo(mods, gatillo));

    pasos
}

fn lineas_accion_up(mods: &[&str], gatillo: &str) -> Vec<String> {
    lineas_arriba(mods, gatillo)
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
//
// REPETIR vuelve a la línea marcada con INICIO_BUCLE si
// la receta la tiene (ej. Normal: primera salida + espera
// distinta antes de arrancar el bucle propiamente dicho).
// Si la receta no tiene esa marca, vuelve a la línea 0,
// igual que siempre (Turbo, Mantener, etc. no cambian).
// INICIO_BUCLE en sí misma no ejecuta ningún paso físico,
// pero SÍ es un punto de chequeo de detener — igual que
// REPETIR — para cortar antes de arrancar la primera vuelta
// del bucle si la orden de detener ya llegó durante la
// espera previa (sin este chequeo, una receta con dos
// [ACCION] antes del primer REPETIR, como Normal, dispararía
// esa segunda salida aunque ya se haya soltado la tecla).
// ======================================================

fn ejecutar_lineas(id: String, lineas: Vec<String>) {
    INSTANCIAS.lock().unwrap().insert(id.clone(), false);

    let inicio_bucle = lineas
        .iter()
        .position(|linea| linea == "INICIO_BUCLE")
        .unwrap_or(0);

    let mut i = 0;

    while i < lineas.len() {
        let linea = &lineas[i];

        if linea.is_empty() {
            i += 1;

            continue;
        }

        // INICIO_BUCLE es también un punto de chequeo: si ya llegó
        // la orden de detener durante la espera previa (ej. la
        // salida inicial de Normal, si soltaste antes de que
        // arranque el bucle), corta ACÁ — antes de ejecutar la
        // primera vuelta del bucle. Sin este chequeo, una tecla
        // Normal tocada rápido igual dispara una segunda salida.
        if linea == "INICIO_BUCLE" {
            if debe_detenerse(&id) {
                break;
            }

            i += 1;

            continue;
        }

        if linea == "REPETIR" {
            if debe_detenerse(&id) {
                break;
            }

            i = inicio_bucle;

            continue;
        }

        ejecutar_linea(&id, linea);

        i += 1;
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
