// ======================================================
// 🖥️ back_app
// ======================================================
// ETAPA X DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Backend encargado de comunicarse con Windows para
// obtener información relacionada con procesos y aplicaciones.
// Actualmente es responsable de:
// • Enumerar procesos (todos, tengan o no ventana propia;
//   la UI decide cómo separarlos en las dos listas).
// • Obtener la aplicación en primer plano.
// • Obtener la ruta real del ejecutable.
// • Extraer el ícono del ejecutable.
//
// En el futuro también será responsable de:
// • Detectar cambio de ventana activa.
// • Detectar apertura de procesos.
// • Detectar cierre de procesos.
// • Informar esos cambios a cache.
//
// Flujo:
// Windows
//      ↓
// back_procesos
//      ↓
// cache::actualizar_estado_app()
// ------------------------------------------------------
// 2. ¿Qué información recibe?
// Recibe consultas realizadas por:
// Solicitudes internas de información sobre Windows.
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
// Es utilizado por:
// • Comandos Tauri.
// • Cache (futuro estado de aplicaciones).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve:
// • Lista de procesos.
// • Programa activo.
// • Ruta del ejecutable.
// • Ícono.
// • (Futuro) cambios de estado de aplicaciones.
// ------------------------------------------------------
// 5. Funciones del archivo
// enumerar_procesos_ventana()
//     Lista procesos disponibles.
// obtener_programa_activo()
//     Devuelve la aplicación en primer plano.
// obtener_pid_y_ruta_activo()
//     PID + ruta completa del ejecutable en primer plano, en una
//     sola consulta a Windows (evita repetir la misma cadena de
//     llamadas que ya hace obtener_programa_activo() cuando el
//     que llama necesita ambos datos, no solo el nombre).
// obtener_ruta_proceso()
//     Obtiene la ruta del ejecutable.
// extraer_icono()
//     Extrae el ícono del ejecutable.
// extraer_icono_ruta()
//     Extrae el ícono asociado a cualquier ruta (carpeta/documento/.lnk), vía SHGetFileInfoW.
// es_proceso_windows()
//     Filtra procesos propios de Windows.
// iniciar_monitor()
//     Instala el aviso de cambio de foco de Windows (una sola vez) y arranca su loop de mensajes en un hilo aparte.
// revisar_apps()
//     Reevalúa el estado de todas las apps vigiladas y actualiza cache.
// esta_abierta()
//     Indica si un ejecutable aparece entre los procesos corriendo ahora mismo.
// alternar_foco()
//     Núcleo compartido: dado un HWND, si ya está en primer plano lo minimiza, si no lo restaura/enfoca. Toggle usado por enfocar_proceso() y enfocar_carpeta().
// enfocar_proceso()
//     Busca una ventana "real" (ver es_ventana_real) del proceso indicado y alterna minimizar/restaurar (alternar_foco). Usado por "Abrir Archivo/App" con Instancias Única cuando no hay un archivo puntual que matchear en el título (caso .exe).
// enfocar_carpeta() / enfocar_documento()
//     Envoltorios de enfocar_por_titulo(): matchean proceso + título de ventana conteniendo el nombre de la carpeta/archivo (por nombre de proceso solo no alcanza — ver bug de Notepad++ con Múltiples archivos).
// enfocar_por_titulo()
//     Núcleo compartido por enfocar_carpeta/enfocar_documento.
// es_ventana_real()
//     Filtro "tipo Alt-Tab" (sin dueño, sin WS_EX_TOOLWINDOW) — evita matchear paletas/diálogos de una app en vez de su ventana principal.
// listar_ventanas_visibles()
//     Foto de todos los HWND visibles del sistema en este instante. Usado por runtime.rs::abrir_archivo() como "antes" de lanzar.
// buscar_ventana_nueva()
//     Compara contra un snapshot anterior y devuelve una ventana visible que no estaba ahí (prioriza coincidir con un PID si se conoce). Usado por runtime.rs::abrir_archivo() para el forzado de primer plano tras lanzar.
// forzar_foco()
//     AttachThreadInput + un toque de ALT simulado (ver simular_pulsacion_alt) + SetForegroundWindow: fuerza el foco a una ventana puntual, evitando la protección anti robo-de-foco de Windows incluso cuando la ventana ya está visible en 2º plano sin cambio de estado.
// reafirmar_modo_ventana()
//     Tras encontrar la ventana nueva de un lanzamiento reciente, reafirma el modo pedido (Minimizado/Maximizado/Ventana) con ShowWindow explícito — no solo el foco. Usado por runtime.rs::forzar_primer_plano().
// ======================================================

use std::collections::HashSet;

use std::path::Path;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};

use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
    DIB_RGB_COLORS,
};

use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};

use windows_sys::Win32::UI::Shell::{
    ExtractIconExW, SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetForegroundWindow, GetIconInfo, GetWindowThreadProcessId, ICONINFO,
};

// ======================================================
// PROCESO
// ======================================================

pub struct ProcesoVentana {
    pub nombre: String,

    pub ruta: String,
}

// ======================================================
// ÍCONO CRUDO - Píxeles en formato RGBA.
// ======================================================

pub struct IconoRaw {
    pub ancho: u32,

    pub alto: u32,

    pub pixeles: Vec<u8>,
}

// ======================================================
// ENUMERAR PROCESOS
// ======================================================

pub fn enumerar_procesos_ventana() -> Vec<ProcesoVentana> {
    let mut lista = Vec::new();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);

        if snapshot == INVALID_HANDLE_VALUE {
            return lista;
        }

        let mut entrada: PROCESSENTRY32W = std::mem::zeroed();

        entrada.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut exito = Process32FirstW(snapshot, &mut entrada);

        while exito != 0 {
            let pid = entrada.th32ProcessID;

            if let Some(ruta) = obtener_ruta_proceso(pid) {
                if !es_proceso_windows(&ruta) {
                    let nombre = Path::new(&ruta)
                        .file_name()
                        .map(|nombre| nombre.to_string_lossy().to_string())
                        .unwrap_or_else(|| ruta.clone());

                    lista.push(ProcesoVentana { nombre, ruta });
                }
            }

            exito = Process32NextW(snapshot, &mut entrada);
        }

        CloseHandle(snapshot);
    }

    // ==================================================
    // DEDUPLICAR POR NOMBRE DE EJECUTABLE
    // ==================================================

    let mut vistos = HashSet::new();

    lista.retain(|proceso| vistos.insert(proceso.nombre.to_lowercase()));

    lista
}

// ======================================================
// 🖥️ PROGRAMA EN PRIMER PLANO
// ======================================================

pub fn obtener_programa_activo() -> Option<String> {
    let ventana = unsafe { GetForegroundWindow() };

    if ventana.is_null() {
        return None;
    }

    let mut pid = 0;

    unsafe {
        GetWindowThreadProcessId(ventana, &mut pid);
    }

    let ruta = unsafe { obtener_ruta_proceso(pid) }?;

    Path::new(&ruta)
        .file_name()
        .map(|nombre| nombre.to_string_lossy().to_string())
}

// ======================================================
// 🆔 PID DEL PROGRAMA EN PRIMER PLANO
// ------------------------------------------------------
// Igual a la primera mitad de obtener_programa_activo(), pero
// devuelve el PID en vez de resolverlo hasta el nombre de archivo —
// lo usa back_pegado_personalizado.rs, que necesita el PID para
// pedir la ruta completa del ejecutable vía obtener_ruta_proceso().
// ======================================================

pub fn obtener_pid_activo() -> Option<u32> {
    let ventana = unsafe { GetForegroundWindow() };

    if ventana.is_null() {
        return None;
    }

    let mut pid = 0;

    unsafe {
        GetWindowThreadProcessId(ventana, &mut pid);
    }

    if pid == 0 {
        return None;
    }

    Some(pid)
}

// ======================================================
// 🆔📄 PID + RUTA DEL PROGRAMA EN PRIMER PLANO (una sola consulta)
// ------------------------------------------------------
// Combina en una sola consulta a Windows lo que back_pegado_
// personalizado.rs venía haciendo con dos llamadas separadas
// (obtener_programa_activo() para decidir si es Photoshop, después
// obtener_pid_activo()+obtener_ruta_proceso() para relanzarlo) — cada
// una de esas repetía la misma cadena completa (GetForegroundWindow
// + GetWindowThreadProcessId + OpenProcess +
// QueryFullProcessImageNameW), consultando dos veces el mismo dato.
// Devuelve PID y ruta completa juntos; quien llama puede sacar el
// nombre de archivo de la ruta con Path::file_name() si solo
// necesita eso (ver es_photoshop() en back_pegado_personalizado.rs).
// ======================================================

pub fn obtener_pid_y_ruta_activo() -> Option<(u32, String)> {
    let ventana = unsafe { GetForegroundWindow() };

    if ventana.is_null() {
        return None;
    }

    let mut pid = 0;

    unsafe {
        GetWindowThreadProcessId(ventana, &mut pid);
    }

    if pid == 0 {
        return None;
    }

    let ruta = unsafe { obtener_ruta_proceso(pid) }?;

    Some((pid, ruta))
}

// ======================================================
// DETERMINAR SI ES PROCESO DE WINDOWS
// ======================================================

fn es_proceso_windows(ruta: &str) -> bool {
    let ruta = ruta.to_lowercase();

    let carpeta_windows = std::env::var("WINDIR")
        .unwrap_or_else(|_| "C:\\Windows".to_string())
        .to_lowercase();

    ruta.starts_with(&format!("{}\\", carpeta_windows,))
}

// ======================================================
// RUTA DEL EJECUTABLE
// ======================================================

pub unsafe fn obtener_ruta_proceso(pid: u32) -> Option<String> {
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);

    if handle.is_null() {
        return None;
    }

    let mut buffer: [u16; 1024] = [0; 1024];

    let mut tamano = buffer.len() as u32;

    let exito = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut tamano);

    CloseHandle(handle);

    if exito == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&buffer[..tamano as usize]))
}

// ======================================================
// EXTRAER ÍCONO (desde un .exe/.dll, vía ExtractIconExW)
// ======================================================

pub fn extraer_icono(ruta: &str) -> Option<IconoRaw> {
    unsafe {
        let ruta_ancha: Vec<u16> = ruta.encode_utf16().chain(std::iter::once(0)).collect();

        let mut icono_grande: *mut std::ffi::c_void = std::ptr::null_mut();

        let mut icono_pequeno: *mut std::ffi::c_void = std::ptr::null_mut();

        let extraidos = ExtractIconExW(
            ruta_ancha.as_ptr(),
            0,
            &mut icono_grande,
            &mut icono_pequeno,
            1,
        );

        // icono_grande nunca se usa — se descarta apenas se obtiene,
        // en vez de recién en cada punto de salida como antes (mismo
        // resultado, sin repetir el chequeo en cada return).
        if !icono_grande.is_null() {
            DestroyIcon(icono_grande);
        }

        if extraidos == 0 || icono_pequeno.is_null() {
            return None;
        }

        icono_desde_hicon(icono_pequeno)
    }
}

// ======================================================
// EXTRAER ÍCONO POR RUTA (carpeta/documento, vía SHGetFileInfoW)
// ------------------------------------------------------
// A diferencia de extraer_icono() (pensada para .exe/.dll), esta
// sirve para cualquier ruta — carpeta, documento, .lnk — usando el
// ícono que Windows ya le asocia (el mismo que se ve en el
// Explorador). Usada por el botón "Seleccionar..." del tipo "Abrir
// Archivo/App" (comandos.rs::obtener_icono_ruta) y por el listado de
// "Abrir con" (Etapa 7B). No requiere que la ruta exista en un
// proceso corriendo, a diferencia de extraer_icono().
// ======================================================

pub fn extraer_icono_ruta(ruta: &str) -> Option<IconoRaw> {
    unsafe {
        let ruta_ancha: Vec<u16> = ruta.encode_utf16().chain(std::iter::once(0)).collect();

        let mut info: SHFILEINFOW = std::mem::zeroed();

        let resultado = SHGetFileInfoW(
            ruta_ancha.as_ptr(),
            0,
            &mut info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        );

        if resultado == 0 || info.hIcon.is_null() {
            return None;
        }

        icono_desde_hicon(info.hIcon)
    }
}

// ======================================================
// ÍCONO DESDE HICON → PÍXELES RGBA
// ------------------------------------------------------
// Núcleo compartido por extraer_icono() y extraer_icono_ruta(): dado
// un HICON ya obtenido (por el método que sea), lo convierte a
// píxeles RGBA planos y SIEMPRE lo destruye (DestroyIcon) antes de
// volver, en cualquiera de sus salidas — el llamador no debe volver
// a tocar el HICON después de pasarlo acá.
// ======================================================

unsafe fn icono_desde_hicon(hicon: *mut std::ffi::c_void) -> Option<IconoRaw> {
    let mut icon_info: ICONINFO = std::mem::zeroed();

    if GetIconInfo(hicon, &mut icon_info) == 0 {
        DestroyIcon(hicon);

        return None;
    }

    let hbm_color = icon_info.hbmColor;

    let hbm_mask = icon_info.hbmMask;

    if hbm_color.is_null() {
        DeleteObject(hbm_mask);

        DestroyIcon(hicon);

        return None;
    }

    let mut bitmap: BITMAP = std::mem::zeroed();

    let bytes_bitmap = std::mem::size_of::<BITMAP>() as i32;

    let resultado = GetObjectW(
        hbm_color,
        bytes_bitmap,
        &mut bitmap as *mut BITMAP as *mut std::ffi::c_void,
    );

    if resultado == 0 {
        DeleteObject(hbm_color);

        DeleteObject(hbm_mask);

        DestroyIcon(hicon);

        return None;
    }

    let ancho = bitmap.bmWidth as u32;

    let alto = bitmap.bmHeight as u32;

    if ancho == 0 || alto == 0 {
        DeleteObject(hbm_color);

        DeleteObject(hbm_mask);

        DestroyIcon(hicon);

        return None;
    }

    let hdc = CreateCompatibleDC(std::ptr::null_mut());

    if hdc.is_null() {
        DeleteObject(hbm_color);

        DeleteObject(hbm_mask);

        DestroyIcon(hicon);

        return None;
    }

    let mut bitmap_info: BITMAPINFO = std::mem::zeroed();

    bitmap_info.bmiHeader.biSize = std::mem::size_of_val(&bitmap_info.bmiHeader) as u32;

    bitmap_info.bmiHeader.biWidth = ancho as i32;

    bitmap_info.bmiHeader.biHeight = -(alto as i32);

    bitmap_info.bmiHeader.biPlanes = 1;

    bitmap_info.bmiHeader.biBitCount = 32;

    bitmap_info.bmiHeader.biCompression = 0;

    let mut pixeles = vec![0u8; (ancho * alto * 4) as usize];

    let filas_copiadas = GetDIBits(
        hdc,
        hbm_color,
        0,
        alto,
        pixeles.as_mut_ptr() as *mut std::ffi::c_void,
        &mut bitmap_info,
        DIB_RGB_COLORS,
    );

    DeleteDC(hdc);

    DeleteObject(hbm_color);

    DeleteObject(hbm_mask);

    DestroyIcon(hicon);

    if filas_copiadas == 0 {
        return None;
    }

    // ==================================================
    // BGRA → RGBA
    // ==================================================

    let mut indice = 0;

    while indice < pixeles.len() {
        pixeles.swap(indice, indice + 2);

        indice += 4;
    }

    Some(IconoRaw {
        ancho,
        alto,
        pixeles,
    })
}

// ======================================================
// 👁️ MONITOR DE APPS (foco)
// ======================================================

use crate::cache;
use crate::perfil_cache::{AppCache, IniciarVentana};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, KEYEVENTF_KEYUP, VK_MENU, VK_SHIFT,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, EnumWindows, GetMessageW, GetWindow, GetWindowLongPtrW, GetWindowTextW,
    IsIconic, IsWindowVisible, IsZoomed, SetForegroundWindow, ShowWindow, TranslateMessage,
    EVENT_SYSTEM_FOREGROUND, GWL_EXSTYLE, GW_OWNER, MSG, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE,
    WINEVENT_OUTOFCONTEXT, WS_EX_TOOLWINDOW,
};

pub fn iniciar_monitor() {
    std::thread::spawn(|| unsafe {
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            std::ptr::null_mut(),
            Some(en_cambio_foco),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        if hook.is_null() {
            println!("⚠️ No se pudo instalar el monitor de apps.");

            return;
        }

        let mut mensaje: MSG = std::mem::zeroed();

        while GetMessageW(&mut mensaje, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&mensaje);
            DispatchMessageW(&mensaje);
        }
    });
}

unsafe extern "system" fn en_cambio_foco(
    _hook: HWINEVENTHOOK,
    _evento: u32,
    _hwnd: HWND,
    _id_objeto: i32,
    _id_hijo: i32,
    _hilo_generador: u32,
    _instante: u32,
) {
    revisar_apps();
}

pub fn revisar_apps() {
    let vigiladas = cache::apps_a_vigilar();

    if vigiladas.is_empty() {
        return;
    }

    let activo = obtener_programa_activo();

    for app in vigiladas {
        let AppCache::Programa {
            nombre,
            segundo_plano,
        } = &app
        else {
            continue;
        };

        let activa = if *segundo_plano {
            esta_abierta(nombre)
        } else {
            activo.as_deref() == Some(nombre.as_str())
        };

        cache::actualizar_estado_app(app, activa);
    }
}

fn esta_abierta(nombre: &str) -> bool {
    enumerar_procesos_ventana()
        .iter()
        .any(|proceso| proceso.nombre.eq_ignore_ascii_case(nombre))
}

// ======================================================
// 🔁 ALTERNAR FOCO (toggle minimizar/restaurar)
// ------------------------------------------------------
// Núcleo compartido por enfocar_proceso() y enfocar_carpeta(): dado
// un HWND ya encontrado, si esa ventana ya es la que está en primer
// plano ahora mismo (y no está minimizada) la minimiza; en
// cualquier otro caso (minimizada, o en segundo plano) la restaura y
// le fuerza el foco vía forzar_foco() (AttachThreadInput +
// SetForegroundWindow).
// ======================================================

// ======================================================
// 🪟 ¿ES UNA VENTANA "REAL"? (filtro tipo Alt-Tab)
// ------------------------------------------------------
// BUG 5: EnumWindows visita TODAS las ventanas visibles de un
// proceso, en el orden de Z que sea — incluidas paletas, diálogos y
// otras ventanas "herramienta" que un proceso como Photoshop crea
// además de su ventana principal. Si el callback de matching agarra
// una de esas por error, GetForegroundWindow() == hwnd nunca da
// true aunque la app sí esté visualmente en primer plano (el
// usuario ve la ventana principal en foco, no esa herramienta), así
// que alternar_foco() nunca dispara el minimizado.
// Filtro estándar: una ventana "real" (la que un Alt-Tab mostraría)
// no tiene dueño (GetWindow GW_OWNER) y no tiene el estilo extendido
// WS_EX_TOOLWINDOW.
// ======================================================

unsafe fn es_ventana_real(hwnd: HWND) -> bool {
    if !GetWindow(hwnd, GW_OWNER).is_null() {
        return false;
    }

    let estilo_extendido = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;

    estilo_extendido & WS_EX_TOOLWINDOW == 0
}

fn alternar_foco(hwnd: HWND) {
    unsafe {
        let ya_en_foco = GetForegroundWindow() == hwnd && IsIconic(hwnd) == 0;

        if ya_en_foco {
            ShowWindow(hwnd, SW_MINIMIZE);
        } else {
            forzar_foco(hwnd);
        }
    }
}

// ======================================================
// 🔎 ENFOCAR PROCESO (toggle minimizar/restaurar)
// ------------------------------------------------------
// Usado por runtime.rs::abrir_archivo() cuando Instancias == Única y
// se conoce un nombre de ejecutable puntual (.exe directo, abrir_con,
// o el programa predeterminado resuelto por back_registro para un
// documento): busca entre las ventanas visibles del sistema una que
// pertenezca al proceso `nombre` (nombre de archivo del ejecutable,
// ej. "notepad.exe") y le aplica alternar_foco(). Devuelve false si
// no encontró ninguna — el llamador decide entonces lanzar el
// proceso de nuevo.
//
// NO se usa para carpetas — ver enfocar_carpeta() más abajo, ya que
// matchear solo por nombre de proceso ("explorer.exe") traería
// cualquier ventana de Explorer, no la carpeta puntual que se pidió.
// ======================================================

struct ContextoEnfoque {
    nombre: String,
    hwnd: HWND,
}

pub fn enfocar_proceso(nombre: &str) -> bool {
    let mut contexto = ContextoEnfoque {
        nombre: nombre.to_lowercase(),
        hwnd: std::ptr::null_mut(),
    };

    unsafe {
        EnumWindows(Some(callback_enfoque), &mut contexto as *mut _ as isize);
    }

    if contexto.hwnd.is_null() {
        return false;
    }

    alternar_foco(contexto.hwnd);

    true
}

unsafe extern "system" fn callback_enfoque(hwnd: HWND, lparam: isize) -> i32 {
    let contexto = &mut *(lparam as *mut ContextoEnfoque);

    if IsWindowVisible(hwnd) == 0 || !es_ventana_real(hwnd) {
        return 1;
    }

    let mut pid = 0u32;

    GetWindowThreadProcessId(hwnd, &mut pid);

    if let Some(ruta) = obtener_ruta_proceso(pid) {
        let nombre_proceso = Path::new(&ruta)
            .file_name()
            .map(|nombre| nombre.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if nombre_proceso == contexto.nombre {
            contexto.hwnd = hwnd;

            return 0;
        }
    }

    1
}

// ======================================================
// 🔎 ENFOCAR CARPETA / DOCUMENTO (toggle minimizar/restaurar, por título)
// ------------------------------------------------------
// Usado por runtime.rs::abrir_archivo() cuando Instancias == Única y
// la ruta es una carpeta (enfocar_carpeta) o un documento con programa
// conocido, propio o vía "Abrir con" (enfocar_documento). A diferencia
// de enfocar_proceso(), acá el nombre de proceso solo no alcanza para
// identificar LA carpeta/archivo puntual — cualquier ventana del mismo
// proceso lo matchearía (ej. otra carpeta ya abierta en Explorer, u
// otro archivo ya abierto en Notepad++).
//
// BUG 6: antes, Instancias Única para documentos usaba enfocar_proceso
// (solo por nombre de proceso) — con Notepad++ ya corriendo, abrir un
// SEGUNDO archivo distinto encontraba la ventana existente, hacía
// toggle sobre ELLA (minimizaba/restauraba el primer archivo) y nunca
// llegaba a lanzar el segundo. Al exigir también que el título de la
// ventana contenga el nombre del archivo pedido, un archivo distinto
// ya no matchea acá — cae a lanzar igual, que es justo lo que hace que
// apps de instancia única como Notepad++ agreguen una pestaña nueva
// sola.
//
// Se combina: proceso == nombre_proceso Y título de la ventana
// contiene el texto buscado (heurístico — tanto Explorer como la
// inmensa mayoría de editores/visores muestran el nombre de
// carpeta/archivo en el título de su ventana). Devuelve false si no
// encontró ninguna coincidencia — el llamador decide lanzar de nuevo.
// ======================================================

pub fn enfocar_carpeta(ruta: &str) -> bool {
    let nombre_carpeta = Path::new(ruta)
        .file_name()
        .map(|nombre| nombre.to_string_lossy().to_string())
        .unwrap_or_default();

    if nombre_carpeta.is_empty() {
        return false;
    }

    enfocar_por_titulo("explorer.exe", &nombre_carpeta)
}

pub fn enfocar_documento(nombre_proceso: &str, nombre_archivo: &str) -> bool {
    if nombre_proceso.is_empty() || nombre_archivo.is_empty() {
        return false;
    }

    enfocar_por_titulo(nombre_proceso, nombre_archivo)
}

struct ContextoEnfoqueTitulo {
    nombre_proceso: String,
    texto_titulo: String,
    hwnd: HWND,
}

fn enfocar_por_titulo(nombre_proceso: &str, texto_titulo: &str) -> bool {
    let mut contexto = ContextoEnfoqueTitulo {
        nombre_proceso: nombre_proceso.to_lowercase(),
        texto_titulo: texto_titulo.to_lowercase(),
        hwnd: std::ptr::null_mut(),
    };

    unsafe {
        EnumWindows(
            Some(callback_enfoque_titulo),
            &mut contexto as *mut _ as isize,
        );
    }

    if contexto.hwnd.is_null() {
        return false;
    }

    alternar_foco(contexto.hwnd);

    true
}

unsafe extern "system" fn callback_enfoque_titulo(hwnd: HWND, lparam: isize) -> i32 {
    let contexto = &mut *(lparam as *mut ContextoEnfoqueTitulo);

    if IsWindowVisible(hwnd) == 0 || !es_ventana_real(hwnd) {
        return 1;
    }

    let mut pid = 0u32;

    GetWindowThreadProcessId(hwnd, &mut pid);

    let Some(ruta_proceso) = obtener_ruta_proceso(pid) else {
        return 1;
    };

    let nombre_proceso = Path::new(&ruta_proceso)
        .file_name()
        .map(|nombre| nombre.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if nombre_proceso != contexto.nombre_proceso {
        return 1;
    }

    let mut buffer = [0u16; 512];

    let largo = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);

    if largo <= 0 {
        return 1;
    }

    let titulo = String::from_utf16_lossy(&buffer[..largo as usize]).to_lowercase();

    if titulo.contains(&contexto.texto_titulo) {
        contexto.hwnd = hwnd;

        return 0;
    }

    1
}

// ======================================================
// 🪟 SNAPSHOT DE VENTANAS VISIBLES
// ------------------------------------------------------
// Usado por runtime.rs::abrir_archivo() como el "antes" de lanzar,
// para poder detectar después cuál ventana es nueva (ver
// buscar_ventana_nueva). No filtra por proceso — es una foto cruda
// de todos los HWND visibles del sistema en este instante.
// ======================================================

pub type VentanaSnapshot = HashSet<HWND>;

pub fn listar_ventanas_visibles() -> VentanaSnapshot {
    let mut vistas: VentanaSnapshot = HashSet::new();

    unsafe {
        EnumWindows(
            Some(callback_listar_visibles),
            &mut vistas as *mut _ as isize,
        );
    }

    vistas
}

unsafe extern "system" fn callback_listar_visibles(hwnd: HWND, lparam: isize) -> i32 {
    let vistas = &mut *(lparam as *mut VentanaSnapshot);

    if IsWindowVisible(hwnd) != 0 {
        vistas.insert(hwnd);
    }

    1
}

// ======================================================
// 🔎 BUSCAR VENTANA NUEVA (tras lanzar)
// ------------------------------------------------------
// Reemplaza a la vieja búsqueda por PID exacto: ShellExecuteExW NO
// siempre entrega un PID utilizable — al abrir una CARPETA, Windows
// no lanza un explorer.exe nuevo, reusa la instancia que ya está
// corriendo (se lo pide por mensaje interno), así que hProcess llega
// nulo. Lo mismo puede pasar con un documento cuyo programa asociado
// ya estaba abierto y lo recibe por DDE en vez de lanzar una
// instancia nueva. Buscar solo por PID en esos casos además sería
// CONTRAPRODUCENTE: ese PID ya es dueño de ventanas viejas (las que
// tenía abiertas de antes), así que el primer match encontrado podía
// ser una ventana vieja sin ninguna relación con lo que se acaba de
// abrir — foco robado al lugar equivocado, no solo foco faltante.
//
// Estrategia: comparar contra el snapshot tomado ANTES de lanzar
// (ver listar_ventanas_visibles) y devolver una ventana visible que
// no estaba ahí — es decir, que apareció como consecuencia del
// lanzamiento. Si se conoce el PID, se prioriza una ventana nueva de
// ESE proceso puntual (caso normal .exe/abrir_con, más preciso); si
// no hay PID o ninguna ventana nueva es de ese proceso todavía, cae
// a "la primera ventana nueva que sea" — mejor esfuerzo para el caso
// de reuso de proceso (carpeta/documento).
//
// Devuelve None si todavía no apareció ninguna ventana nueva (puede
// pasar varias veces seguidas mientras el proceso arranca, o Windows
// termina de procesar el pedido) — el llamador decide cuántas veces
// reintentar.
// ======================================================

struct ContextoVentanaNueva {
    anteriores: VentanaSnapshot,
    pid: Option<u32>,
    con_pid: Option<HWND>,
    cualquiera: Option<HWND>,
}

pub fn buscar_ventana_nueva(anteriores: &VentanaSnapshot, pid: Option<u32>) -> Option<HWND> {
    let mut contexto = ContextoVentanaNueva {
        anteriores: anteriores.clone(),
        pid,
        con_pid: None,
        cualquiera: None,
    };

    unsafe {
        EnumWindows(
            Some(callback_ventana_nueva),
            &mut contexto as *mut _ as isize,
        );
    }

    contexto.con_pid.or(contexto.cualquiera)
}

unsafe extern "system" fn callback_ventana_nueva(hwnd: HWND, lparam: isize) -> i32 {
    let contexto = &mut *(lparam as *mut ContextoVentanaNueva);

    if IsWindowVisible(hwnd) == 0 || contexto.anteriores.contains(&hwnd) {
        return 1;
    }

    if contexto.cualquiera.is_none() {
        contexto.cualquiera = Some(hwnd);
    }

    if let Some(pid_buscado) = contexto.pid {
        let mut pid = 0u32;

        GetWindowThreadProcessId(hwnd, &mut pid);

        if pid == pid_buscado {
            contexto.con_pid = Some(hwnd);

            // Ya encontramos el mejor candidato posible (ventana
            // nueva Y del proceso exacto) — no hace falta seguir
            // enumerando el resto de las ventanas del sistema.
            return 0;
        }
    }

    1
}

// ======================================================
// 🔝 FORZAR FOCO (AttachThreadInput)
// ------------------------------------------------------
// SetForegroundWindow() solo, sin más, puede fallar en silencio: la
// protección anti robo-de-foco de Windows solo deja que un hilo le dé
// foco a una ventana si ese hilo está "pegado" (misma cola de
// entrada) al hilo que tiene el foco actual. Acá se fuerza esa
// condición con AttachThreadInput: se pega temporalmente el hilo
// actual al hilo dueño de la ventana en foco AHORA, se pide el foco
// para `hwnd`, y se despega. Con eso SetForegroundWindow ya no
// depende del historial de interacción del usuario ni de qué proceso
// lo llama.
// ======================================================

pub fn forzar_foco(hwnd: HWND) -> bool {
    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }

        let ventana_actual = GetForegroundWindow();

        let hilo_actual = GetCurrentThreadId();

        let mut pid_actual = 0u32;

        let hilo_de_ventana_actual = GetWindowThreadProcessId(ventana_actual, &mut pid_actual);

        // Ventana en foco puede no existir (escritorio vacío) o ser
        // la misma que ya queremos enfocar — en ambos casos no hace
        // falta pegar/despegar hilos, SetForegroundWindow directo
        // alcanza.
        if ventana_actual.is_null() || hilo_de_ventana_actual == hilo_actual {
            return SetForegroundWindow(hwnd) != 0;
        }

        // BUG 3 (resto): con la ventana ya visible en 2º plano y SIN
        // cambio de estado (no estaba minimizada, así que arriba no
        // se llamó a ShowWindow/SW_RESTORE), AttachThreadInput solo
        // no siempre alcanza para saltar la protección anti
        // robo-de-foco. El truco documentado: simular una pulsación
        // de ALT justo antes — Windows lo trata como si el pedido de
        // foco viniera de un input real del usuario, no de otro
        // proceso. Se hace acá (no en alternar_foco) para que
        // cualquier llamador de forzar_foco se beneficie por igual.
        simular_pulsacion_alt();

        AttachThreadInput(hilo_actual, hilo_de_ventana_actual, 1);

        let resultado = SetForegroundWindow(hwnd) != 0;

        AttachThreadInput(hilo_actual, hilo_de_ventana_actual, 0);

        resultado
    }
}

// ======================================================
// 🔁 ROBAR Y DEVOLVER FOCO (una sola pulsación de Shift, no Alt)
// ------------------------------------------------------
// Pensada para "robar el foco un instante y devolverlo ya" (usado por
// Portapapeles), a diferencia de forzar_foco() que es "dame el foco a
// esta ventana y quedate ahí" (Abrir Archivo/App). Se probó primero
// sin ningún refuerzo (solo AttachThreadInput) — no alcanzó para que
// Photoshop realmente reciba el ciclo de desactivar/reactivar (mismo
// motivo por el que forzar_foco() necesita el refuerzo de Alt: ver
// comentario ahí, "AttachThreadInput solo no siempre alcanza"). Se
// probó después con el mismo refuerzo de Alt que forzar_foco(), pero
// llamado dos veces seguidas (una para robar, otra para devolver) —
// eso mandaba DOS pulsaciones de Alt reales e independientes, que no
// siempre caían sobre la misma ventana ni se cancelaban entre sí:
// togglea el modo de mnemónicos de menú en apps clásicas (confirmado
// en Paint) y revela la barra de menú superior en Firefox.
//
// Acá se resuelven los dos problemas a la vez: un solo
// AttachThreadInput (robar y devolver sin soltar el hilo en el medio,
// así como mucho se manda una sola tecla de refuerzo por pegado, no
// dos) y esa tecla es Shift en vez de Alt — Shift solo, sin ninguna
// otra tecla, no tiene ningún atajo ni toggle conocido a nivel de
// Windows ni en apps comunes (a diferencia de Alt, que universalmente
// abre el modo de menú por teclado en apps clásicas).
// ======================================================

pub fn robar_y_devolver_foco(hwnd_temporal: HWND, hwnd_original: HWND) -> bool {
    unsafe {
        if hwnd_original.is_null() || hwnd_temporal.is_null() {
            return false;
        }

        let hilo_actual = GetCurrentThreadId();

        let mut pid_original = 0u32;

        let hilo_de_ventana_original = GetWindowThreadProcessId(hwnd_original, &mut pid_original);

        // Ya somos el dueño del foco (caso raro acá) — no hace falta
        // pegar/despegar hilos ni ningún refuerzo.
        if hilo_de_ventana_original == hilo_actual {
            SetForegroundWindow(hwnd_temporal);
            return SetForegroundWindow(hwnd_original) != 0;
        }

        AttachThreadInput(hilo_actual, hilo_de_ventana_original, 1);

        keybd_event(VK_SHIFT as u8, 0, 0, 0);
        keybd_event(VK_SHIFT as u8, 0, KEYEVENTF_KEYUP, 0);

        SetForegroundWindow(hwnd_temporal);
        let resultado = SetForegroundWindow(hwnd_original) != 0;

        AttachThreadInput(hilo_actual, hilo_de_ventana_original, 0);

        resultado
    }
}

// ======================================================
// 🔑 SIMULAR PULSACIÓN DE ALT (refuerzo de forzar_foco)
// ------------------------------------------------------
// keybd_event (en vez de SendInput) alcanza para este propósito
// puntual: no importa que esté deprecada, solo se necesita que el
// sistema registre una pulsación de tecla real para que
// SetForegroundWindow deje de estar bloqueado — no hace falta que
// otra ventana la "reciba" ni que el usuario la vea. Se usa dentro
// de forzar_foco justo antes de SetForegroundWindow.
// ======================================================

unsafe fn simular_pulsacion_alt() {
    keybd_event(VK_MENU as u8, 0, 0, 0);

    keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
}

// ======================================================
// 🔁 REAFIRMAR MODO DE VENTANA (BUG 7 y parte de BUG 8)
// ------------------------------------------------------
// El nShow que se le pasa a ShellExecuteExW es solo una SUGERENCIA —
// muchas apps (frameworks Qt/Electron modernos, y Lightburn en
// particular) la ignoran y llaman su propio ShowWindow al terminar de
// cargar, pisando el modo pedido. runtime.rs::forzar_primer_plano()
// ya encuentra la ventana nueva tras lanzar (para el foco); esta
// función reutiliza ese mismo hallazgo para, además, reafirmar el
// modo de ventana pedido con un ShowWindow explícito:
// • Minimizado: si la app se mostró igual (no está minimizada), se
//   fuerza SW_MINIMIZE — y a propósito NO se roba el foco acá (no
//   tendría sentido traer al frente una ventana que se pidió
//   minimizada).
// • Maximizado: si no quedó maximizada, se fuerza SW_MAXIMIZE, y
//   además sí se reafirma el foco (forzar_foco) — antes solo se
//   reafirmaba el foco, nunca el tamaño.
// • Ventana: comportamiento de siempre, solo reafirma el foco.
//
// LÍMITE: esto solo tiene efecto sobre una ventana NUEVA (lanzamiento
// fresco). Para apps de instancia única que reusan su ventana ya
// abierta (ej. Notepad++ agregando una pestaña) no hay ventana nueva
// que ajustar — ShellExecuteExW ni siquiera crea proceso/ventana en
// ese caso, así que el modo de ventana pedido no tiene forma de
// aplicarse ahí sin controlar la app por dentro (fuera de alcance).
// ======================================================

pub fn reafirmar_modo_ventana(hwnd: HWND, iniciar: &IniciarVentana) {
    unsafe {
        match iniciar {
            IniciarVentana::Minimizado => {
                if IsIconic(hwnd) == 0 {
                    ShowWindow(hwnd, SW_MINIMIZE);
                }
            }

            IniciarVentana::Maximizado => {
                if IsZoomed(hwnd) == 0 {
                    ShowWindow(hwnd, SW_MAXIMIZE);
                }

                forzar_foco(hwnd);
            }

            IniciarVentana::Ventana => {
                forzar_foco(hwnd);
            }
        }
    }
}
