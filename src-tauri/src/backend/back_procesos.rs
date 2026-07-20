// ======================================================
// 🖥️ Back_Procesos RemapH V3
// ------------------------------------------------------
// Backend Platform.
//
// Responsable de:
//   - Enumerar procesos con ventana visible (estilo Alt+Tab).
//   - Extraer el ícono de un ejecutable.
//
// No conoce:
//   - Perfiles.
//   - Runtime.
//   - UI.
// ======================================================

use std::collections::HashSet;
use std::path::Path;

use windows_sys::Win32::Foundation::{
    CloseHandle,
    HWND,
    LPARAM,
    BOOL,
};

use windows_sys::Win32::System::Threading::{
    OpenProcess,
    QueryFullProcessImageNameW,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows,
    IsWindowVisible,
    GetWindowTextLengthW,
    GetWindowThreadProcessId,
    GetWindowLongW,
    GetIconInfo,
    DestroyIcon,
    ICONINFO,
    GWL_EXSTYLE,
    WS_EX_TOOLWINDOW,
    WS_EX_APPWINDOW,
};

use windows_sys::Win32::UI::Shell::{
    SHGetFileInfoW,
    SHFILEINFOW,
    SHGFI_ICON,
    SHGFI_SMALLICON,
};

use windows_sys::Win32::Graphics::Gdi::{
    GetObjectW,
    CreateCompatibleDC,
    GetDIBits,
    DeleteDC,
    DeleteObject,
    BITMAP,
    BITMAPINFO,
    DIB_RGB_COLORS,
};


// ======================================================
// 🪟 PROCESO CON VENTANA
// ======================================================

pub struct ProcesoVentana {

    pub nombre:
        String,

    pub ruta:
        String,

}


// ======================================================
// 🎨 ÍCONO CRUDO
// ------------------------------------------------------
// Píxeles en formato RGBA, listos para volcarse en un
// ImageData del lado de la UI.
// ======================================================

pub struct IconoRaw {

    pub ancho:
        u32,

    pub alto:
        u32,

    pub pixeles:
        Vec<u8>,

}


// ======================================================
// 📋 ENUMERAR PROCESOS CON VENTANA VISIBLE
// ------------------------------------------------------
// Equivalente a la lista de Alt+Tab: ventanas visibles,
// con título, que no sean ventanas "herramienta".
//
// Deduplicado por nombre de ejecutable.
// ======================================================

pub fn enumerar_procesos_ventana() -> Vec<ProcesoVentana> {

    let mut lista: Vec<ProcesoVentana> =
        Vec::new();

    unsafe {

        EnumWindows(
            Some(callback_enumerar),
            &mut lista as *mut _ as isize,
        );

    }

    let mut vistos: HashSet<String> =
        HashSet::new();

    lista.retain(
        |proceso|
            vistos.insert(
                proceso.nombre.to_lowercase()
            )
    );

    lista

}


// ======================================================
// 🔁 CALLBACK DE ENUMERACIÓN
// ======================================================

unsafe extern "system" fn callback_enumerar(
    hwnd: HWND,
    lparam: LPARAM,
) -> BOOL {

    let lista =
        &mut *(lparam as *mut Vec<ProcesoVentana>);

    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }

    if GetWindowTextLengthW(hwnd) == 0 {
        return 1;
    }

    let estilo_ex =
        GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

    let es_herramienta =
        (estilo_ex & WS_EX_TOOLWINDOW) != 0;

    let es_app =
        (estilo_ex & WS_EX_APPWINDOW) != 0;

    if es_herramienta && !es_app {
        return 1;
    }

    let mut pid: u32 = 0;

    GetWindowThreadProcessId(
        hwnd,
        &mut pid,
    );

    if pid == 0 {
        return 1;
    }

    if let Some(ruta) = obtener_ruta_proceso(pid) {

        let nombre =
            Path::new(&ruta)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ruta.clone());

        lista.push(
            ProcesoVentana {
                nombre,
                ruta,
            }
        );

    }

    1

}


// ======================================================
// 📄 RUTA DEL EJECUTABLE A PARTIR DEL PID
// ======================================================

unsafe fn obtener_ruta_proceso(
    pid: u32,
) -> Option<String> {

    let handle =
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            0,
            pid,
        );

    if handle == 0 {
        return None;
    }

    let mut buffer: [u16; 260] =
        [0; 260];

    let mut tamano: u32 =
        buffer.len() as u32;

    let exito =
        QueryFullProcessImageNameW(
            handle,
            0,
            buffer.as_mut_ptr(),
            &mut tamano,
        );

    CloseHandle(handle);

    if exito == 0 {
        return None;
    }

    Some(
        String::from_utf16_lossy(
            &buffer[..tamano as usize]
        )
    )

}


// ======================================================
// 🎨 EXTRAER ÍCONO DE UN EJECUTABLE
// ------------------------------------------------------
// Recibe la ruta completa de un .exe y devuelve su ícono
// pequeño (16x16 / 32x32 según el sistema) como píxeles
// RGBA planos.
// ======================================================

pub fn extraer_icono(
    ruta: &str,
) -> Option<IconoRaw> {

    unsafe {

        let ruta_ancha: Vec<u16> =
            ruta
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

        let mut info: SHFILEINFOW =
            std::mem::zeroed();

        let resultado =
            SHGetFileInfoW(
                ruta_ancha.as_ptr(),
                0,
                &mut info,
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON,
            );

        if resultado == 0 || info.hIcon == 0 {
            return None;
        }

        let hicon =
            info.hIcon;

        let mut icon_info: ICONINFO =
            std::mem::zeroed();

        if GetIconInfo(hicon, &mut icon_info) == 0 {

            DestroyIcon(hicon);

            return None;

        }

        let mut bitmap: BITMAP =
            std::mem::zeroed();

        GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bitmap as *mut BITMAP as *mut core::ffi::c_void,
        );

        let ancho =
            bitmap.bmWidth as u32;

        let alto =
            bitmap.bmHeight as u32;

        if ancho == 0 || alto == 0 {

            DeleteObject(icon_info.hbmColor);
            DeleteObject(icon_info.hbmMask);
            DestroyIcon(hicon);

            return None;

        }

        let hdc =
            CreateCompatibleDC(0);

        let mut bmi: BITMAPINFO =
            std::mem::zeroed();

        bmi.bmiHeader.biSize =
            std::mem::size_of_val(&bmi.bmiHeader) as u32;

        bmi.bmiHeader.biWidth =
            ancho as i32;

        bmi.bmiHeader.biHeight =
            -(alto as i32);

        bmi.bmiHeader.biPlanes =
            1;

        bmi.bmiHeader.biBitCount =
            32;

        bmi.bmiHeader.biCompression =
            0; // BI_RGB

        let mut pixeles: Vec<u8> =
            vec![0u8; (ancho * alto * 4) as usize];

        let filas_copiadas =
            GetDIBits(
                hdc,
                icon_info.hbmColor,
                0,
                alto,
                pixeles.as_mut_ptr() as *mut core::ffi::c_void,
                &mut bmi,
                DIB_RGB_COLORS,
            );

        DeleteDC(hdc);
        DeleteObject(icon_info.hbmColor);
        DeleteObject(icon_info.hbmMask);
        DestroyIcon(hicon);

        if filas_copiadas == 0 {
            return None;
        }

        // BGRA → RGBA
        let mut i = 0;

        while i < pixeles.len() {

            pixeles.swap(i, i + 2);

            i += 4;

        }

        Some(
            IconoRaw {
                ancho,
                alto,
                pixeles,
            }
        )

    }

}