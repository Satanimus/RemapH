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
use crate::compilador::ResultadoCompilacion;
use crate::config;
use crate::perfil;
use crate::perfil_json::perfil_json;
use crate::perfil_ui::{convertir_perfil, FilaUI, ResultadoPerfil, TriggerCapturaUI};
use crate::pulsadores;

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

// ======================================================
// 🌐 COMANDOS TRADUCTOR (pulsadores.tsv)
// ------------------------------------------------------
// Puente entre columnas del diccionario para la UI (ver
// core_traductor.ts). La UI nunca lee pulsadores.tsv
// directamente — pide traducciones puntuales acá.
// ======================================================

#[tauri::command]
pub fn traducir_pulsador(valor: String, origen: String, destino: String) -> Option<String> {
    pulsadores::traducir(&valor, &origen, &destino)
}

#[tauri::command]
pub fn traducir_pulsador_lote(
    valores: Vec<String>,
    origen: String,
    destino: String,
) -> std::collections::HashMap<String, String> {
    pulsadores::traducir_lote(&valores, &origen, &destino)
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
pub fn compilar_perfil(filas: Vec<FilaUI>) -> Result<ResultadoCompilacion, String> {
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
// ⚡🪟 MENU EXPRESS — TAMAÑOS CONFIGURABLES
// ------------------------------------------------------
// Un solo comando de lectura combinada (no uno por valor, como
// tiempo_doble/etc.): la ventana de MenuExpress necesita los 9
// valores juntos al cargar (ver menu_express_main.ts), no de a
// uno. Los setters individuales sí siguen el patrón par
// obtener/establecer del resto del archivo — quedan listos para
// un futuro panel de configuración, aunque hoy ningún popup de
// la UI los expone todavía (fuera del alcance de MenuExpress).
// ======================================================

#[derive(Serialize)]
pub struct MenuExpressTamanosJson {
    pub boton_pequeno: (u64, u64),
    pub boton_mediano: (u64, u64),
    pub boton_grande: (u64, u64),
    pub texto_pequeno: u64,
    pub texto_mediano: u64,
    pub texto_grande: u64,
}

#[tauri::command]
pub fn obtener_tamanos_menu_express() -> MenuExpressTamanosJson {
    MenuExpressTamanosJson {
        boton_pequeno: config::menu_boton_pequeno(),
        boton_mediano: config::menu_boton_mediano(),
        boton_grande: config::menu_boton_grande(),
        texto_pequeno: config::menu_texto_pequeno(),
        texto_mediano: config::menu_texto_mediano(),
        texto_grande: config::menu_texto_grande(),
    }
}

#[tauri::command]
pub fn establecer_menu_boton_pequeno(ancho: u64, alto: u64) {
    config::establecer_menu_boton_pequeno(ancho, alto)
}

#[tauri::command]
pub fn establecer_menu_boton_mediano(ancho: u64, alto: u64) {
    config::establecer_menu_boton_mediano(ancho, alto)
}

#[tauri::command]
pub fn establecer_menu_boton_grande(ancho: u64, alto: u64) {
    config::establecer_menu_boton_grande(ancho, alto)
}

#[tauri::command]
pub fn establecer_menu_texto_pequeno(valor: u64) {
    config::establecer_menu_texto_pequeno(valor)
}

#[tauri::command]
pub fn establecer_menu_texto_mediano(valor: u64) {
    config::establecer_menu_texto_mediano(valor)
}

#[tauri::command]
pub fn establecer_menu_texto_grande(valor: u64) {
    config::establecer_menu_texto_grande(valor)
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
// 🎨 OBTENER ICONO POR RUTA
// ------------------------------------------------------
// A diferencia de obtener_icono_programa (busca por nombre entre
// los procesos corriendo), esta recibe una ruta directa — usada por
// el tipo "Abrir Archivo/App" para mostrar el ícono de lo que se
// eligió con "Seleccionar..." (archivo, carpeta o programa), sin
// depender de que esté corriendo ahora mismo.
// ======================================================

#[tauri::command]
pub fn obtener_icono_ruta(ruta: String) -> Option<IconoJson> {
    back_app::extraer_icono_ruta(&ruta).map(convertir_icono)
}

// ======================================================
// 📂 SELECTOR NATIVO DE ARCHIVO/CARPETA
// ------------------------------------------------------
// Usados por el tipo "Abrir Archivo/App": seleccionar_archivo() para
// el botón "Seleccionar..." de la columna Acción (sin filtro) y para
// la opción "Examinar..." del listado de "Abrir con" (filtrada a
// .exe, ver Etapa 11); seleccionar_carpeta() para cuando el ítem
// elegido es una carpeta. rfd no ofrece un diálogo nativo que
// combine archivo+carpeta en una sola ventana — la UI (Etapa 10)
// decide cómo ofrecer ambas opciones.
// ======================================================

#[tauri::command]
pub fn seleccionar_archivo(extensiones: Option<Vec<String>>) -> Option<String> {
    let mut dialogo = rfd::FileDialog::new();

    if let Some(extensiones) = &extensiones {
        let filtros: Vec<&str> = extensiones.iter().map(String::as_str).collect();

        dialogo = dialogo.add_filter("Programas", &filtros);
    }

    dialogo
        .pick_file()
        .map(|ruta| ruta.to_string_lossy().to_string())
}

#[tauri::command]
pub fn seleccionar_carpeta() -> Option<String> {
    rfd::FileDialog::new()
        .pick_folder()
        .map(|ruta| ruta.to_string_lossy().to_string())
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
pub async fn abrir_ventana_captura_coordenada(
    app: tauri::AppHandle,
    ubicacion: String,
    modo_ventana: String,
    punto_referencia: String,
) -> Result<(), String> {
    // ⚠️ IMPORTANTE: este comando tiene que ser `async fn`. En Windows,
    // WebviewWindowBuilder::build() hace DEADLOCK si se lo llama desde
    // un comando síncrono (`fn` normal) — es un problema documentado
    // de Tauri/WebView2, no algo nuestro:
    // https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
    // Síntoma exacto que daba antes de este cambio: la ventana nativa
    // se creaba (con barra, arrastrable, minimizable) pero quedaba en
    // blanco, sin webview adjunto (sin clic derecho, sin F12) y sin
    // poder cerrarse — clásico deadlock.

    // Re-captura: si ya había una ventana abierta (el usuario volvió
    // a hacer clic en 📌 Capturar sin cerrar la anterior), se cierra
    // primero para no dejar dos overlays sueltos.
    //
    // close() es ASINCRÓNICO: solo pide el cierre, no lo espera. Si
    // se sigue de largo y se crea de inmediato una ventana nueva con
    // el mismo label ("captura_coordenada"), puede que la vieja
    // todavía no se haya destruido del todo — WebviewWindowBuilder
    // falla (o queda en estado inconsistente) si el label sigue
    // ocupado. Por eso se espera, con un tope de 1s, a que
    // get_webview_window() confirme que ya no existe antes de seguir.
    if let Some(existente) = app.get_webview_window(VENTANA_CAPTURA_COORDENADA) {
        let _ = existente.close();

        for _ in 0..50 {
            if app.get_webview_window(VENTANA_CAPTURA_COORDENADA).is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    // Se fija la config ANTES de crear la ventana: captura.html puede
    // consultarla apenas termina de cargar, sin carrera posible.
    captura_coordenada::activar(ubicacion, modo_ventana, punto_referencia);

    let ventana = WebviewWindowBuilder::new(
        &app,
        VENTANA_CAPTURA_COORDENADA,
        WebviewUrl::App("captura.html".into()),
    )
    .title("RemapH — Captura")
    .inner_size(320.0, 120.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(true)
    .devtools(true)
    .build()
    .map_err(|error| error.to_string())?;

    // Abre las devtools de ESTA ventana automáticamente en debug — ya
    // cumplió su propósito de diagnóstico (confirmó el deadlock de
    // WebviewWindowBuilder en comando síncrono). Comentado por ahora;
    // descomentar si hace falta diagnosticar algo de nuevo.
    // #[cfg(debug_assertions)]
    // ventana.open_devtools();

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

// ======================================================
// ⚡🪟 MENU EXPRESS — VENTANA FLOTANTE
// ------------------------------------------------------
// abrir_o_alternar() NO es un comando Tauri — runtime.rs la
// llama directo (el trigger llega desde el hilo de entrada
// física, no desde JS). Acá solo lo que la propia ventana
// necesita invocar: leer sus datos una vez al cargar, pedir su
// propio cierre (botón [x]), y ejecutar/detener un botón de
// adentro en cada mousedown/mouseup (etapa 7) — la lógica real
// de esto último vive en back_menu_express.rs (incluida la
// llamada a runtime::ejecutar), nunca acá, para no romper la
// regla de este archivo (ver header: "Comandos NO: ejecuta
// Runtime").
// ======================================================

#[tauri::command]
pub fn obtener_datos_menu_express(
    id: String,
) -> Option<crate::back_menu_express::MenuExpressDatosUI> {
    crate::back_menu_express::obtener_datos(&id)
}

#[tauri::command]
pub fn cerrar_menu_express(id: String) {
    crate::back_menu_express::cerrar(&id);
}

#[tauri::command]
pub fn menu_express_boton_down(fila_id: String) {
    crate::back_menu_express::boton_down(&fila_id);
}

#[tauri::command]
pub fn menu_express_boton_up(id_menu: String, fila_id: String) -> bool {
    crate::back_menu_express::boton_up(&id_menu, &fila_id)
}

// ======================================================
// 📋 PORTAPAPELES — VENTANA FLOTANTE
// ------------------------------------------------------
// Igual criterio que MenuExpress: abrir_o_alternar() NO es un
// comando Tauri (runtime.rs la llama directo, Etapa I). Acá solo lo
// que la propia ventana necesita invocar. Los comandos de mutación
// (fijar/desfijar/renombrar/editar/eliminar/limpiar_todo/toggle
// Registro/pegar) devuelven Option<PortapapelesDatosUI> — el mismo
// back_portapapeles::refrescar_datos() tras aplicar el cambio, para
// que la ventana reciba el estado ya actualizado en la misma
// respuesta y no necesite un segundo viaje. None si la mutación
// falló (además del propio Result<_, String> que reporta el motivo)
// o si la ventana ya se había cerrado mientras la operación estaba
// en vuelo.
// ======================================================

#[tauri::command]
pub fn obtener_datos_portapapeles(
    id: String,
) -> Option<crate::back_portapapeles::PortapapelesDatosUI> {
    crate::back_portapapeles::obtener_datos(&id)
}

#[tauri::command]
pub fn cerrar_portapapeles(id: String) {
    crate::back_portapapeles::cerrar(&id);
}

#[tauri::command]
pub fn portapapeles_toggle_registro(
    id: String,
    activar: bool,
    limite: u32,
) -> Option<crate::back_portapapeles::PortapapelesDatosUI> {
    if activar {
        crate::back_portapapeles::activar_registro(&id, limite);
    } else {
        crate::back_portapapeles::desactivar_registro(&id);
    }

    crate::back_portapapeles::refrescar_datos(&id)
}

#[tauri::command]
pub fn portapapeles_fijar(
    id: String,
    ruta: String,
) -> Result<Option<crate::back_portapapeles::PortapapelesDatosUI>, String> {
    crate::back_portapapeles::fijar(std::path::Path::new(&ruta), &id)?;

    Ok(crate::back_portapapeles::refrescar_datos(&id))
}

#[tauri::command]
pub fn portapapeles_desfijar(
    id: String,
    ruta: String,
) -> Result<Option<crate::back_portapapeles::PortapapelesDatosUI>, String> {
    crate::back_portapapeles::desfijar(std::path::Path::new(&ruta))?;

    Ok(crate::back_portapapeles::refrescar_datos(&id))
}

#[tauri::command]
pub fn portapapeles_renombrar(
    id: String,
    ruta: String,
    nuevo_nombre: String,
) -> Result<Option<crate::back_portapapeles::PortapapelesDatosUI>, String> {
    crate::back_portapapeles::renombrar(std::path::Path::new(&ruta), &nuevo_nombre)?;

    Ok(crate::back_portapapeles::refrescar_datos(&id))
}

#[tauri::command]
pub fn portapapeles_editar(
    id: String,
    ruta: String,
    contenido: String,
) -> Result<Option<crate::back_portapapeles::PortapapelesDatosUI>, String> {
    crate::back_portapapeles::editar_texto(std::path::Path::new(&ruta), &contenido)?;

    Ok(crate::back_portapapeles::refrescar_datos(&id))
}

#[tauri::command]
pub fn portapapeles_marcar_reciente(ruta: String) -> Result<(), String> {
    crate::back_portapapeles::marcar_reciente(std::path::Path::new(&ruta))
}

// La ventana se crea con WS_EX_NOACTIVATE (no le roba el foco a la
// app activa del usuario) — pero eso también bloquea el tecleo real
// en los popups de Editar/Renombrar (los únicos con campo de texto).
// enfocar_ventana se llama al abrirlos, desenfocar_ventana al
// cerrarlos (ver portapapeles_main.ts::abrirPopupRenombrar/
// abrirPopupEditar/cerrarPopup).
#[tauri::command]
pub fn portapapeles_enfocar_ventana(id: String) {
    crate::back_portapapeles::enfocar_para_edicion(&id);
}

#[tauri::command]
pub fn portapapeles_desenfocar_ventana(id: String) {
    crate::back_portapapeles::restaurar_no_activacion(&id);
}

#[tauri::command]
pub fn portapapeles_eliminar(
    id: String,
    ruta: String,
) -> Result<Option<crate::back_portapapeles::PortapapelesDatosUI>, String> {
    crate::back_portapapeles::eliminar(std::path::Path::new(&ruta))?;

    Ok(crate::back_portapapeles::refrescar_datos(&id))
}

#[tauri::command]
pub fn portapapeles_limpiar_todo(
    id: String,
) -> Result<Option<crate::back_portapapeles::PortapapelesDatosUI>, String> {
    // "Limpiar todo" borra los ROTATIVOS (spec: "Botón 'Limpiar
    // todo' Borra todos los rotativos") — los fijados de esta fila
    // no se tocan, son un pool aparte (Etapa E).
    for elemento in crate::back_portapapeles::listar_rotativos()? {
        crate::back_portapapeles::eliminar(&elemento.ruta)?;
    }

    Ok(crate::back_portapapeles::refrescar_datos(&id))
}

#[tauri::command]
pub fn portapapeles_pegar(ruta: String) -> Result<(), String> {
    crate::back_portapapeles::pegar(std::path::Path::new(&ruta))
}

// ======================================================
// 📋 PORTAPAPELES — TAMAÑOS CONFIGURABLES
// ------------------------------------------------------
// Solo tamaño de BOTÓN tiene funciones propias (portapapeles_boton_
// pequeno/mediano/grande en config.rs, Etapa C) — el tamaño de TEXTO
// reusa menu_texto_pequeno/mediano/grande tal cual (ver config.rs),
// así que ya están cubiertos por establecer_menu_texto_pequeno/
// mediano/grande, comandos existentes de MenuExpress, sin duplicar
// acá. Mismo criterio par obtener/establecer que
// obtener_tamanos_menu_express: un solo comando de lectura combinada
// (la ventana necesita los 3 juntos al cargar), setters individuales
// para un futuro panel de configuración.
// ======================================================

#[derive(Serialize)]
pub struct PortapapelesTamanosJson {
    pub boton_pequeno: (u64, u64),
    pub boton_mediano: (u64, u64),
    pub boton_grande: (u64, u64),
}

#[tauri::command]
pub fn obtener_tamanos_portapapeles() -> PortapapelesTamanosJson {
    PortapapelesTamanosJson {
        boton_pequeno: config::portapapeles_boton_pequeno(),
        boton_mediano: config::portapapeles_boton_mediano(),
        boton_grande: config::portapapeles_boton_grande(),
    }
}

#[tauri::command]
pub fn establecer_portapapeles_boton_pequeno(ancho: u64, alto: u64) {
    config::establecer_portapapeles_boton_pequeno(ancho, alto)
}

#[tauri::command]
pub fn establecer_portapapeles_boton_mediano(ancho: u64, alto: u64) {
    config::establecer_portapapeles_boton_mediano(ancho, alto)
}

#[tauri::command]
pub fn establecer_portapapeles_boton_grande(ancho: u64, alto: u64) {
    config::establecer_portapapeles_boton_grande(ancho, alto)
}
