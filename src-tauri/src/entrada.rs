// ======================================================
// 🚀 Entrada RemapH V3
// ------------------------------------------------------
// Orquesta los backends físicos.
//
// Backend
//    ↓
// InputEvent
//    ↓
// AnalizadorTrigger
//    ↓
// Captura o Runtime
// ======================================================

use crate::analizador_trigger::AnalizadorTrigger;
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

// ======================================================
// 🧪 MODO ACTUAL
// ======================================================

const MODO: Modo = Modo::Portable;

// ======================================================
// 🖥️ CONTEXTO APP
// ======================================================

fn actualizar_contexto_cache(ultima_actualizacion: &mut Instant) {
    if ultima_actualizacion.elapsed() < Duration::from_millis(250) {
        return;
    }

    let programa_activo = crate::backend::back_procesos::obtener_programa_activo();

    let procesos_activos: HashSet<String> =
        crate::backend::back_procesos::enumerar_procesos_ventana()
            .into_iter()
            .map(|proceso| proceso.nombre.to_lowercase())
            .collect();

    cache::actualizar_contexto(programa_activo.as_deref(), &procesos_activos);

    *ultima_actualizacion = Instant::now();
}

// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar() {
    let (tx, rx) = mpsc::channel::<AccionCache>();

    match MODO {
        Modo::Full => {
            iniciar_full(tx, rx);
        }

        Modo::Portable => {
            iniciar_portable(tx, rx);
        }
    }
}

// ======================================================
// ⚡ FULL
// ======================================================

fn iniciar_full(tx: mpsc::Sender<AccionCache>, rx: mpsc::Receiver<AccionCache>) {
    std::thread::spawn(move || {
        let ict = crate::backend::back_interception::crear();

        let salida = crate::backend::back_salida::crear();

        let mut runtime = runtime::Estado::nuevo();

        let mut analizador = AnalizadorTrigger::nuevo();

        let mut ultima_actualizacion = Instant::now() - Duration::from_secs(1);

        loop {
            let Some((_device, stroke)) = crate::backend::back_interception::recibir(&ict) else {
                continue;
            };

            let Some(evento) = crate::backend::back_interception::traducir(&stroke) else {
                continue;
            };

            procesar_evento(
                evento,
                &mut analizador,
                &mut runtime,
                &tx,
                &rx,
                &mut ultima_actualizacion,
                Some(&salida),
            );
        }
    });
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
            procesar_evento(
                evento,
                &mut analizador,
                &mut runtime,
                &tx,
                &rx,
                &mut ultima_actualizacion,
                None,
            );

            false
        });
    });
}

// ======================================================
// 🧠 PROCESAR EVENTO CENTRAL
// ======================================================

fn procesar_evento(
    evento: InputEvent,

    analizador: &mut AnalizadorTrigger,

    runtime: &mut runtime::Estado,

    tx: &mpsc::Sender<AccionCache>,

    rx: &mpsc::Receiver<AccionCache>,

    ultima_actualizacion: &mut Instant,

    salida: Option<&crate::backend::back_salida::Salida>,
) {
    actualizar_contexto_cache(ultima_actualizacion);

    let Some(trigger) = analizador.procesar(evento) else {
        return;
    };

    // ==================================================
    // 🎹 CAPTURA ACTIVA
    // ==================================================

    if captura::activa() {
        captura::recibir(trigger);

        return;
    }

    // ==================================================
    // ⚙️ RUNTIME
    // ==================================================

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
}

// ======================================================
// 🪟 EJECUTAR PORTABLE
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
