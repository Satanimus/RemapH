// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ------------------------------------------------------
// Analiza únicamente posibles triggers.
//
// No ejecuta acciones.
// No conoce Runtime.
// No conoce Captura.
//
// Solo responde:
//
//   Esperar
//   Trigger
//   Liberar
//
// Maneja:
//   - Simple.
//   - Doble.
//   - Mantenido.
//
// Mantiene:
//   - Inputs físicamente presionados.
//   - Candidatos temporales.
// ======================================================

use crate::evento_trigger::EventoTrigger;
use crate::eventos::{InputEvent, InputId, InputState};
use crate::perfil_cache::CondicionTrigger;

use std::time::{Duration, Instant};

// ======================================================
// 🔎 RESULTADO
// ======================================================

pub enum ResultadoTrigger {
    Esperar,

    Trigger(EventoTrigger),

    Liberar(Vec<InputEvent>),
}

// ======================================================
// 🧠 CANDIDATO TEMPORAL
// ======================================================

struct CandidatoTrigger {
    modificadores: Vec<InputId>,

    gatillo: InputId,

    condicion: CondicionTrigger,

    instante_down: u64,

    instante_up: Option<u64>,
}

// ======================================================
// 🧠 ANALIZADOR
// ======================================================

pub struct AnalizadorTrigger {
    buffer: Vec<InputEvent>,

    presionados: Vec<InputId>,

    candidato: Option<CandidatoTrigger>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl AnalizadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            buffer: Vec::new(),

            presionados: Vec::new(),

            candidato: None,
        }
    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn procesar(&mut self, evento: InputEvent) -> ResultadoTrigger {
        self.buffer.push(evento.clone());

        match evento.state {
            InputState::Down => {
                self.agregar_presionado(evento.input.clone());

                self.procesar_down(evento)
            }

            InputState::Up => {
                self.quitar_presionado(&evento.input);

                self.procesar_up(evento)
            }

            InputState::Pulse => ResultadoTrigger::Liberar(self.buffer.clone()),
        }
    }

    // ==================================================
    // ⬇️ DOWN
    // ==================================================

    fn procesar_down(&mut self, evento: InputEvent) -> ResultadoTrigger {
        if let Some(candidato) = &self.candidato {
            if candidato.condicion == CondicionTrigger::Doble && candidato.gatillo == evento.input {
                let resultado = EventoTrigger::doble(
                    candidato.modificadores.clone(),
                    candidato.gatillo.clone(),
                );

                self.limpiar();

                return ResultadoTrigger::Trigger(resultado);
            }
        }

        let activos = self.presionados.clone();

        let gatillo = evento.input.clone();

        let modificadores = activos
            .iter()
            .filter(|input| **input != gatillo)
            .cloned()
            .collect();

        let Some(remapeo) = crate::cache::buscar(&activos, &gatillo) else {
            if crate::cache::existe_prefijo(&activos) {
                return ResultadoTrigger::Esperar;
            }

            let eventos = self.buffer.clone();

            self.limpiar();

            return ResultadoTrigger::Liberar(eventos);
        };

        match remapeo.trigger.condicion {
            CondicionTrigger::Simple => {
                self.limpiar();

                ResultadoTrigger::Trigger(EventoTrigger::simple(modificadores, gatillo))
            }

            CondicionTrigger::Doble | CondicionTrigger::Mantenido => {
                self.candidato = Some(CandidatoTrigger {
                    modificadores,

                    gatillo,

                    condicion: remapeo.trigger.condicion,

                    instante_down: evento.instante,

                    instante_up: None,
                });

                ResultadoTrigger::Esperar
            }
        }
    }

    // ==================================================
    // ⬆️ UP
    // ==================================================

    fn procesar_up(&mut self, evento: InputEvent) -> ResultadoTrigger {
        let Some(candidato) = &mut self.candidato else {
            return ResultadoTrigger::Esperar;
        };

        if candidato.gatillo != evento.input {
            return ResultadoTrigger::Esperar;
        }

        candidato.instante_up = Some(evento.instante);

        match candidato.condicion {
            CondicionTrigger::Mantenido => {
                let duracion = evento.instante - candidato.instante_down;

                if duracion >= crate::config::tiempo_mantenido() {
                    let resultado = EventoTrigger::mantenido(
                        candidato.modificadores.clone(),
                        candidato.gatillo.clone(),
                    );

                    self.limpiar();

                    return ResultadoTrigger::Trigger(resultado);
                }

                let eventos = self.buffer.clone();

                self.limpiar();

                ResultadoTrigger::Liberar(eventos)
            }

            CondicionTrigger::Doble => ResultadoTrigger::Esperar,

            CondicionTrigger::Simple => ResultadoTrigger::Esperar,
        }
    }

    // ==================================================
    // ⏱️ TIMEOUT
    // ==================================================

    pub fn comprobar_timeout(&mut self) -> ResultadoTrigger {
        let Some(candidato) = &self.candidato else {
            return ResultadoTrigger::Esperar;
        };

        let Some(instante_up) = candidato.instante_up else {
            return ResultadoTrigger::Esperar;
        };

        let ahora = crate::instante::ahora();

        if ahora - instante_up < crate::config::tiempo_doble() {
            return ResultadoTrigger::Esperar;
        }

        match candidato.condicion {
            CondicionTrigger::Doble => {
                let resultado = EventoTrigger::simple(
                    candidato.modificadores.clone(),
                    candidato.gatillo.clone(),
                );

                self.limpiar();

                ResultadoTrigger::Trigger(resultado)
            }

            _ => ResultadoTrigger::Esperar,
        }
    }

    // ==================================================
    // 📦 ANALIZAR CAPTURA
    // --------------------------------------------------
    // Analiza eventos físicos sin consultar Cache.
    //
    // Uso:
    // creación de triggers desde UI.
    //
    // No busca remapeos.
    // No ejecuta Runtime.
    // ==================================================

    pub fn analizar_captura(&mut self, eventos: &[InputEvent]) -> Option<EventoTrigger> {
        self.limpiar();

        for evento in eventos {
            match self.procesar(evento.clone()) {
                ResultadoTrigger::Trigger(trigger) => {
                    self.limpiar();

                    return Some(trigger);
                }

                ResultadoTrigger::Liberar(_) => {
                    self.limpiar();

                    return None;
                }

                ResultadoTrigger::Esperar => {}
            }
        }

        match self.comprobar_timeout() {
            ResultadoTrigger::Trigger(trigger) => {
                self.limpiar();

                Some(trigger)
            }

            _ => None,
        }
    }

    // ==================================================
    // ➕ AGREGAR PRESIONADO
    // ==================================================

    fn agregar_presionado(&mut self, input: InputId) {
        if !self.presionados.contains(&input) {
            self.presionados.push(input);
        }
    }

    // ==================================================
    // ➖ QUITAR PRESIONADO
    // ==================================================

    fn quitar_presionado(&mut self, input: &InputId) {
        self.presionados.retain(|actual| actual != input);
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.buffer.clear();

        self.presionados.clear();

        self.candidato = None;
    }
}
