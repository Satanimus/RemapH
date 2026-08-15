// ======================================================
// 🎨 back_pegado_personalizado
// ======================================================
// ETAPA 1 DEL PLAN "PEGADO PERSONALIZADO"
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Módulo aislado: decide si la app activa necesita un camino de
// pegado distinto al genérico (Ctrl+V simulado) y, si corresponde,
// lo ejecuta. Por ahora solo conoce Photoshop — el script vive
// embebido en el binario (include_str!, carpeta scripts/ del repo),
// no como archivo suelto en disco del usuario. Si en el futuro
// aparecen más programas con el mismo problema, se agrega acá un
// caso más (mismo patrón), sin tocar nada externo.
// ------------------------------------------------------
// 2. ¿Quién llama este archivo?
// Etapa 2: back_portapapeles.rs::pegar(), justo donde hoy se llama
// incondicionalmente a crate::runtime::emitir_ctrl_v() — ahí se va a
// intentar primero el camino personalizado y, si no corresponde,
// seguir con el camino genérico de siempre. Todavía no está
// conectado.
// ------------------------------------------------------
// 3. ¿Qué información recibe?
// Nada como parámetro — consulta directo la app activa (back_app).
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// bool: true si se ejecutó el script personalizado (Photoshop),
// false si la app activa no tiene camino personalizado y debe
// seguir el pegado genérico.
// ------------------------------------------------------
// 5. Funciones del archivo
// intentar()
//     Punto de entrada único. Revisa la app activa y, si es
//     Photoshop, escribe el script embebido a un archivo temporal y
//     lo manda a ejecutar en la instancia ya abierta.
// es_photoshop()
//     Compara el nombre de archivo del proceso activo.
// ejecutar_script_photoshop()
//     Resuelve PID→ruta del ejecutable, escribe el .jsx embebido a
//     %TEMP% y lanza "<photoshop.exe> <script.jsx>".
// ======================================================

use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::back_app;

const SCRIPT_PHOTOSHOP: &str = include_str!("../scripts/photoshop_pegar.jsx");

// Contador para que cada invocación use un nombre de archivo
// temporal distinto — evita que Photoshop pueda tratar dos pedidos
// seguidos como "el mismo de antes" si llegan cerca en el tiempo.
static CONTADOR: AtomicU32 = AtomicU32::new(0);

// ======================================================
// 🎯 PUNTO DE ENTRADA
// ======================================================

pub fn intentar() -> bool {
    let Some(nombre_activo) = back_app::obtener_programa_activo() else {
        return false;
    };

    if !es_photoshop(&nombre_activo) {
        return false;
    }

    println!("🎨 [diag] pegado personalizado: app activa es Photoshop, uso script dedicado");

    // DIAGNÓSTICO: delay grande a propósito (600) para confirmar si el
    // desfasaje de "pega el ítem anterior" es un tema de timing —
    // Photoshop necesitando más tiempo del que le dábamos (600ms,
    // heredado del ajuste para Paint) para terminar de invalidar su
    // caché interna del portapapeles tras forzar_relectura_
    // portapapeles(). Si con esto el desfasaje desaparece, se ajusta
    // a un valor más razonable después; si sigue igual, el problema
    // no es de timing y hay que mirar otra cosa.
    std::thread::sleep(std::time::Duration::from_millis(600));

    ejecutar_script_photoshop()
}

// ======================================================
// 🔎 ¿ES PHOTOSHOP?
// ======================================================

fn es_photoshop(nombre_proceso: &str) -> bool {
    nombre_proceso.to_lowercase().contains("photoshop")
}

// ======================================================
// ▶️ EJECUTAR SCRIPT EN LA INSTANCIA YA ABIERTA
// ======================================================

fn ejecutar_script_photoshop() -> bool {
    let Some(pid) = back_app::obtener_pid_activo() else {
        println!("🎨 [diag] pegado personalizado: no se pudo obtener PID de la app activa");
        return false;
    };

    let Some(ruta_exe) = (unsafe { back_app::obtener_ruta_proceso(pid) }) else {
        println!("🎨 [diag] pegado personalizado: no se pudo resolver la ruta del ejecutable");
        return false;
    };

    let marca = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duracion| duracion.as_millis())
        .unwrap_or(0);

    let contador = CONTADOR.fetch_add(1, Ordering::Relaxed);

    let ruta_temporal =
        std::env::temp_dir().join(format!("remaph_photoshop_pegar_{}_{}.jsx", marca, contador));

    if let Err(error) = fs::write(&ruta_temporal, SCRIPT_PHOTOSHOP) {
        println!(
            "🎨 [diag] pegado personalizado: error escribiendo script temporal: {}",
            error
        );
        return false;
    }

    // .status() en vez de .spawn(): se espera a que el proceso que
    // reenvía el script a la instancia de Photoshop ya abierta
    // termine de verdad, en vez de devolver el control de inmediato.
    // Sin esto, pegar() volvía OK antes de que el reenvío terminara,
    // y un click siguiente podía escribir contenido nuevo al
    // portapapeles mientras el pedido anterior todavía no había
    // llegado a ejecutarse adentro de Photoshop — eso producía el
    // desfasaje de "un paso atrás" (clickear el ítem 2 pegaba el 1,
    // clickear el 3 pegaba el 2, etc.).
    match Command::new(&ruta_exe).arg(&ruta_temporal).status() {
        Ok(estado) => {
            println!(
                "🎨 [diag] pegado personalizado: script enviado a Photoshop, estado={}",
                estado
            );
            true
        }
        Err(error) => {
            println!(
                "🎨 [diag] pegado personalizado: error lanzando Photoshop con el script: {}",
                error
            );
            false
        }
    }
}
