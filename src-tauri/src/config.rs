// ======================================================
// ⚙️ Config ETAPA 0 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Mantiene la configuración global utilizada
// por todos los módulos del sistema.
//
// Contiene únicamente valores compartidos.
//
// NO almacena:
//
// - Perfiles.
// - Remapeos.
// - Estado.
// - Runtime.
// - UI.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe cambios desde:
//
// • Configuración persistente.
// • UI mediante comandos Tauri.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Cualquier módulo que necesite un valor
// de configuración global.
//
// Principalmente:
//
// • AnalizadorTrigger.
// • Runtime.
// • Backends.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve:
//
// • Tiempos.
// • Sensibilidades.
// • Parámetros globales.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// tiempo_doble()
// establecer_tiempo_doble()
//     Tiempo máximo para detectar Doble.
//     También define cuánto espera AnalizadorTrigger antes de decidir que una secuencia terminó.
//
// tiempo_triple()
// establecer_tiempo_triple()
//     Tiempo máximo, contado desde el Up del primer toque, para
//     detectar Triple. Reemplaza a tiempo_doble en ese rol cuando
//     existe al menos un binding Triple candidato para la entrada
//     (ver analizador_trigger.rs, fase Triple) — es una ventana más
//     larga porque tiene que alcanzar a entrar un tercer toque, no
//     solo un segundo.
//
// tiempo_mantenido()
// establecer_tiempo_mantenido()
//     Tiempo mínimo para detectar Mantenido.
//
// tiempo_repeticion()
// establecer_tiempo_repeticion()
//     Intervalo entre repeticiones de Turbo.
//
// tiempo_espera_normal()
// establecer_tiempo_espera_normal()
//     Espera entre la primera salida y el inicio del
//     bucle de repetición, para el Extra "Normal".
//
// sensibilidad_rueda()
// establecer_sensibilidad_rueda()
//     Cantidad de movimientos minimos necesarios para considerar una acción mantenida.
//
// tiempo_maximo_retenido()
// establecer_tiempo_maximo_retenido()
//     Red de seguridad de entrada.rs: si un RETENIDO lleva más de
//     este tiempo sin resolverse (nunca debería pasar — indica un
//     bug en cache.rs/analizador_trigger.rs), se fuerza a soltarlo
//     en vez de dejar el teclado/mouse trabado para siempre. Un log
//     de advertencia avisa por consola cuando esto se activa, para
//     poder distinguir "activó la red de seguridad" de "otra cosa".
//
// tiempo_inactividad_captura()
// establecer_tiempo_inactividad_captura()
//     Red de seguridad de Modo Captura: tiempo absoluto desde que
//     se activó la captura hasta que se fuerza su cierre si no
//     terminó sola. No se reinicia con la actividad — a propósito,
//     para cubrir el caso de una tecla trabada que sigue mandando
//     Down sin Up nunca.
//
// tecla_guardar_coordenada()
// establecer_tecla_guardar_coordenada()
//     Atajo (AtajoSimple: modificadores + gatillo) que la ventana de
//     captura de "Click en coordenada" escucha para guardar la
//     posición actual (F1 sin modificadores, por defecto).
//
// tecla_toggle_perfil()
// establecer_tecla_toggle_perfil()
//     Atajo (AtajoSimple: modificadores + gatillo) global para
//     Activar/Desactivar el perfil (Ctrl+F1 por defecto). Funciona
//     con el perfil desactivado, cubre Interception y Modo Portable.
//
// intervalo_captura_coordenada()
// establecer_intervalo_captura_coordenada()
//     Cada cuántos ms la ventana de captura de "Click en coordenada"
//     vuelve a leer cursor/ventana activa y a chequear si se pidió
//     guardar (100ms por defecto — solo feedback visual, no necesita
//     ser más fino).
//
// delay_entre_salida_doble()
// establecer_delay_entre_salida_doble()
//     Pausa entre el 1º y 2º combo cuando la Acción capturada
//     (accion_trigger, tipo tecla_mouse) tiene condición Doble.
//
// delay_rueda_repeticion()
// establecer_delay_rueda_repeticion()
//     Pausa entre cada salida en cola de una fila con Extra
//     RepeticionRueda (un pulso de rueda = una salida encolada) —
//     ver PLAN_RUEDA_REPETICION.md.
//
// tiempo_simple_teclas()
// establecer_tiempo_simple_teclas()
//     Pausa entre el DOWN y el UP de un toque de tecla/botón (todo
//     "toque" pasa por acá: Simple, cada repetición de Doble/Triple,
//     y cada ciclo de Normal/Turbo). 0ms por defecto — existe lista
//     para el caso de una app que necesite un mínimo de detección.
//
// delta_volumen()
// establecer_delta_volumen()
//     Cuánto sube/baja el volumen por pulsación (Acción Multimedia,
//     alcance En App).
//
// menu_boton_pequeno() / menu_boton_mediano() / menu_boton_grande()
// establecer_menu_boton_pequeno() / _mediano() / _grande()
//     Ancho/alto en px de los 3 tamaños de botón de MenuExpress
//     (menu_extra.tamanoBoton). Única fuente de verdad — antes
//     hardcodeados por triplicado (CSS + TS + Rust), ver
//     comentario en la sección de abajo.
//
// menu_texto_pequeno() / menu_texto_mediano() / menu_texto_grande()
// establecer_menu_texto_pequeno() / _mediano() / _grande()
//     Tamaño de fuente en px de los 3 tamaños de texto de
//     MenuExpress (menu_extra.tamanoTexto). Mismo criterio.
//
// portapapeles_boton_pequeno() / _mediano() / _grande()
// establecer_portapapeles_boton_pequeno() / _mediano() / _grande()
//     Ancho/alto en px de los 3 tamaños de botón de Portapapeles
//     (portapapeles_extra.tamanoBoton). Valores propios (botones
//     alargados, no cuadrados como MenuExpress) — mismo criterio de
//     única fuente de verdad que menu_boton_*. El tamaño de TEXTO de
//     Portapapeles reusa menu_texto_pequeno/mediano/grande tal cual,
//     no tiene funciones propias.
//
// tiempo_ignorar_cambio_portapapeles()
// establecer_tiempo_ignorar_cambio_portapapeles()
//     Ventana (desde que se pega algo) durante la cual se ignora el
//     próximo aviso de cambio de portapapeles, para no generar un
//     rotativo duplicado del mismo contenido que ya existía (el
//     propio pegado dispara su propio aviso de cambio).
//
// tiempo_espera_pegado_imagen()
// establecer_tiempo_espera_pegado_imagen()
//     Pausa en pegar(), solo para contenido IMAGEN en apps SIN camino
//     personalizado (no Photoshop), entre forzar_relectura_
//     portapapeles() y el Ctrl+V simulado — le da tiempo a la app de
//     "asentar" la imagen nueva antes del pegado automático. Antes
//     era un único valor genérico para todo contenido; se separó
//     porque solo hace falta para imagen (ver tiempo_espera_pegado_
//     texto()).
//
// tiempo_espera_pegado_texto()
// establecer_tiempo_espera_pegado_texto()
//     Misma pausa que tiempo_espera_pegado_imagen(), pero para
//     contenido TEXTO — mucho más corta porque el texto no necesita
//     "asentarse" como una imagen pesada. También la usa Photoshop
//     cuando el contenido es texto (ver delay_imagen_photoshop()).
//
// delay_imagen_photoshop()
// establecer_delay_imagen_photoshop()
//     Solo Photoshop y solo con contenido IMAGEN: pausa entre el
//     relanzamiento de activación (script vacío, reutilizado) y el
//     Ctrl+V simulado que pega de verdad. Con contenido TEXTO,
//     Photoshop no usa este valor — usa tiempo_espera_pegado_texto()
//     como cualquier app genérica. Ver back_pegado_personalizado.rs.
// ------------------------------------------------------
// Transformación:
//
// UI
//      ↓
// Configuración
//      ↓
// Config
//      ↓
// Todos los módulos
// ======================================================

use std::sync::atomic::{AtomicU64, Ordering};

use crate::eventos::InputId;

// ======================================================
// 📦 APP
// ======================================================

pub const NOMBRE_APP: &str = "RemapH";

// ======================================================
// ⏱️ TIEMPO DOBLE
// ======================================================

static TIEMPO_DOBLE: AtomicU64 = AtomicU64::new(250);

// ======================================================
// ⏱️ TIEMPO TRIPLE
// ======================================================

static TIEMPO_TRIPLE: AtomicU64 = AtomicU64::new(380);

// ======================================================
// ⏳ TIEMPO MANTENIDO
// ======================================================

static TIEMPO_MANTENIDO: AtomicU64 = AtomicU64::new(300);

// ======================================================
// 🖱️ SENSIBILIDAD RUEDA
// ======================================================

static SENSIBILIDAD_RUEDA: AtomicU64 = AtomicU64::new(5);

// ======================================================
// 📥 LEER TIEMPO DOBLE
// ======================================================

pub fn tiempo_doble() -> u64 {
    TIEMPO_DOBLE.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO DOBLE
// ======================================================

pub fn establecer_tiempo_doble(valor: u64) {
    TIEMPO_DOBLE.store(valor, Ordering::Relaxed);
}

// ======================================================
// 📥 LEER TIEMPO TRIPLE
// ======================================================

pub fn tiempo_triple() -> u64 {
    TIEMPO_TRIPLE.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO TRIPLE
// ======================================================

pub fn establecer_tiempo_triple(valor: u64) {
    TIEMPO_TRIPLE.store(valor, Ordering::Relaxed);
}

// ======================================================
// 📥 LEER TIEMPO MANTENIDO
// ======================================================

pub fn tiempo_mantenido() -> u64 {
    TIEMPO_MANTENIDO.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO MANTENIDO
// ======================================================

pub fn establecer_tiempo_mantenido(valor: u64) {
    TIEMPO_MANTENIDO.store(valor, Ordering::Relaxed);
}

// ======================================================
// 📥 LEER SENSIBILIDAD RUEDA
// ======================================================

pub fn sensibilidad_rueda() -> u64 {
    SENSIBILIDAD_RUEDA.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR SENSIBILIDAD RUEDA
// ======================================================

pub fn establecer_sensibilidad_rueda(valor: u64) {
    SENSIBILIDAD_RUEDA.store(valor, Ordering::Relaxed);
}

// ======================================================
// ⚡ TIEMPO REPETICIÓN
// ======================================================

static TIEMPO_REPETICION: AtomicU64 = AtomicU64::new(30);

// ======================================================
// 📥 LEER TIEMPO REPETICIÓN
// ======================================================

pub fn tiempo_repeticion() -> u64 {
    TIEMPO_REPETICION.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO REPETICIÓN
// ======================================================

pub fn establecer_tiempo_repeticion(valor: u64) {
    TIEMPO_REPETICION.store(valor, Ordering::Relaxed);
}

// ======================================================
// 🕐 TIEMPO ESPERA NORMAL
// ------------------------------------------------------
// Extra "Normal": al mantener presionado, la espera entre
// la primera salida y el inicio del bucle de repetición
// (que luego usa tiempo_repeticion() como cualquier Turbo).
// Simula el comportamiento de una tecla física de Windows
// (se dibuja una vez, y recién después de esta espera
// empieza a repetirse en bucle).
// ======================================================

static TIEMPO_ESPERA_NORMAL: AtomicU64 = AtomicU64::new(500);

// ======================================================
// 📥 LEER TIEMPO ESPERA NORMAL
// ======================================================

pub fn tiempo_espera_normal() -> u64 {
    TIEMPO_ESPERA_NORMAL.load(Ordering::Relaxed)
}

// ======================================================
// 📤 ESCRIBIR TIEMPO ESPERA NORMAL
// ======================================================

pub fn establecer_tiempo_espera_normal(valor: u64) {
    TIEMPO_ESPERA_NORMAL.store(valor, Ordering::Relaxed);
}

// ======================================================
// 🛟 TIEMPO MÁXIMO RETENIDO (red de seguridad)
// ------------------------------------------------------
// entrada.rs nunca debería necesitar esto — es un "por las dudas"
// para que un RETENIDO jamás quede trabado para siempre por un bug
// que todavía no vimos. Separado en su propia constante (no
// reutiliza tiempo_mantenido/tiempo_doble) justo para poder
// distinguir, si algún día se activa, si fue esta red de seguridad
// la que actuó o si el problema es otro.
// ======================================================

static TIEMPO_MAXIMO_RETENIDO: AtomicU64 = AtomicU64::new(5000);

pub fn tiempo_maximo_retenido() -> u64 {
    TIEMPO_MAXIMO_RETENIDO.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_maximo_retenido(valor: u64) {
    TIEMPO_MAXIMO_RETENIDO.store(valor, Ordering::Relaxed);
}

// ======================================================
// 🛟 TIEMPO INACTIVIDAD CAPTURA (red de seguridad)
// ------------------------------------------------------
// Modo Captura consume físicamente todo lo que llega (ver
// entrada.rs) — si el usuario lo abre y se distrae sin terminar
// el gesto, o si una tecla queda trabada físicamente (sigue
// repitiendo Down sin soltar nunca el Up), el teclado/mouse queda
// mudo indefinidamente. Este es el "vigía" que lo evita: tiempo
// absoluto desde que se activó la captura, sin reiniciarse con
// la actividad, tras el cual se fuerza su cierre si no terminó
// sola.
// ======================================================

static TIEMPO_INACTIVIDAD_CAPTURA: AtomicU64 = AtomicU64::new(10000);

pub fn tiempo_inactividad_captura() -> u64 {
    TIEMPO_INACTIVIDAD_CAPTURA.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_inactividad_captura(valor: u64) {
    TIEMPO_INACTIVIDAD_CAPTURA.store(valor, Ordering::Relaxed);
}

// ======================================================
// 🎹 ATAJO SIMPLE (modificadores + gatillo, sin condición)
// ------------------------------------------------------
// Formato compartido por tecla_guardar_coordenada y
// tecla_toggle_perfil: ambos deben funcionar con el perfil
// desactivado, fuera del pipeline de AnalizadorTrigger/Cache
// — quedan limitados a Simple (sin Doble/Triple/Mantenido),
// así que no necesitan condicion, solo modificadores+gatillo.
//
// Texto plano: "mod1,mod2|gatillo" (cada entrada en formato
// InputId "fuente:control", ej. "keyboard:F1"). Sin
// modificadores: "|keyboard:F1".
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtajoSimple {
    pub modificadores: Vec<InputId>,

    pub gatillo: InputId,
}

impl AtajoSimple {
    pub fn a_texto(&self) -> String {
        let mods = self
            .modificadores
            .iter()
            .map(|input| input.to_string())
            .collect::<Vec<_>>()
            .join(",");

        format!("{}|{}", mods, self.gatillo)
    }

    /// None si el texto no respeta el formato "mod,mod|gatillo"
    /// (separador '|' ausente, o alguna entrada sin "fuente:control").
    pub fn desde_texto(texto: &str) -> Option<Self> {
        let (mods_texto, gatillo_texto) = texto.split_once('|')?;

        let modificadores = mods_texto
            .split(',')
            .filter(|entrada| !entrada.is_empty())
            .map(input_id_desde_texto)
            .collect::<Option<Vec<_>>>()?;

        let gatillo = input_id_desde_texto(gatillo_texto)?;

        Some(Self {
            modificadores,
            gatillo,
        })
    }
}

fn input_id_desde_texto(texto: &str) -> Option<InputId> {
    let (fuente, control) = texto.split_once(':')?;

    Some(InputId::new(fuente, control))
}

// ======================================================
// 📌 TECLA GUARDAR COORDENADA (ventana de captura)
// ------------------------------------------------------
// Atajo (modificadores + gatillo, ver AtajoSimple) que la
// ventana de captura de "Click en coordenada" escucha para
// guardar la posición actual. Configurable, no fijo — ver
// captura_coordenada.rs (quien la usa) y comandos.rs (quien
// la expone a la UI).
// ======================================================

static TECLA_GUARDAR_COORDENADA: std::sync::Mutex<Option<AtajoSimple>> =
    std::sync::Mutex::new(None);

pub fn tecla_guardar_coordenada() -> AtajoSimple {
    TECLA_GUARDAR_COORDENADA
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| AtajoSimple {
            modificadores: Vec::new(),
            gatillo: InputId::new("keyboard", "F1"),
        })
}

pub fn establecer_tecla_guardar_coordenada(valor: AtajoSimple) {
    *TECLA_GUARDAR_COORDENADA.lock().unwrap() = Some(valor);
}

// ======================================================
// 🎚️ TECLA TOGGLE PERFIL (Activar/Desactivar, atajo global)
// ------------------------------------------------------
// Atajo (modificadores + gatillo, ver AtajoSimple) que activa
// o desactiva el perfil actual. Global: funciona con el
// perfil desactivado y cubre Interception y Modo Portable —
// ver entrada.rs (quien lo detecta) y comandos.rs (quien lo
// expone a la UI).
// ======================================================

static TECLA_TOGGLE_PERFIL: std::sync::Mutex<Option<AtajoSimple>> = std::sync::Mutex::new(None);

pub fn tecla_toggle_perfil() -> AtajoSimple {
    TECLA_TOGGLE_PERFIL
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| AtajoSimple {
            modificadores: vec![InputId::new("keyboard", "LeftControl")],
            gatillo: InputId::new("keyboard", "F1"),
        })
}

pub fn establecer_tecla_toggle_perfil(valor: AtajoSimple) {
    *TECLA_TOGGLE_PERFIL.lock().unwrap() = Some(valor);
}

// ======================================================
// 🔴 TECLA GRABAR MACRO (toggle Iniciar/Detener grabación)
// ------------------------------------------------------
// Atajo (modificadores + gatillo, ver AtajoSimple) que
// inicia/detiene la grabación de macro desde el editor de
// Macro. Configurable, no fijo — mismo patrón que
// tecla_guardar_coordenada.
// ======================================================

static TECLA_GRABAR_MACRO: std::sync::Mutex<Option<AtajoSimple>> = std::sync::Mutex::new(None);

pub fn tecla_grabar_macro() -> AtajoSimple {
    TECLA_GRABAR_MACRO
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| AtajoSimple {
            modificadores: Vec::new(),
            gatillo: InputId::new("keyboard", "F9"),
        })
}

pub fn establecer_tecla_grabar_macro(valor: AtajoSimple) {
    *TECLA_GRABAR_MACRO.lock().unwrap() = Some(valor);
}

// ======================================================
// ⏱️ INTERVALO CAPTURA COORDENADA (polling ventana overlay)
// ======================================================

static INTERVALO_CAPTURA_COORDENADA: AtomicU64 = AtomicU64::new(100);

pub fn intervalo_captura_coordenada() -> u64 {
    INTERVALO_CAPTURA_COORDENADA.load(Ordering::Relaxed)
}

pub fn establecer_intervalo_captura_coordenada(valor: u64) {
    INTERVALO_CAPTURA_COORDENADA.store(valor, Ordering::Relaxed);
}

// ======================================================
// ⏱️ DELAY ENTRE SALIDA DOBLE (Acción con condición Doble)
// ======================================================

static DELAY_ENTRE_SALIDA_DOBLE: AtomicU64 = AtomicU64::new(30);

pub fn delay_entre_salida_doble() -> u64 {
    DELAY_ENTRE_SALIDA_DOBLE.load(Ordering::Relaxed)
}

pub fn establecer_delay_entre_salida_doble(valor: u64) {
    DELAY_ENTRE_SALIDA_DOBLE.store(valor, Ordering::Relaxed);
}

// ======================================================
// 🖱️ DELAY ENTRE REPETICIONES DE RUEDA (Extra RepeticionRueda)
// ======================================================

static DELAY_RUEDA_REPETICION: AtomicU64 = AtomicU64::new(30);

pub fn delay_rueda_repeticion() -> u64 {
    DELAY_RUEDA_REPETICION.load(Ordering::Relaxed)
}

pub fn establecer_delay_rueda_repeticion(valor: u64) {
    DELAY_RUEDA_REPETICION.store(valor, Ordering::Relaxed);
}

// ======================================================
// ⏱️ TIEMPO SIMPLE TECLAS (pausa entre DOWN y UP de un toque)
// ======================================================

static TIEMPO_SIMPLE_TECLAS: AtomicU64 = AtomicU64::new(0);

pub fn tiempo_simple_teclas() -> u64 {
    TIEMPO_SIMPLE_TECLAS.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_simple_teclas(valor: u64) {
    TIEMPO_SIMPLE_TECLAS.store(valor, Ordering::Relaxed);
}

// ======================================================
// 🔊 DELTA DE VOLUMEN (Acción Multimedia, alcance En App)
// ------------------------------------------------------
// Cuánto sube/baja el volumen de la sesión de audio de un programa
// por cada pulsación de Subir/Bajar en alcance "En App"
// (back_multimedia.rs, vía winmix). El alcance Global no usa este
// valor — VK_VOLUME_UP/DOWN los maneja Windows con su propio paso
// nativo, no aceptan un delta custom.
// ======================================================

static DELTA_VOLUMEN: AtomicU64 = AtomicU64::new(10);

pub fn delta_volumen() -> u64 {
    DELTA_VOLUMEN.load(Ordering::Relaxed)
}

pub fn establecer_delta_volumen(valor: u64) {
    DELTA_VOLUMEN.store(valor, Ordering::Relaxed);
}

// ======================================================
// ⚡🪟 MENU EXPRESS — TAMAÑOS DE BOTÓN
// ------------------------------------------------------
// Ancho/alto en px para cada uno de los 3 tamaños de botón
// (menu_extra.tamanoBoton). Antes vivían hardcodeados en
// menu_express.css / TAMANOS_BOTON_PX (menu_express_main.ts) /
// TAMANOS_BOTON_PX (back_menu_express.rs) — desde acá pasan a
// ser la única fuente de verdad real y configurable; los otros
// dos lugares los LEEN por comando/consulta en vez de tener su
// propia copia fija (ver obtener_tamanos_menu_express en
// comandos.rs y calcular_tamano_ventana en back_menu_express.rs).
// Valores por defecto: los mismos que ya venían de la etapa 5/6.
// ======================================================

static MENU_BOTON_PEQUENO_ANCHO: AtomicU64 = AtomicU64::new(60);
static MENU_BOTON_PEQUENO_ALTO: AtomicU64 = AtomicU64::new(30);

static MENU_BOTON_MEDIANO_ANCHO: AtomicU64 = AtomicU64::new(80);
static MENU_BOTON_MEDIANO_ALTO: AtomicU64 = AtomicU64::new(40);

static MENU_BOTON_GRANDE_ANCHO: AtomicU64 = AtomicU64::new(100);
static MENU_BOTON_GRANDE_ALTO: AtomicU64 = AtomicU64::new(50);

pub fn menu_boton_pequeno() -> (u64, u64) {
    (
        MENU_BOTON_PEQUENO_ANCHO.load(Ordering::Relaxed),
        MENU_BOTON_PEQUENO_ALTO.load(Ordering::Relaxed),
    )
}

pub fn establecer_menu_boton_pequeno(ancho: u64, alto: u64) {
    MENU_BOTON_PEQUENO_ANCHO.store(ancho, Ordering::Relaxed);
    MENU_BOTON_PEQUENO_ALTO.store(alto, Ordering::Relaxed);
}

pub fn menu_boton_mediano() -> (u64, u64) {
    (
        MENU_BOTON_MEDIANO_ANCHO.load(Ordering::Relaxed),
        MENU_BOTON_MEDIANO_ALTO.load(Ordering::Relaxed),
    )
}

pub fn establecer_menu_boton_mediano(ancho: u64, alto: u64) {
    MENU_BOTON_MEDIANO_ANCHO.store(ancho, Ordering::Relaxed);
    MENU_BOTON_MEDIANO_ALTO.store(alto, Ordering::Relaxed);
}

pub fn menu_boton_grande() -> (u64, u64) {
    (
        MENU_BOTON_GRANDE_ANCHO.load(Ordering::Relaxed),
        MENU_BOTON_GRANDE_ALTO.load(Ordering::Relaxed),
    )
}

pub fn establecer_menu_boton_grande(ancho: u64, alto: u64) {
    MENU_BOTON_GRANDE_ANCHO.store(ancho, Ordering::Relaxed);
    MENU_BOTON_GRANDE_ALTO.store(alto, Ordering::Relaxed);
}

// ======================================================
// ⚡🔤 MENU EXPRESS — TAMAÑOS DE TEXTO
// ------------------------------------------------------
// Tamaño de fuente en px para cada uno de los 3 tamaños de texto
// (menu_extra.tamanoTexto). Mismo criterio que los de botón — ver
// comentario arriba.
// ======================================================

static MENU_TEXTO_PEQUENO: AtomicU64 = AtomicU64::new(10);
static MENU_TEXTO_MEDIANO: AtomicU64 = AtomicU64::new(13);
static MENU_TEXTO_GRANDE: AtomicU64 = AtomicU64::new(16);

pub fn menu_texto_pequeno() -> u64 {
    MENU_TEXTO_PEQUENO.load(Ordering::Relaxed)
}

pub fn establecer_menu_texto_pequeno(valor: u64) {
    MENU_TEXTO_PEQUENO.store(valor, Ordering::Relaxed);
}

pub fn menu_texto_mediano() -> u64 {
    MENU_TEXTO_MEDIANO.load(Ordering::Relaxed)
}

pub fn establecer_menu_texto_mediano(valor: u64) {
    MENU_TEXTO_MEDIANO.store(valor, Ordering::Relaxed);
}

pub fn menu_texto_grande() -> u64 {
    MENU_TEXTO_GRANDE.load(Ordering::Relaxed)
}

pub fn establecer_menu_texto_grande(valor: u64) {
    MENU_TEXTO_GRANDE.store(valor, Ordering::Relaxed);
}

// ======================================================
// 📋 PORTAPAPELES — TAMAÑOS DE BOTÓN
// ------------------------------------------------------
// Ancho/alto en px para cada uno de los 3 tamaños de botón
// (portapapeles_extra.tamanoBoton). Valores PROPIOS (no reusa
// menu_boton_*): los botones de Portapapeles son filas alargadas
// (ícono + nombre + acciones), no cuadrados como los de MenuExpress
// — mismo criterio de "única fuente de verdad configurable" que ya
// aplica ahí (ver comentario de MENU EXPRESS — TAMAÑOS DE BOTÓN más
// arriba); portapapeles.css / portapapeles_main.ts /
// back_portapapeles.rs (etapas D en adelante) los van a LEER por
// comando/consulta en vez de tener su propia copia fija.
//
// El tamaño de TEXTO de Portapapeles no tiene su propia sección acá
// — reusa menu_texto_pequeno/mediano/grande tal cual (ver
// core_portapapeles.ts / AccionCache::Portapapeles en
// perfil_cache.rs), así que no hace falta duplicar esos 3 valores.
// ======================================================

static PORTAPAPELES_BOTON_PEQUENO_ANCHO: AtomicU64 = AtomicU64::new(140);
static PORTAPAPELES_BOTON_PEQUENO_ALTO: AtomicU64 = AtomicU64::new(26);

static PORTAPAPELES_BOTON_MEDIANO_ANCHO: AtomicU64 = AtomicU64::new(180);
static PORTAPAPELES_BOTON_MEDIANO_ALTO: AtomicU64 = AtomicU64::new(32);

static PORTAPAPELES_BOTON_GRANDE_ANCHO: AtomicU64 = AtomicU64::new(220);
static PORTAPAPELES_BOTON_GRANDE_ALTO: AtomicU64 = AtomicU64::new(40);

pub fn portapapeles_boton_pequeno() -> (u64, u64) {
    (
        PORTAPAPELES_BOTON_PEQUENO_ANCHO.load(Ordering::Relaxed),
        PORTAPAPELES_BOTON_PEQUENO_ALTO.load(Ordering::Relaxed),
    )
}

pub fn establecer_portapapeles_boton_pequeno(ancho: u64, alto: u64) {
    PORTAPAPELES_BOTON_PEQUENO_ANCHO.store(ancho, Ordering::Relaxed);
    PORTAPAPELES_BOTON_PEQUENO_ALTO.store(alto, Ordering::Relaxed);
}

pub fn portapapeles_boton_mediano() -> (u64, u64) {
    (
        PORTAPAPELES_BOTON_MEDIANO_ANCHO.load(Ordering::Relaxed),
        PORTAPAPELES_BOTON_MEDIANO_ALTO.load(Ordering::Relaxed),
    )
}

pub fn establecer_portapapeles_boton_mediano(ancho: u64, alto: u64) {
    PORTAPAPELES_BOTON_MEDIANO_ANCHO.store(ancho, Ordering::Relaxed);
    PORTAPAPELES_BOTON_MEDIANO_ALTO.store(alto, Ordering::Relaxed);
}

pub fn portapapeles_boton_grande() -> (u64, u64) {
    (
        PORTAPAPELES_BOTON_GRANDE_ANCHO.load(Ordering::Relaxed),
        PORTAPAPELES_BOTON_GRANDE_ALTO.load(Ordering::Relaxed),
    )
}

pub fn establecer_portapapeles_boton_grande(ancho: u64, alto: u64) {
    PORTAPAPELES_BOTON_GRANDE_ANCHO.store(ancho, Ordering::Relaxed);
    PORTAPAPELES_BOTON_GRANDE_ALTO.store(alto, Ordering::Relaxed);
}

// ======================================================
// 📋 PORTAPAPELES — TIMERS DE PEGADO
// ------------------------------------------------------
// Los 3 tiempos de espera involucrados en pegar() (back_portapapeles.
// rs) y en el camino personalizado de Photoshop (back_pegado_
// personalizado.rs). Antes hardcodeados, ahora únicos valores de
// verdad y configurables desde acá — mismo criterio que el resto de
// este archivo.
// ======================================================

static TIEMPO_IGNORAR_CAMBIO_PORTAPAPELES: AtomicU64 = AtomicU64::new(600);
static TIEMPO_ESPERA_PEGADO_IMAGEN: AtomicU64 = AtomicU64::new(450);
static TIEMPO_ESPERA_PEGADO_TEXTO: AtomicU64 = AtomicU64::new(50);
static DELAY_IMAGEN_PHOTOSHOP: AtomicU64 = AtomicU64::new(1350);

pub fn tiempo_ignorar_cambio_portapapeles() -> u64 {
    TIEMPO_IGNORAR_CAMBIO_PORTAPAPELES.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_ignorar_cambio_portapapeles(valor: u64) {
    TIEMPO_IGNORAR_CAMBIO_PORTAPAPELES.store(valor, Ordering::Relaxed);
}

pub fn tiempo_espera_pegado_imagen() -> u64 {
    TIEMPO_ESPERA_PEGADO_IMAGEN.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_espera_pegado_imagen(valor: u64) {
    TIEMPO_ESPERA_PEGADO_IMAGEN.store(valor, Ordering::Relaxed);
}

pub fn tiempo_espera_pegado_texto() -> u64 {
    TIEMPO_ESPERA_PEGADO_TEXTO.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_espera_pegado_texto(valor: u64) {
    TIEMPO_ESPERA_PEGADO_TEXTO.store(valor, Ordering::Relaxed);
}

pub fn delay_imagen_photoshop() -> u64 {
    DELAY_IMAGEN_PHOTOSHOP.load(Ordering::Relaxed)
}

pub fn establecer_delay_imagen_photoshop(valor: u64) {
    DELAY_IMAGEN_PHOTOSHOP.store(valor, Ordering::Relaxed);
}
