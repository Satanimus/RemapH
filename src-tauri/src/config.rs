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
