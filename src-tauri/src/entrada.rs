// ======================================================
// 🚀 Entrada RemapH V3
// ------------------------------------------------------
// Orquesta:
//
// Input físico
//      ↓
// CapturaTrigger / AnalizadorTrigger
//      ↓
// Runtime
// ======================================================

use crate::analizador_trigger::{AnalizadorTrigger, ResultadoTrigger};
use crate::capturador_trigger::CapturadorTrigger;

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

        let mut capturador = CapturadorTrigger::nuevo();

        let mut ultima_actualizacion = Instant::now() - Duration::from_secs(1);

        crate::backend::back_windows::iniciar(move |evento, _emitir| {
            let resultado = procesar_evento(
                evento,
                &mut analizador,
                &mut capturador,
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

    capturador: &mut CapturadorTrigger,

    runtime: &mut runtime::Estado,

    tx: &mpsc::Sender<AccionCache>,

    rx: &mpsc::Receiver<AccionCache>,

    ultima_actualizacion: &mut Instant,

    salida: Option<&crate::backend::back_salida::Salida>,
) -> runtime::Resultado {
    actualizar_contexto_cache(ultima_actualizacion);

    // ==================================================
    // CAPTURA DE NUEVO TRIGGER
    // ==================================================

    if captura::activa() {
        capturador.recibir(evento);

        if let Some(trigger) = capturador.comprobar_timeout() {
            captura::recibir(trigger);

            return runtime::Resultado::Consumir;
        }

        return runtime::Resultado::Pasar;
    }

    // ==================================================
    // ANALIZADOR NORMAL
    // ==================================================

    match analizador.procesar(evento) {
        ResultadoTrigger::Trigger(trigger) => {
            let resultado = runtime.procesar(trigger, tx);

            ejecutar_salidas(rx, salida);

            resultado
        }

        ResultadoTrigger::Liberar(eventos) => {
            for evento in eventos {
                crate::backend::back_windows::emitir_evento(evento);
            }

            analizador.limpiar();

            runtime::Resultado::Consumir
        }

        ResultadoTrigger::Esperar => match analizador.comprobar_timeout() {
            ResultadoTrigger::Liberar(eventos) => {
                for evento in eventos {
                    crate::backend::back_windows::emitir_evento(evento);
                }

                analizador.limpiar();

                runtime::Resultado::Consumir
            }

            ResultadoTrigger::Trigger(trigger) => {
                let resultado = runtime.procesar(trigger, tx);

                ejecutar_salidas(rx, salida);

                resultado
            }

            ResultadoTrigger::Esperar => runtime::Resultado::Consumir,
        },
    }
}

// ======================================================
// ⚡ EJECUTAR SALIDAS
// ======================================================

fn ejecutar_salidas(
    rx: &mpsc::Receiver<AccionCache>,

    salida: Option<&crate::backend::back_salida::Salida>,
) {
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
