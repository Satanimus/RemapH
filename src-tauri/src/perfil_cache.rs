// ======================================================
// 📦 perfil_cache RemapH V3
// ======================================================
// ETAPA 4 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
// Modelo interno compilado utilizado por Runtime.
// El compilador solamente transforma la información necesaria para acelerar la detección de triggers.
// La respuesta NO se interpreta.
// De esta forma agregar nuevos tipos de respuesta no obliga a modificar todo el motor.
// No guarda: Color. - Nota.- Remapeos OFF.
//
// Flujo:
// perfil_json
//      ↓
// Compilador
//      ↓
// perfil_cache
//      ↓
// Runtime
// ------------------------------------------------------
// 2. ¿Qué información recibe?
// Recibe información compilada desde perfil_json.
// El Trigger llega optimizado:
// - App.
// - Entrada
// - Condición.
// La Respuesta mantiene prácticamente el mismo formato que perfil_json.
//
// Ejemplo:
// Trigger:
// Firefox
// CTRL
// A
// Doble
//
// Respuesta:
// Tipo = tecla_mouse
// Acción = [keyboard:B]
// Ejecución = turbo
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
// Recibe:
// - Compilador.
//
// Lo utilizan:
// - Cache.
// - Runtime.
// - AnalizadorTrigger.
// ------------------------------------------------------
// 4. ¿Qué información entrega?
// TriggerCache:
// Información optimizada para responder:"¿Existe coincidencia?"
// RespuestaCache:
// Información necesaria para responder:"¿Qué debo ejecutar?"
//
// RemapeoCache:
// { id,trigger,respuesta }
// ------------------------------------------------------
// 5. Funciones y estructuras
// AppCache
//      Contexto de aplicación.
// CondicionTrigger
//      Tipo de disparador.
// RemapeoCache
//      Une Trigger + Respuesta.
// TriggerCache
//      Parte compilada del remapeo.
// RespuestaCache
//      Parte NO compilada del remapeo.
// ------------------------------------------------------
// Filosofía del compilador
// ✔ Se compila únicamente aquello que mejora el rendimiento del motor de búsqueda del Trigger.
// ✔ La Respuesta no se interpreta. Se transporta casi exactamente igual que en perfil_json.
// Así el motor permanece estable mientras la cantidad de tipos de respuesta puede crecer libremente.
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

    pub respuesta: RespuestaCache,
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
// ⚡ RESPUESTA CACHE
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct RespuestaCache {
    pub tipo: String,

    pub accion: String,

    pub ejecucion: String,
}
