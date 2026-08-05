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
// tiempo_mantenido()
// establecer_tiempo_mantenido()
//     Tiempo mínimo para detectar Mantenido.
//
// tiempo_repeticion()
// establecer_tiempo_repeticion()
//     Intervalo entre repeticiones de Turbo.
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
// tecla_guardar_coordenada()
// establecer_tecla_guardar_coordenada()
//     Tecla que la ventana de captura de "Click en coordenada"
//     escucha para guardar la posición actual ("F1" por defecto).
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
// tiempo_salida_mantenido()
// establecer_tiempo_salida_mantenido()
//     Cuánto queda abajo la tecla/botón de salida cuando la Acción
//     capturada tiene condición Mantenido, antes de soltarse sola.
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

// ======================================================
// 📦 APP
// ======================================================

pub const NOMBRE_APP: &str = "RemapH";

// ======================================================
// ⏱️ TIEMPO DOBLE
// ======================================================

static TIEMPO_DOBLE: AtomicU64 = AtomicU64::new(250);

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
// 📌 TECLA GUARDAR COORDENADA (ventana de captura)
// ------------------------------------------------------
// Código interno de tecla (mismo vocabulario que
// pulsadores.tsv, ej. "F1") que la ventana de captura de
// "Click en coordenada" escucha para guardar la posición
// actual. Configurable, no fija — ver captura_coordenada.rs
// (quien la usa) y comandos.rs (quien la expone a la UI).
// ======================================================

static TECLA_GUARDAR_COORDENADA: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

pub fn tecla_guardar_coordenada() -> String {
    let valor = TECLA_GUARDAR_COORDENADA.lock().unwrap();

    if valor.is_empty() {
        "F1".to_string()
    } else {
        valor.clone()
    }
}

pub fn establecer_tecla_guardar_coordenada(valor: String) {
    *TECLA_GUARDAR_COORDENADA.lock().unwrap() = valor;
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
// ⏳ TIEMPO SALIDA MANTENIDO (Acción con condición Mantenido)
// ======================================================

static TIEMPO_SALIDA_MANTENIDO: AtomicU64 = AtomicU64::new(300);

pub fn tiempo_salida_mantenido() -> u64 {
    TIEMPO_SALIDA_MANTENIDO.load(Ordering::Relaxed)
}

pub fn establecer_tiempo_salida_mantenido(valor: u64) {
    TIEMPO_SALIDA_MANTENIDO.store(valor, Ordering::Relaxed);
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
