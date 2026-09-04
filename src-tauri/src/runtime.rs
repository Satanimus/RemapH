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
// back_portapapeles.rs — llama emitir_ctrl_v() directo (no
//     a través de ejecutar()/OrdenRuntime) para el pegado
//     automático tras escribir al portapapeles. Ver comentario
//     largo en emitir_ctrl_v() más abajo.
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
// motor::emitir_evento() para cada paso físico (motor.rs
// decide si eso termina en back_interception o back_windows,
// según el modo activo).
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
// B) AccionCache — Emitir/Macro/Ui siguen siendo tupla; AbrirArchivo
//    es struct (ruta, iniciar, instancias, abrir_con, argumento —
//    ver perfil_cache.rs). Si se vuelve a tocar esa variante, revisar
//    también compilador.rs y este archivo — ya se rompió una vez por
//    un desajuste entre los tres.
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
//    la maquinaria de INSTANCIAS. AbrirArchivo también corre en
//    su propio hilo (ShellExecuteExW + buscar la ventana del PID
//    lanzado para forzarle el foco puede tardar) pero sin pasar
//    por INSTANCIAS —es de una sola vez, no hay nada que Detener
//    a mitad de camino.
//    Solo Ui sigue sin hilo — es de verdad instantánea, no
//    produce eventos físicos encadenados.
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
//    esperar_detener() (Mantener/Click Sostenido) es la única
//    excepción a "se revisa antes de cada REPETIR": ahí no hay
//    ningún REPETIR que visitar mientras se espera el Up físico,
//    así que en vez de sondear duerme en INSTANCIAS_CONDVAR,
//    acoplado al mismo Mutex — ver esa función más abajo.
//
//    [FIX] Carrera Iniciar+Detener "pegados" (sin demora real
//    entre uno y otro — típico de un tap rápido sobre un
//    trigger diferido por ambigüedad, ver cache.rs): Cache
//    encola la generación en GENERACIONES de forma síncrona
//    (nueva_id_ejecucion(), ANTES de spawnear el hilo), pero
//    la entrada real en INSTANCIAS recién se crea DENTRO del
//    hilo spawneado, al llegar a ejecutar_lineas(). Si
//    detener_ejecucion() corría con get_mut() (no hace nada
//    si la clave no existe todavía), un Detener que llegaba
//    antes de que el hilo nuevo alcanzara a arrancar se
//    perdía en silencio — el hilo, al arrancar después,
//    insertaba `false` sin saber que ya le habían pedido
//    pararse, y corría para siempre (bug: "1" en bucle
//    infinito que no se detiene; o Mantenido que nunca manda
//    el Up). Ahora detener_ejecucion() PRE-REGISTRA la orden
//    con insert() (crea la entrada si no existía), y
//    ejecutar_lineas() usa entry().or_insert(false) en vez de
//    insert() a secas, para no pisar un `true` que ya haya
//    llegado antes de que el hilo arrancara.
//
// G) Backend de salida: emitir_evento() usa
//    back_interception exclusivamente. Ya no existe el
//    modo dual con back_windows (descartado para la 1.0,
//    ver decisiones de backend/). Esto también aplica a
//    emitir_ctrl_v() (usado por back_portapapeles.rs, ver
//    comentario ahí): antes esa función usaba SendInput/
//    WinAPI directo, único lugar del proyecto que lo hacía
//    para emitir teclas — se confirmó que Paint (UWP) no
//    reaccionaba a esos eventos aunque SendInput reportara
//    insertarlos sin objeción, mientras que el mismo Ctrl+V
//    emitido vía back_interception (como cualquier otro
//    Emitir del motor) sí funcionaba. La primera versión de
//    emitir_ctrl_v() llamaba emitir_combo() directo desde el
//    hilo que originó el pegado (comando Tauri) — a
//    diferencia de un Emitir real, que SIEMPRE pasa por
//    COLA_SALIDA (ver E) y nunca corre en el hilo que
//    disparó el trigger. emitir_ctrl_v() ahora encola en
//    COLA_SALIDA igual que cualquier Emitir, para eliminar
//    esa única diferencia de contexto que quedaba entre un
//    atajo de Menú Express y el pegado automático.
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
//     condición: Simple → un toque (DOWN, espera
//     config::tiempo_simple_teclas(), UP). Doble/Triple →
//     mods abajo UNA sola vez, el gatillo se golpea 2/3
//     veces (cada golpe con tiempo_simple_teclas entre su
//     DOWN y su UP, config::delay_entre_salida_doble() entre
//     golpes), mods arriba al final. Mantenido →
//     emitir_combo_abajo(), espera config::tiempo_mantenido(),
//     emitir_combo_arriba().
// emitir_combo_abajo(inputs) / emitir_combo_arriba(inputs)
//     Ejecuta solo la mitad de un combo completo (mods+gatillo
//     juntos): DOWN de los mods en orden + DOWN del gatillo, o
//     UP del gatillo + UP de los mods en orden inverso. Usado
//     por Mantenido (abajo/arriba del bloque entero) y por
//     lineas_abajo/lineas_arriba del lado del Idioma Runtime.
// emitir_ctrl_v()
//     Atajo público de conveniencia: arma [LeftControl, V]
//     como InputId y los ENCOLA en COLA_SALIDA — mismo camino
//     real, mismo hilo dedicado, que un Emitir configurado por
//     el usuario. Usado por back_portapapeles::pegar() para el
//     pegado automático.
// ejecutar_macro_en_hilo(id, ruta)
//     [Etapa 8B] Eliminada — el ejecutor de macro de texto plano
//     (basado en split_whitespace, sin soporte de rutas/argumentos)
//     se reemplazó por runt_macro.rs, que interpreta el JSON de
//     pasos directo. ejecutar_lineas/ejecutar_linea NO se tocaron
//     (Turbo/Normal/Mantener los siguen usando vía runt_extra).
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
// detener(id) / detener_ejecucion(id) / debe_detenerse(id) /
// limpiar_instancia(id) / nueva_id_ejecucion(id)
//     Manejo de la bandera compartida en INSTANCIAS. La
//     clave real ahí adentro es siempre un id ÚNICO POR
//     EJECUCIÓN ("idFila#generación", ver
//     nueva_id_ejecucion()/GENERACIONES) — nunca el id de
//     fila solo, para que dos ejecuciones superpuestas de la
//     misma fila (ej. dos toques rápidos de una tecla Normal)
//     nunca compartan bandera. detener(id) recibe el id de
//     FILA (es lo único que Cache conoce) y lo traduce a la
//     ejecución real más vieja todavía pendiente para esa
//     fila; detener_ejecucion(id) recibe directo el id de
//     ejecución (lo usa el comando "DETENER" de una receta,
//     que se refiere a sí mismo).
// resolver_input(interno) / ejecutar_down() /
// ejecutar_up() / ejecutar_pulse() / emitir() / esperar()
//     Los pasos físicos individuales del Idioma Runtime.
// abrir_archivo(ruta, iniciar, instancias, abrir_con, argumento)
//     En su propio hilo: si Instancias es Única, intenta encontrar
//     una ventana ya abierta antes de lanzar nada nuevo — carpeta vía
//     back_app::enfocar_carpeta() (proceso explorer.exe + título);
//     .exe directo vía back_app::enfocar_proceso() (solo proceso, no
//     hay archivo que matchear en el título); cualquier documento
//     (propio o vía abrir_con) vía back_app::enfocar_documento()
//     (proceso objetivo conocido — .exe directo, abrir_con, el
//     programa predeterminado de la extensión resuelto por
//     back_registro, o el mapeo de apps UWP conocidas de
//     proceso_uwp_conocido() — Y título de la ventana conteniendo el
//     nombre de ESE archivo puntual, para no confundirlo con otro
//     archivo ya abierto en el mismo programa). Las tres hacen TOGGLE
//     minimizar/restaurar si la encuentran. Si no encuentra nada (o
//     Instancias es Múltiple), arma el comando según el tipo de ítem
//     (.exe: ejecuta con argumento; .lnk: abre el acceso directo;
//     carpeta: fuerza explorer.exe nuevo; abrir_con: el programa
//     elegido; documento sin abrir_con: el programa predeterminado de
//     la extensión resuelto por back_registro, lanzado directo — evita
//     depender del "open" de Windows, que puede reusar una instancia
//     ya corriendo e ignorar Instancias/el modo de ventana) y lo lanza
//     con el modo de ventana de iniciar. Siempre busca después la
//     ventana NUEVA que apareció tras lanzar (comparando contra un
//     snapshot tomado antes — ver back_app::buscar_ventana_nueva, con
//     reintentos cortos; el PID recién lanzado no siempre existe, ej.
//     carpeta) y le reafirma el modo pedido con
//     back_app::reafirmar_modo_ventana() — no solo el foco: también el
//     tamaño/estado de ventana, para las apps que ignoran el nShow que
//     se les pasó y se muestran igual una vez que cargan.
//     LÍMITE CONOCIDO: apps de instancia única por diseño propio (ej.
//     el Notepad moderno de Windows 11, la app Fotos) pueden seguir
//     enrutando la apertura a la ventana ya abierta pase lo que pase
//     acá — no hay forma de forzar una ventana/proceso realmente nuevo
//     desde afuera en esos casos, y por lo tanto tampoco de reafirmar
//     el modo de ventana sobre esa reutilización (reafirmar_modo_ventana
//     solo actúa sobre una ventana nueva).
// mostrar_ui()
//     Sin implementar todavía (fuera de esta sesión de
//     trabajo) — queda como punto de entrada ya conectado.
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
//     ├─ Ui → ejecución directa
//     ├─ AbrirArchivo → hilo propio (ShellExecuteExW + forzar
//     │                 primer plano por PID, sin INSTANCIAS)
//     └─ con Extra o Macro de archivo → hilo propio (id)
//               ↓
//         ejecutar_lineas() [loop, revisa INSTANCIAS
//         antes de cada REPETIR]
//               ↓
//         ejecutar_linea() por cada paso
//               ↓
//         motor::emitir_evento()
//               ↓
//         Dispositivo físico
// ======================================================
use crate::back_app;
use crate::back_coordenada;
use crate::back_registro;

use crate::motor;

use crate::back_menu_express;
use crate::back_portapapeles;

pub use crate::cache::OrdenRuntime;

use crate::config;

use crate::eventos::InputId;

use crate::perfil_cache::{
    AccionCache, CondicionTrigger, CoordenadaCache, ExtraCache, IniciarVentana, InstanciasAbrir,
    PostAccionCache,
};

use crate::runt_extra;

use std::collections::HashMap;

use std::collections::HashSet;

use std::collections::VecDeque;

use std::path::Path;

use std::sync::mpsc::Sender;

use std::sync::Condvar;
use std::sync::Mutex;

use std::thread;

use std::time::Duration;

use windows_sys::Win32::Foundation::CloseHandle;

use windows_sys::Win32::System::Threading::GetProcessId;

use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_SHOWMAXIMIZED, SW_SHOWMINIMIZED, SW_SHOWNORMAL,
};

// ======================================================
// 🗂️ INSTANCIAS ACTIVAS
// ------------------------------------------------------
// Un id -> true si le llegó orden de detener. Cada hilo
// de ejecución consulta esto antes de cada REPETIR.
//
// OJO: la clave de este mapa NUNCA es el id de fila
// (RemapeoCache.id) solo — es un id ÚNICO POR EJECUCIÓN
// (ver nueva_id_ejecucion()/GENERACIONES más abajo). Antes
// se usaba directo el id de fila, fijo, siempre el mismo
// para "3>A" sin importar cuántas veces se dispare: si una
// segunda pulsación de la misma fila arrancaba su hilo
// (INSTANCIAS.insert(id, false)) mientras el hilo de la
// pulsación anterior todavía estaba dormido esperando su
// checkpoint (INICIO_BUCLE/REPETIR) sin haber leído todavía
// el `true` que su propio Detener ya había puesto, ese
// insert nuevo pisaba la bandera vieja — el hilo viejo
// despertaba, encontraba `false` (puesto por el hilo nuevo)
// y entraba igual al bucle de repetición. Con id único por
// ejecución, dos hilos de la misma fila nunca comparten
// entrada en este mapa.
// ======================================================

static INSTANCIAS: std::sync::LazyLock<Mutex<HashMap<String, bool>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

// Condvar acoplado al MISMO Mutex que protege INSTANCIAS (no un
// Mutex<()> aparte, a diferencia de runt_macro::NOTIFICADOR) — así
// esperar_detener() jamás suelta el lock entre "leer la bandera" y
// "dormirse esperando": wait() libera el lock y bloquea el hilo en
// un solo paso atómico a nivel de SO, sin ninguna ventana en la que
// una notificación pueda llegar y perderse entre medio. Toda
// escritura que ponga `true` en INSTANCIAS (detener_ejecucion(),
// detener_todas_las_instancias()) debe notificar acá después.
static INSTANCIAS_CONDVAR: std::sync::LazyLock<Condvar> = std::sync::LazyLock::new(Condvar::new);

// ======================================================
// 🧬 GENERACIONES — id de fila -> ids de ejecución en curso
// ------------------------------------------------------
// Cache solo conoce el id de fila (fijo) — nunca vio, ni
// puede ver, el id único de ejecución que vive acá adentro.
// Cuando manda OrdenRuntime::Detener{id} (id de fila), hay
// que traducirlo a QUÉ ejecución real corresponde detener.
//
// Como el físico no puede tener la misma tecla presionada
// dos veces a la vez, Cache siempre empareja sus Iniciar/
// Detener de una misma fila en el mismo orden en que
// ocurrieron (ACTIVAS los busca con position(), que respeta
// orden de inserción) — así que alcanza con una cola FIFO
// por fila: cada nueva ejecución se encola al final: cada
// Detener saca la más vieja todavía sin resolver.
// ======================================================

static GENERACIONES: std::sync::LazyLock<Mutex<HashMap<String, VecDeque<u64>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static SIGUIENTE_GENERACION: std::sync::LazyLock<Mutex<u64>> =
    std::sync::LazyLock::new(|| Mutex::new(0));

/// Da de alta una ejecución nueva para `id_fila` y devuelve el id
/// combinado ("idFila#generación") que hay que usar como clave real
/// en INSTANCIAS durante toda esa ejecución (ejecutar_lineas y todo lo
/// que cuelga de ahí). Encola la generación en GENERACIONES para que
/// detener(id_fila) sepa, más adelante, a cuál apuntar.
///
/// pub(crate) desde la Etapa 8B: runt_macro.rs necesita el mismo
/// mecanismo de id único por ejecución para sus propias instancias
/// (Una ejecución/Toggle registra su propio id de ejecución; Tecla
/// mantenida se anota en GENERACIONES para que el Up físico real la
/// encuentre, ver runt_macro.rs).
pub(crate) fn nueva_id_ejecucion(id_fila: &str) -> String {
    let generacion = {
        let mut siguiente = SIGUIENTE_GENERACION.lock().unwrap();
        *siguiente += 1;
        *siguiente
    };

    GENERACIONES
        .lock()
        .unwrap()
        .entry(id_fila.to_string())
        .or_default()
        .push_back(generacion);

    format!("{id_fila}#{generacion}")
}

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

        // Etapa 8B: el ejecutor real vive en runt_macro.rs — arma
        // los pasos, Bucle/Marcador y decide qué hacer según
        // Comportamiento. Runtime solo despacha. A diferencia del
        // resto de las variantes de este match, acá NO se genera
        // ningún id de ejecución en GENERACIONES/INSTANCIAS desde
        // este punto — runt_macro::iniciar() decide por su cuenta
        // (registro propio para Una ejecución/Toggle, o
        // nueva_id_ejecucion() + INSTANCIAS solo para Tecla
        // mantenida, ver runt_macro.rs).
        AccionCache::Macro {
            nombre,
            programa,
            comportamiento,
            indicador_ejecucion,
        } => {
            crate::runt_macro::iniciar(id, nombre, programa, comportamiento, indicador_ejecucion);
        }

        AccionCache::AbrirArchivo {
            ruta,
            iniciar,
            instancias,
            abrir_con,
            argumento,
        } => {
            abrir_archivo(ruta, iniciar, instancias, abrir_con, argumento);
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

        // Alternar la ventana flotante de Portapapeles — mismo
        // criterio que el brazo MenuExpress de arriba:
        // abrir_o_alternar() decide sola si toca crear la ventana o
        // cerrar la que ya estaba abierta para este id (mismo
        // trigger = toggle). El id de la ORDEN (no de la Acción) es
        // el mismo RemapeoJson::id de la fila Portapapeles — el
        // mismo que identifica sus fijados y su entrada en ACTIVOS
        // (ver back_portapapeles.rs).
        AccionCache::Portapapeles {
            nombre,
            comportamiento,
            ubicacion,
            tamano_boton,
            tamano_texto,
            limite,
            color,
        } => {
            back_portapapeles::abrir_o_alternar(
                id,
                back_portapapeles::PortapapelesPaquete {
                    nombre,
                    comportamiento,
                    ubicacion,
                    tamano_boton,
                    tamano_texto,
                    limite,
                    color,
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
    // [FIX] Mismo motivo que en ejecutar_extra_en_hilo: si hay Extra
    // (Normal/Turbo/Mantener/etc.), esta ejecución necesita un id
    // ÚNICO POR EJECUCIÓN para poder detenerse — generado ACÁ,
    // síncrono, ANTES de spawnear el hilo, y encolado en GENERACIONES
    // para que detener(id) (el Up físico) sepa a qué ejecución real
    // apuntar.
    //
    // Antes esta función pasaba el id de FILA crudo a ejecutar_lineas(),
    // sin pasar nunca por nueva_id_ejecucion(). Esa ejecución nunca
    // quedaba anotada en GENERACIONES, así que cuando llegaba el Up y
    // Cache mandaba Detener{id}, detener() buscaba una cola vacía en
    // GENERACIONES para ese id de fila y cortaba en silencio sin tocar
    // INSTANCIAS — la orden de detener se perdía siempre. El loop de
    // REPETIR (Normal/Turbo) o el ESPERAR DETENER (Mantener) nunca se
    // enteraban de que había que parar y quedaban repitiendo para
    // siempre (bug: bucle de salida con Coordenada + Extra
    // Normal/Turbo). Sin Extra no pasaba nada de esto (rama None de
    // abajo no usa INSTANCIAS), y sin coordenada tampoco (ese camino
    // va por ejecutar_extra_en_hilo, que sí generaba el id a tiempo).
    let id_ejecucion = extra.as_ref().map(|_| nueva_id_ejecucion(&id));

    thread::spawn(move || {
        let origen = match coordenada.post_accion {
            PostAccionCache::Inicial => Some(back_coordenada::obtener_cursor()),
            PostAccionCache::Final => None,
        };

        let destino = back_coordenada::calcular_destino(&coordenada.ubicacion);
        back_coordenada::mover_cursor(destino.0, destino.1, &|| false);

        match extra {
            Some(extra) => {
                let lineas = runt_extra::obtener(&extra);
                let lineas = sustituir_accion(lineas, &accion);
                ejecutar_lineas(
                    id_ejecucion.expect("generado arriba cuando extra es Some"),
                    lineas,
                );
            }

            None => {
                if let AccionCache::Emitir(inputs, condicion) = &accion {
                    ejecutar_emitir(inputs, condicion);
                }
            }
        }

        if let Some((x, y)) = origen {
            back_coordenada::mover_cursor(x, y, &|| false);
        }
    });
}

// ======================================================
// 📂 ABRIR ARCHIVO
// ------------------------------------------------------
// Corre en su propio hilo (mismo motivo que ejecutar_macro_en_hilo:
// no bloquear el hilo de captura). Sin registrarse en INSTANCIAS —
// es de una sola vez, no hay Extra ni loop que Detener a mitad de
// camino.
//
// "Única": el proceso objetivo se conoce de antemano en varios casos
// — ruta es un .exe (objetivo = su propio nombre de archivo), hay
// abrir_con (objetivo = el nombre de archivo del programa
// alternativo), es un documento sin abrir_con (objetivo = el programa
// predeterminado de la extensión, resuelto por
// back_registro::programa_predeterminado() — mismo programa que
// abrir_archivo() va a lanzar más abajo si no encuentra nada), o cae
// al mapeo de apps UWP conocidas (proceso_uwp_conocido(), ej. la app
// Fotos para imágenes — mitigación del BUG 4, ver su comentario). Para
// .exe directo se usa back_app::enfocar_proceso() (solo nombre de
// proceso, no por PID: a diferencia del forzado de primer plano de
// más abajo, acá se busca un proceso que puede llevar rato corriendo,
// no uno que se acaba de lanzar). Para cualquier documento (con o sin
// abrir_con) se usa back_app::enfocar_documento(): además del nombre
// de proceso, exige que el TÍTULO de la ventana contenga el nombre de
// ESTE archivo puntual (ver BUG 6 más abajo) — sin eso, con el mismo
// programa ya abierto en OTRO archivo, se le hacía toggle a ese otro
// archivo y nunca se llegaba a abrir el pedido.
//
// Para .lnk (puede apuntar a cualquier destino, sin resolver por
// ahora) Única no tiene efecto y siempre se lanza de nuevo.
//
// CARPETA es un caso aparte: aunque el programa se conoce de
// antemano (siempre "explorer.exe"), matchear solo por nombre de
// proceso traería la primera ventana de Explorer que encuentre —
// podría ser la de OTRA carpeta ya abierta, no la que se pidió. Por
// eso usa back_app::enfocar_carpeta() (mismo mecanismo de título que
// enfocar_documento, especializado a "explorer.exe" + nombre de
// carpeta).
//
// PRIMER PLANO / MODO DE VENTANA: se lanza con ShellExecuteExW en vez
// de ShellExecuteW para obtener el HANDLE del proceso creado
// (SEE_MASK_NOCLOSEPROCESS) y de ahí su PID cuando lo hay. Ese PID NO
// siempre existe: un documento cuyo programa asociado ya estaba
// abierto puede recibir la apertura por DDE (hProcess llega nulo, sin
// proceso nuevo). Por eso el reforzado de después no depende solo del
// PID: se toma un snapshot de ventanas visibles ANTES de lanzar
// (back_app::listar_ventanas_visibles, siempre — también con
// Minimizado, ver BUG 7/8) y después se busca la ventana NUEVA que
// apareció (back_app::buscar_ventana_nueva — prioriza el PID si lo
// hay, si no cae a "la primera ventana nueva que sea"), reafirmándole
// el modo de ventana pedido con back_app::reafirmar_modo_ventana()
// (no solo el foco — también Minimizar/Maximizar explícito, para las
// apps que ignoran el nShow que se le pasó a ShellExecuteExW y se
// muestran igual una vez que terminan de cargar). El foco en sí usa
// AttachThreadInput + un toque de ALT simulado (back_app::forzar_foco),
// que sí tiene efecto garantizado a diferencia de un
// SetForegroundWindow suelto (Windows bloquea el robo de foco de
// ventanas recién creadas o ya visibles en 2º plano si el hilo que lo
// pide no está "pegado" al hilo en foco actual). CARPETA es un caso
// aparte (ver el `if Path::new(&ruta).is_dir()` de abajo):
// ShellExecuteExW "open" sobre una ruta de carpeta reusa la ventana de
// Explorer que ya esté corriendo (misma negociación por DDE que un
// doble clic normal) en vez de crear una nueva — ni PID ni ventana
// nueva garantizados, el snapshot-diff de arriba no tiene nada que
// detectar. Por eso ahí se lanza "explorer.exe" como programa con la
// ruta de parámetro en vez del verbo "open" directo: fuerza proceso y
// ventana genuinamente nuevos, mismo camino confiable que ya funciona
// para .exe/abrir_con.
// ======================================================

// pub(crate) desde la Etapa 8B: runt_macro.rs reusa esta misma
// función para el paso "Abrir Archivo/Programa" — mismo criterio que
// AccionCache::AbrirArchivo, sin reimplementar nada de bajo nivel.
// Sigue spawneando su propio hilo interno (ver comentario largo más
// abajo) — llamarla desde dentro del hilo de una macro es seguro
// (no hay estado compartido con el hilo llamador más allá del propio
// spawn), aunque signifique un hilo "extra" anidado; no vale la pena
// duplicar la función solo para evitarlo.
pub(crate) fn abrir_archivo(
    ruta: String,
    iniciar: IniciarVentana,
    instancias: InstanciasAbrir,
    abrir_con: Option<String>,
    argumento: String,
) {
    // Calculado antes del hilo: instancias == Única lo necesita para
    // decidir la rama de matching (.exe / documento / carpeta), y la
    // rama de lanzamiento más abajo también lo necesita — un único
    // cálculo, sin repetirlo.
    let extension = extension_de(&ruta);

    thread::spawn(move || {
        if instancias == InstanciasAbrir::Unica {
            if Path::new(&ruta).is_dir() {
                // Carpeta: por nombre de proceso solo no alcanza
                // (cualquier ventana de Explorer matchearía) — ver
                // back_app::enfocar_carpeta().
                if back_app::enfocar_carpeta(&ruta) {
                    return;
                }
            } else if extension == "exe" {
                // .exe directo: no hay un archivo/documento que
                // matchear en el título, solo el proceso en sí.
                if let Some(nombre) = nombre_proceso_objetivo(&ruta, &abrir_con) {
                    if back_app::enfocar_proceso(&nombre) {
                        return;
                    }
                }
            } else if extension != "lnk" {
                // BUG 6: documento (propio o vía "Abrir con") — matchear
                // SOLO por proceso traía la ventana de CUALQUIER archivo
                // ya abierto en ese programa (ej. Notepad++ con otro
                // archivo distinto) y le hacía toggle, sin nunca llegar a
                // abrir el archivo pedido. Ahora exige además que el
                // título de la ventana contenga el nombre de este
                // archivo puntual — ver back_app::enfocar_documento().
                if let Some(nombre_proceso) = nombre_proceso_objetivo(&ruta, &abrir_con) {
                    if let Some(nombre_archivo) = nombre_archivo(&ruta) {
                        if back_app::enfocar_documento(&nombre_proceso, &nombre_archivo) {
                            return;
                        }
                    }
                }
            }
            // .lnk: puede apuntar a cualquier destino sin resolver por
            // ahora — Única no tiene efecto, cae a lanzar siempre.
        }

        let (archivo, parametros) = if extension == "exe" {
            (ruta.clone(), argumento)
        } else if extension == "lnk" {
            (ruta.clone(), String::new())
        } else if let Some(programa) = abrir_con {
            (programa, format!("\"{}\"", ruta))
        } else if Path::new(&ruta).is_dir() {
            // Carpeta: NO se usa ShellExecuteExW "open" sobre la ruta
            // sola — ese verbo le pide a Windows que reuse la ventana
            // de Explorer que ya esté corriendo (la misma negociación
            // por DDE que usa un doble clic normal), y en ese caso ni
            // siquiera aparece una ventana nueva para detectar más
            // abajo (a veces ni un PID nuevo). Lanzando "explorer.exe"
            // como programa, con la ruta de parámetro, se evita esa
            // reutilización: Windows crea una ventana (y proceso)
            // nuevos siempre, igual que ejecutarlo así desde
            // Ejecutar/cmd — mismo comportamiento confiable que ya
            // funciona para .exe/abrir_con.
            ("explorer.exe".to_string(), format!("\"{}\"", ruta))
        } else {
            // Documento sin abrir_con personalizado: en vez de
            // delegar en ShellExecuteExW "open" sobre la ruta sola
            // (que puede reutilizar una instancia ya corriendo vía
            // DDE, ignorando Instancias Múltiple y el modo de
            // ventana), se resuelve el programa predeterminado de la
            // extensión por registro (back_registro) y se lo lanza
            // directo con la ruta como parámetro — mismo patrón que
            // abrir_con. Si no se pudo resolver, cae al "open" de
            // siempre.
            match back_registro::programa_predeterminado(&extension) {
                Some(programa) => (programa, format!("\"{}\"", ruta)),
                None => (ruta.clone(), String::new()),
            }
        };

        // BUG 7 / BUG 8: antes, con Minimizado no se tomaba snapshot y
        // nunca se volvía a tocar la ventana nueva — asumía que
        // ShellExecuteExW con nShow=SW_SHOWMINIMIZED bastaba. Muchas
        // apps (frameworks Qt/Electron, y Lightburn en particular)
        // ignoran esa sugerencia y se muestran igual una vez que
        // cargan. Ahora se toma el snapshot siempre, y
        // forzar_primer_plano() reafirma el modo pedido (no solo el
        // foco) para los tres casos — ver back_app::reafirmar_modo_ventana.
        let snapshot_previo = back_app::listar_ventanas_visibles();

        let pid = ejecutar_shell_execute(&archivo, &parametros, mostrar_para_iniciar(&iniciar));

        forzar_primer_plano(pid, snapshot_previo, &iniciar);
    });
}

// ======================================================
// 🔝 FORZAR PRIMER PLANO (reintentos cortos + reafirmar modo)
// ------------------------------------------------------
// La ventana nueva puede tardar unos instantes en existir — se
// reintenta buscar_ventana_nueva() cada 100ms hasta encontrarla o
// agotar los intentos. `pid` puede ser None (ver ejecutar_shell_execute):
// pasa siempre igual, buscar_ventana_nueva() cae a "cualquier ventana
// nueva" en ese caso — necesario porque ShellExecuteExW no siempre
// entrega un PID (ej. carpeta: reusa el explorer.exe ya corriendo).
//
// BUG 7 / BUG 8: una vez encontrada la ventana, ya no solo se le
// fuerza el foco — back_app::reafirmar_modo_ventana() además reafirma
// el modo pedido (Minimizado/Maximizado/Ventana) con un ShowWindow
// explícito, para las apps que ignoran el nShow que se le pasó a
// ShellExecuteExW y se muestran igual una vez que terminan de cargar.
// ======================================================

fn forzar_primer_plano(
    pid: Option<u32>,
    snapshot_previo: back_app::VentanaSnapshot,
    iniciar: &IniciarVentana,
) {
    const INTENTOS: u32 = 20;
    const ESPERA: Duration = Duration::from_millis(100);

    for _ in 0..INTENTOS {
        if let Some(hwnd) = back_app::buscar_ventana_nueva(&snapshot_previo, pid) {
            back_app::reafirmar_modo_ventana(hwnd, iniciar);

            return;
        }

        thread::sleep(ESPERA);
    }
}

fn nombre_proceso_objetivo(ruta: &str, abrir_con: &Option<String>) -> Option<String> {
    if let Some(programa) = abrir_con {
        return nombre_archivo(programa);
    }

    let extension = extension_de(ruta);

    if extension == "exe" {
        return nombre_archivo(ruta);
    }

    if extension == "lnk" {
        // Un .lnk puede apuntar a cualquier cosa — resolver su
        // destino real queda fuera de alcance por ahora. Única no
        // tiene efecto para este caso (mismo comportamiento previo).
        return None;
    }

    // Documento: mismo programa predeterminado que abrir_archivo()
    // va a lanzar si no lo encuentra abierto (ver back_registro). Si
    // no se pudo resolver por registro, cae al mapeo de apps UWP
    // conocidas (ver proceso_uwp_conocido) — mitigación puntual del
    // BUG 4.
    back_registro::programa_predeterminado(&extension)
        .and_then(|programa| nombre_archivo(&programa))
        .or_else(|| proceso_uwp_conocido(&extension).map(str::to_string))
}

// ======================================================
// 🖼️ MITIGACIÓN BUG 4 — VISOR "FOTOS" (UWP)
// ------------------------------------------------------
// La app moderna "Fotos" de Windows es un paquete UWP: no se registra
// en HKCR\<ext>\shell\open\command como un programa.exe común (usa
// activación COM/DelegateExecute), así que
// back_registro::programa_predeterminado() no encuentra nada para
// extensiones de imagen y nombre_proceso_objetivo() quedaba sin poder
// intentar el toggle de Instancias Única — siempre abría una ventana
// nueva.
//
// Esto es una LIMITACIÓN REAL, no un bug corregible del todo: no hay
// un ejecutable puntual que lanzar ni una instancia controlable desde
// afuera con ShellExecuteExW para una app UWP. Como mitigación
// parcial, se hardcodea el nombre de proceso conocido de "Fotos"
// (Microsoft.Photos.exe) para extensiones de imagen comunes — esto
// permite que el TOGGLE de Única funcione (si "Fotos" ya está
// corriendo con esa imagen en el título, se enfoca/minimiza en vez de
// abrir otra). "Abrir nuevo" (primera vez, o si el toggle no
// encuentra coincidencia) sigue sin control fino: cae al "open" de
// siempre sobre la ruta, que Windows resuelve por su cuenta vía COM y
// siempre abre una ventana nueva — eso no tiene vuelta con las
// herramientas usadas acá.
// ======================================================

fn proceso_uwp_conocido(extension: &str) -> Option<&'static str> {
    match extension {
        "jpg" | "jpeg" | "png" | "bmp" | "gif" | "tif" | "tiff" | "webp" => {
            Some("Microsoft.Photos.exe")
        }

        _ => None,
    }
}

fn nombre_archivo(ruta: &str) -> Option<String> {
    Path::new(ruta)
        .file_name()
        .map(|nombre| nombre.to_string_lossy().to_string())
}

fn extension_de(ruta: &str) -> String {
    Path::new(ruta)
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn mostrar_para_iniciar(iniciar: &IniciarVentana) -> SHOW_WINDOW_CMD {
    match iniciar {
        IniciarVentana::Ventana => SW_SHOWNORMAL,
        IniciarVentana::Minimizado => SW_SHOWMINIMIZED,
        IniciarVentana::Maximizado => SW_SHOWMAXIMIZED,
    }
}

// ======================================================
// 🚀 EJECUTAR SHELLEXECUTEEXW
// ------------------------------------------------------
// Variante "Ex" de ShellExecuteW: misma operación ("open" sobre
// archivo+parámetros+modo de ventana), pero con SEE_MASK_NOCLOSEPROCESS
// deja en hProcess un HANDLE abierto al proceso recién creado —
// de ahí se saca el PID (GetProcessId) que necesita
// forzar_primer_plano() para encontrar su ventana. El handle se
// cierra acá mismo apenas se lee el PID (CloseHandle) — no hace
// falta mantenerlo abierto más que eso.
//
// Devuelve None si ShellExecuteExW falló (ruta inválida, sin
// permisos, etc.) o si por algún motivo no llegó a entregar un
// hProcess utilizable — en ese caso simplemente no hay forzado de
// primer plano, el resto de la apertura no se ve afectada.
//
// Tipos verificados contra el fuente real de windows-sys (0.59 y
// 0.60): fMask es u32 (SEE_MASK_NOCLOSEPROCESS también, sin cast) y
// nShow es i32 (de ahí el `mostrar as i32`, ya que SHOW_WINDOW_CMD es
// alias de i32). SHELLEXECUTEINFOW y ShellExecuteExW están detrás de
// `#[cfg(feature = "Win32_System_Registry")]` dentro del crate — ver
// Cargo.toml, esa feature se agregó puntualmente por esto.
// ======================================================

fn ejecutar_shell_execute(
    archivo: &str,
    parametros: &str,
    mostrar: SHOW_WINDOW_CMD,
) -> Option<u32> {
    let operacion: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();

    let archivo_ancho: Vec<u16> = archivo.encode_utf16().chain(std::iter::once(0)).collect();

    let parametros_ancho: Vec<u16> = parametros
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let parametros_ptr = if parametros.is_empty() {
        std::ptr::null()
    } else {
        parametros_ancho.as_ptr()
    };

    let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };

    info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    info.fMask = SEE_MASK_NOCLOSEPROCESS;
    info.lpVerb = operacion.as_ptr();
    info.lpFile = archivo_ancho.as_ptr();
    info.lpParameters = parametros_ptr;
    info.nShow = mostrar as i32;

    let exito = unsafe { ShellExecuteExW(&mut info) };

    if exito == 0 || info.hProcess.is_null() {
        return None;
    }

    let pid = unsafe { GetProcessId(info.hProcess) };

    unsafe {
        CloseHandle(info.hProcess);
    }

    if pid == 0 {
        None
    } else {
        Some(pid)
    }
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
    let evento = crate::eventos::InputEvent::pulse(input);

    emitir_evento(evento);
}

// ======================================================
// ⚡ EJECUTAR EMITIR (según condición: Simple/Doble/Triple/Mantenido)
// ------------------------------------------------------
// Único lugar que decide, para un Emitir sin Extra, cómo
// ejecutar el combo según la condición capturada en la
// Acción (ver perfil_cache.rs / compilador.rs):
// • Simple    → mods abajo, DOWN+espera tiempo_simple_teclas+UP
//               del gatillo, mods arriba.
// • Doble     → mods abajo UNA sola vez, el gatillo golpea 2
//               veces (delay_entre_salida_doble entre golpes),
//               mods arriba al final — no repite el combo
//               completo (los mods no vuelven a soltarse ni
//               apretarse entre golpes).
// • Triple    → igual que Doble, 3 golpes de gatillo.
// • Mantenido → solo el DOWN del combo, espera
//               config::tiempo_mantenido(), y recién ahí manda
//               el UP.
// Llamado tanto desde el hilo de salida dedicado
// (COLA_SALIDA) como desde Click en coordenada sin Extra.
// ======================================================

fn ejecutar_emitir(inputs: &[InputId], condicion: &CondicionTrigger) {
    let Some((gatillo, mods)) = inputs.split_last() else {
        return;
    };

    match condicion {
        CondicionTrigger::Simple => emitir_un_toque(mods, gatillo),

        CondicionTrigger::Doble => emitir_multiples_toques(mods, gatillo, 2),

        CondicionTrigger::Triple => emitir_multiples_toques(mods, gatillo, 3),

        CondicionTrigger::Mantenido => {
            emitir_combo_abajo(inputs);

            thread::sleep(Duration::from_millis(config::tiempo_mantenido()));

            emitir_combo_arriba(inputs);
        }
    }
}

// ======================================================
// 👆 EMITIR UN TOQUE (mods + un DOWN/UP de gatillo)
// ------------------------------------------------------
// DOWN mods → DOWN gatillo → espera tiempo_simple_teclas()
// → UP gatillo → UP mods en orden inverso.
// ======================================================

// ======================================================
// 🔁 EMITIR MODIFICADORES ABAJO (soporta duplicados)
// ------------------------------------------------------
// `mods` puede traer el mismo código repetido consecutivo
// (multi-tap aplanado por el grabador de macro, ver
// core_analisis_grabacion.ts: "1+2+2+2+2"). Un DOWN sobre una
// tecla que ya está abajo es un no-op físico — para que cada
// repetición produzca un toque real, cuando el código se repite
// se cierra primero con un UP y recién ahí se manda el DOWN de
// nuevo. pub(crate): usada también desde runt_macro.rs en las
// ramas Down/Mantenido de "Simular teclas".
// ======================================================

pub(crate) fn emitir_mods_abajo(mods: &[InputId]) {
    let mut anterior: Option<&InputId> = None;

    for modificador in mods {
        if anterior == Some(modificador) {
            emitir_up_input(modificador.clone());
            thread::sleep(Duration::from_millis(config::delay_entre_salida_doble()));
        }

        emitir_down_input(modificador.clone());
        anterior = Some(modificador);
    }
}

// pub(crate) desde la Etapa 8B: runt_macro.rs las usa directo para
// el paso "Simular teclas" (Simple/Doble/Triple ya cubiertos por
// estas dos; Mantenido y Normal/Turbo se arman a mano en
// runt_macro.rs con emitir_down_input/emitir_up_input/esperar, ver
// ahí). Antes privadas, hoy visibles en todo el crate — ninguna
// cambia de firma ni de comportamiento.
pub(crate) fn emitir_un_toque(mods: &[InputId], gatillo: &InputId) {
    emitir_mods_abajo(mods);

    emitir_down_input(gatillo.clone());

    thread::sleep(Duration::from_millis(config::tiempo_simple_teclas()));

    emitir_up_input(gatillo.clone());

    for modificador in mods.iter().rev() {
        emitir_up_input(modificador.clone());
    }
}

// ======================================================
// 👆👆 EMITIR MÚLTIPLES TOQUES (Doble/Triple)
// ------------------------------------------------------
// Mods abajo UNA sola vez, el gatillo se golpea n veces
// (tiempo_simple_teclas entre el DOWN y el UP de cada
// golpe, delay_entre_salida_doble entre golpes), mods
// arriba al final. Antes repetía el combo completo n
// veces (mods incluidos) — bug: "Q+Wx2" salía "qwqw" en
// vez de "qww".
// ======================================================

pub(crate) fn emitir_multiples_toques(mods: &[InputId], gatillo: &InputId, n: u8) {
    emitir_mods_abajo(mods);

    for indice in 0..n {
        if indice > 0 {
            thread::sleep(Duration::from_millis(config::delay_entre_salida_doble()));
        }

        emitir_down_input(gatillo.clone());

        thread::sleep(Duration::from_millis(config::tiempo_simple_teclas()));

        emitir_up_input(gatillo.clone());
    }

    for modificador in mods.iter().rev() {
        emitir_up_input(modificador.clone());
    }
}

// ======================================================
// 📤 EMITIR COMBO ABAJO/ARRIBA (mitades, para Mantenido)
// ------------------------------------------------------
// inputs = [mod1, mod2, ..., gatillo] (último = gatillo,
// ver perfil_cache.rs).
// ======================================================

/// Emite Ctrl+V por el mismo camino que un combo de Emitir real
/// (back_interception, nivel driver) — NO SendInput/WinAPI. Usado
/// por back_portapapeles::pegar() para el pegado automático tras
/// escribir al portapapeles: se comprobó que el SendInput de WinAPI,
/// aunque reportaba insertar los eventos sin objeción, no lograba
/// que Paint (UWP) reaccionara — mientras que un atajo de Menú
/// Express con el mismo Ctrl+V, emitido por este camino
/// (COLA_SALIDA → back_interception), sí funcionaba.
///
/// IMPORTANTE — encola en COLA_SALIDA en vez de llamar
/// emitir_combo() directo. Un atajo de Menú Express llama
/// runtime::ejecutar(OrdenRuntime::Iniciar { accion:
/// AccionCache::Emitir(...), .. }) — eso pasa por ejecutar_accion(),
/// que para Emitir SIEMPRE encola en COLA_SALIDA (ver comentario
/// largo ahí: "Emitir tampoco corre [en el hilo que llamó a
/// ejecutar()]... Por eso Emitir va a COLA_SALIDA: un canal + un
/// único hilo de salida de por vida"), nunca llama a emitir_combo()
/// desde el hilo que originó el trigger. Esta función replica
/// exactamente eso — encola, no ejecuta directo — para correr en el
/// mismo hilo dedicado que cualquier Emitir real, sin ninguna
/// diferencia de contexto entre un atajo de Menú Express y este
/// pegado automático.
///
/// Los nombres "LeftControl" y "V" son la columna "interno" de
/// pulsadores.tsv (mismo criterio que compilador.rs::convertir_input,
/// que arma los InputId de un Emitir configurado por el usuario a
/// partir de esa misma columna vía perfil.json) — no la columna
/// "interception" que usa resolver_input() más abajo en este archivo
/// para las líneas DOWN/UP/pulse del Idioma Runtime, que es un
/// camino distinto.
pub fn emitir_ctrl_v() {
    let inputs = vec![
        InputId::new("keyboard", "LeftControl"),
        InputId::new("keyboard", "V"),
    ];

    let _ = COLA_SALIDA
        .lock()
        .unwrap()
        .send((inputs, CondicionTrigger::Simple));
}

/// Misma emisión que emitir_ctrl_v() (mismo camino
/// back_interception vía ejecutar_emitir, ver comentario largo de
/// arriba), pero SÍNCRONA — corre ejecutar_emitir() directo en el
/// hilo del llamador en vez de encolar en COLA_SALIDA, y por lo
/// tanto BLOQUEA hasta que el combo terminó de emitirse (incluye el
/// sleep de tiempo_simple_teclas() entre DOWN y UP del gatillo, ver
/// emitir_un_toque()).
///
/// Usada por back_portapapeles::pegar() cuando quien pega es un paso
/// de Macro (runt_macro.rs::ejecutar_paso_pegar) — a diferencia del
/// pegado automático original (una tecla remapeada dispara UN
/// pegar() y no hay nada más compitiendo por el portapapeles
/// después), una Macro puede encadenar varios pasos "Pegar" seguidos
/// sin pausa. emitir_ctrl_v() normal solo ENCOLA y retorna al
/// instante — el paso siguiente de la Macro (que ya corre en su
/// propio hilo dedicado, ejecutar_macro_completa, nunca el hilo de
/// captura del sistema) alcanzaba a sobreescribir el portapapeles
/// con el SIGUIENTE texto antes de que el hilo consumidor de
/// COLA_SALIDA llegara a procesar el Ctrl+V del paso anterior — de
/// ahí el bug reportado ("Pegar 1", "Pegar 2" pegaba "22"; con 5
/// pasos pegaba "23455", el consumidor siempre iba unos pasos atrás
/// de las escrituras). Bloquear acá hasta que el Ctrl+V realmente se
/// emitió es seguro porque el runtime de macro ya corre aislado en
/// su propio hilo — no congela el hilo de captura de eventos del
/// sistema, que es el único motivo por el que Emitir usa el canal en
/// el resto de los casos.
pub fn emitir_ctrl_v_bloqueante() {
    let inputs = vec![
        InputId::new("keyboard", "LeftControl"),
        InputId::new("keyboard", "V"),
    ];

    ejecutar_emitir(&inputs, &CondicionTrigger::Simple);
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

    emitir_mods_abajo(mods);

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

// ======================================================
// 🛟 SALIDAS ABAJO (Etapa 8C — red de seguridad global)
// ------------------------------------------------------
// Qué salidas están físicamente abajo AHORA MISMO por acción del
// motor. Se llena/vacía en los mismos dos puntos donde ya se emite
// un Down/Up real (emitir_down_input/emitir_up_input, ver abajo) —
// así cualquier camino de salida que exista hoy o se agregue después
// (Emitir, runt_extra, runt_macro) queda cubierto automáticamente,
// sin tener que tocarlo aparte. Usado únicamente por
// detener_todo()/soltar_salidas_pendientes() más abajo.
// ======================================================

static SALIDAS_ABAJO: std::sync::LazyLock<Mutex<HashSet<InputId>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashSet::new()));

// pub(crate) desde la Etapa 8B — ver nota en emitir_un_toque() más
// arriba. Desde la Etapa 8C también registra la salida en
// SALIDAS_ABAJO (ver arriba) — insert() ANTES de emitir_evento(),
// para que quede registrada incluso si el evento tarda en salir.
pub(crate) fn emitir_down_input(input: InputId) {
    SALIDAS_ABAJO.lock().unwrap().insert(input.clone());

    let evento = crate::eventos::InputEvent::down(input);

    emitir_evento(evento);
}

// Desde la Etapa 8C: saca la salida de SALIDAS_ABAJO — un Up real,
// sea el normal del flujo o el forzado por detener_todo(), siempre
// pasa por acá.
pub(crate) fn emitir_up_input(input: InputId) {
    SALIDAS_ABAJO.lock().unwrap().remove(&input);

    let evento = crate::eventos::InputEvent::up(input);

    emitir_evento(evento);
}

// ======================================================
// ⏱️ ESPERAR
// ======================================================

// pub(crate) desde la Etapa 8B: runt_macro.rs la usa para el paso
// "Tiempo de espera" y para las duraciones fijas de "Simular teclas"
// (Mantenido con Extra Ninguno, Normal/Turbo) — ver comentario largo
// de espera interrumpible en runt_macro.rs, que NO usa esta función
// para esos puntos (usa su propio mecanismo interrumpible) pero sí
// para cualquier espera fija que no necesite cortarse a mitad de
// camino.
pub(crate) fn esperar(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

// ======================================================
// ⏸️ ESPERAR DETENER
// ------------------------------------------------------
// Bloquea el hilo de la instancia hasta que llegue la
// orden de detener (detener(id)) — usado por Mantener y
// Click Sostenido para no soltar la acción hasta que el
// físico se suelte (Cache es quien manda el Detener en
// ese momento).
//
// 100% reactivo, sin sondeo: el hilo se bloquea en
// INSTANCIAS_CONDVAR.wait() (espera real a nivel de SO,
// cero CPU, cero despertares periódicos) hasta que
// detener_ejecucion() o detener_todas_las_instancias()
// escriban `true` para este id y llamen notify_all(). Al
// despertar, vuelve a mirar su propia entrada — si la
// notificación era para otro id, sigue esperando sin
// gastar ninguna vuelta de más. No hace falta timeout de
// respaldo (a diferencia de runt_macro::esperar_interrumpible):
// el Condvar está acoplado al mismo Mutex que guarda la
// bandera, así que no hay ninguna ventana de carrera entre
// leerla y dormirse.
// ======================================================

fn esperar_detener(id: &str) {
    let mut guard = INSTANCIAS.lock().unwrap();

    while guard.get(id).copied() == Some(false) {
        guard = INSTANCIAS_CONDVAR.wait(guard).unwrap();
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
    // Generado ACÁ, síncrono, antes de spawnear — así, si Cache manda
    // Iniciar+Detener juntos (extras que no requiere_up_real), la
    // generación ya está encolada en GENERACIONES cuando el Detener
    // llega, sin importar cuándo el hilo nuevo alcance a arrancar.
    let id_ejecucion = nueva_id_ejecucion(&id);

    thread::spawn(move || {
        let lineas = runt_extra::obtener(&extra);

        let lineas = sustituir_accion(lineas, &accion);

        ejecutar_lineas(id_ejecucion, lineas);
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
// Un "toque" es un DOWN+UP del gatillo. La condición decide
// cuántos toques hacen falta antes de la mitad final:
// • Simple           → 1 toque, con mods incluidos (down antes,
//                       up después, todo en el mismo toque).
// • Doble            → 2 toques de gatillo, mods abajo UNA sola
//                       vez antes del primero y arriba UNA sola
//                       vez después del último — no se sueltan
//                       entre toque y toque.
// • Triple           → igual que Doble, 3 toques de gatillo.
// • Mantenido        → no tiene "toques": es DOWN, sostenido, UP.
// Todo DOWN/UP de gatillo espera tiempo_simple_teclas() entre
// medio; entre toques de Doble/Triple se espera
// delay_entre_salida_doble().
//
// [ACCION] (unidad completa, la usan Normal/Turbo en su bucle):
// para Mantenido, DOWN + espera tiempo_mantenido() + UP
// (sostenido artificial, igual que ejecutar_emitir sin Extra).
//
// [ACCION_DOWN]/[ACCION_UP] (mitades separadas, las usan Mantener/
// Click Sostenido para sostener hasta el Up físico real):
// [ACCION_DOWN] manda mods abajo una sola vez, todos los toques
// de gatillo salvo el último completos, y deja el último toque
// solo con su DOWN (sostenido) — ej. Triple: DOWN mods, toque,
// espera, toque, espera, DOWN gatillo. [ACCION_UP] siempre es
// solamente la mitad de arriba (UP gatillo + UP mods en reversa):
// no importa la condición, lo único que queda pendiente cuando
// llega el Up físico real es soltar ese último DOWN sostenido.
// Para Mantenido, [ACCION_DOWN] es solo DOWN mods+gatillo — el
// sostenido real que pide el Extra (ESPERAR DETENER) reemplaza
// cualquier espera fija acá.
// ======================================================

fn lineas_un_toque(mods: &[&str], gatillo: &str) -> Vec<String> {
    let mut pasos = lineas_abajo(mods, gatillo);

    pasos.push(format!("ESPERAR {}", config::tiempo_simple_teclas()));

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

        CondicionTrigger::Doble => lineas_multiples_toques_gatillo(mods, gatillo, 2),

        CondicionTrigger::Triple => lineas_multiples_toques_gatillo(mods, gatillo, 3),

        CondicionTrigger::Mantenido => {
            let mut pasos = lineas_abajo(mods, gatillo);

            pasos.push(format!("ESPERAR {}", config::tiempo_mantenido()));

            pasos.extend(lineas_arriba(mods, gatillo));

            pasos
        }
    }
}

// Mods abajo una sola vez, el gatillo golpea n veces
// (tiempo_simple_teclas entre el DOWN y el UP de cada golpe,
// delay_entre_salida_doble entre golpes), mods arriba al
// final. Antes repetía lineas_un_toque n veces completo (mods
// incluidos) — bug: "Q+Wx2" salía "qwqw" en vez de "qww".
fn lineas_multiples_toques_gatillo(mods: &[&str], gatillo: &str, n: u8) -> Vec<String> {
    let mut pasos = lineas_down_mods(mods);

    for indice in 0..n {
        if indice > 0 {
            pasos.push(format!("ESPERAR {}", config::delay_entre_salida_doble()));
        }

        pasos.push(format!("DOWN {gatillo}"));

        pasos.push(format!("ESPERAR {}", config::tiempo_simple_teclas()));

        pasos.push(format!("UP {gatillo}"));
    }

    pasos.extend(lineas_up_mods(mods));

    pasos
}

fn lineas_accion_down(mods: &[&str], gatillo: &str, condicion: &CondicionTrigger) -> Vec<String> {
    let toques_previos = match condicion {
        CondicionTrigger::Simple | CondicionTrigger::Mantenido => 0,

        CondicionTrigger::Doble => 1,

        CondicionTrigger::Triple => 2,
    };

    let mut pasos = lineas_down_mods(mods);

    for _ in 0..toques_previos {
        pasos.push(format!("DOWN {gatillo}"));

        pasos.push(format!("ESPERAR {}", config::tiempo_simple_teclas()));

        pasos.push(format!("UP {gatillo}"));

        pasos.push(format!("ESPERAR {}", config::delay_entre_salida_doble()));
    }

    pasos.push(format!("DOWN {gatillo}"));

    pasos
}

fn lineas_accion_up(mods: &[&str], gatillo: &str) -> Vec<String> {
    lineas_arriba(mods, gatillo)
}

fn lineas_down_mods(mods: &[&str]) -> Vec<String> {
    mods.iter()
        .map(|modificador| format!("DOWN {modificador}"))
        .collect()
}

fn lineas_up_mods(mods: &[&str]) -> Vec<String> {
    mods.iter()
        .rev()
        .map(|modificador| format!("UP {modificador}"))
        .collect()
}

fn lineas_abajo(mods: &[&str], gatillo: &str) -> Vec<String> {
    let mut pasos = lineas_down_mods(mods);

    pasos.push(format!("DOWN {gatillo}"));

    pasos
}

fn lineas_arriba(mods: &[&str], gatillo: &str) -> Vec<String> {
    let mut pasos = vec![format!("UP {gatillo}")];

    pasos.extend(lineas_up_mods(mods));

    pasos
}

// ======================================================
// ⏹️ DETENER
// ======================================================

// ------------------------------------------------------
// `id` acá SIEMPRE es el id de fila (fijo) — el único que
// Cache conoce (ver OrdenRuntime::Detener). Se traduce a la
// ejecución real vía GENERACIONES (ver comentario ahí
// arriba): saca la generación más vieja todavía en cola para
// esta fila y marca esa ejecución puntual como detenida. Si
// no hay ninguna en cola (fila sin Extra/Macro corriendo,
// ej. MenuExpress/Portapapeles, o llegó un Detener de más),
// no hace nada — igual que antes.
// ------------------------------------------------------
fn detener(id: String) {
    let generacion = GENERACIONES
        .lock()
        .unwrap()
        .get_mut(&id)
        .and_then(|cola| cola.pop_front());

    let Some(generacion) = generacion else {
        return;
    };

    detener_ejecucion(&format!("{id}#{generacion}"));
}

/// Marca como detenida una ejecución puntual, ya identificada por su
/// id combinado ("idFila#generación"). A diferencia de detener(id) de
/// arriba (que recibe el id de FILA y necesita traducirlo vía
/// GENERACIONES), esto lo usa quien YA tiene el id de ejecución en la
/// mano — hoy, el propio comando "DETENER" dentro de una receta (ver
/// ejecutar_linea) corre dentro de su propia ejecución y se refiere a
/// sí mismo, no a la fila en general.
///
/// [FIX] Antes usaba `get_mut()`, que no hace nada si la clave
/// todavía no existe en INSTANCIAS. Pero para un trigger diferido
/// (Normal/Turbo/Mantenido/ClickSostenido) cuyo Iniciar+Detener
/// llegan pegados — sin demora real entre uno y otro, típicamente un
/// tap rápido sobre un trigger ambiguo con uno más largo (ver
/// cache.rs) resuelto retroactivamente por el Up — hay una carrera
/// real: nueva_id_ejecucion() encola la generación en GENERACIONES
/// de forma síncrona ANTES de spawnear el hilo, así que detener(id)
/// SIEMPRE encuentra a qué ejecución apunta, pero el hilo recién
/// spawneado puede no haber llegado todavía a ejecutar_lineas()
/// (que es quien recién ahí inserta la entrada en INSTANCIAS) para
/// cuando este Detener llega. Con get_mut(), esa orden se perdía en
/// silencio: el hilo, al arrancar más tarde, insertaba `false` sin
/// saber que ya le habían pedido pararse, y corría para siempre sin
/// recibir nunca la señal (bug: "1" en bucle infinito que no se
/// detiene al soltar; o Mantenido que nunca llega a mandar el Up).
/// Ahora se PRE-REGISTRA la orden con insert() (crea la entrada si
/// todavía no existía) — y ejecutar_lineas(), más abajo, usa
/// entry().or_insert(false) en vez de insert() a secas, para no
/// pisar un `true` que ya haya llegado antes de que el hilo
/// arrancara.
fn detener_ejecucion(id_ejecucion: &str) {
    INSTANCIAS
        .lock()
        .unwrap()
        .insert(id_ejecucion.to_string(), true);

    // El lock ya se soltó (el guard de arriba era temporal, murió al
    // terminar la sentencia) — notify_all() se llama sin el lock
    // tomado, cualquier hilo despierto en esperar_detener() vuelve a
    // pedirlo él mismo dentro de wait().
    INSTANCIAS_CONDVAR.notify_all();
}

// ======================================================
// ❓ DEBE DETENERSE
// ------------------------------------------------------
// Si el id ya no está registrado (nunca existió, o ya se
// limpió), se considera que debe detenerse — nunca se
// entra a un ciclo sin garantía de poder pararlo.
//
// pub(crate) desde la Etapa 8B: runt_macro.rs la consulta antes de
// cada paso (esté esperando o no), mismo criterio que
// ejecutar_lineas() antes de cada REPETIR.
// ======================================================

pub(crate) fn debe_detenerse(id: &str) -> bool {
    INSTANCIAS.lock().unwrap().get(id).copied().unwrap_or(true)
}

// ======================================================
// 🛑 DETENER TODO (Etapa 8C — red de seguridad global)
// ------------------------------------------------------
// Llamada por perfil.rs (junto a cada punto donde ya se llama
// cache::borrar_cache() — activar/desactivar/guardar/clonar/
// renombrar/eliminar/crear/seleccionar perfil) y por lib.rs (al
// cierre del programa). Corta cualquier cosa en ejecución y suelta
// cualquier tecla que haya quedado físicamente abajo por culpa del
// motor, en este orden:
//
// 1) Marca como detenida cualquier ejecución activa del mecanismo
//    general (INSTANCIAS) — Turbo/Normal/Mantener/ClickSostenido
//    (vía runt_extra) y Macro con Comportamiento Tecla mantenida
//    (Etapa 8B, que se anota acá mismo).
// 2) Limpia el registro propio de runt_macro.rs de ejecuciones Una
//    ejecución/Toggle (Etapa 8B) — independiente de INSTANCIAS.
// 3) Despierta el mecanismo de espera interrumpible de runt_macro.rs
//    (Etapa 8B) — para que cualquier hilo de Macro dormido (con
//    cualquiera de los tres Comportamientos) se entere YA de las
//    banderas puestas en 1) y 2), en vez de esperar a que venza su
//    propio plazo.
// 4) Recién al final, suelta (Up real) cualquier salida que haya
//    quedado pendiente en SALIDAS_ABAJO — después de 1-3, para darle
//    a cada hilo la chance de soltar sus propias teclas de forma
//    ordenada antes de forzar lo que haya quedado.
// ======================================================

pub fn detener_todo() {
    detener_todas_las_instancias();

    crate::runt_macro::detener_todas_las_activas();

    crate::runt_macro::notificar_todas();

    soltar_salidas_pendientes();
}

fn detener_todas_las_instancias() {
    {
        let mut instancias = INSTANCIAS.lock().unwrap();

        for detenida in instancias.values_mut() {
            *detenida = true;
        }
    }

    INSTANCIAS_CONDVAR.notify_all();
}

fn soltar_salidas_pendientes() {
    // Se copia y vacía el registro con el lock tomado solo un
    // instante — emitir_up_input() (que vuelve a pedir el lock para
    // sacar cada una, ya redundante pero inofensivo) no debe correr
    // con SALIDAS_ABAJO todavía tomado.
    let pendientes: Vec<InputId> = SALIDAS_ABAJO.lock().unwrap().drain().collect();

    for input in pendientes {
        emitir_up_input(input);
    }
}

// ======================================================
// 📜 (Etapa 8B) El viejo ejecutor de macro de texto plano
//     (ejecutar_macro_en_hilo, basado en split_whitespace) se
//     eliminó acá — reemplazado por runt_macro.rs, que interpreta
//     el JSON de pasos y no depende de este intérprete de líneas.
//     ejecutar_lineas/ejecutar_linea NO se tocan: el resto de la
//     app los sigue usando para Turbo/Normal/Mantener vía
//     runt_extra (ver comentario en el header de este archivo).
// ======================================================

// ======================================================
// 📝 REGISTRAR INSTANCIA
// ------------------------------------------------------
// Extraída de ejecutar_lineas() en la Etapa 8B para que
// runt_macro.rs pueda registrar sus propias ejecuciones en
// INSTANCIAS con la misma protección — ver el comentario "[FIX]"
// original más abajo (ejecutar_lineas ahora llama a esta función en
// vez de hacer el insert a mano).
// ======================================================

pub(crate) fn registrar_instancia(id: &str) {
    INSTANCIAS
        .lock()
        .unwrap()
        .entry(id.to_string())
        .or_insert(false);
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
    // [FIX] Antes: `insert(id.clone(), false)` a secas — pisaba
    // siempre con `false`, incluso si detener_ejecucion() ya había
    // pre-registrado un `true` para este id mientras el hilo todavía
    // no había alcanzado a arrancar (carrera Iniciar+Detener
    // pegados — ver el comentario largo en detener_ejecucion()).
    // `entry().or_insert(false)` solo escribe `false` si la entrada
    // TODAVÍA NO existía; si ya existía (pre-registrada en `true`
    // por un Detener que llegó primero), la deja tal cual está. Ver
    // registrar_instancia() más arriba (extraída en la Etapa 8B).
    registrar_instancia(&id);

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
            // `id` acá ya es el id de ejecución (no el de fila) — se
            // marca directo, sin pasar por la traducción de
            // GENERACIONES (esa es solo para cuando el llamador
            // conoce el id de fila, ej. Cache).
            detener_ejecucion(id);
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
// ------------------------------------------------------
// pub(crate) desde la Etapa 8B: runt_macro.rs la llama al terminar
// una ejecución (natural o cortada), mismo criterio que
// ejecutar_lineas() al final de su loop.
// ======================================================

pub(crate) fn limpiar_instancia(id: String) {
    INSTANCIAS.lock().unwrap().remove(&id);
}

// ======================================================
// 🔎 RESOLVER INPUT
// ======================================================

fn resolver_input(interno: &str) -> Option<InputId> {
    // [FIX] Antes: `InputId::new(&pulsador.fuente, &pulsador.interception)`.
    // Esta función reconstruye un InputId a partir del nombre INTERNO
    // que viaja en las líneas del idioma Runtime (ver sustituir_accion:
    // "DOWN {gatillo}", donde gatillo = InputId::control(), siempre un
    // nombre interno). InputId::control() se usa después en
    // back_teclas::convertir_salida(), que espera recibir justamente
    // el nombre INTERNO ahí (llama a pulsadores::interno_a_interception()
    // para traducirlo recién en ese punto) — armar acá el InputId ya
    // con el nombre "interception" pisaba ese contrato: convertir_salida
    // volvía a tratarlo como si fuera interno, buscaba un pulsador con
    // ESE nombre en la columna interno (no existe) y descartaba el
    // evento en silencio.
    //
    // Para la mayoría de las teclas (letras, números, F1-F12,
    // modificadores) interno == interception, así que el bug quedaba
    // invisible — el lookup fallido "acertaba" igual por coincidencia
    // de nombres. Se notaba solo en las teclas donde difieren: los
    // símbolos propios del layout español (Ñ/SemiColon, ¡/Equals,
    // °/Grave, ´/Apostrophe, ç/BackSlash, +/RightBracket, etc.) —
    // exactamente las que se perdían con Extra Normal/Mantenido/Turbo
    // (que pasan por acá) pero no con Extra Simple (que emite el
    // InputId ya armado por el compilador, sin pasar por acá).
    //
    // `interno` ya es el nombre interno correcto (así lo busca
    // por_interno() ahí abajo) — no hace falta ir a buscar
    // pulsador.interception, alcanza con reusarlo directo.
    let pulsador = crate::pulsadores::por_interno(interno)?;

    Some(InputId::new(&pulsador.fuente, interno))
}

// ======================================================
// ⬇️ DOWN
// ======================================================

fn ejecutar_down(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    let evento = crate::eventos::InputEvent::down(input);

    emitir_evento(evento);
}

// ======================================================
// ⬆️ UP
// ======================================================

fn ejecutar_up(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    let evento = crate::eventos::InputEvent::up(input);

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
    motor::emitir_evento(evento);
}
