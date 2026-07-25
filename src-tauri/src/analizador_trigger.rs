// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ------------------------------------------------------
// Analiza secuencias físicas.
//
// Reglas:
//
// • Respeta el orden de Down.
// • Último Down = Gatillo.
// • Downs anteriores = Modificadores.
// • Espera hasta que expire la ventana de análisis.
// • El tiempo proviene de config.rs.
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
    eventos: Vec<InputEvent>,

    ultimo_evento: Option<Instant>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl AnalizadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            eventos: Vec::new(),

            ultimo_evento: None,
        }
    }

    // ==================================================
    // 📥 RECIBIR EVENTO
    // ==================================================

    pub fn procesar(&mut self, evento: InputEvent) -> ResultadoTrigger {
        self.eventos.push(evento);

        self.ultimo_evento = Some(Instant::now());

        self.analizar()
    }

    // ==================================================
    // ⏱️ COMPROBAR TIMEOUT
    // ==================================================

    pub fn comprobar_timeout(&mut self) -> ResultadoTrigger {
        let Some(instante) = self.ultimo_evento else {
            return ResultadoTrigger::Esperar;
        };

        if instante.elapsed().as_millis() < crate::config::tiempo_doble() as u128 {
            return ResultadoTrigger::Esperar;
        }

        let eventos = self.eventos.clone();

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

        if crate::cache::tiene_prefijo(&activos) {
            return ResultadoTrigger::Esperar;
        }

        ResultadoTrigger::Esperar
    }

    // ==================================================
    // ⬇️ INPUTS DOWN
    // ==================================================

    fn down_activos(&self) -> Vec<InputId> {
        self.eventos
            .iter()
            .filter(|evento| evento.state == InputState::Down)
            .map(|evento| evento.input.clone())
            .collect()
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.eventos.clear();

        self.ultimo_evento = None;
    }
}
