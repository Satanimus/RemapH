// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ------------------------------------------------------
// Analiza secuencias físicas.
//
// Regla:
// - Respeta orden de Down.
// - Último Down = gatillo.
// - Todo Down anterior = modificadores.
// - Espera ventana antes de liberar.
// ======================================================

use crate::evento_trigger::EventoTrigger;
use crate::eventos::{InputEvent, InputId, InputState};

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
// 🧠 ANALIZADOR
// ======================================================

pub struct AnalizadorTrigger {
    eventos_pendientes: Vec<InputEvent>,

    ultimo_evento: Option<Instant>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl AnalizadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            eventos_pendientes: Vec::new(),

            ultimo_evento: None,
        }
    }

    // ==================================================
    // 📥 EVENTO
    // ==================================================

    pub fn procesar(&mut self, evento: InputEvent) -> ResultadoTrigger {
        self.ultimo_evento = Some(Instant::now());

        self.eventos_pendientes.push(evento);

        self.analizar()
    }

    // ==================================================
    // ⏱️ TIMEOUT
    // ==================================================

    pub fn comprobar_timeout(&mut self) -> ResultadoTrigger {
        let Some(ultimo) = self.ultimo_evento else {
            return ResultadoTrigger::Esperar;
        };

        if ultimo.elapsed() < Duration::from_millis(250) {
            return ResultadoTrigger::Esperar;
        }

        let eventos = self.eventos_pendientes.clone();

        self.limpiar();

        ResultadoTrigger::Liberar(eventos)
    }

    // ==================================================
    // 🔎 ANALIZAR
    // ==================================================

    fn analizar(&self) -> ResultadoTrigger {
        let inputs = self.inputs_down();

        if inputs.is_empty() {
            return ResultadoTrigger::Esperar;
        }

        let Some(gatillo) = inputs.last() else {
            return ResultadoTrigger::Esperar;
        };

        let modificadores = if inputs.len() > 1 {
            inputs[..inputs.len() - 1].to_vec()
        } else {
            Vec::new()
        };

        if crate::cache::buscar(&inputs, gatillo).is_some() {
            return ResultadoTrigger::Trigger(EventoTrigger::simple(
                modificadores,
                gatillo.clone(),
            ));
        }

        if crate::cache::tiene_prefijo(&inputs) {
            return ResultadoTrigger::Esperar;
        }

        ResultadoTrigger::Esperar
    }

    // ==================================================
    // 🔎 DOWN ACTIVOS
    // ==================================================

    fn inputs_down(&self) -> Vec<InputId> {
        self.eventos_pendientes
            .iter()
            .filter(|evento| evento.state == InputState::Down)
            .map(|evento| evento.input.clone())
            .collect()
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.eventos_pendientes.clear();

        self.ultimo_evento = None;
    }
}
