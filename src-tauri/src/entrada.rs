// ======================================================
// 🚪 Entrada RemapH V3
// ======================================================
// ETAPA 4 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Es el portero del sistema.
//
// Recibe cada InputEvent generado por el backend
// de captura y lo retiene temporalmente mientras
// el resto del flujo decide su destino.
//
// Entrada NO interpreta:
//
// - Gatillos.
// - Remapeos.
// - Perfiles.
// - Acciones.
// - Runtime.
//
// Su única responsabilidad es decidir:
//
// • El evento continúa hacia Runtime.
// • El evento vuelve al sistema operativo.
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// InputEvent
//
// El evento ya contiene:
//
// - Input.
// - Estado.
// - Instante.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Backend de captura.
//
// Flujo:
//
// Dispositivo físico
//      ↓
// Backend captura
//      ↓
// Entrada
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Puede enviar el InputEvent hacia:
//
// • AnalizadorTrigger.
//
// Finalmente el evento terminará en uno de dos caminos:
//
// A)
// Runtime.
//
// B)
// Backend Windows
// para devolver el Input original.
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// iniciar()
//     Entrega el punto de entrada al backend
//     de captura.
//
// procesar_evento()
//     Recibe el InputEvent.
//     Lo retiene temporalmente.
//     Lo entrega al AnalizadorTrigger.
//
// consumir()
//     Descarta el Input físico cuando
//     Cache encuentra coincidencia.
//
// devolver()
//     Devuelve el Input físico al backend
//     para continuar hacia el sistema
//     operativo.
// ------------------------------------------------------
// Transformación:
//
// Dispositivo físico
//      ↓
// Backend captura
//      ↓
// Entrada
//      ↓
// AnalizadorTrigger
//      ↓
// Cache
//      ↓
//
// ├── Runtime
//
// └── Windows
//
// ======================================================

use crate::analizador_trigger::AnalizadorTrigger;

use crate::eventos::InputEvent;

// ======================================================
// 🚀 INICIAR
// ======================================================

pub fn iniciar() {
    let mut analizador = AnalizadorTrigger::nuevo();

    crate::back_interception::iniciar(move |evento| {
        procesar_evento(evento, &mut analizador);
    });
}

// ======================================================
// 🚪 PROCESAR EVENTO
// ======================================================

fn procesar_evento(evento: InputEvent, analizador: &mut AnalizadorTrigger) {
    analizador.procesar(evento.clone());

    let _ = evento;
}

// ======================================================
// ❌ CONSUMIR
// ======================================================

fn consumir() {
    // El Input físico se descarta.
}

// ======================================================
// ↩️ DEVOLVER
// ======================================================

fn devolver(evento: InputEvent) {
    crate::back_interception::emitir_evento(evento);
}
