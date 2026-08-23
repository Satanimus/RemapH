// ======================================================
// 🖱️ Back_Interception
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Único punto de contacto con el driver Interception.
//
// ENTRADA: escucha teclado/mouse en un loop bloqueante,
// traduce cada Stroke físico a InputEvent (vía
// back_teclas/back_mouse) y se lo entrega a quien lo
// inició.
//
// SALIDA: recibe un InputEvent (da igual si es el evento
// original sin cambios, o una acción remapeada — el
// código es el mismo en los dos casos) y lo emite
// físicamente.
//
// No decide qué hacer con un evento. No conoce Runtime,
// Cache ni AnalizadorTrigger.
//
// EXCEPCIÓN puntual (fix bug "interfiere sin trigger en
// caché" / tecla no soportada que se comía el input):
// iniciar() recibe además un segundo callback,
// debe_tragar_no_traducible, que le inyecta quien lo llama
// (lib.rs, con analizador_trigger::captura_activa). Sigue sin
// importar AnalizadorTrigger directamente — solo recibe un
// predicado genérico Fn() -> bool — pero es la única función
// de este archivo cuyo comportamiento depende de algo ajeno a
// Interception/back_teclas. Motivo: antes, un Stroke que
// back_teclas::convertir() no reconocía (ScanCode no listado
// en pulsadores.tsv) se perdía en silencio — ni se procesaba
// NI se reenviaba a Windows. Eso interfería con el sistema (la
// tecla quedaba muda) incluso sin trigger activo ni caché
// compilada. Ahora, si no se puede traducir, se reenvía el
// Stroke crudo tal cual, salvo que haya una Captura en curso
// (ahí sí se traga, para no ensuciar lo que se está grabando).
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// lib.rs llama precargar_desde_config() una sola vez al
//     arrancar, ANTES de spawnear el hilo de iniciar() (ver
//     comentario en "Reglas de dispositivo primario" y en
//     precargar_desde_config() más abajo). Después llama
//     iniciar() una sola vez, pasándole
//     entrada::procesar_evento y
//     analizador_trigger::captura_activa.
// entrada.rs llama emitir_evento() para devolver un
// evento sin consumir. Runtime (más adelante) llama
// emitir_evento() para ejecutar una acción remapeada,
// desde el hilo propio de cada instancia activa (por id).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// iniciar(procesar, debe_tragar_no_traducible): un callback
// FnMut(InputEvent) llamado una vez por cada evento traducido,
// y un predicado Fn() -> bool que decide si un Stroke NO
// traducible se traga (true) o se reenvía crudo a Windows
// (false).
// emitir_evento(evento): un InputEvent completo.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// iniciar() no retorna — bloquea para siempre.
// emitir_evento() no retorna nada.
// ------------------------------------------------------
// Reglas de dispositivo primario:
//
// La sesión de ENTRADA y la de SALIDA son sesiones de
// Interception separadas (la de entrada vive atrapada en
// el loop bloqueante; la de salida se crea sola, perezosa,
// la primera vez que hace falta emitir algo).
//
// Como son sesiones distintas, la de salida no sabe a
// qué número de dispositivo mandar. Se resuelve así:
// la primera vez que iniciar() recibe un evento real de
// teclado, guarda ese número de dispositivo como
// "teclado primario" — mismo criterio para "mouse
// primario". Emitir_evento() usa esos valores guardados.
// Quedan fijos el resto de la sesión (no se vuelve a
// tocar una vez "confirmados" por ese primer evento real,
// ver DispositivoPrimario más abajo). Asume 1.0: un solo
// teclado y un solo mouse conectados.
//
// [FIX] Bug "trigger de mouse no genera salida de teclado
// hasta tocar una tecla": con la regla de arriba sola, el
// teclado primario queda en None hasta el primer evento de
// teclado real — un trigger de mouse que dispara una salida
// de TECLADO (ej. Emitir letra desde un botón lateral) antes
// de que el usuario haya tocado el teclado físico ni una vez
// se perdía en silencio (emitir_teclado() corta en el primer
// `let Some(device) = teclado_primario() else { return }`).
//
// Ahora lib.rs llama precargar_desde_config() ANTES de
// spawnear el hilo de iniciar() — carga el número guardado la
// sesión anterior (configuracion_usuario::leer_dispositivo_
// teclado/mouse, ver ese archivo) como valor INICIAL, pero SIN
// marcarlo "confirmado". Con eso alcanza para tapar el hueco:
// emitir_teclado()/emitir_mouse() ya tienen un `device` usable
// desde el primer instante, incluso sin haber recibido ningún
// evento físico todavía.
//
// El valor precargado es un punto de partida, no un techo: el
// primer evento físico real de la sesión SIGUE ganando siempre
// (por si el número cambió entre sesiones — Interception puede
// reordenar los IDs si el dispositivo se reconectó en otro
// puerto, por ejemplo). Recién en ESE momento se marca
// "confirmado" (se deja de tocar el resto de la sesión, igual
// que antes) y, si el número resultó distinto del precargado,
// se persiste el nuevo — una sola escritura reactiva por
// sesión, nunca un guardado periódico. Ver registrar_teclado()/
// registrar_mouse() más abajo.
//
// Regla de threading para la sesión de salida (decisión
// de diseño, no migrar sin volver a leer esto):
// Runtime puede tener varias instancias activas a la vez,
// cada una en su propio hilo (ej: una macro de fondo +
// un atajo disparado en simultáneo) — no hay un único
// hilo fijo de Runtime. El crate interception marca su
// tipo Interception como !Send y !Sync (no se puede
// compartir entre hilos), así que la sesión de salida es
// thread_local: cada hilo que llega a emitir_evento() por
// primera vez crea y guarda su propia sesión, sin
// compartirla con los demás hilos. Asume que el driver
// permite varias sesiones de salida abiertas al mismo
// tiempo — es lo primero a verificar si algo falla acá en
// la práctica.
//
// Reglas de Pulse en salida (decisión de diseño, no
// migrar sin volver a leer esto):
// - Rueda de mouse → un solo envío, el estado del evento
//   (Down/Up/Pulse) no importa, la rueda no tiene ciclo.
// - Tecla o botón → Down manda un solo stroke (down), Up
//   manda un solo stroke (up), Pulse manda los dos
//   strokes seguidos sin delay artificial entre medio.
// ------------------------------------------------------
// 5. Funciones del archivo
// precargar_desde_config()
//     Llamada por lib.rs, una sola vez, antes de spawnear
//     iniciar(). Carga el teclado/mouse primario guardado la
//     sesión anterior como valor inicial (sin confirmar) —
//     ver "Reglas de dispositivo primario" arriba.
// activar_filtros() / desactivar_filtros()
//     La sesión de Interception para ENTRADA (SESION_ENTRADA)
//     se crea UNA sola vez por vida del proceso (perezosa,
//     thread_local, nunca se destruye entre transiciones de
//     modo — ver comentario junto a SESION_ENTRADA).
//     activar_filtros()
//     prende la captura de teclado/mouse al entrar a
//     Interception; desactivar_filtros() la apaga (filtro
//     vacío) al volver a Portable, sin tocar el contexto.
// recibir()
//     Bloquea hasta el próximo evento, devuelve el
//     Stroke crudo junto a su Device de origen.
// traducir()
//     Stroke → Option<InputEvent>, delegando en
//     back_teclas/back_mouse según el tipo.
// iniciar()
//     Loop principal de ENTRADA. Arranca la sesión,
//     registra el dispositivo primario de cada tipo la
//     primera vez que aparece, y llama al callback con
//     cada evento traducido. Si un Stroke NO se puede
//     traducir, lo reenvía crudo a Windows tal cual —
//     salvo que debe_tragar_no_traducible() diga true
//     (captura en curso), en cuyo caso se traga.
// con_sesion_salida()
//     Da acceso a la sesión de Interception de SALIDA
//     del hilo actual, creándola la primera vez que ese
//     hilo la necesita (thread_local, ver regla arriba).
// emitir_evento()
//     InputEvent → Stroke(s) físicos reales, enviados
//     por la sesión de salida del hilo actual al
//     dispositivo primario correspondiente.
// ------------------------------------------------------
// Transformación:
//
// ENTRADA:
// Stroke físico (Interception)
//     ↓
// back_teclas / back_mouse . convertir()
//     ↓
// InputEvent
//     ↓
// callback(evento)   [quien llamó a iniciar()]
//
// SALIDA:
// InputEvent
//     ↓
// back_teclas / back_mouse . convertir_salida()
//     ↓
// Stroke físico (Interception, sesión de salida
// thread_local del hilo que emite)
//     ↓
// dispositivo primario (teclado o mouse)
// ======================================================

use crate::back_mouse::{self, MouseOutput};
use crate::back_teclas;
use crate::eventos::{InputEvent, InputState};
use interception::{
    Device, Filter, Interception, KeyFilter, KeyState, MouseFilter, ScanCode, Stroke,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

// ======================================================
// 🆔 DISPOSITIVOS PRIMARIOS
// ------------------------------------------------------
// `confirmado` distingue un valor precargado desde disco
// (posible punto de partida, todavía puede pisarse) de uno
// confirmado por un evento físico real (fijo el resto de la
// sesión) — ver "Reglas de dispositivo primario" en el
// header de este archivo.
// ======================================================

struct DispositivoPrimario {
    valor: Option<Device>,
    confirmado: bool,
}

impl DispositivoPrimario {
    const fn vacio() -> Self {
        Self {
            valor: None,
            confirmado: false,
        }
    }
}

static TECLADO_PRIMARIO: OnceLock<Mutex<DispositivoPrimario>> = OnceLock::new();
static MOUSE_PRIMARIO: OnceLock<Mutex<DispositivoPrimario>> = OnceLock::new();

// ======================================================
// 💾 PRECARGAR DESDE CONFIG
// ------------------------------------------------------
// Ver comentario largo en el header ("Reglas de dispositivo
// primario"). Sin confirmar todavía — un evento físico real
// sigue pudiendo pisar este valor (registrar_teclado/
// registrar_mouse más abajo).
// ======================================================

pub fn precargar_desde_config() {
    if let Ok(Some(device)) = crate::configuracion_usuario::leer_dispositivo_teclado() {
        TECLADO_PRIMARIO
            .get_or_init(|| Mutex::new(DispositivoPrimario::vacio()))
            .lock()
            .unwrap()
            .valor = Some(device);
    }

    if let Ok(Some(device)) = crate::configuracion_usuario::leer_dispositivo_mouse() {
        MOUSE_PRIMARIO
            .get_or_init(|| Mutex::new(DispositivoPrimario::vacio()))
            .lock()
            .unwrap()
            .valor = Some(device);
    }
}

fn registrar_teclado(device: Device) {
    let mutex = TECLADO_PRIMARIO.get_or_init(|| Mutex::new(DispositivoPrimario::vacio()));
    let mut guardia = mutex.lock().unwrap();

    if guardia.confirmado {
        return;
    }

    let valor_previo = guardia.valor;

    guardia.valor = Some(device);
    guardia.confirmado = true;

    drop(guardia);

    if valor_previo != Some(device) {
        if let Err(error) = crate::configuracion_usuario::guardar_dispositivo_teclado(device) {
            eprintln!("⚠️ No se pudo guardar el teclado primario: {}", error);
        }
    }
}

fn registrar_mouse(device: Device) {
    let mutex = MOUSE_PRIMARIO.get_or_init(|| Mutex::new(DispositivoPrimario::vacio()));
    let mut guardia = mutex.lock().unwrap();

    if guardia.confirmado {
        return;
    }

    let valor_previo = guardia.valor;

    guardia.valor = Some(device);
    guardia.confirmado = true;

    drop(guardia);

    if valor_previo != Some(device) {
        if let Err(error) = crate::configuracion_usuario::guardar_dispositivo_mouse(device) {
            eprintln!("⚠️ No se pudo guardar el mouse primario: {}", error);
        }
    }
}

fn teclado_primario() -> Option<Device> {
    TECLADO_PRIMARIO
        .get_or_init(|| Mutex::new(DispositivoPrimario::vacio()))
        .lock()
        .unwrap()
        .valor
}

fn mouse_primario() -> Option<Device> {
    MOUSE_PRIMARIO
        .get_or_init(|| Mutex::new(DispositivoPrimario::vacio()))
        .lock()
        .unwrap()
        .valor
}

// ======================================================
// 🚀 SESIÓN DE ENTRADA (thread_local, creada UNA sola vez)
// ------------------------------------------------------
// [FIX] Bug "Portable→Interception deja teclado/mouse
// bloqueados (solo se mueve el puntero)": antes, crear()
// llamaba Interception::new() de nuevo en cada transición
// Portable→Interception, y el contexto anterior se cerraba
// (Drop) recién un instante antes al salir de Portable. El
// driver Interception queda en un estado roto al crear un
// segundo contexto tan pronto después de destruir el
// primero dentro del mismo proceso — no lanza error
// (Interception::new() no falla), pero el wait()/receive()
// de la sesión nueva deja de traer eventos aunque el filtro
// sí sigue capturando a nivel driver (por eso el teclado/
// clics quedan mudos pero el puntero se sigue moviendo: el
// mouse MOVE nunca estuvo en el filtro).
//
// Fix: el contexto de entrada se crea UNA sola vez por vida
// del proceso (perezoso, primera vez que se necesita) y
// nunca se destruye entre transiciones de modo. Lo que
// cambia entre Interception y Portable es el FILTRO, no el
// contexto: activar_filtros() prende la captura al entrar a
// Interception, desactivar_filtros() la apaga (filtro vacío
// = el driver deja pasar todo, tal como si no hubiera
// contexto) al volver a Portable. Mismo patrón que
// SESION_SALIDA más abajo — misma restricción !Send/!Sync,
// por eso thread_local en vez de estático simple.
// ======================================================

thread_local! {
    static SESION_ENTRADA: RefCell<Option<Interception>> = RefCell::new(None);
}

fn activar_filtros(ict: &Interception) {
    ict.set_filter(
        interception::is_keyboard,
        // E0/E1: sin esto, el driver descarta antes de llegar acá
        // cualquier tecla de scancode extendido — flechas, Insert,
        // Supr, Inicio/Fin, RePág/AvPág, el "/" del numpad, Pause,
        // etc. Las del numpad normal (0-9, *, +, -, .) no son
        // extendidas, por eso esas sí funcionaban sin este flag.
        Filter::KeyFilter(KeyFilter::DOWN | KeyFilter::UP | KeyFilter::E0 | KeyFilter::E1),
    );

    ict.set_filter(
        interception::is_mouse,
        Filter::MouseFilter(
            MouseFilter::LEFT_BUTTON_DOWN
                | MouseFilter::LEFT_BUTTON_UP
                | MouseFilter::RIGHT_BUTTON_DOWN
                | MouseFilter::RIGHT_BUTTON_UP
                | MouseFilter::MIDDLE_BUTTON_DOWN
                | MouseFilter::MIDDLE_BUTTON_UP
                | MouseFilter::BUTTON_4_DOWN
                | MouseFilter::BUTTON_4_UP
                | MouseFilter::BUTTON_5_DOWN
                | MouseFilter::BUTTON_5_UP
                | MouseFilter::WHEEL,
        ),
    );

    println!("📥 Backend de entrada iniciado.");
}

// Deja pasar todo sin capturar nada — se llama al salir hacia
// Portable, para que el contexto siga vivo (evita el
// Interception::new() repetido, ver comentario de arriba) sin
// seguir bloqueando teclado/mouse a nivel driver mientras
// Portable maneja la entrada con hooks WinAPI.
fn desactivar_filtros(ict: &Interception) {
    ict.set_filter(
        interception::is_keyboard,
        Filter::KeyFilter(KeyFilter::empty()),
    );
    ict.set_filter(
        interception::is_mouse,
        Filter::MouseFilter(MouseFilter::empty()),
    );
}

// ======================================================
// 📥 RECIBIR
// ======================================================

fn recibir(ict: &Interception) -> Option<(Device, Stroke)> {
    let device = ict.wait();

    let mut strokes = [Stroke::Mouse {
        state: MouseFilter::empty(),
        flags: interception::MouseFlags::empty(),
        rolling: 0,
        x: 0,
        y: 0,
        information: 0,
    }];

    if ict.receive(device, &mut strokes) <= 0 {
        return None;
    }

    Some((device, strokes[0]))
}

// ======================================================
// 🔄 TRADUCIR
// ======================================================

fn traducir(stroke: &Stroke) -> Option<InputEvent> {
    match stroke {
        Stroke::Keyboard { code, state, .. } => {
            // OJO: NO comparar `*state == KeyState::DOWN` — state es un
            // bitflag y una tecla extendida (flechas, Insert/Supr,
            // etc.) SIEMPRE trae también el bit E0 prendido, así que
            // esa igualdad exacta daba falso incluso estando presionada
            // (todo E0 se leía como "Up", nunca como "Down", y el
            // evento se perdía por completo). Hay que preguntar por
            // cada bit por separado.
            let es_extendida = state.contains(KeyState::E0);
            let presionado = !state.contains(KeyState::UP);

            back_teclas::convertir(*code, es_extendida, presionado)
        }

        Stroke::Mouse { state, rolling, .. } => back_mouse::convertir(*state, *rolling),
    }
}

// ======================================================
// 🔁 INICIAR (loop de entrada)
// ======================================================

// ======================================================
// 🛑 DETENER
// ------------------------------------------------------
// Mecanismo de parada limpia para el cambio de modo en
// caliente (Etapa D). solicitar_detener() pone la bandera;
// el loop de iniciar() la revisa tras cada recibir() y
// sale limpiamente sin matar el hilo a la fuerza.
// La dirección Interception→Portable necesita este
// mecanismo reactivo (el loop solo despierta ante un
// evento físico real — ver Regla 10 del plan de Modo
// Portable). La dirección opuesta (Portable→Interception)
// es instantánea desde afuera vía back_windows::detener().
// ======================================================

static DETENER: AtomicBool = AtomicBool::new(false);

pub fn solicitar_detener() {
    DETENER.store(true, Ordering::SeqCst);
}

// ======================================================
// 🔁 INICIAR (loop de entrada)
// ======================================================

pub fn iniciar(mut procesar: impl FnMut(InputEvent), debe_tragar_no_traducible: impl Fn() -> bool) {
    SESION_ENTRADA.with(|celda| {
        let mut sesion = celda.borrow_mut();

        if sesion.is_none() {
            *sesion = Some(Interception::new().expect("No se pudo iniciar Interception (entrada)"));
        }

        let ict = sesion.as_ref().unwrap();

        activar_filtros(ict);
        iniciar_loop(ict, &mut procesar, &debe_tragar_no_traducible);
        desactivar_filtros(ict);
    });
}

fn iniciar_loop(
    ict: &Interception,
    procesar: &mut impl FnMut(InputEvent),
    debe_tragar_no_traducible: &impl Fn() -> bool,
) {
    loop {
        let Some((device, stroke)) = recibir(ict) else {
            continue;
        };

        if DETENER.swap(false, Ordering::SeqCst) {
            break;
        }

        match &stroke {
            Stroke::Keyboard { .. } => registrar_teclado(device),
            Stroke::Mouse { .. } => registrar_mouse(device),
        }

        match traducir(&stroke) {
            Some(evento) => procesar(evento),

            // No reconocida por back_teclas/back_mouse (ScanCode fuera de
            // pulsadores.tsv). Antes esto se perdía en silencio: ni se
            // procesaba ni se reenviaba, y la tecla quedaba muda en
            // Windows aunque no hubiera ningún trigger ni caché
            // compilada — justo lo que este programa promete no hacer.
            // Ahora se reenvía cruda, salvo que haya una Captura en
            // curso (ahí se traga, para no ensuciar lo que se graba).
            //
            // EXCEPCIÓN — fake-shift de Impr Pant (E0+LeftShift): este
            // stroke puntual NO se reenvía nunca acá, ni siquiera
            // crudo. emitir_teclado() lo reconstruye por su cuenta,
            // pegado al stroke real de PrintScreen, cada vez que ese
            // evento se emite (ver back_teclas.rs, TABLA_EXTENDIDA).
            // Si además lo dejáramos pasar acá, Windows vería un
            // Shift-down duplicado (el crudo inmediato + el
            // reconstruido después) y deja de reconocer la
            // combinación — es justo el bug que esto corrige.
            None => {
                let es_fake_shift_impr_pant = matches!(
                    stroke,
                    Stroke::Keyboard { code: ScanCode::LeftShift, state, .. }
                        if state.contains(KeyState::E0)
                );

                if !es_fake_shift_impr_pant && !debe_tragar_no_traducible() {
                    ict.send(device, &[stroke]);
                }
            }
        }
    }
}

// ======================================================
// 🚀 SESIÓN DE SALIDA (thread_local, perezosa)
// ======================================================

thread_local! {
    static SESION_SALIDA: RefCell<Option<Interception>> = RefCell::new(None);
}

fn con_sesion_salida<R>(usar: impl FnOnce(&Interception) -> R) -> R {
    SESION_SALIDA.with(|celda| {
        let mut sesion = celda.borrow_mut();

        if sesion.is_none() {
            *sesion = Some(Interception::new().expect("No se pudo iniciar Interception (salida)"));
        }

        usar(sesion.as_ref().unwrap())
    })
}

// ======================================================
// 📤 EMITIR EVENTO
// ======================================================

pub fn emitir_evento(evento: InputEvent) {
    match evento.input.fuente() {
        Some("keyboard") => emitir_teclado(&evento),
        Some("mouse") => emitir_mouse(&evento),
        _ => {}
    }
}

fn emitir_teclado(evento: &InputEvent) {
    let Some(device) = teclado_primario() else {
        return;
    };

    // Impr Pant es un caso especial: la tecla física manda un PAR de
    // strokes pegados (fake-shift E0+2A seguido del real E0+37) y
    // Windows solo lo reconoce como Impr Pant si le llegan juntos, en
    // el orden correcto, desde el mismo envío. Si se emitiera como un
    // stroke suelto (como cualquier otra tecla, vía back_teclas::
    // convertir_salida + enviar_tecla), Windows lo traduce a nada (ni
    // "*" ni captura) — ver nota en back_teclas.rs, TABLA_EXTENDIDA.
    // Por eso NO pasa por el camino genérico: se arma acá el par
    // completo, replicando byte a byte la secuencia física real.
    if evento.input.control() == Some("PrintScreen") {
        con_sesion_salida(|ict| match evento.state {
            InputState::Down => enviar_impr_pant(ict, device, true),
            InputState::Up => enviar_impr_pant(ict, device, false),
            InputState::Pulse => {
                enviar_impr_pant(ict, device, true);
                enviar_impr_pant(ict, device, false);
            }
        });

        return;
    }

    let Some((code, es_extendida)) = back_teclas::convertir_salida(&evento.input) else {
        return;
    };

    con_sesion_salida(|ict| match evento.state {
        InputState::Down => enviar_tecla(ict, device, code, KeyState::DOWN, es_extendida),

        InputState::Up => enviar_tecla(ict, device, code, KeyState::UP, es_extendida),

        InputState::Pulse => {
            enviar_tecla(ict, device, code, KeyState::DOWN, es_extendida);
            enviar_tecla(ict, device, code, KeyState::UP, es_extendida);
        }
    });
}

// ======================================================
// 🖨️ IMPR PANT (par fake-shift + tecla real)
// ------------------------------------------------------
// Secuencia física real (Set 1, ambas extendidas E0):
// Press:   E0 2A (shift) luego E0 37 (tecla)
// Release: E0 B7 (tecla) luego E0 AA (shift) — orden inverso
// ======================================================

fn enviar_impr_pant(ict: &Interception, device: Device, presionar: bool) {
    if presionar {
        enviar_tecla(ict, device, ScanCode::LeftShift, KeyState::DOWN, true);
        enviar_tecla(ict, device, ScanCode::NumpadMultiply, KeyState::DOWN, true);
    } else {
        enviar_tecla(ict, device, ScanCode::NumpadMultiply, KeyState::UP, true);
        enviar_tecla(ict, device, ScanCode::LeftShift, KeyState::UP, true);
    }
}

fn enviar_tecla(
    ict: &Interception,
    device: Device,
    code: interception::ScanCode,
    estado: KeyState,
    es_extendida: bool,
) {
    // Mismo motivo que en traducir(): sin el bit E0 en lo que se
    // manda, esto saldría como si fuera la tecla de numpad que
    // comparte su mismo ScanCode (ver back_teclas.rs, TABLA_EXTENDIDA).
    let estado = if es_extendida {
        estado | KeyState::E0
    } else {
        estado
    };

    let stroke = Stroke::Keyboard {
        code,
        state: estado,
        information: 0,
    };

    ict.send(device, &[stroke]);
}

fn emitir_mouse(evento: &InputEvent) {
    let Some(salida) = back_mouse::convertir_salida(&evento.input) else {
        return;
    };

    let Some(device) = mouse_primario() else {
        return;
    };

    con_sesion_salida(|ict| match salida {
        // Si el evento trae magnitud real (rueda física, ver
        // back_mouse::convertir/InputEvent::pulse_con_magnitud),
        // se reenvía tal cual entró — así un giro de varias
        // muescas no se aplana a una sola cuando no hay trigger
        // que lo intercepte. Si no trae magnitud (evento
        // sintético de una Acción remapeada), se usa el valor
        // fijo de siempre (120/-120).
        MouseOutput::Wheel(cantidad) => {
            enviar_rueda(ict, device, evento.magnitud.unwrap_or(cantidad))
        }

        MouseOutput::Button { down, up } => match evento.state {
            InputState::Down => enviar_boton(ict, device, down),

            InputState::Up => enviar_boton(ict, device, up),

            InputState::Pulse => {
                enviar_boton(ict, device, down);
                enviar_boton(ict, device, up);
            }
        },
    });
}

fn enviar_boton(ict: &Interception, device: Device, estado: MouseFilter) {
    let stroke = Stroke::Mouse {
        state: estado,
        flags: interception::MouseFlags::empty(),
        rolling: 0,
        x: 0,
        y: 0,
        information: 0,
    };

    ict.send(device, &[stroke]);
}

fn enviar_rueda(ict: &Interception, device: Device, cantidad: i16) {
    let stroke = Stroke::Mouse {
        state: MouseFilter::WHEEL,
        flags: interception::MouseFlags::empty(),
        rolling: cantidad,
        x: 0,
        y: 0,
        information: 0,
    };

    ict.send(device, &[stroke]);
}
