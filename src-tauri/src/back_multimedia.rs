// ======================================================
// 🎚️ Back_Multimedia
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Ejecuta físicamente una AccionCache::Multimedia.
//
// Alcance Global: emite el comando con SendInput/keybd_event de
// Windows usando el Virtual-Key real de cada comando
// (VK_VOLUME_UP, VK_MEDIA_PLAY_PAUSE, etc.) — a propósito NO pasa
// por Interception/back_interception, porque estas teclas son
// virtuales del sistema, no scancodes físicos de un dispositivo.
//
// Alcance En App: busca la sesión de audio del proceso indicado vía
// winmix y lee/escribe su volumen o mute — Volumen únicamente
// (Reproducción no tiene equivalente por sesión de audio, solo
// existe como Global).
//
// No conoce Cache. No conoce AnalizadorTrigger. No decide cuándo
// ejecutar — solo ejecuta lo que Runtime le pide.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// runtime.rs (ejecutar_accion, brazo AccionCache::Multimedia).
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// ejecutar(): un ComandoMultimedia + un AlcanceMultimedia ya
// resueltos por compilador.rs (el programa de En App ya viene
// como String, no hace falta volver a mirar el perfil).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// Nada — efecto físico (tecla virtual emitida o volumen de una
// sesión de audio modificado).
// ------------------------------------------------------
// 5. Funciones del archivo
//
// ejecutar()
//     Punto de entrada único. Despacha según AlcanceMultimedia.
//
// ejecutar_global()
//     Arma el VK del comando y lo emite con SendInput
//     (down + up, un solo pulse).
//
// vk_de_comando()
//     ComandoMultimedia → Virtual-Key code real de Windows.
//
// enviar_vk()
//     SendInput crudo: un INPUT de teclado down, uno de teclado up.
//
// ejecutar_en_app()
//     Alcance En App (winmix): busca la sesión de audio del proceso
//     y ajusta volumen (Subir/Bajar) o alterna mute (Silenciar).
//
// coincide_programa()
//     Compara la ruta de una sesión de winmix contra el nombre de
//     programa guardado en la fila (mismo criterio que back_app.rs).
//
// ajustar_volumen() / alternar_mute()
//     Operación real sobre una sesión de winmix ya encontrada.
// ------------------------------------------------------

use crate::config;

use crate::perfil_cache::{AlcanceMultimedia, ComandoMultimedia};

use std::path::Path;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP,
    VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
};

// ======================================================
// 🚀 EJECUTAR
// ======================================================

pub fn ejecutar(comando: &ComandoMultimedia, alcance: &AlcanceMultimedia) {
    match alcance {
        AlcanceMultimedia::Global => ejecutar_global(comando),

        AlcanceMultimedia::EnApp { programa } => ejecutar_en_app(comando, programa),
    }
}

// ======================================================
// 🌐 EJECUTAR GLOBAL (SendInput, VK real)
// ======================================================

fn ejecutar_global(comando: &ComandoMultimedia) {
    enviar_vk(vk_de_comando(comando));
}

// ======================================================
// 🔑 VK DE COMANDO
// ------------------------------------------------------
// Las 7 teclas multimedia reales de Windows. Silenciar/Play-Pausa/
// Detener/Siguiente/Anterior no tienen "estado" que consultar acá
// (son pulse puro) — a diferencia del alcance En App, donde
// Silenciar sí necesita leer el mute actual antes de togglearlo.
// ======================================================

fn vk_de_comando(comando: &ComandoMultimedia) -> VIRTUAL_KEY {
    match comando {
        ComandoMultimedia::VolumenSubir => VK_VOLUME_UP,
        ComandoMultimedia::VolumenBajar => VK_VOLUME_DOWN,
        ComandoMultimedia::Silenciar => VK_VOLUME_MUTE,
        ComandoMultimedia::PlayPausa => VK_MEDIA_PLAY_PAUSE,
        ComandoMultimedia::Detener => VK_MEDIA_STOP,
        ComandoMultimedia::Siguiente => VK_MEDIA_NEXT_TRACK,
        ComandoMultimedia::Anterior => VK_MEDIA_PREV_TRACK,
    }
}

// ======================================================
// 📤 ENVIAR VK (SendInput crudo: down + up)
// ------------------------------------------------------
// Las teclas multimedia son teclas extendidas del teclado virtual
// de Windows — se marcan con KEYEVENTF_EXTENDEDKEY para que el
// sistema las trate igual que las manda un teclado multimedia real.
// ======================================================

fn enviar_vk(vk: VIRTUAL_KEY) {
    let mut eventos = [input_teclado(vk, false), input_teclado(vk, true)];

    unsafe {
        SendInput(
            eventos.len() as u32,
            eventos.as_mut_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
    }
}

fn input_teclado(vk: VIRTUAL_KEY, soltar: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if soltar {
                    KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_EXTENDEDKEY
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

// ======================================================
// 🎧 EJECUTAR EN APP (winmix)
// ------------------------------------------------------
// Solo tiene sentido para los 3 comandos de Volumen — Reproducción
// nunca llega acá (compilador.rs siempre produce Global para esos,
// ver convertir_alcance_multimedia). Si no se encuentra ninguna
// sesión de audio para `programa` (la app está cerrada, o nunca
// generó sonido todavía), no hace nada — no hay nada que ajustar.
// ======================================================

fn ejecutar_en_app(comando: &ComandoMultimedia, programa: &str) {
    let winmix = winmix::WinMix::default();

    let Ok(sesiones) = (unsafe { winmix.enumerate() }) else {
        return;
    };

    let Some(sesion) = sesiones
        .into_iter()
        .find(|sesion| coincide_programa(&sesion.path, programa))
    else {
        return;
    };

    match comando {
        ComandoMultimedia::VolumenSubir => ajustar_volumen(&sesion, delta_normalizado()),

        ComandoMultimedia::VolumenBajar => ajustar_volumen(&sesion, -delta_normalizado()),

        ComandoMultimedia::Silenciar => alternar_mute(&sesion),

        // No debería llegar acá — compilador.rs solo produce EnApp
        // para comandos de Volumen (ver ComandoMultimedia::es_de_volumen).
        _ => {}
    }
}

// ======================================================
// 🔎 COINCIDE PROGRAMA
// ------------------------------------------------------
// `programa` (columna App de la fila) guarda solo el nombre de
// archivo (ej. "firefox.exe" — ver back_app.rs::obtener_programa_activo).
// `ruta` es la ruta completa que entrega winmix (ej.
// "C:\...\firefox.exe") — se compara por el nombre de archivo final,
// case-insensitive, mismo criterio que back_app.rs::esta_abierta.
// ======================================================

fn coincide_programa(ruta: &str, programa: &str) -> bool {
    Path::new(ruta)
        .file_name()
        .map(|nombre| nombre.to_string_lossy().eq_ignore_ascii_case(programa))
        .unwrap_or(false)
}

// ======================================================
// 🔊 DELTA NORMALIZADO
// ------------------------------------------------------
// config::delta_volumen() está en unidades 0-100 (mismo criterio que
// un % en la UI) — winmix trabaja en 0.0-1.0.
// ======================================================

fn delta_normalizado() -> f32 {
    config::delta_volumen() as f32 / 100.0
}

// ======================================================
// 🔊 AJUSTAR VOLUMEN (Subir/Bajar, En App)
// ------------------------------------------------------
// winmix es absoluto (no hay "subir un paso" nativo, a diferencia de
// VK_VOLUME_UP/DOWN en Global): hay que leer el volumen actual,
// sumar/restar el delta, y volver a escribir — con clamp para no
// pasarse de 0.0/1.0.
// ======================================================

fn ajustar_volumen(sesion: &winmix::Session, delta: f32) {
    let Ok(actual) = (unsafe { sesion.vol.get_master_volume() }) else {
        return;
    };

    let nuevo = (actual + delta).clamp(0.0, 1.0);

    let _ = unsafe { sesion.vol.set_master_volume(nuevo) };
}

// ======================================================
// 🔇 ALTERNAR MUTE (Silenciar, En App)
// ------------------------------------------------------
// Toggle real: lee el mute actual y lo invierte (mismo criterio que
// VK_VOLUME_MUTE en Global, que también alterna).
// ======================================================

fn alternar_mute(sesion: &winmix::Session) {
    let Ok(actual) = (unsafe { sesion.vol.get_mute() }) else {
        return;
    };

    let _ = unsafe { sesion.vol.set_mute(!actual) };
}
