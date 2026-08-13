// ======================================================
// 🚀 src-tauri/src Lib.rs
// ------------------------------------------------------
// Punto de entrada del backend.
//
// Inicializa el motor principal.
// ======================================================

mod back_app;
mod back_coordenada;
mod back_interception;
mod back_menu_express;
mod back_mouse;
mod back_multimedia;
mod back_portapapeles;
mod back_portapapeles_captura;
mod back_registro;
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
        back_interception::iniciar(entrada::procesar_evento, cache::captura_activa);
    });
    back_app::iniciar_monitor();
    tauri::Builder::default()
        .device_event_filter(tauri::DeviceEventFilter::Always)
        .setup(|app| {
            // AppHandle global para back_menu_express.rs — el trigger
            // que abre una ventana MenuExpress llega desde el hilo de
            // entrada física (ver runtime.rs), no desde un comando
            // Tauri, así que no hay forma de recibirlo como parámetro
            // en ese momento. Se fija acá, una única vez, apenas Tauri
            // termina de inicializar.
            back_menu_express::inicializar(app.handle().clone());

            // Mismo motivo/momento que arriba, para back_portapapeles.rs
            // (Etapa G/H) — abrir_o_alternar() también va a llegar
            // desde el hilo de entrada física (Etapa I), no desde un
            // comando Tauri.
            back_portapapeles::inicializar(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comandos::compilar_perfil,
            comandos::activar_perfil,
            comandos::desactivar_perfil,
            comandos::iniciar_captura,
            comandos::obtener_captura,
            comandos::obtener_perfil_actual,
            comandos::obtener_perfiles,
            comandos::traducir_pulsador,
            comandos::traducir_pulsador_lote,
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
            comandos::obtener_icono_ruta,
            comandos::seleccionar_archivo,
            comandos::seleccionar_carpeta,
            comandos::obtener_programas_abrir_con,
            comandos::obtener_tiempo_doble,
            comandos::establecer_tiempo_doble,
            comandos::abrir_selector_emoji,
            comandos::abrir_ventana_captura_coordenada,
            comandos::cerrar_ventana_captura_coordenada,
            comandos::obtener_cursor_captura,
            comandos::obtener_ventana_activa_captura,
            comandos::obtener_config_captura_activa,
            comandos::consultar_guardado_coordenada,
            comandos::guardar_resultado_coordenada,
            comandos::obtener_resultado_coordenada,
            comandos::obtener_tecla_guardar_coordenada,
            comandos::establecer_tecla_guardar_coordenada,
            comandos::obtener_intervalo_captura_coordenada,
            comandos::obtener_datos_menu_express,
            comandos::cerrar_menu_express,
            comandos::menu_express_boton_down,
            comandos::menu_express_boton_up,
            comandos::obtener_tamanos_menu_express,
            comandos::establecer_menu_boton_pequeno,
            comandos::establecer_menu_boton_mediano,
            comandos::establecer_menu_boton_grande,
            comandos::establecer_menu_texto_pequeno,
            comandos::establecer_menu_texto_mediano,
            comandos::establecer_menu_texto_grande,
            comandos::obtener_datos_portapapeles,
            comandos::cerrar_portapapeles,
            comandos::portapapeles_toggle_registro,
            comandos::portapapeles_fijar,
            comandos::portapapeles_desfijar,
            comandos::portapapeles_renombrar,
            comandos::portapapeles_editar,
            comandos::portapapeles_marcar_reciente,
            comandos::portapapeles_enfocar_ventana,
            comandos::portapapeles_desenfocar_ventana,
            comandos::portapapeles_eliminar,
            comandos::portapapeles_limpiar_todo,
            comandos::portapapeles_pegar,
            comandos::obtener_tamanos_portapapeles,
            comandos::establecer_portapapeles_boton_pequeno,
            comandos::establecer_portapapeles_boton_mediano,
            comandos::establecer_portapapeles_boton_grande,
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar Tauri");
}
