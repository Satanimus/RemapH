// ======================================================
// 🚀 src-tauri/src Lib.rs RemapH V3
// ------------------------------------------------------
// Punto de entrada del backend.
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
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
            ]
        )
        .run(
            tauri::generate_context!()
        )
        .expect(
            "error al ejecutar Tauri"
        );

}