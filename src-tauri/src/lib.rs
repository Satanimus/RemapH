// ======================================================
// 🚀 src-tauri/src Lib.rs
// ------------------------------------------------------
// Punto de entrada del backend.
//
// Inicializa el motor principal.
// ======================================================

mod analizador_trigger;
mod back_app;
mod back_coordenada;
mod back_interception;
mod back_mouse;
mod back_teclas;
mod cache;
mod captura_coordenada;
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
            comandos::abrir_selector_emoji,
            comandos::abrir_ventana_captura_coordenada,
            comandos::cerrar_ventana_captura_coordenada,
            comandos::obtener_cursor_captura,
            comandos::obtener_ventana_activa_captura,
            comandos::consultar_guardado_coordenada,
            comandos::guardar_resultado_coordenada,
            comandos::obtener_resultado_coordenada,
            comandos::obtener_tecla_guardar_coordenada,
            comandos::establecer_tecla_guardar_coordenada,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar Tauri");
}
