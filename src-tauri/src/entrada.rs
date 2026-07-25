// ======================================================
// 🚀 Entrada RemapH V3
// ------------------------------------------------------
// Orquesta:
// Input físico
//      ↓
// AnalizadorTrigger
//      ↓
// Captura / Runtime
// ======================================================

use crate::analizador_trigger::{AnalizadorTrigger, ResultadoTrigger};

use crate::cache;
use crate::captura;
use crate::eventos::InputEvent;
use crate::perfilcache::AccionCache;
use crate::runtime;

use std::collections::HashSet;
use std::sync::mpsc;
use std::time::{Duration, Instant};

// ======================================================
// ⚙️ MODO
// ======================================================

#[derive(Clone, Copy)]
pub enum Modo {
    Full,

    Portable,
}

const MODO: Modo = Modo::Portable;

// ======================================================
// 🖥️ CONTEXTO CACHE
// ======================================================

fn actualizar_contexto_cache(ultima: &mut Instant) {
    if ultima.elapsed() < Duration::from_millis(250) {
        return;
    }

    let programa = crate::backend::back_procesos::obtener_programa_activo();

    let procesos: HashSet<String> = crate::backend::back_procesos::enumerar_procesos_ventana()
        .into_iter()
        .map(|p| p.nombre.to_lowercase())
        .collect();

    cache::actualizar_contexto(programa.as_deref(), &procesos);

    *ultima = Instant::now();
}

// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar() {
    let (tx, rx) = mpsc::channel::<AccionCache>();

    match MODO {
        Modo::Portable => iniciar_portable(tx, rx),

        Modo::Full => {
            println!("Modo Full pendiente");
        }
    }
}

// ======================================================
// 🪟 PORTABLE
// ======================================================

fn iniciar_portable(tx: mpsc::Sender<AccionCache>, rx: mpsc::Receiver<AccionCache>) {
    std::thread::spawn(move || {
        let mut runtime = runtime::Estado::nuevo();

        let mut analizador = AnalizadorTrigger::nuevo();

        let mut ultima_actualizacion = Instant::now() - Duration::from_secs(1);

        crate::backend::back_windows::iniciar(move |evento, _emitir| {
            let resultado = procesar_evento(
                evento,
                &mut analizador,
                &mut runtime,
                &tx,
                &rx,
                &mut ultima_actualizacion,
                None,
            );

            matches!(resultado, runtime::Resultado::Consumir)
        });
    });
}

// ======================================================
// 🧠 PROCESAR EVENTO
// ======================================================

fn procesar_evento(
    evento: InputEvent,

    analizador: &mut AnalizadorTrigger,

    runtime: &mut runtime::Estado,

    tx: &mpsc::Sender<AccionCache>,

    rx: &mpsc::Receiver<AccionCache>,

    ultima_actualizacion: &mut Instant,

    salida: Option<&crate::backend::back_salida::Salida>,
) -> runtime::Resultado {
    actualizar_contexto_cache(ultima_actualizacion);

    let resultado_trigger = analizador.procesar(evento);

    match resultado_trigger {
        crate::analizador_trigger::ResultadoTrigger::Trigger(trigger) => {
            if captura::activa() {
                captura::recibir(trigger);

                return runtime::Resultado::Consumir;
            }

            let resultado = runtime.procesar(trigger, tx);

            while let Ok(accion) = rx.try_recv() {
                match salida {
                    Some(salida) => {
                        salida.ejecutar(accion);
                    }

                    None => {
                        ejecutar_portable(accion);
                    }
                }
            }

            println!("[ENTRADA] Resultado Runtime -> {:?}", resultado);

            resultado
        }

        // El analizador todavía no sabe si es trigger.
        // Bloqueamos para evitar que Windows ejecute la tecla.
        crate::analizador_trigger::ResultadoTrigger::Esperar => runtime::Resultado::Consumir,

        // No era trigger.
        // Liberamos los eventos físicos originales.
        crate::analizador_trigger::ResultadoTrigger::Liberar(eventos) => {
            for evento in eventos {
                println!("[ENTRADA] Liberando evento pendiente -> {:?}", evento);

                crate::backend::back_windows::emitir_evento(evento);
            }

            analizador.limpiar();

            runtime::Resultado::Consumir
        }
    }
}

// ======================================================
// ⚡ EJECUTAR SALIDAS
// ======================================================

fn ejecutar_salidas(rx: &mpsc::Receiver<AccionCache>) {
    while let Ok(accion) = rx.try_recv() {
        ejecutar_portable(accion);
    }
}

// ======================================================
// 🪟 EMITIR PORTABLE
// ======================================================

fn ejecutar_portable(accion: AccionCache) {
    match accion {
        AccionCache::Emitir(input) => {
            crate::backend::back_windows::emitir_evento(InputEvent::pulse(
                input,
                crate::instante::ahora(),
            ));
        }
    }
}
