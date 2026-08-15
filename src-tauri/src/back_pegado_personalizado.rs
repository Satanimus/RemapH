// ======================================================
// 🎨 back_pegado_personalizado
// ======================================================
// ETAPA 1 DEL PLAN "PEGADO PERSONALIZADO"
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Módulo aislado: decide si la app activa necesita un camino de
// pegado distinto al genérico (Ctrl+V simulado) y, si corresponde,
// lo ejecuta. Por ahora solo conoce Photoshop — el script vacío de
// activación vive embebido en el binario (include_str! / const, sin
// archivos sueltos en disco del usuario). Si en el futuro aparecen
// más programas con el mismo problema, se agrega acá un caso más
// (mismo patrón), sin tocar nada externo.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// back_portapapeles.rs::pegar(), justo donde antes se llamaba
// incondicionalmente a crate::runtime::emitir_ctrl_v() — ahí se
// intenta primero el camino personalizado y, si no corresponde,
// sigue con el camino genérico de siempre.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Nada como parámetro — consulta directo la app activa (back_app,
// una sola vez por click, PID+ruta juntos).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// bool: true si se ejecutó el camino personalizado (Photoshop),
// false si la app activa no tiene camino personalizado y debe
// seguir el pegado genérico.
// ------------------------------------------------------
// 5. Funciones del archivo
// intentar()
//     Punto de entrada único. Resuelve PID+ruta del programa activo
//     en una sola consulta (back_app::obtener_pid_y_ruta_activo) y,
//     si es Photoshop, dispara la secuencia activar→esperar→pegar.
// es_photoshop()
//     Compara el nombre de archivo del proceso activo.
// ejecutar_pegado_photoshop()
//     Relanza Photoshop con el script vacío ya escrito en disco (solo
//     para provocar la activación real de la ventana), espera
//     config::delay_entre_scripts_photoshop() y después pega con el
//     MISMO camino que usa el resto de RemapH: crate::runtime::
//     emitir_ctrl_v(). Ya no arma ni lanza un segundo script .jsx de
//     pegado — un relanzamiento menos de Photoshop por click.
// ruta_script_vacio()
//     Devuelve la ruta del script vacío en disco, escribiéndolo una
//     única vez (primer uso) en vez de en cada click — el contenido
//     nunca cambia, así que no hace falta recrearlo cada vez.
// lanzar_script_photoshop()
//     Manda un archivo de script ya existente a ejecutar en la
//     instancia de Photoshop ya abierta.
// ======================================================

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use crate::back_app;

// Script "vacío" (no-op): no hace nada dentro de Photoshop, su único
// propósito es viajar en un relanzamiento propio para provocar la
// activación real de la ventana de Photoshop — la misma activación
// que, según la hipótesis en investigación, es la que de verdad hace
// que Photoshop revise si hay algo nuevo en el portapapeles (no
// ningún truco nuestro de robo-y-devuelve-foco).
const SCRIPT_VACIO: &str = "// no-op: solo dispara el relanzamiento/activación, no pega nada\n";

// ======================================================
// ⏱️ Ver config::delay_entre_scripts_photoshop() / establecer_
// delay_entre_scripts_photoshop() (config.rs) — espera entre el
// relanzamiento con SCRIPT_VACIO (activación) y el Ctrl+V simulado
// (pegado real). Ya no es una constante local: se dejó de fase de
// pruebas fijas para pasar a ser un valor configurable único, junto
// con el resto de los timers de portapapeles.
// ======================================================

// Ruta del script vacío en disco. Se escribe una sola vez (primer
// llamado a ruta_script_vacio() en toda la vida del proceso) y se
// reutiliza en todos los clicks siguientes — antes se reescribía a un
// archivo temporal con nombre único en CADA click, que era trabajo
// (y tiempo) de más para un contenido que nunca cambia.
static RUTA_SCRIPT_VACIO: OnceLock<Option<PathBuf>> = OnceLock::new();

// ======================================================
// 🎯 PUNTO DE ENTRADA
// ======================================================

pub fn intentar() -> bool {
    let Some((_pid, ruta_exe)) = back_app::obtener_pid_y_ruta_activo() else {
        return false;
    };

    let Some(nombre_activo) = std::path::Path::new(&ruta_exe)
        .file_name()
        .map(|nombre| nombre.to_string_lossy().to_string())
    else {
        return false;
    };

    if !es_photoshop(&nombre_activo) {
        return false;
    }

    println!("🎨 [diag] pegado personalizado: app activa es Photoshop, uso activación + Ctrl+V");

    ejecutar_pegado_photoshop(&ruta_exe)
}

// ======================================================
// 🔎 ¿ES PHOTOSHOP?
// ======================================================

fn es_photoshop(nombre_proceso: &str) -> bool {
    nombre_proceso.to_lowercase().contains("photoshop")
}

// ======================================================
// ▶️ ACTIVAR (script vacío reutilizado), ESPERAR, PEGAR (Ctrl+V)
// ======================================================

fn ejecutar_pegado_photoshop(ruta_exe: &str) -> bool {
    println!(
        "🎨 [diag] pegado personalizado: relanzamiento de activación (script vacío reutilizado)"
    );

    let activacion_ok = match ruta_script_vacio() {
        Some(ruta_script) => lanzar_script_photoshop(ruta_exe, &ruta_script),
        None => {
            println!("🎨 [diag] pegado personalizado: no hay script vacío disponible, salto la activación");
            false
        }
    };

    if !activacion_ok {
        println!("🎨 [diag] pegado personalizado: el relanzamiento de activación falló (sigo igual con el Ctrl+V)");
    }

    let delay = crate::config::delay_entre_scripts_photoshop();
    println!(
        "🎨 [diag] pegado personalizado: espero {}ms antes del Ctrl+V",
        delay
    );
    std::thread::sleep(std::time::Duration::from_millis(delay));

    // Mismo camino que usa pegar() para cualquier app sin ruta
    // personalizada — antes acá se lanzaba un segundo script .jsx
    // (app.activeDocument.paste()) en una segunda relanzada de
    // Photoshop; ahora se simula Ctrl+V como el camino genérico, sin
    // ese segundo relanzamiento.
    println!("🎨 [diag] pegado personalizado: disparo Ctrl+V simulado (mismo camino que el pegado genérico)");
    crate::runtime::emitir_ctrl_v();

    true
}

// ======================================================
// 📄 RUTA DEL SCRIPT VACÍO (se escribe una sola vez)
// ======================================================

fn ruta_script_vacio() -> Option<PathBuf> {
    RUTA_SCRIPT_VACIO
        .get_or_init(|| {
            let ruta = std::env::temp_dir().join("remaph_photoshop_activar.jsx");

            match fs::write(&ruta, SCRIPT_VACIO) {
                Ok(()) => {
                    println!(
                        "🎨 [diag] pegado personalizado: script vacío escrito una vez en {:?}",
                        ruta
                    );
                    Some(ruta)
                }
                Err(error) => {
                    println!(
                        "🎨 [diag] pegado personalizado: error escribiendo el script vacío: {}",
                        error
                    );
                    None
                }
            }
        })
        .clone()
}

// ======================================================
// ▶️ EJECUTAR UN SCRIPT YA ESCRITO EN LA INSTANCIA YA ABIERTA
// ======================================================

fn lanzar_script_photoshop(ruta_exe: &str, ruta_script: &std::path::Path) -> bool {
    // .status() en vez de .spawn(): se espera a que el proceso que
    // reenvía el script a la instancia de Photoshop ya abierta
    // termine de verdad, en vez de devolver el control de inmediato.
    match Command::new(ruta_exe).arg(ruta_script).status() {
        Ok(estado) => {
            println!(
                "🎨 [diag] pegado personalizado: script de activación enviado a Photoshop, estado={}",
                estado
            );
            true
        }
        Err(error) => {
            println!(
                "🎨 [diag] pegado personalizado: error lanzando Photoshop con el script de activación: {}",
                error
            );
            false
        }
    }
}
