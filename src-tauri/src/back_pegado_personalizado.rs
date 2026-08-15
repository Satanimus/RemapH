// ======================================================
// 🎨 back_pegado_personalizado
// ======================================================
// ETAPA 1 DEL PLAN "PEGADO PERSONALIZADO"
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Módulo aislado: decide si la app activa necesita un camino de
// pegado distinto al genérico (Ctrl+V simulado) y, si corresponde,
// lo ejecuta. Por ahora solo conoce Photoshop — los scripts viven
// embebidos en el binario (include_str! / const, sin archivos
// sueltos en disco del usuario). Si en el futuro aparecen más
// programas con el mismo problema, se agrega acá un caso más
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
//     si es Photoshop, dispara la secuencia de doble relanzamiento.
// es_photoshop()
//     Compara el nombre de archivo del proceso activo.
// ejecutar_doble_script_photoshop()
//     Resuelve PID→ruta del ejecutable UNA sola vez y hace dos
//     relanzamientos separados por config::delay_entre_scripts_
//     photoshop(): primero un script vacío (solo para provocar la
//     activación real de Photoshop), después el script que pega de
//     verdad.
// lanzar_script_photoshop()
//     Escribe un contenido de script a un archivo temporal único y
//     lo manda a ejecutar en la instancia ya abierta.
// ======================================================

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::back_app;

const SCRIPT_PHOTOSHOP: &str = include_str!("../scripts/photoshop_pegar.jsx");

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
// relanzamiento con SCRIPT_VACIO (activación) y el relanzamiento con
// SCRIPT_PHOTOSHOP (pegado real). Ya no es una constante local:
// se dejó de fase de pruebas fijas para pasar a ser un valor
// configurable único, junto con el resto de los timers de
// portapapeles.
// ======================================================

// Contador para que cada invocación use un nombre de archivo
// temporal distinto — evita que Photoshop pueda tratar dos pedidos
// seguidos como "el mismo de antes" si llegan cerca en el tiempo.
static CONTADOR: AtomicU32 = AtomicU32::new(0);

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

    println!("🎨 [diag] pegado personalizado: app activa es Photoshop, uso doble relanzamiento");

    ejecutar_doble_script_photoshop(&ruta_exe)
}

// ======================================================
// 🔎 ¿ES PHOTOSHOP?
// ======================================================

fn es_photoshop(nombre_proceso: &str) -> bool {
    nombre_proceso.to_lowercase().contains("photoshop")
}

// ======================================================
// ▶️▶️ DOS RELANZAMIENTOS: ACTIVAR, ESPERAR, PEGAR
// ======================================================

fn ejecutar_doble_script_photoshop(ruta_exe: &str) -> bool {
    println!("🎨 [diag] pegado personalizado: relanzamiento 1/2 (activación, script vacío)");
    let activacion_ok = lanzar_script_photoshop(ruta_exe, SCRIPT_VACIO, "activar");

    let delay = crate::config::delay_entre_scripts_photoshop();
    println!(
        "🎨 [diag] pegado personalizado: espero {}ms entre relanzamientos",
        delay
    );
    std::thread::sleep(std::time::Duration::from_millis(delay));

    println!("🎨 [diag] pegado personalizado: relanzamiento 2/2 (pegado real)");
    let pegado_ok = lanzar_script_photoshop(ruta_exe, SCRIPT_PHOTOSHOP, "pegar");

    if !activacion_ok {
        println!("🎨 [diag] pegado personalizado: el relanzamiento de activación falló (sigo igual con el de pegado)");
    }

    pegado_ok
}

// ======================================================
// ▶️ EJECUTAR UN SCRIPT EN LA INSTANCIA YA ABIERTA
// ======================================================

fn lanzar_script_photoshop(ruta_exe: &str, contenido_script: &str, etiqueta: &str) -> bool {
    let marca = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duracion| duracion.as_millis())
        .unwrap_or(0);

    let contador = CONTADOR.fetch_add(1, Ordering::Relaxed);

    let ruta_temporal = std::env::temp_dir().join(format!(
        "remaph_photoshop_{}_{}_{}.jsx",
        etiqueta, marca, contador
    ));

    if let Err(error) = fs::write(&ruta_temporal, contenido_script) {
        println!(
            "🎨 [diag] pegado personalizado ({}): error escribiendo script temporal: {}",
            etiqueta, error
        );
        return false;
    }

    // .status() en vez de .spawn(): se espera a que el proceso que
    // reenvía el script a la instancia de Photoshop ya abierta
    // termine de verdad, en vez de devolver el control de inmediato.
    match Command::new(ruta_exe).arg(&ruta_temporal).status() {
        Ok(estado) => {
            println!(
                "🎨 [diag] pegado personalizado ({}): script enviado a Photoshop, estado={}",
                etiqueta, estado
            );
            true
        }
        Err(error) => {
            println!(
                "🎨 [diag] pegado personalizado ({}): error lanzando Photoshop con el script: {}",
                etiqueta, error
            );
            false
        }
    }
}
