// ======================================================
// 🎮 Comandos Tauri RemapH V3
// ======================================================
// ETAPA UI DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Punto de entrada entre TypeScript y el backend.
//
// Su responsabilidad:
//
// - Exponer funciones mediante Tauri.
// - Recibir datos desde UI.
// - Convertir modelos UI.
// - Devolver resultados serializables.
//
// Comandos NO:
//
// - Gestiona perfiles.
// - Gestiona cache.
// - Ejecuta Runtime.
// - Procesa entradas.
// - Accede directamente a Windows.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// - Solicitudes Tauri.
// - Datos enviados desde TypeScript.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Tauri.
//
// Flujo:
//
// TypeScript
//      ↓
// comandos.rs
//      ↓
// Módulo correspondiente
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Entrega:
//
// - Respuestas serializables.
// - Modelos preparados para UI.
// - Datos de captura.
// - Datos de aplicaciones.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// ======================================================
// 🧩 MODELOS UI
// ======================================================
//
// AppUI
//     Modelo de aplicación recibido desde UI.
//
// FilaUI
//     Modelo completo de una fila editable.
//
// TriggerUI
//     Modelo de trigger recibido desde UI.
//
// EntradaUI
//     Modelo de entrada recibido desde UI.
//
// EntradaCapturaUI
//     Modelo de entrada mostrado en captura.
//
// TriggerCapturaUI
//     Modelo de trigger mostrado en captura.
//
// ResultadoPerfil
//     Respuesta de perfil hacia UI.
//
// EstadoCachePerfil
//     Estado visual de cache de perfil.
//
// IconoJson
//     Modelo serializable de icono.
//
// ProcesoIconoJson
//     Modelo serializable de proceso.
//
// ======================================================
// 🎹 CAPTURA
// ======================================================
//
// iniciar_captura()
//
//     Solicita iniciar captura.
//
// obtener_captura()
//
//     Devuelve captura actual.
//
// convertir_input_captura()
//
//     Convierte InputId interno a formato UI.
//
// convertir_trigger_captura()
//
//     Convierte EventoTrigger interno a formato UI.
//
// ======================================================
// 🖥️ APLICACIONES
// ======================================================
//
// convertir_icono()
//
//     Convierte IconoRaw interno a formato UI.
//
// listar_procesos_ventana()
//
//     Entrega procesos disponibles para selector UI.
//
// obtener_icono_programa()
//
//     Entrega icono de programa.
//
// ======================================================

use crate::back_app;
use crate::config;
use crate::perfil;
use crate::perfil_json::perfil_json;
use crate::perfil_ui::{convertir_perfil, FilaUI, ResultadoPerfil, TriggerCapturaUI};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde::Serialize;

// ======================================================
// 🎹 COMANDOS PERFIL
// ======================================================

#[tauri::command]
pub fn activar_perfil() -> Result<bool, String> {
    perfil::activar_perfil()
}

#[tauri::command]
pub fn desactivar_perfil() {
    perfil::desactivar_perfil();
}

#[tauri::command]
pub fn obtener_perfil_actual() -> Result<perfil_json, String> {
    perfil::obtener_perfil_actual()
}

#[tauri::command]
pub fn obtener_perfiles() -> Result<Vec<String>, String> {
    perfil::obtener_perfiles()
}

#[tauri::command]
pub fn obtener_nombre_perfil_actual() -> Result<String, String> {
    perfil::obtener_nombre_actual()
}

#[tauri::command]
pub fn obtener_estado_cache() -> bool {
    perfil::obtener_estado_cache()
}

#[tauri::command]
pub fn restaurar_perfil_actual() -> Result<ResultadoPerfil, String> {
    perfil::restaurar_perfil_actual()
}

#[tauri::command]
pub fn crear_perfil_nuevo() -> Result<ResultadoPerfil, String> {
    perfil::crear_perfil_nuevo()
}

#[tauri::command]
pub fn seleccionar_perfil(nombre: String) -> Result<ResultadoPerfil, String> {
    perfil::seleccionar_perfil(nombre)
}

#[tauri::command]
pub fn renombrar_perfil(nuevo_nombre: String) -> Result<ResultadoPerfil, String> {
    perfil::renombrar_perfil(nuevo_nombre)
}

#[tauri::command]
pub fn eliminar_perfil_actual() -> Result<ResultadoPerfil, String> {
    perfil::eliminar_perfil_actual()
}

#[tauri::command]
pub fn compilar_perfil(filas: Vec<FilaUI>) -> Result<bool, String> {
    let perfil = convertir_perfil(filas);

    perfil::guardar_perfil(perfil)
}

#[tauri::command]
pub fn clonar_perfil(filas: Vec<FilaUI>) -> Result<ResultadoPerfil, String> {
    let perfil = convertir_perfil(filas);

    perfil::clonar_perfil(perfil)
}

#[tauri::command]
pub fn obtener_tiempo_doble() -> u64 {
    config::tiempo_doble()
}

#[tauri::command]
pub fn establecer_tiempo_doble(valor: u64) {
    config::establecer_tiempo_doble(valor)
}

// ======================================================
// 🎹 CAPTURA
// ======================================================

#[tauri::command]
pub fn iniciar_captura(fila_id: String, columna: String) {
    crate::perfil_ui::iniciar_captura(fila_id, columna);

    println!("🎹 Captura iniciada");
}

#[tauri::command]
pub fn obtener_captura() -> Option<(String, String, TriggerCapturaUI)> {
    crate::perfil_ui::obtener_captura()
}

// ======================================================
// 🖼️ MODELOS DE ÍCONO
// ======================================================

#[derive(Serialize)]
pub struct IconoJson {
    pub ancho: u32,

    pub alto: u32,

    pub pixeles: String,
}

#[derive(Serialize)]
pub struct ProcesoIconoJson {
    pub nombre: String,

    pub icono: Option<IconoJson>,
}

// ======================================================
// 🖥️ APLICACIONES / ICONOS
// ======================================================

fn convertir_icono(icono: back_app::IconoRaw) -> IconoJson {
    IconoJson {
        ancho: icono.ancho,

        alto: icono.alto,

        pixeles: BASE64.encode(icono.pixeles),
    }
}

// ======================================================
// 📋 LISTAR PROCESOS
// ======================================================

#[tauri::command]
pub fn listar_procesos_ventana() -> Vec<ProcesoIconoJson> {
    back_app::enumerar_procesos_ventana()
        .into_iter()
        .map(|proceso| {
            let icono = back_app::extraer_icono(&proceso.ruta).map(convertir_icono);

            ProcesoIconoJson {
                nombre: proceso.nombre,

                icono,
            }
        })
        .collect()
}

// ======================================================
// 🎨 OBTENER ICONO PROGRAMA
// ======================================================

#[tauri::command]
pub fn obtener_icono_programa(nombre: String) -> Option<IconoJson> {
    let proceso = back_app::enumerar_procesos_ventana()
        .into_iter()
        .find(|proceso| proceso.nombre.eq_ignore_ascii_case(&nombre))?;

    back_app::extraer_icono(&proceso.ruta).map(convertir_icono)
}
