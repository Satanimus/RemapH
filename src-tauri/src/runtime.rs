// ======================================================
// ⚙️ Runtime RemapH V3
// ======================================================
// ETAPA 5 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Motor encargado de ejecutar órdenes compiladas.
//
// Recibe una orden desde Cache y la transforma
// en una secuencia de pasos de ejecución.
// *Nota: detener o reiniciar siempre se hace al finalizar un paso para no dejar teclas tomadas.
// Runtime:
//
// - Ejecuta acciones.
// - Ejecuta macros desde archivos.
// - Controla esperas.
// - Administra instancias de ejecución.
// - Coordina la comunicación con los backends.
//
// Runtime NO conoce:
//
// - UI.
// - Perfil.json.
// - Captura.
// - Cache.
// - Cómo se genera un trigger.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// OrdenRuntime:
//
// - Acción.
// - Extra.
// - Identificador.
//
// La acción ya viene preparada desde perfil_cache.
//
// Runtime no interpreta perfiles.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Principalmente:
//
// Cache.
//
// Flujo:
//
// Entrada física
//      ↓
// Cache
//      ↓
// OrdenRuntime
//      ↓
// Runtime
//      ↓
// Backend correspondiente
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Envía instrucciones directamente al backend
// correspondiente para cada paso.
//
// Un mismo proceso puede utilizar varios backends.
//
// Ejemplo:
//
// Ctrl Down
//      ↓
// back_teclado
//
// Click Down
//      ↓
// back_mouse
//
// Click Up
//      ↓
// back_mouse
//
// Ctrl Up
//      ↓
// back_teclado
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// ejecutar()
//     Punto principal de entrada.
//
// ejecutar_accion()
//     Decide qué tipo de acción ejecutar.
//
// ejecutar_macro()
//     Ejecuta un archivo Runtime paso a paso.
//
// ejecutar_linea()
//     Interpreta una línea del idioma Runtime.
//
// resolver_input()
//     Convierte un identificador interno en InputId.
//
// emitir()
//     Envía un paso al backend correspondiente.
//
// ejecutar_down()
//     Ejecuta un Down.
//
// ejecutar_up()
//     Ejecuta un Up.
//
// ejecutar_pulse()
//     Ejecuta un Pulse.
//
// esperar()
//     Ejecuta pausas temporales.
//
// ejecutar_extra()
//     Ejecuta comportamientos especiales.
//
// detener()
//     Finaliza una instancia de ejecución.
//
// ------------------------------------------------------
// Idioma Runtime:
//
// Una línea = un paso.
//
// Ejemplos:
//
// keyboard::A
// keyboard::Ctrl
// mouse::LeftButton
//
// DOWN keyboard::Ctrl
// DOWN mouse::LeftButton
// UP mouse::LeftButton
// UP keyboard::Ctrl
//
// ESPERAR 50
//
// Los identificadores corresponden al nombre
// "interno" de pulsadores.tsv.
//
// Runtime nunca deduce el dispositivo.
// Siempre recibe el identificador completo.
//
// ------------------------------------------------------
// Transformación:
//
// pulsadores.tsv
//       ↓
// Perfil
//       ↓
// Cache
//       ↓
// OrdenRuntime
//       ↓
// Runtime
//       ↓
// Back correspondiente
//       ↓
// Dispositivo físico
//
// ======================================================

use crate::cache::OrdenRuntime;

use crate::eventos::InputId;

use crate::perfil_cache::{AccionCache, ExtraCache};

use std::fs::File;

use std::io::{BufRead, BufReader};

use std::thread;

use std::time::Duration;

// ======================================================
// 🚀 EJECUTAR ORDEN
// ======================================================

pub fn ejecutar(orden: OrdenRuntime) {
    match orden {
        OrdenRuntime::Iniciar { id, accion, extra } => {
            ejecutar_accion(id, accion, extra);
        }

        OrdenRuntime::Detener { id } => {
            detener(id);
        }
    }
}

// ======================================================
// ⚡ EJECUTAR ACCIÓN
// ======================================================

fn ejecutar_accion(id: String, accion: AccionCache, extra: Option<ExtraCache>) {
    match accion {
        AccionCache::Emitir(input) => {
            emitir(input);
        }

        AccionCache::Macro(ruta) => {
            ejecutar_macro(id, ruta);
        }

        AccionCache::AbrirArchivo { ruta } => {
            abrir_archivo(ruta);
        }

        AccionCache::Ui(valor) => {
            mostrar_ui(valor);
        }
    }

    if let Some(extra) = extra {
        ejecutar_extra(extra);
    }
}

// ======================================================
// 📂 ABRIR ARCHIVO
// ======================================================

fn abrir_archivo(ruta: String) {
    // El backend decide cómo abrir la ruta
    // según corresponda (programa, documento,
    // carpeta, URL, etc.).

    let _ = ruta;
}

// ======================================================
// 🪟 MOSTRAR UI
// ======================================================

fn mostrar_ui(valor: String) {
    // Enviar al Back_UI.
    // El backend interpreta la orden y
    // construye el elemento visual solicitado.

    let _ = valor;
}

// ======================================================
// 📤 EMITIR
// ======================================================

fn emitir(input: InputId) {
    let evento = crate::eventos::InputEvent::pulse(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⏱️ ESPERAR
// ======================================================

fn esperar(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

// ======================================================
// 🧩 EJECUTAR EXTRA
// ======================================================

fn ejecutar_extra(extra: ExtraCache) {
    let ruta_macro = runt_extra::generar_macro(extra);

    ejecutar_macro("extra".to_string(), ruta_macro);
}

// ======================================================
// ⏹️ DETENER
// ======================================================

fn detener(id: String) {
    let _ = id;
}

// ======================================================
// 📜 EJECUTAR MACRO
// ======================================================

fn ejecutar_macro(id: String, ruta: String) {
    let archivo = match File::open(&ruta) {
        Ok(valor) => valor,

        Err(_) => {
            // FALTA CREAR:
            // sistema global de notificaciones y registro.

            limpiar_instancia(id);

            return;
        }
    };

    let lector = BufReader::new(archivo);

    for linea in lector.lines() {
        let Ok(linea) = linea else {
            continue;
        };

        let linea = linea.trim();

        if linea.is_empty() {
            continue;
        }

        ejecutar_linea(&id, linea);
    }

    limpiar_instancia(id);
}

// ======================================================
// 🧠 INTERPRETAR LÍNEA
// ======================================================

fn ejecutar_linea(id: &str, linea: &str) {
    let partes: Vec<&str> = linea.split_whitespace().collect();

    if partes.is_empty() {
        return;
    }

    match partes[0] {
        // ==============================================
        // ⏱️ ESPERAR
        // ==============================================
        "ESPERAR" => {
            if let Some(valor) = partes.get(1) {
                if let Ok(ms) = valor.parse::<u64>() {
                    esperar(ms);
                }
            }
        }

        // ==============================================
        // ⬇️ DOWN
        // ==============================================
        "DOWN" => {
            if let Some(control) = partes.get(1) {
                ejecutar_down(control);
            }
        }

        // ==============================================
        // ⬆️ UP
        // ==============================================
        "UP" => {
            if let Some(control) = partes.get(1) {
                ejecutar_up(control);
            }
        }

        // ==============================================
        // 🔁 REPETIR
        // ==============================================
        "REPETIR" => {
            repetir(id);
        }

        // ==============================================
        // ⏹️ DETENER
        // ==============================================
        "DETENER" => {
            detener(id.to_string());
        }

        // ==============================================
        // ⚡ PULSE
        // ==============================================
        _ => {
            ejecutar_pulse(partes[0]);
        }
    }
}

// ======================================================
// 🧹 LIMPIAR INSTANCIA
// ======================================================

fn limpiar_instancia(id: String) {
    let _ = id;
}

// ======================================================
// 🔎 RESOLVER INPUT
// ======================================================

fn resolver_input(interno: &str) -> Option<InputId> {
    let pulsador = crate::pulsadores::por_interno(interno)?;

    Some(InputId::new(&pulsador.fuente, &pulsador.interception))
}

// ======================================================
// ⬇️ DOWN
// ======================================================

fn ejecutar_down(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    let evento = crate::eventos::InputEvent::down(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⬆️ UP
// ======================================================

fn ejecutar_up(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    let evento = crate::eventos::InputEvent::up(input, crate::instante::ahora());

    emitir_evento(evento);
}

// ======================================================
// ⚡ PULSE
// ======================================================

fn ejecutar_pulse(identificador: &str) {
    let Some(input) = resolver_input(identificador) else {
        return;
    };

    emitir(input);
}

// ======================================================
// 📤 EMITIR EVENTO
// ======================================================

fn emitir_evento(evento: crate::eventos::InputEvent) {
    back_windows::emitir_evento(evento); // ahora solo existe interception, ver quien lo reemplazará en la v1
}
