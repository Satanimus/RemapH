// ======================================================
// 🚀 src-tauri/src Lib.rs
// ------------------------------------------------------
// Punto de entrada del backend.
//
// Inicializa el motor principal.
// ======================================================

mod ayuda;
mod back_app;
mod back_coordenada;
mod banco_coordenadas;
mod back_interception;
mod back_menu_express;
mod back_mouse;
mod back_multimedia;
mod back_pegado_personalizado;
mod back_portapapeles;
mod back_portapapeles_captura;
mod back_registro;
mod back_teclas;
mod back_windows;
mod cache;
mod captura_coordenada;
mod comandos;
mod compilador;
mod config;
mod configuracion_usuario;
mod entrada;
mod eventos;
mod grabacion_macro;
mod macro_cache;
mod macro_json;
mod macro_usuario;
mod macros;
mod motor;
mod perfil;
mod perfil_cache;
mod perfil_json;
mod perfil_ui;
mod pulsadores;
mod runt_extra;
mod runt_macro;
mod runtime;
mod usuario;

// ======================================================
// 🚀 INICIO TAURI
// ======================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    // Carga el modo de motor guardado en Configuracion_Usuario.txt
    // antes de arrancar el hilo de entrada — si el usuario eligió
    // Portable en la sesión anterior, el backend correcto arranca
    // desde el primer evento físico.
    motor::cargar_modo_desde_config();

    // El punto único de despacho (motor.rs) decide, según el modo
    // activo, si arranca back_interception (con su propia precarga
    // de dispositivo primario) o back_windows — ver motor::iniciar().
    // Tiene que ir antes de que arranque el hilo de entrada, no
    // adentro: ver comentario largo en back_interception.rs
    // ("Reglas de dispositivo primario") y en motor::iniciar().
    std::thread::spawn(|| {
        motor::iniciar(entrada::procesar_evento, cache::captura_activa);
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

            // Aplica sobre config.rs los overrides guardados en
            // Configuracion_Usuario.txt (pestaña General de la
            // Ventana de Configuración). Debe ir después de que
            // exista la carpeta de usuario, pero no depende de
            // ninguna ventana — se hace apenas arranca.
            configuracion_usuario::cargar_al_iniciar();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comandos::compilar_perfil,
            comandos::activar_perfil,
            comandos::desactivar_perfil,
            comandos::iniciar_captura,
            comandos::obtener_captura,
            comandos::coincide_con_atajo_reservado,
            comandos::obtener_perfil_actual,
            comandos::obtener_perfiles,
            comandos::traducir_pulsador,
            comandos::traducir_pulsador_lote,
            comandos::obtener_nombre_perfil_actual,
            comandos::obtener_estado_cache,
            comandos::restaurar_perfil_actual,
            comandos::guardar_perfil_como,
            comandos::renombrar_perfil,
            comandos::eliminar_perfil_actual,
            comandos::crear_perfil_nuevo,
            comandos::seleccionar_perfil,
            comandos::macro_listar,
            comandos::macro_nueva,
            comandos::macro_abrir,
            comandos::macro_guardar,
            comandos::macro_guardar_paso,
            comandos::macro_cancelar,
            comandos::macro_guardar_como,
            comandos::macro_renombrar,
            comandos::macro_eliminar,
            comandos::listar_procesos_ventana,
            comandos::obtener_icono_programa,
            comandos::obtener_icono_ruta,
            comandos::seleccionar_archivo,
            comandos::seleccionar_carpeta,
            comandos::obtener_programas_abrir_con,
            comandos::obtener_tiempo_doble,
            comandos::establecer_tiempo_doble,
            comandos::obtener_tiempo_mantenido,
            comandos::abrir_selector_emoji,
            comandos::abrir_ventana_captura_coordenada,
            comandos::cerrar_ventana_captura_coordenada,
            comandos::abrir_ventana_preview_coordenada,
            comandos::cerrar_ventana_preview_coordenada,
            comandos::obtener_destino_preview_coordenada,
            comandos::obtener_xy_preview_coordenada,
            comandos::actualizar_xy_preview_en_vivo,
            comandos::guardar_posicion_preview_coordenada,
            comandos::probar_coordenada,
            comandos::obtener_cursor_captura,
            comandos::obtener_ventana_activa_captura,
            comandos::obtener_config_captura_activa,
            comandos::obtener_config_preview_coordenada,
            comandos::consultar_guardado_coordenada,
            comandos::guardar_resultado_coordenada,
            comandos::obtener_resultado_coordenada,
            comandos::obtener_tecla_guardar_coordenada,
            comandos::establecer_tecla_guardar_coordenada,
            comandos::obtener_tecla_grabar_macro,
            comandos::establecer_tecla_grabar_macro,
            comandos::abrir_ventana_grabacion_macro,
            comandos::cerrar_ventana_grabacion_macro,
            comandos::armar_grabacion_macro,
            comandos::obtener_estado_grabacion_macro,
            comandos::detener_grabacion_macro,
            comandos::tomar_eventos_grabacion_macro,
            comandos::obtener_tecla_toggle_perfil,
            comandos::establecer_tecla_toggle_perfil,
            comandos::obtener_intervalo_captura_coordenada,
            comandos::abrir_ventana_coordenadas,
            comandos::coordenadas_listar,
            comandos::coordenadas_agregar,
            comandos::coordenadas_editar,
            comandos::coordenadas_eliminar,
            comandos::coordenadas_listar_grupos,
            comandos::coordenadas_reordenar,
            comandos::seleccionar_coordenada_banco,
            comandos::obtener_seleccion_coordenada_banco,
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
            comandos::portapapeles_listar_otros,
            comandos::portapapeles_listar_fijados_de,
            comandos::portapapeles_fijar_como,
            comandos::obtener_tamanos_portapapeles,
            comandos::establecer_portapapeles_boton_pequeno,
            comandos::establecer_portapapeles_boton_mediano,
            comandos::establecer_portapapeles_boton_grande,
            comandos::abrir_ventana_configuracion,
            comandos::configuracion_listar_general,
            comandos::configuracion_guardar_lote,
            comandos::configuracion_restablecer_seccion,
            comandos::configuracion_restablecer_claves,
            comandos::configuracion_listar_teclas,
            comandos::configuracion_guardar_lote_teclas,
            comandos::configuracion_listar_apariencia,
            comandos::configuracion_guardar_lote_apariencia,
            comandos::configuracion_restablecer_claves_css,
            comandos::configuracion_guardar_tema,
            comandos::configuracion_cargar_tema,
            comandos::configuracion_tema_listar,
            comandos::configuracion_tema_cargar,
            comandos::configuracion_tema_guardar_como,
            comandos::configuracion_tema_guardar_editado,
            comandos::configuracion_tema_renombrar,
            comandos::configuracion_tema_eliminar,
            comandos::configuracion_apariencia_iniciar_sesion,
            comandos::abrir_carpeta_usuario,
            comandos::obtener_overrides_apariencia,
            comandos::configuracion_refrescar_ventanas_apariencia,
            comandos::motor_obtener_modo,
            comandos::motor_solicitar_cambio_modo,
            comandos::obtener_ayuda,
            comandos::obtener_ancho_panel_ayuda,
            comandos::establecer_ancho_panel_ayuda,
            comandos::obtener_visible_panel_ayuda,
            comandos::establecer_visible_panel_ayuda,
            comandos::obtener_primer_inicio_ayuda,
        ])
        .build(tauri::generate_context!())
        .expect("error al construir Tauri")
        // Etapa 8C: enganche al cierre del programa (antes no había
        // ninguno) — ExitRequested se dispara apenas el programa
        // empieza a cerrarse (con el proceso todavía completo), antes
        // de que Tauri termine de desmontar nada, así que es el punto
        // seguro para runtime::detener_todo() (soltar teclas que
        // hayan quedado físicamente abajo, cortar ejecuciones activas
        // — ver runtime.rs).
        .run(|_app_handle, evento| {
            if let tauri::RunEvent::ExitRequested { .. } = evento {
                runtime::detener_todo();
            }
        });
}
