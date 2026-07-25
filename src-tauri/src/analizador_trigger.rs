// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ------------------------------------------------------
// Buffer lógico de entrada.
//
// Recibe eventos físicos.
// Retiene temporalmente.
// Decide si existe trigger.
// Si no existe, libera eventos originales.
//
// No ejecuta acciones.
// No conoce Runtime.
// ======================================================

use crate::evento_trigger::EventoTrigger;
use crate::eventos::{InputEvent, InputId, InputState};

use std::time::Instant;

// ======================================================
// 🔎 RESULTADO
// ======================================================

pub enum ResultadoTrigger {
    Esperar,

    Trigger(EventoTrigger),

    Liberar(Vec<InputEvent>),
}

// ======================================================
// 🧠 ANALIZADOR
// ======================================================

pub struct AnalizadorTrigger {
    buffer: Vec<InputEvent>,

    ultimo_evento: Option<Instant>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl AnalizadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            buffer: Vec::new(),

            ultimo_evento: None,
        }
    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn procesar(&mut self, evento: InputEvent) -> ResultadoTrigger {
        self.buffer.push(evento);

        self.ultimo_evento = Some(Instant::now());

        self.analizar()
    }

    // ==================================================
    // ⏱️ TIMEOUT
    // ==================================================

    pub fn comprobar_timeout(&mut self) -> ResultadoTrigger {
        let Some(instante) = self.ultimo_evento else {
            return ResultadoTrigger::Esperar;
        };

        if instante.elapsed().as_millis() < crate::config::tiempo_doble() as u128 {
            return ResultadoTrigger::Esperar;
        }

        let eventos = self.buffer.clone();

        self.limpiar();

        ResultadoTrigger::Liberar(eventos)
    }

    // ==================================================
    // 🔎 ANALIZAR
    // ==================================================

    fn analizar(&self) -> ResultadoTrigger {
        let activos = self.down_activos();

        if activos.is_empty() {
            return ResultadoTrigger::Esperar;
        }

        let gatillo = activos.last().unwrap().clone();

        let modificadores = if activos.len() > 1 {
            activos[..activos.len() - 1].to_vec()
        } else {
            Vec::new()
        };

        if crate::cache::buscar(&activos, &gatillo).is_some() {
            return ResultadoTrigger::Trigger(EventoTrigger::simple(modificadores, gatillo));
        }

        ResultadoTrigger::Esperar
    }

    // ==================================================
    // ⬇️ DOWN ACTIVOS
    // ==================================================

    fn down_activos(&self) -> Vec<InputId> {
        self.buffer
            .iter()
            .filter(|evento| evento.state == InputState::Down)
            .map(|evento| evento.input.clone())
            .collect()
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.buffer.clear();

        self.ultimo_evento = None;
    }
}
