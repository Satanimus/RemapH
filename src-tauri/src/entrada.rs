// ======================================================
// 🚪 Entrada
// ======================================================
// 1. ¿Qué hace este archivo?
//
// El portero: recibe cada InputEvent físico del backend
// activo (back_interception o back_windows, según el modo —
// ver motor.rs), se lo entrega al AnalizadorTrigger,
// y según la ResolucionEntrada que termine llegando de
// Cache, decide si el input vuelve a Windows, se bloquea,
// o queda pendiente.
//
// Mantiene DOS cosas por separado (no un único "pendiente"):
//
// RETENIDO — a lo sumo uno, global. Hay un input bloqueado
//     esperando a ver si termina siendo un match o no.
//     Es único a propósito: Cache tampoco soporta más de
//     una "pregunta al timer" en simultáneo (ver
//     PREGUNTA_PENDIENTE en cache.rs) — acá se refleja la
//     misma regla, no es una limitación nueva.
//
//     Red de seguridad: nunca debería quedar abierto para
//     siempre (Cache siempre termina llamando a pasar() o
//     consumir()) — pero si algún bug hiciera que eso no
//     pase, CADA evento físico nuevo (de lo que sea) se
//     seguiría sumando a este mismo buffer sin límite,
//     dejando el teclado y el mouse completamente mudos.
//     Por eso, cada vez que se abre un RETENIDO nuevo, se
//     arranca un vigía en otro hilo
//     (config::tiempo_maximo_retenido(), 5s por defecto):
//     si para entonces sigue sin resolverse, se fuerza a
//     soltar todo el buffer tal cual — y se avisa por
//     consola, para poder distinguir "se activó la red de
//     seguridad" (hay un bug real en otro lado) de
//     cualquier otra cosa.
// DEVOLVIENDO — una lista, uno por cada grupo de teclas que
//     YA se dejó pasar a Windows (con match o sin él da lo
//     mismo el motivo) y todavía no soltó todas sus teclas.
//     Mientras un grupo esté acá, sus repeats/Ups pasan
//     derecho, sin analizar — es lo único que le permite al
//     portero saber, más adelante, qué Up le corresponde a
//     qué Down ya emitido. SIN esto, un Up nunca vuelve a
//     pasar por acá y la tecla queda "pegada" en Windows.
//
//     [FIX] También se abre un grupo DEVOLVIENDO cuando
//     consumir() resuelve un match DIFERIDO (Extra Normal/
//     Turbo/Mantener/ClickSostenido, ver requiere_up_real()
//     en perfil_cache.rs): ahí NO se emitió nada a Windows
//     (el "1" que ve el usuario es enteramente simulado por
//     runtime::ejecutar), pero la tecla física remapeada
//     (ej. "q") sigue abajo y sus repeats/Up real TODAVÍA
//     van a llegar acá. Sin un grupo DEVOLVIENDO que los
//     intercepte, esos eventos volvían a caer en la rama (c)
//     como si fueran "nuevos" — con nadie en cache.rs
//     llamando retener()/pasar()/consumir() para ellos (se
//     filtraban en silencio por el propio chequeo de
//     `presionadas` de cache.rs) — y entrada.rs los dejaba
//     pasar tal cual a Windows sin bloquear ni decidir nada.
//     Ahora, para ese caso, `faltan_soltar` se siembra con
//     las teclas indicadas por Cache en vez de quedar vacío,
//     así que sus repeats se bloquean acá (nunca se emiten,
//     ver el "pasa derecho" de la rama (a) — que emite
//     siempre; para bloquear un repeat sin emitirlo hace
//     falta el chequeo explícito de abajo) y su Up real cae
//     en la rama (a), avisa a cache::soltar_fisico() y cierra
//     el grupo normalmente.
//
// Una tecla físicamente nueva, sin relación con nada de lo
// anterior, no espera a que nada termine: se manda derecho
// al analizador (comportamiento normal).
//
// EXCEPCIÓN — Modo Captura: mientras haya una captura activa
// (cache::captura_activa()), este archivo no
// aplica NADA de lo anterior. Ni RETENIDO, ni DEVOLVIENDO, ni
// el corte por cache::esta_vacia(). Todo evento se reenvía
// directo a cache::procesar_evento_captura() y
// NUNCA se emite a Windows — la captura consume físicamente
// todo lo que llega (así un clic derecho capturado no abre
// menú contextual, ni un atajo ya guardado se dispara durante
// la captura de uno nuevo). Es el primer chequeo de la
// función, antes que cualquier otra cosa (incluido el corte
// de cache vacía: con captura activa, da igual si hay algo
// compilado o no).
//
// TAP PASIVO — captura_coordenada::observar_evento(): distinto
// del Modo Captura de arriba. Se llama SIEMPRE, antes que
// cualquier otra cosa (incluida la excepción de arriba), y
// nunca cambia el flujo — solo mira si llegó la tecla de
// guardar coordenada. Windows sigue recibiendo todo normal.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// motor::iniciar() — le entrega cada InputEvent físico
//     capturado por el backend activo (back_interception o
//     back_windows, según el modo — ver motor.rs).
// cache.rs — le avisa retener() / pasar() / consumir() sin
//     pasarle ningún dato salvo, en el caso de consumir(),
//     la lista de teclas que siguen físicamente vivas tras
//     un match diferido (ver EVENTO_EN_CURSO más abajo, y el
//     FIX de arriba).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Cada InputEvent (Down/Up/Pulse) tal como lo entrega el
// backend.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// No retorna nada — actúa directo llamando a
// motor::emitir_evento() cuando corresponde
// dejar pasar algo.
// ------------------------------------------------------
// 5. Comportamiento
//
// Al llegar un evento, en este orden:
//
// a) ¿La tecla del evento está en algún grupo DEVOLVIENDO?
//    Si el grupo nació de pasar() (o del reinyectado de un
//    RETENIDO), pasa derecho a Windows, sin análisis. Si el
//    grupo nació de un match diferido vía consumir() (ver
//    FIX arriba), el evento se BLOQUEA en vez de emitirse
//    (nunca se emitió el Down original a Windows, así que
//    tampoco corresponde emitir sus repeats ni su Up) — pero
//    igual se usa para llevar la cuenta de qué falta soltar,
//    y el Up real avisa a cache::soltar_fisico() igual que
//    siempre. Si es Up, se saca del faltan_soltar de ese
//    grupo; si el grupo queda vacío, se descarta (esa tecla
//    vuelve a estar "libre").
//
// b) Si no, ¿hay un RETENIDO en curso? Se agrega el evento a
//    su buffer (en orden, tal cual llegó) y además se manda
//    al analizador igual (para que evalúe si esto extiende o
//    resuelve el match).
//
// c) Si no hay ninguno de los dos: es un evento nuevo. Se
//    guarda en EVENTO_EN_CURSO (por si Cache, más abajo en
//    la misma pila de llamada, decide retener/pasar/consumir
//    sin pasarle el evento explícitamente) y se manda al
//    analizador.
//
// Lo que responde Cache (siempre sobre "lo que está pasando
// ahora"):
//
// - retener() → si no había RETENIDO, se crea uno, sembrado
//     con el EVENTO_EN_CURSO del hilo que llama (la llamada
//     es síncrona, dentro de la misma pila que
//     procesar_evento, así que el thread_local es válido).
// - consumir(vivas: &[InputId]) → hubo match real. Se
//     DESCARTA el RETENIDO entero (si había) sin reinyectar
//     nada — esos eventos ya fueron el match, nunca se
//     emitieron a Windows. Si `vivas` no está vacío (match
//     diferido con la tecla todavía físicamente abajo — ver
//     FIX arriba), se abre un grupo DEVOLVIENDO con esas
//     teclas, marcado para BLOQUEAR en vez de emitir, así sus
//     repeats no se cuelan y su Up real cierra el grupo
//     normalmente.
// - pasar() → no hubo match.
//     - Si había RETENIDO: se reinyecta su buffer completo,
//       en el mismo orden, sin delay artificial. Lo que
//       quede sin soltar pasa a un grupo nuevo en
//       DEVOLVIENDO (modo emitir).
//     - Si NO había RETENIDO (caso más común: el evento
//       actual se resolvió "Pasar" de una): se emite el
//       EVENTO_EN_CURSO tal cual, y si era un Down, se abre
//       igual un grupo DEVOLVIENDO para esa tecla (modo
//       emitir) — así su Up, cuando llegue, cae en el paso
//       (a) de arriba en vez de perderse.
// ------------------------------------------------------
// 6. Funciones del archivo
//
// procesar_evento(evento: InputEvent)
//     Punto de entrada único. Ver comportamiento (5).
// retener()
//     Ver comportamiento (5).
// pasar()
//     Ver comportamiento (5).
// consumir(vivas: &[InputId])
//     Ver comportamiento (5). El parámetro es nuevo (ver FIX):
//     antes no tomaba argumentos.
// ------------------------------------------------------
// Transformación:
//
// InputEvent físico
//     ↓
// ¿pertenece a un grupo DEVOLVIENDO? → pasa derecho (o se
//     bloquea, si el grupo es de tipo "consumido")
//     ↓ no
// ¿hay un RETENIDO? → se suma a su buffer + se analiza
//     ↓ no
// se guarda como EVENTO_EN_CURSO + se analiza
//     ↓ (implícito, vía Cache)
// retener() | pasar() | consumir(vivas)
// ======================================================

use crate::cache;
use crate::captura_coordenada;
use crate::config;
use crate::eventos::{InputEvent, InputId, InputState};
use crate::grabacion_macro;
use crate::motor;
use crate::perfil;
use std::cell::RefCell;
use std::sync::Mutex;
use std::time::Duration;

struct GrupoRetenido {
    buffer: Vec<InputEvent>,

    // Identifica esta apertura puntual de RETENIDO, para que el
    // vigía de la red de seguridad (ver retener()) sepa si todavía
    // está hablando del mismo RETENIDO que abrió, o si ya se
    // resolvió y se volvió a abrir otro distinto mientras dormía.
    generacion: u64,
}

struct GrupoDevolviendo {
    faltan_soltar: Vec<InputId>,

    // [FIX] false (comportamiento de siempre) = los eventos que
    // pertenecen a este grupo se emiten a Windows tal cual (grupo
    // nacido de pasar()). true = se BLOQUEAN, nunca se emiten (grupo
    // nacido de consumir() con un match diferido — ver header,
    // punto 5a): el Down original de esta tecla nunca llegó a
    // Windows, así que sus repeats y su Up tampoco deben llegar.
    // Sigue usándose igual para saber qué falta soltar y para
    // avisar a cache::soltar_fisico() en el Up.
    bloquear: bool,
}

static RETENIDO: Mutex<Option<GrupoRetenido>> = Mutex::new(None);
static DEVOLVIENDO: Mutex<Vec<GrupoDevolviendo>> = Mutex::new(Vec::new());
static SIGUIENTE_GENERACION_RETENIDO: Mutex<u64> = Mutex::new(0);

/// [FIX] Bug "Ctrl+Q=1 esporádico entre 1 y q" (Extra diferido:
/// Normal/Mantenido/Turbo — Extra Simple no lo sufre, ver más abajo
/// por qué). Causa raíz: `consumir()`/`pasar()` empujaban SIEMPRE un
/// grupo DEVOLVIENDO nuevo para sus teclas, sin revisar si alguna de
/// ellas ya estaba viva en OTRO grupo existente. Con un modificador
/// sostenido (Ctrl) y el gatillo tocado varias veces seguidas (Q),
/// cada match nuevo abre su propio grupo [Ctrl, Q] mientras el grupo
/// del toque anterior, que ya soltó su Q pero todavía no su Ctrl
/// (Ctrl sigue físicamente abajo), sigue existiendo con [Ctrl] solo
/// — Ctrl queda duplicado en dos grupos a la vez. El Up real de Ctrl
/// (rama a, `position()`) solo cierra el PRIMERO que encuentra —
/// siempre el más viejo — dejando el otro como zombie para siempre.
/// Ese zombie bloquea en silencio el PRÓXIMO Down real de Ctrl (rama
/// a, `bloquear:true`) antes de que llegue a cache.rs: `RUNTIME.
/// presionadas` nunca se entera de que Ctrl está abajo, así que la
/// próxima vez que se toca Q, la sesión nace sin Ctrl, no matchea
/// ningún trigger compilado (todos piden Ctrl+Q), y Q se reenvía
/// crudo a Windows sin remapear — el "q" suelto del reporte,
/// alternando con "1" según cuántos grupos zombie se hayan
/// acumulado. Extra Simple no lo sufre porque, al no ser diferido,
/// `resolver_match()` llama a `consumir(&[])` (vivas vacío a
/// propósito) y nunca abre ningún grupo DEVOLVIENDO para sus teclas
/// — no hay grupo que pueda duplicarse.
///
/// Fix: antes de abrir un grupo nuevo para `inputs`, sacarlos de
/// CUALQUIER otro grupo que ya los tuviera (y descartar ese grupo si
/// queda vacío) — así una tecla nunca vive en dos grupos a la vez, y
/// el más nuevo siempre "gana" la propiedad de esa tecla.
fn purgar_de_devolviendo(inputs: &[InputId]) {
    let mut devolviendo = DEVOLVIENDO.lock().unwrap();
    for grupo in devolviendo.iter_mut() {
        grupo.faltan_soltar.retain(|i| !inputs.contains(i));
    }
    devolviendo.retain(|grupo| !grupo.faltan_soltar.is_empty());
}

thread_local! {
    // El evento que está siendo procesado ahora mismo en este hilo.
    // Cache lo necesita indirectamente: cuando llama a retener() o
    // pasar() (sin pasarle el evento, por diseño), acá es donde lo
    // recuperamos. Válido porque esas llamadas son síncronas, dentro
    // de la misma pila que procesar_evento().
    static EVENTO_EN_CURSO: RefCell<Option<InputEvent>> = RefCell::new(None);
}

// ======================================================
// 🎚️ ATAJO GLOBAL TOGGLE PERFIL
// ------------------------------------------------------
// Detección independiente de RETENIDO/DEVOLVIENDO/Cache: debe
// funcionar con el perfil desactivado (cache vacía). [FIX] SÍ lo
// afecta el Modo Captura (cache::captura_activa()): mientras se está
// grabando un combo nuevo, tocar el combo del atajo debe quedar
// registrado por la captura, no disparar el toggle — ver el chequeo
// en procesar_evento(). Lleva su propio registro de teclas
// físicamente abajo (no reutiliza el de Cache, que no existe cuando
// el perfil está desactivado).
// ======================================================

static TECLAS_ABAJO_TOGGLE: Mutex<Vec<InputId>> = Mutex::new(Vec::new());

/// Compara un set de teclas físicamente abajo contra un AtajoSimple
/// (modificadores + gatillo), sin importar el orden de los
/// modificadores. Mismo criterio que
/// perfil_ui::coincide_con_atajo_reservado, pero contra un solo
/// atajo (acá no aplica tecla_guardar_coordenada).
fn coincide_con_combo(atajo: &config::AtajoSimple, gatillo: &InputId, abajo: &[InputId]) -> bool {
    &atajo.gatillo == gatillo
        && atajo.modificadores.len() == abajo.len().saturating_sub(1)
        && atajo
            .modificadores
            .iter()
            .all(|modificador| abajo.contains(modificador))
}

/// Actualiza TECLAS_ABAJO_TOGGLE con el evento físico y, en el Down
/// del gatillo, evalúa si el combo resultante coincide con
/// config::tecla_toggle_perfil(). Devuelve true solo en ese caso
/// (nunca en Up/Pulse, ni en un Down que solo sea un modificador).
///
/// [FIX] Windows repite WM_KEYDOWN mientras una tecla sigue
/// físicamente abajo (auto-repeat) — sin este chequeo, mantener
/// apretado el gatillo disparaba ejecutar_toggle_perfil() una vez
/// por cada repeat, activando/desactivando el perfil en ráfaga
/// mientras se sostiene la tecla (mismo patrón que ya rompía
/// Interception al recrear su contexto en sucesión rápida — ver
/// back_interception.rs). Se dispara solo en la transición real
/// (la tecla no estaba abajo todavía); un Down repetido de una
/// tecla ya registrada nunca reevalúa el combo.
fn detectar_toggle(evento: &InputEvent) -> bool {
    let mut abajo = TECLAS_ABAJO_TOGGLE.lock().unwrap();

    match evento.state {
        InputState::Down => {
            if abajo.contains(&evento.input) {
                return false;
            }

            abajo.push(evento.input.clone());

            let atajo = config::tecla_toggle_perfil();
            coincide_con_combo(&atajo, &evento.input, &abajo)
        }
        InputState::Up => {
            abajo.retain(|i| i != &evento.input);
            false
        }
        InputState::Pulse => false,
    }
}

/// Consulta cache::esta_vacia() (activado/desactivado) y llama a
/// perfil::activar_perfil() / perfil::desactivar_perfil() según
/// corresponda. Errores de activar_perfil() (ej. perfil corrupto)
/// solo se loguean por consola: entrada.rs no tiene forma de
/// mostrarlos en la UI.
fn ejecutar_toggle_perfil() {
    if cache::esta_vacia() {
        if let Err(error) = perfil::activar_perfil() {
            eprintln!("⚠️ Atajo toggle: no se pudo activar el perfil: {error}");
        }
    } else {
        perfil::desactivar_perfil();
    }
}

pub fn procesar_evento(evento: InputEvent) {
    // Atajo global Activar/Desactivar perfil: va antes que el resto
    // del flujo, para que funcione sin importar si el perfil está
    // activado o desactivado. Igual se llama a detectar_toggle()
    // incondicionalmente (necesita ver cada Down/Up para mantener
    // TECLAS_ABAJO_TOGGLE al día), pero [FIX] la acción NO se
    // dispara si hay Modo Captura en curso (cache::captura_activa()):
    // mientras se está grabando un combo nuevo desde el Botón
    // Capturador (incluida la captura del propio atajo o de
    // tecla_guardar_coordenada — Regla 7), tocar ese combo debe
    // quedar registrado por la captura, no disparar el toggle. En
    // ese caso el evento NO se consume acá; sigue el flujo normal y
    // cae en la excepción de Modo Captura de abajo, que sí lo
    // consume.
    if detectar_toggle(&evento) && !cache::captura_activa() {
        ejecutar_toggle_perfil();
        return;
    }

    // Tap pasivo para la ventana de captura de "Click en coordenada"
    // (ver captura_coordenada.rs): nunca decide nada sobre el evento,
    // solo observa. Va primero y no retorna nada — todo lo de abajo
    // sigue exactamente igual, con o sin una captura de coordenada
    // activa. A propósito NO es lo mismo que el "Modo Captura" de más
    // abajo (ese sí consume todo); acá Windows sigue funcionando
    // normal.
    captura_coordenada::observar_evento(&evento);

    // Tap pasivo para la captura cruda del Grabador de Macro (Etapa
    // D, ver grabacion_macro.rs): mismo criterio que el de arriba,
    // nunca decide nada sobre el evento.
    grabacion_macro::observar_evento(&evento);

    // EXCEPCIÓN — Modo Captura: se consume TODO, incondicionalmente, y
    // ni se mira RETENIDO/DEVOLVIENDO ni el estado de la cache. Esto va
    // primero que cualquier otra cosa (ver header, punto 5).
    if cache::captura_activa() {
        cache::procesar_evento_captura(evento);
        return;
    }

    // Diagnóstico + optimización: sin ningún remapeo compilado, no hay
    // nada que evaluar — se devuelve directo, sin tocar RETENIDO,
    // DEVOLVIENDO ni el analizador. (Solo aplica fuera de una captura:
    // si hay captura activa, la rama de arriba ya se hizo cargo y esta
    // línea ni se evalúa.)
    if cache::esta_vacia() {
        motor::emitir_evento(evento);
        return;
    }

    // a) ¿Pertenece a algún grupo que ya se dejó pasar (o consumir) y
    //    todavía no soltó todas sus teclas? Se resuelve sin análisis:
    //    se emite o se bloquea según el tipo de grupo (ver
    //    GrupoDevolviendo::bloquear).
    {
        let mut devolviendo = DEVOLVIENDO.lock().unwrap();

        if let Some(indice) = devolviendo
            .iter()
            .position(|grupo| grupo.faltan_soltar.contains(&evento.input))
        {
            if !devolviendo[indice].bloquear {
                motor::emitir_evento(evento.clone());
            }

            if evento.state == InputState::Up {
                devolviendo[indice]
                    .faltan_soltar
                    .retain(|i| i != &evento.input);

                if devolviendo[indice].faltan_soltar.is_empty() {
                    devolviendo.remove(indice);
                }

                drop(devolviendo);

                // Este Up nunca llega a cache::procesar_evento_runtime()
                // (cortamos acá con el return de abajo) — sin este
                // aviso, su conjunto interno de "presionados ahora"
                // queda pensando que la tecla sigue abajo para
                // siempre, y la próxima Down de esa tecla se descarta
                // como si fuera un repeat.
                cache::soltar_fisico(evento.input.clone());
            }
            return;
        }
    }

    // b) ¿Hay una retención en curso? Se suma al buffer y se analiza
    //    igual, para que el analizador evalúe si esto extiende o
    //    resuelve el match.
    {
        let mut retenido = RETENIDO.lock().unwrap();

        if let Some(grupo) = retenido.as_mut() {
            grupo.buffer.push(evento.clone());
            drop(retenido);
            cache::procesar_evento_runtime(evento);
            return;
        }
    }

    // c) Evento nuevo, sin nada pendiente.
    EVENTO_EN_CURSO.with(|c| *c.borrow_mut() = Some(evento.clone()));
    cache::procesar_evento_runtime(evento);
    EVENTO_EN_CURSO.with(|c| *c.borrow_mut() = None);
}

/// [FIX] Llamada por Cache al activar Modo Captura, por cada tecla que
/// ya estaba físicamente abajo en ese momento: emite su Up a Windows
/// (deja el estado del sistema limpio) Y la purga de cualquier grupo
/// DEVOLVIENDO que la tuviera pendiente. Lo segundo es indispensable:
/// sin purgar, ese grupo queda zombie (nunca ve el Up real — Captura
/// se lo come primero, ver la excepción de arriba) y termina
/// interceptando en silencio el PRÓXIMO Down real de esa tecla (rama
/// a: lo reemite crudo y hace `return` antes de llegar a
/// cache::procesar_evento_runtime()) — la tecla queda invisible para
/// RUNTIME.presionadas en la vuelta siguiente. Efecto observado: la
/// captura funciona una vez sí, una vez no, alternando.
pub(crate) fn soltar_forzado(inputs: &[InputId]) {
    for input in inputs {
        motor::emitir_evento(InputEvent::up(input.clone()));
    }
    purgar_de_devolviendo(inputs);
}

/// Llamada por Cache (síncrona o desde el timer): no hay match posible.
/// Si había un RETENIDO, se reinyecta su buffer completo en orden, sin
/// delay, y lo que quede sin soltar pasa a un grupo DEVOLVIENDO (modo
/// emitir). Si no había nada retenido, es el evento en curso ahora
/// mismo en este hilo: se emite tal cual y, si era un Down, abre igual
/// su propio grupo DEVOLVIENDO (modo emitir) para que su Up no se
/// pierda.
pub fn pasar() {
    let mut retenido = RETENIDO.lock().unwrap();

    let Some(grupo) = retenido.take() else {
        drop(retenido);

        let evento = EVENTO_EN_CURSO.with(|c| c.borrow().clone());

        if let Some(evento) = evento {
            motor::emitir_evento(evento.clone());

            if evento.state == InputState::Down {
                purgar_de_devolviendo(std::slice::from_ref(&evento.input)); // ⚠️ ver FIX en la definición
                DEVOLVIENDO.lock().unwrap().push(GrupoDevolviendo {
                    faltan_soltar: vec![evento.input],
                    bloquear: false,
                });
            }
        }
        return;
    };

    drop(retenido);

    let mut faltan_soltar: Vec<InputId> = Vec::new();

    for evento in grupo.buffer {
        motor::emitir_evento(evento.clone());

        match evento.state {
            InputState::Down => {
                if !faltan_soltar.contains(&evento.input) {
                    faltan_soltar.push(evento.input);
                }
            }
            InputState::Up => faltan_soltar.retain(|i| i != &evento.input),
            InputState::Pulse => {}
        }
    }

    if !faltan_soltar.is_empty() {
        purgar_de_devolviendo(&faltan_soltar); // ⚠️ ver FIX en la definición
        DEVOLVIENDO.lock().unwrap().push(GrupoDevolviendo {
            faltan_soltar,
            bloquear: false,
        });
    }
}

/// Llamada por Cache: todavía puede llegar a ser un match. Abre (si no
/// existía) el RETENIDO, sembrado con el evento en curso, y arranca su
/// vigía de red de seguridad (ver vigilar_retenido()).
pub fn retener() {
    let mut retenido = RETENIDO.lock().unwrap();

    if retenido.is_none() {
        let evento_inicial = EVENTO_EN_CURSO.with(|c| c.borrow().clone());

        let generacion = {
            let mut g = SIGUIENTE_GENERACION_RETENIDO.lock().unwrap();
            *g += 1;
            *g
        };

        *retenido = Some(GrupoRetenido {
            buffer: evento_inicial.into_iter().collect(),
            generacion,
        });
        drop(retenido);

        std::thread::spawn(move || {
            vigilar_retenido(generacion);
        });
    }
}

/// Red de seguridad: si el RETENIDO abierto con esta generación sigue
/// siendo el mismo (nadie lo resolvió) después de
/// config::tiempo_maximo_retenido(), se fuerza a soltar su buffer tal
/// cual — un bug en otro lado no debería poder dejar el teclado o el
/// mouse mudos para siempre. El aviso por consola permite distinguir
/// esto de cualquier otro problema.
fn vigilar_retenido(generacion: u64) {
    std::thread::sleep(Duration::from_millis(config::tiempo_maximo_retenido()));

    let sigue_siendo_este = RETENIDO
        .lock()
        .unwrap()
        .as_ref()
        .map(|g| g.generacion == generacion)
        .unwrap_or(false);

    if !sigue_siendo_este {
        return; // ya se resolvió (o se abrió otro distinto) antes de esto
    }

    eprintln!(
        "⚠️ Red de seguridad: un RETENIDO llevaba más de {} ms sin resolverse — se fuerza a soltar. Esto NO debería pasar; revisar cache.rs.",
        config::tiempo_maximo_retenido()
    );

    pasar();
}

/// Llamada por Cache: hubo match real y ya se avisó a Runtime. Lo
/// retenido (si algo había) YA fue el match — se descarta sin
/// reinyectar nada (nunca se emitió nada de esto a Windows).
///
/// [FIX] `vivas` son las teclas de ese match que Cache determinó que
/// siguen físicamente presionadas AHORA MISMO (ver
/// cache::algo_sigue_presionado — típicamente, un match diferido de
/// Extra Normal/Turbo/Mantener/ClickSostenido, donde la instancia
/// activa recién se va a cerrar con el Up real de esa tecla). Antes
/// esta función no recibía nada y jamás abría un grupo DEVOLVIENDO,
/// así que esas teclas quedaban sin ningún rastro en entrada.rs: sus
/// repeats de Down y su Up real volvían a caer en la rama (c) como
/// "eventos nuevos", cache.rs los descartaba en silencio por su
/// propio chequeo de `presionadas` (sin llamar nunca a retener/pasar/
/// consumir), y entrada.rs — al no tener ninguna decisión explícita
/// para ellos — terminaba dejándolos pasar sin bloquear (la tecla
/// remapeada se colaba a Windows en cada repeat) y el Up real nunca
/// cerraba el ciclo acá (aunque sí cerraba la InstanciaActiva del
/// lado de cache.rs, vía runtime.activas).
///
/// Si `vivas` está vacío (caso más común: match no diferido, o
/// diferido pero la tecla ya se soltó antes de resolverse), el
/// comportamiento es exactamente el de antes: no se abre nada.
pub fn consumir(vivas: &[InputId]) {
    *RETENIDO.lock().unwrap() = None;

    if !vivas.is_empty() {
        purgar_de_devolviendo(vivas); // ⚠️ ver FIX en la definición — evita el modificador duplicado en dos grupos
        DEVOLVIENDO.lock().unwrap().push(GrupoDevolviendo {
            faltan_soltar: vivas.to_vec(),
            bloquear: true,
        });
    }
}
