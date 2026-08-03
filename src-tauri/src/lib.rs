// ======================================================
// 🚀 src-tauri/src Lib.rs RemapH V3
// ------------------------------------------------------
// Punto de entrada del backend.
//
// Inicializa el motor principal.
// ======================================================

mod analizador_trigger;
mod back_app;
mod back_interception;
mod back_mouse;
mod back_teclas;
mod cache;
mod comandos;
mod compilador;
mod config;
mod entrada;
mod eventos;
mod instante;
mod perfil;
mod perfil_cache;
mod perfil_json;
mod perfil_ui;
mod pulsadores;
mod runt_extra;
mod runtime;
mod usuario;

// ======================================================
// 🚀 INICIO TAURI
// ======================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    std::thread::spawn(|| {
        back_interception::iniciar(entrada::procesar_evento, analizador_trigger::captura_activa);
    });
    back_app::iniciar_monitor();
    tauri::Builder::default()
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .invoke_handler(tauri::generate_handler![
            comandos::compilar_perfil,
            comandos::activar_perfil,
            comandos::desactivar_perfil,
            comandos::iniciar_captura,
            comandos::obtener_captura,
            comandos::obtener_perfil_actual,
            comandos::obtener_perfiles,
            comandos::obtener_nombre_perfil_actual,
            comandos::obtener_estado_cache,
            comandos::restaurar_perfil_actual,
            comandos::clonar_perfil,
            comandos::renombrar_perfil,
            comandos::eliminar_perfil_actual,
            comandos::crear_perfil_nuevo,
            comandos::seleccionar_perfil,
            comandos::listar_procesos_ventana,
            comandos::obtener_icono_programa,
            comandos::obtener_tiempo_doble,
            comandos::establecer_tiempo_doble,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar Tauri");
}
