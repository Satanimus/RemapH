// ======================================================
// 🖥️ back_app RemapH V3
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
// obtener_ruta_proceso()
//     Obtiene la ruta del ejecutable.
// extraer_icono()
//     Extrae el ícono del ejecutable.
// es_proceso_windows()
//     Filtra procesos propios de Windows.
// iniciar_monitor()
//     Instala el aviso de cambio de foco de Windows (una sola vez) y arranca su loop de mensajes en un hilo aparte.
// revisar_apps()
//     Reevalúa el estado de todas las apps vigiladas y actualiza cache.
// esta_abierta()
//     Indica si un ejecutable aparece entre los procesos corriendo ahora mismo.
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

use windows_sys::Win32::UI::Shell::ExtractIconExW;

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

unsafe fn obtener_ruta_proceso(pid: u32) -> Option<String> {
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
// EXTRAER ÍCONO
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

        if extraidos == 0 {
            return None;
        }

        if icono_pequeno.is_null() {
            if !icono_grande.is_null() {
                DestroyIcon(icono_grande);
            }

            return None;
        }

        let hicon = icono_pequeno;

        let mut icon_info: ICONINFO = std::mem::zeroed();

        if GetIconInfo(hicon, &mut icon_info) == 0 {
            DestroyIcon(hicon);

            if !icono_grande.is_null() {
                DestroyIcon(icono_grande);
            }

            return None;
        }

        let hbm_color = icon_info.hbmColor;

        let hbm_mask = icon_info.hbmMask;

        if hbm_color.is_null() {
            DeleteObject(hbm_mask);

            DestroyIcon(hicon);

            if !icono_grande.is_null() {
                DestroyIcon(icono_grande);
            }

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

            if !icono_grande.is_null() {
                DestroyIcon(icono_grande);
            }

            return None;
        }

        let ancho = bitmap.bmWidth as u32;

        let alto = bitmap.bmHeight as u32;

        if ancho == 0 || alto == 0 {
            DeleteObject(hbm_color);

            DeleteObject(hbm_mask);

            DestroyIcon(hicon);

            if !icono_grande.is_null() {
                DestroyIcon(icono_grande);
            }

            return None;
        }

        let hdc = CreateCompatibleDC(std::ptr::null_mut());

        if hdc.is_null() {
            DeleteObject(hbm_color);

            DeleteObject(hbm_mask);

            DestroyIcon(hicon);

            if !icono_grande.is_null() {
                DestroyIcon(icono_grande);
            }

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

        if !icono_grande.is_null() {
            DestroyIcon(icono_grande);
        }

        if filas_copiadas == 0 {
            return None;
        }

        // ==============================================
        // BGRA → RGBA
        // ==============================================

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
}

// ======================================================
// 👁️ MONITOR DE APPS (foco)
// ======================================================

use crate::cache;
use crate::perfil_cache::AppCache;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG,
    WINEVENT_OUTOFCONTEXT,
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
