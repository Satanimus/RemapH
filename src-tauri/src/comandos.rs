// ======================================================
// 🎮 Comandos Tauri
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
//     Devuelve captura actual. El trigger es Option: None
//     significa "hubo un resultado, pero se descartó" (ver
//     perfil_ui::recibir_condicion).
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
use crate::back_coordenada;
use crate::captura_coordenada;
use crate::config;
use crate::perfil;
use crate::perfil_json::perfil_json;
use crate::perfil_ui::{convertir_perfil, FilaUI, ResultadoPerfil, TriggerCapturaUI};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde::Serialize;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

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
pub fn obtener_captura() -> Option<(String, String, Option<TriggerCapturaUI>)> {
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

// ======================================================
// 😎 SELECTOR EMOJI
// ======================================================
#[tauri::command]
pub fn abrir_selector_emoji() {
    unsafe {
        let mut inputs: [INPUT; 4] = std::mem::zeroed();

        // Win down
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki.wVk = VK_LWIN;

        // . down
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki.wVk = 0xBE;

        // . up
        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki.wVk = 0xBE;
        inputs[2].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        // Win up
        inputs[3].r#type = INPUT_KEYBOARD;
        inputs[3].Anonymous.ki.wVk = VK_LWIN;
        inputs[3].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

// ======================================================
// 🖱️📌 CLICK EN COORDENADA — VENTANA DE CAPTURA
// ------------------------------------------------------
// abrir_ventana_captura_coordenada() / cerrar_...()
//     Crea/destruye la ventana overlay bajo demanda (no
//     vive montada). Al abrir, activa el tap pasivo de
//     captura_coordenada.rs; al cerrar (Cancelar, guardado,
//     o cierre externo), lo desactiva.
//
// obtener_cursor_captura() / obtener_ventana_activa_captura()
//     Polling en vivo desde la ventana de captura (posición
//     del cursor y datos de la ventana activa).
//
// consultar_guardado_coordenada()
//     Polling desde la ventana de captura: ¿se apretó la
//     tecla de guardar desde la última consulta?
//
// guardar_resultado_coordenada() / obtener_resultado_coordenada()
//     La ventana de captura entrega el resultado ya
//     calculado; el popup de la fila del perfil lo retira.
//
// obtener_config_captura_activa()
//     La ventana de captura la consulta una sola vez al cargar
//     (ubicación/modo/punto de referencia de la fila que la abrió).
//
// obtener_tecla_guardar_coordenada() / establecer_...()
//     Config de la tecla de guardado (F1 por defecto).
//
// obtener_intervalo_captura_coordenada()
//     Cada cuántos ms debe sondear captura.html (config.rs).
// ======================================================

#[derive(Serialize)]
pub struct ConfigCapturaJson {
    pub ubicacion: String,

    pub modo_ventana: String,

    pub punto_referencia: String,
}

#[derive(Serialize)]
pub struct VentanaActivaJson {
    pub titulo: String,

    pub x: i32,

    pub y: i32,

    pub ancho: i32,

    pub alto: i32,
}

const VENTANA_CAPTURA_COORDENADA: &str = "captura_coordenada";

#[tauri::command]
pub fn abrir_ventana_captura_coordenada(
    app: tauri::AppHandle,
    ubicacion: String,
    modo_ventana: String,
    punto_referencia: String,
) -> Result<(), String> {
    // Re-captura: si ya había una ventana abierta (el usuario volvió
    // a hacer clic en 📌 Capturar sin cerrar la anterior), se cierra
    // primero para no dejar dos overlays sueltos.
    if let Some(existente) = app.get_webview_window(VENTANA_CAPTURA_COORDENADA) {
        let _ = existente.close();
    }

    // Se fija la config ANTES de crear la ventana: captura.html puede
    // consultarla apenas termina de cargar, sin carrera posible.
    captura_coordenada::activar(ubicacion, modo_ventana, punto_referencia);

    WebviewWindowBuilder::new(
        &app,
        VENTANA_CAPTURA_COORDENADA,
        WebviewUrl::App("captura.html".into()),
    )
    .title("RemapH — Captura")
    .inner_size(320.0, 120.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(true)
    .devtools(true)
    .build()
    .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cerrar_ventana_captura_coordenada(app: tauri::AppHandle) {
    if let Some(ventana) = app.get_webview_window(VENTANA_CAPTURA_COORDENADA) {
        let _ = ventana.close();
    }

    captura_coordenada::desactivar();
}

#[tauri::command]
pub fn obtener_cursor_captura() -> (i32, i32) {
    back_coordenada::obtener_cursor()
}

#[tauri::command]
pub fn obtener_ventana_activa_captura() -> Option<VentanaActivaJson> {
    back_coordenada::obtener_ventana_activa().map(|ventana| VentanaActivaJson {
        titulo: ventana.titulo,
        x: ventana.x,
        y: ventana.y,
        ancho: ventana.ancho,
        alto: ventana.alto,
    })
}

#[tauri::command]
pub fn obtener_config_captura_activa() -> Option<ConfigCapturaJson> {
    captura_coordenada::obtener_config_activa().map(|config| ConfigCapturaJson {
        ubicacion: config.ubicacion,
        modo_ventana: config.modo_ventana,
        punto_referencia: config.punto_referencia,
    })
}

#[tauri::command]
pub fn consultar_guardado_coordenada() -> bool {
    captura_coordenada::consultar_guardado()
}

#[tauri::command]
pub fn guardar_resultado_coordenada(x: f64, y: f64) {
    captura_coordenada::guardar_resultado(x, y);
}

#[tauri::command]
pub fn obtener_resultado_coordenada() -> Option<(f64, f64)> {
    captura_coordenada::obtener_resultado()
}

#[tauri::command]
pub fn obtener_tecla_guardar_coordenada() -> String {
    config::tecla_guardar_coordenada()
}

#[tauri::command]
pub fn establecer_tecla_guardar_coordenada(valor: String) {
    config::establecer_tecla_guardar_coordenada(valor)
}

#[tauri::command]
pub fn obtener_intervalo_captura_coordenada() -> u64 {
    config::intervalo_captura_coordenada()
}
