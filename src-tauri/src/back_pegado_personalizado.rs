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
// &ContenidoPortapapeles del contenido que se está pegando (Texto o
// Imagen) — pegar() ya lo tiene calculado, se lo pasa para no tener
// que releerlo. Aparte de eso, consulta directo la app activa
// (back_app, una sola vez por click, PID+ruta juntos).
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
//     Con IMAGEN: relanza Photoshop con el script vacío ya escrito en
//     disco (para provocar la activación real de la ventana) y recién
//     después espera y pega. Con TEXTO: salta directo la activación
//     (no hace falta) y va directo al delay + pegado. En ambos casos
//     pega con el MISMO camino que usa el resto de RemapH:
//     crate::runtime::emitir_ctrl_v(). Ya no arma ni lanza un segundo
//     script .jsx de pegado — un relanzamiento menos de Photoshop por
//     click.
// delay_para_contenido()
//     Con IMAGEN usa el delay independiente de Photoshop
//     (config::delay_imagen_photoshop()); con TEXTO usa el mismo
//     timer corto que cualquier app genérica
//     (config::tiempo_espera_pegado_texto()) — Photoshop no necesita
//     un delay propio para texto, el problema (asentar contenido
//     pesado) es solo de imagen.
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
use crate::back_portapapeles_captura::ContenidoPortapapeles;

// Script "vacío" (no-op): no hace nada dentro de Photoshop, su único
// propósito es viajar en un relanzamiento propio para provocar la
// activación real de la ventana de Photoshop — la misma activación
// que, según la hipótesis en investigación, es la que de verdad hace
// que Photoshop revise si hay algo nuevo en el portapapeles (no
// ningún truco nuestro de robo-y-devuelve-foco).
const SCRIPT_VACIO: &str = "// no-op: solo dispara el relanzamiento/activación, no pega nada\n";

// ======================================================
// ⏱️ Con IMAGEN: config::delay_imagen_photoshop() (delay propio e
// independiente de Photoshop). Con TEXTO: config::tiempo_espera_
// pegado_texto() (el mismo timer corto que usa cualquier app
// genérica — ver delay_para_contenido() más abajo). Ninguno es una
// constante local: son valores configurables únicos, junto con el
// resto de los timers de portapapeles (config.rs).
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

pub fn intentar(contenido: &ContenidoPortapapeles) -> bool {
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

    ejecutar_pegado_photoshop(&ruta_exe, contenido)
}

// ======================================================
// 🔎 ¿ES PHOTOSHOP?
// ======================================================

fn es_photoshop(nombre_proceso: &str) -> bool {
    nombre_proceso.to_lowercase().contains("photoshop")
}

// ======================================================
// ⏱️ DELAY SEGÚN TIPO DE CONTENIDO
// ------------------------------------------------------
// Solo IMAGEN usa el delay independiente de Photoshop — es el único
// caso confirmado (Paint, mismo problema) donde hace falta darle
// tiempo real a la app para "asentar" contenido pesado. TEXTO usa el
// mismo timer corto que cualquier app genérica, Photoshop incluido.
// ======================================================

fn delay_para_contenido(contenido: &ContenidoPortapapeles) -> u64 {
    match contenido {
        ContenidoPortapapeles::Imagen { .. } => crate::config::delay_imagen_photoshop(),
        ContenidoPortapapeles::Texto(_) => crate::config::tiempo_espera_pegado_texto(),
    }
}

// ======================================================
// ▶️ ACTIVAR (solo imagen, script vacío reutilizado), ESPERAR, PEGAR
// (Ctrl+V)
// ------------------------------------------------------
// El relanzamiento de activación (script vacío) solo hace falta con
// IMAGEN — con TEXTO no es necesario, se saltea directo al delay de
// texto + Ctrl+V, un relanzamiento de Photoshop menos.
// ======================================================

fn ejecutar_pegado_photoshop(ruta_exe: &str, contenido: &ContenidoPortapapeles) -> bool {
    let es_imagen = matches!(contenido, ContenidoPortapapeles::Imagen { .. });

    if es_imagen {
        println!("🎨 [diag] pegado personalizado: relanzamiento de activación (script vacío reutilizado)");

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
    } else {
        println!(
            "🎨 [diag] pegado personalizado: contenido texto, salto la activación (no hace falta)"
        );
    }

    let delay = delay_para_contenido(contenido);
    println!(
        "🎨 [diag] pegado personalizado: espero {}ms antes del Ctrl+V ({})",
        delay,
        if es_imagen {
            "imagen, delay propio de Photoshop"
        } else {
            "texto, timer genérico"
        }
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
    // .spawn() en vez de .status(): no se espera a que el proceso
    // mensajero (el que reenvía el script a la instancia de Photoshop
    // ya abierta) termine de verdad — solo que arranque. El delay
    // configurable aplicado por el llamador antes del Ctrl+V (ver
    // delay_para_contenido()) ya cumple el rol de "darle tiempo a
    // Photoshop"; esperar ADEMÁS a que este proceso cierre era tiempo
    // doble. Se pierde la confirmación de "estado=" en el log (ahora
    // solo se sabe que arrancó, no que terminó bien), pero no afecta
    // el pegado en sí.
    match Command::new(ruta_exe).arg(ruta_script).spawn() {
        Ok(_hijo) => {
            println!("🎨 [diag] pegado personalizado: script de activación lanzado (sin esperar a que cierre)");
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
