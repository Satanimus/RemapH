// ======================================================
// 🧠 Analizador Trigger RemapH V3
// ======================================================
// ETAPA 3 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Interpreta únicamente la secuencia física de entradas.
//
// No conoce: • Cache. • Runtime. • Remapeos. • Aplicaciones.
//
// Mantiene temporalmente:
// • Buffer de eventos. • Inputs actualmente presionados.
// • Último Down. • Último Up.
//
// Cuando se solicita:
// • Determina si el último gatillo corresponde a: Simple.  Doble. Mantenido.
// ------------------------------------------------------
// 2. ¿Qué información recibe?
// Recibe:
// • InputEvent.
//
// Opcionalmente:
// • Solicitud para analizar el último gatillo.
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
// • Backend captura. • Módulo de captura UI.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// En Runtime:
// • Cache recibe cada down en orden de llegada para comparar.
// • Resultado del análisis de condición cuando Cache lo solicita.
//
// En Captura:
// • Trigger completo listo para guardar en el botón.
// ------------------------------------------------------
// 5. Funciones del archivo
// procesar()
//     Actualiza el estado interno con cada InputEvent.
// obtener_entrada()
//     Devuelve la entrada completa actualmente activa.
// analizar_condicion()
//     Determina Simple, Doble o Mantenido usando
//     el historial del último gatillo.
// comprobar_timeout()
//     Limpia la memoria cuando vence el tiempo doble.
// limpiar()
//     Reinicia completamente el estado interno.
// ------------------------------------------------------
// Transformación:
// InputEvent
//      ↓
// Analizador Trigger
//      ↓-----------↓
// Cache     o     perfil_ui
//      ↓
// Runtime
// ======================================================

use crate::eventos::{InputEvent, InputId, InputState};
use crate::perfil_cache::CondicionTrigger;

// ======================================================
// 🧠 ANALIZADOR
// ======================================================

pub struct AnalizadorTrigger {
    buffer: Vec<InputEvent>,

    presionados: Vec<InputId>,

    ultimo_down: Option<InputEvent>,

    ultimo_up: Option<InputEvent>,
}

// ======================================================
// 🚀 CREAR
// ======================================================

impl AnalizadorTrigger {
    pub fn nuevo() -> Self {
        Self {
            buffer: Vec::new(),

            presionados: Vec::new(),

            ultimo_down: None,

            ultimo_up: None,
        }
    }

    // ==================================================
    // 📥 PROCESAR
    // ==================================================

    pub fn procesar(&mut self, evento: InputEvent) {
        self.buffer.push(evento.clone());

        match evento.state {
            InputState::Down => {
                if !self.presionados.contains(&evento.input) {
                    self.presionados.push(evento.input.clone());
                }

                self.ultimo_down = Some(evento);
            }

            InputState::Up => {
                self.presionados.retain(|i| i != &evento.input);

                self.ultimo_up = Some(evento);
            }

            InputState::Pulse => {}
        }
    }

    // ==================================================
    // 📦 ENTRADA ACTUAL
    // ==================================================

    pub fn obtener_entrada(&self) -> Vec<InputId> {
        self.presionados.clone()
    }

    // ==================================================
    // ⏱️ ANALIZAR CONDICIÓN
    // ==================================================

    pub fn analizar_condicion(&self) -> Option<CondicionTrigger> {
        let ultimo_down = self.ultimo_down.as_ref()?;

        let gatillo = &ultimo_down.input;

        let eventos: Vec<&InputEvent> = self
            .buffer
            .iter()
            .filter(|evento| evento.input == *gatillo)
            .collect();

        let down: Vec<&InputEvent> = eventos
            .iter()
            .filter(|evento| evento.state == InputState::Down)
            .copied()
            .collect();

        if down.len() >= 2 {
            let diferencia = down[down.len() - 1].instante - down[down.len() - 2].instante;

            if diferencia <= crate::config::tiempo_doble() {
                return Some(CondicionTrigger::Doble);
            }
        }

        let up = eventos
            .iter()
            .rev()
            .find(|evento| evento.state == InputState::Up);

        if let Some(up) = up {
            let tiempo = up.instante - ultimo_down.instante;

            if tiempo >= crate::config::tiempo_mantenido() {
                return Some(CondicionTrigger::Mantenido);
            }
        }

        Some(CondicionTrigger::Simple)
    }

    // ==================================================
    // ⏳ TIMEOUT DOBLE
    // ==================================================

    pub fn comprobar_timeout(&mut self) {
        let Some(up) = &self.ultimo_up else {
            return;
        };

        let ahora = crate::instante::ahora();

        if ahora - up.instante >= crate::config::tiempo_doble() {
            self.limpiar();
        }
    }

    // ==================================================
    // 🧹 LIMPIAR
    // ==================================================

    pub fn limpiar(&mut self) {
        self.buffer.clear();

        self.presionados.clear();

        self.ultimo_down = None;

        self.ultimo_up = None;
    }
}
