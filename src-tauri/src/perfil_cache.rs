// ======================================================
// 📦 perfil_cache  
// ======================================================
// ETAPA 4 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Modelo interno compilado utilizado por Cache y Runtime.
//
// Guarda únicamente información necesaria para:
//
// • Buscar triggers rápidamente.
// • Ejecutar acciones directamente.
//
// No guarda:
// • Color.
// • Nota.
// • Remapeos OFF.
//
// El Trigger es optimizado.
// La Acción ya viene preparada para ejecución.
//
// Flujo:
//
// perfil_json
//      ↓
// Compilador
//      ↓
// perfil_cache
//      ↓
// Cache
//      ↓
// Runtime
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe información compilada desde perfil_json.
//
// Trigger:
//
// • App.
// • Entrada.
// • Condición.
//
// Acción:
//
// • Acción física preparada.
//
// Ejemplo:
//
// Trigger:
//
// Firefox
// CTRL + A
// Doble
//
// Acción:
//
// Emitir keyboard:B
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Recibe:
//
// • Compilador.
//
// Lo utilizan:
//
// • Cache.
// • Runtime.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// TriggerCache:
//
// Información optimizada para búsqueda.
//
// AccionCache:
//
// Información lista para ejecución.
//
// RemapeoCache:
//
// {
//    id,
//    trigger,
//    accion
// }
//
// ------------------------------------------------------
// 5. Funciones y estructuras
//
// AppCache
//      Contexto de aplicación.
//
// CondicionTrigger
//      Tipo de disparador.
//
// RemapeoCache
//      Une Trigger + Acción.
//
// TriggerCache
//      Parte compilada del remapeo.
//
// AccionCache
//      Orden física de ejecución.
//
// ExtraCache
//      Como debe comportarse la Accion.
// ------------------------------------------------------
// Filosofía:
//
// ✔ Cache decide coincidencias.
//
// ✔ Runtime ejecuta acciones.
//
// ✔ Ninguno interpreta respuestas.
//
// ✔ Agregar nuevos tipos de salida modifica únicamente
//   AccionCache y Salida.
//
// ======================================================

use crate::eventos::InputId;

use serde::{Deserialize, Serialize};

// ======================================================
// 🖥️ APP CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppCache {
    Global,

    Programa { nombre: String, segundo_plano: bool },
}

// ======================================================
// 🎯 CONDICIÓN TRIGGER
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CondicionTrigger {
    Simple,

    Doble,

    Mantenido,
}

// ======================================================
// 🧩 REMAPEO CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct RemapeoCache {
    pub id: String,

    pub trigger: TriggerCache,

    pub accion: AccionCache,

    pub extra: Option<ExtraCache>,

    pub coordenada: Option<CoordenadaCache>,
}

// ======================================================
// ⌨️ TRIGGER CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct TriggerCache {
    pub app: AppCache,

    pub entrada: Vec<InputId>,

    pub condicion: CondicionTrigger,
}

// ======================================================
// ⚡ ACCIÓN CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum AccionCache {
    // Modificadores + gatillo, en ese orden (misma convención que
    // TriggerCache::entrada) — el último elemento es siempre el
    // gatillo, los anteriores son modificadores que van DOWN antes
    // y UP después (en orden inverso). Nunca vacío: convertir_accion
    // garantiza al menos el gatillo.
    Emitir(Vec<InputId>),

    Macro(String),

    AbrirArchivo(String),

    Ui(String),
}

// ======================================================
// 🧩 EXTRA CACHE
// ======================================================
//
// Selecciona una receta de runt_extra.
//
// No ejecuta.
// No contiene lógica.
// Runtime solicita la receta.
//

#[derive(Clone, Debug, PartialEq)]
pub enum ExtraCache {
    // Teclado / mouse / joystick
    Turbo,

    Mantener,

    Toggle,

    // Mouse
    DobleClick,

    ClickSostenido,

    // Windows
    AbrirMinimizado,

    // UI
    PopupToggle,
}

impl ExtraCache {
    /// true si la receta de runt_extra para este Extra termina en
    /// "ESPERAR DETENER" — necesita que Cache espere el Up físico
    /// real (vía Iniciar sin Finalizar) en vez de mandar Iniciar +
    /// Detener juntos apenas se confirma el trigger. Independiente
    /// de la Condición que lo disparó (Simple/Doble/Mantenido): lo
    /// que decide si el final es diferido es el Extra, no el
    /// trigger.
    pub fn requiere_up_real(&self) -> bool {
        matches!(
            self,
            ExtraCache::Turbo | ExtraCache::Mantener | ExtraCache::ClickSostenido
        )
    }
}

// ======================================================
// 🖱️ COORDENADA CACHE
// ------------------------------------------------------
// Forma compilada de la columna Extra de "Click en
// coordenada" — ya resuelta a números, lista para que
// Runtime calcule el destino sin volver a interpretar
// strings. La repetición (Normal/Mantener/Turbo) NO vive
// acá: se resuelve al mismo ExtraCache de siempre (ver
// compilador.rs), reutilizando el mecanismo existente.
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum PuntoReferenciaCache {
    SupIzq,
    SupDer,
    Centro,
    InfIzq,
    InfDer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UbicacionCache {
    Absoluta {
        x: f64,
        y: f64,
    },

    RelativaCursor {
        offset_x: f64,
        offset_y: f64,
    },

    RelativaVentanaPorcentaje {
        h: f64,
        v: f64,
    },

    RelativaVentanaPixeles {
        offset_x: f64,
        offset_y: f64,
        referencia: PuntoReferenciaCache,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostAccionCache {
    Inicial,

    Final,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoordenadaCache {
    pub ubicacion: UbicacionCache,

    pub post_accion: PostAccionCache,
}
