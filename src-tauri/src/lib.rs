// ======================================================
// 🚀 src-tauri/src Lib.rs RemapH V3
// ------------------------------------------------------
// Punto de entrada del backend.
//
// Inicializa el motor principal.
// ======================================================


mod backend;


mod cache;


mod captura;


mod comandos;


mod compilador;


mod config;


mod eventos;


mod entrada;


mod estado;


pub mod idioma;


mod perfilcache;


mod persistencia;


mod reentrada;


mod runtime;


mod perfiljson;


mod usuario;


// ======================================================
// 🚀 INICIO TAURI
// ======================================================

#[cfg_attr(

    mobile,

    tauri::mobile_entry_point

)]

pub fn run() {


    entrada::iniciar();


    tauri::Builder::default()


        .invoke_handler(

            tauri::generate_handler![

                comandos::compilar_perfil,

                comandos::activar_perfil,

                comandos::desactivar_perfil,

                comandos::iniciar_captura,

                comandos::obtener_captura,

                comandos::obtener_perfil_actual,

                comandos::obtener_estados_cache_perfiles,

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

            ]

        )


        .run(

            tauri::generate_context!()

        )


        .expect(

            "error al ejecutar Tauri"

        );

}