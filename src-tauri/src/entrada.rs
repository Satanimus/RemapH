// ======================================================
// 🚀 Entrada RemapH V3
// ------------------------------------------------------
// Orquesta los backends físicos.
//
// Full: Interception
// Portable: Windows API
//
// Ambos entregan InputEvent genérico.
// El Runtime es único.
//
// Entrada no:
//   - Interpreta remapeos.
//   - Compila configuraciones.
//   - Conoce teclas concretas.
//   - Ejecuta Accion directamente.
//
// Solo conecta:
//   Backend → Runtime → Salida
// ======================================================

use crate::cache;
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
// 🖥️ ACTUALIZAR CONTEXTO APP
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

        let mut ultima_actualizacion = Instant::now() - Duration::from_secs(1);

        let mut pendientes: Vec<(interception::Device, interception::Stroke)> = Vec::new();

        loop {
            let Some((device, stroke)) = crate::backend::back_interception::recibir(&ict) else {
                continue;
            };

            let Some(evento) = crate::backend::back_interception::traducir(&stroke) else {
                continue;
            };

            if crate::captura::procesar(&evento) {
                continue;
            }

            actualizar_contexto_cache(&mut ultima_actualizacion);

            let resultado = runtime.procesar(evento, &tx);

            match resultado {
                runtime::Resultado::Esperar => {
                    pendientes.push((device, stroke));
                }

                runtime::Resultado::Consumir => {
                    pendientes.clear();
                }

                runtime::Resultado::Pasar => {
                    for (device_pendiente, stroke_pendiente) in pendientes.drain(..) {
                        crate::backend::back_interception::reenviar(
                            &ict,
                            device_pendiente,
                            stroke_pendiente,
                        );
                    }

                    crate::backend::back_interception::reenviar(&ict, device, stroke);
                }
            }

            while let Ok(accion) = rx.try_recv() {
                salida.ejecutar(accion);
            }
        }
    });
}

// ======================================================
// 🪟 PORTABLE
// ======================================================

fn iniciar_portable(tx: mpsc::Sender<AccionCache>, rx: mpsc::Receiver<AccionCache>) {
    std::thread::spawn(move || {
        let mut runtime = runtime::Estado::nuevo();

        let mut ultima_actualizacion = Instant::now() - Duration::from_secs(1);

        let mut pendientes: Vec<InputEvent> = Vec::new();

        crate::backend::back_windows::iniciar(move |evento, emitir| {
            // ----------------------------------
            // 📥 EVENTO RECIBIDO
            // ----------------------------------

            if crate::captura::procesar(&evento) {
                return true;
            }

            actualizar_contexto_cache(&mut ultima_actualizacion);

            // ----------------------------------
            // 🧠 RUNTIME
            // ----------------------------------

            let resultado = runtime.procesar(evento.clone(), &tx);

            let consumir = match resultado {
                runtime::Resultado::Esperar => {
                    pendientes.push(evento);

                    true
                }

                runtime::Resultado::Consumir => {
                    pendientes.clear();
                    true
                }

                runtime::Resultado::Pasar => {
                    for pendiente in pendientes.drain(..) {
                        emitir(pendiente);
                    }
                    false
                }
            };

            // ----------------------------------
            // 📤 ACCIONES
            // ----------------------------------

            while let Ok(accion) = rx.try_recv() {
                ejecutar_portable(accion);
            }
            consumir
        });
    });
}

// ======================================================
// 🪟 EJECUTAR PORTABLE
// ------------------------------------------------------
// Adaptador temporal de salida para Windows.
//
// La acción sigue siendo genérica.
// La traducción física ocurre aquí.
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
