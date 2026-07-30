// ======================================================
// 🗃️ CACHE RemapH V3
// ======================================================
// ETAPA 2 DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Mantiene en memoria:
//
// - Remapeos activos compilados.
// - Estado actual de aplicaciones.
// - La instancia activa actual (si hay una entrada
//   con match en curso).
//
// Cache NO conoce:
//
// - Runtime (más allá de mandarle órdenes).
// - Captura.
// - Cómo se clasifica una condición.
//
// Su responsabilidad es resolver una entrada:
//
// • Sin coincidencia.
// • Match único.
// • Necesita análisis de condición.
//
// Y sostener el ciclo de vida de esa coincidencia:
// avisarle a Runtime cuándo empieza (iniciar) y cuándo
// termina (finalizar).
//
// ------------------------------------------------------
// 2. ¿Qué información recibe?
//
// Recibe:
//
// - Remapeos desde compilador.
// - Escritura o eliminación de filas.
// - Estado de aplicaciones desde back_app.
// - Entrada acumulada (Vec<InputId>) desde Entrada:
//   el conjunto de inputs físicos actualmente
//   presionados, en cada ciclo.
// - Opcionalmente, la condición (Simple/Doble/Mantenido)
//   ya clasificada por AnalizadorTrigger, cuando Entrada
//   vuelve a preguntar tras un AnalizarCondicion.
//
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// - Compilador.
// - Entrada (el portero, con la entrada acumulada
//   del ciclo actual, y opcionalmente la condición).
// - Módulo estado aplicaciones.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Devuelve ResolucionEntrada:
//
// Pasar:
//     El Input físico no coincide con nada.
//     Entrada devuelve el Input físico.
//
// Consumir:
//     El Input físico no debe volver al sistema.
//     Cache ya envió la orden Iniciar a Runtime.
//
// AnalizarCondicion:
//     Solicita al AnalizadorTrigger determinar
//     Simple / Doble / Mantenido. Entrada debe volver
//     a llamar a resolver_entrada con esa condición.
//
// ------------------------------------------------------
// 5. Reglas de resolución (resolver_entrada)
//
// Si viene con condicion = None:
//
//   posibles = 0
//        → Pasar.
//
//   posibles = 1
//   exactas = 1
//        → Iniciar. Solo hay una fila candidata: se
//          manda "iniciar" de inmediato, sin esperar la
//          condición (la condición es información para
//          Runtime, no un criterio de match adicional).
//
//   posibles = exactas
//   exactas > 1
//        → AnalizarCondicion. Varias filas con la misma
//          entrada física pero distinta condición.
//
// Si viene con condicion = Some(x) (ya se pidió
// desambiguar):
//
//   exactas = 1 (filtrando también por condicion = x)
//        → Iniciar.
//
//   exactas = 0
//        → Pasar. Ninguna fila calzó con esa condición.
//
// ------------------------------------------------------
// 6. Iniciar / Finalizar
//
// Cache siempre envía dos órdenes por id a Runtime:
// iniciar y finalizar, sin importar la condición
// (Simple / Doble / Mantenido).
//
// - iniciar: se envía apenas se resuelve el match
//   (ver Reglas de resolución arriba). Cache guarda el
//   id y la entrada de esa instancia como "activa".
//
// - finalizar: se envía cuando el match se pierde, es
//   decir, cuando la entrada actual ya no contiene todos
//   los inputs de la instancia activa (se soltó alguno).
//   Se revisa al principio de cada resolver_entrada.
//
// Runtime decide qué hacer con ese par según el extra
// de la acción:
//
// - Simple / Doble: ejecuta una vez; espera el
//   finalizar solo para descartar la instancia.
//
// - Mantenido: repite en turbo mientras no llegue
//   el finalizar.
//
// Cache se limpia (deja de recordar la instancia activa)
// apenas envía el finalizar.
//
// ------------------------------------------------------
// 7. Funciones del archivo
//
// obtener_cache()
//     Acceso interno a la cache de remapeos.
// obtener_apps()
//     Acceso interno al estado de aplicaciones.
// obtener_activa()
//     Acceso interno a la instancia activa actual.
// escribir_cache()
//     Reemplaza toda la cache.
// escribir_fila()
//     Agrega un remapeo.
// borrar_cache()
//     Elimina todos los remapeos.
// borrar_fila()
//     Elimina un remapeo por ID.
// actualizar_estado_app()
//     Actualiza estado de aplicación.
// app_habilitada()
//     Determina si una fila aplica según la app activa.
// revisar_finalizacion()
//     Si la instancia activa perdió alguno de sus
//     inputs, manda Detener y limpia la instancia.
// iniciar_instancia()
//     Guarda la instancia activa y manda Iniciar.
// resolver_entrada()
//     Compara una entrada (y opcionalmente la condición)
//     contra la cache y aplica las reglas del punto 5.
// ------------------------------------------------------
// Transformación:
//
// perfil_cache
//       ↓
//    Cache
//       ↓
// Resolución de entrada
//       ↓
// Runtime (iniciar / finalizar)
// ======================================================

use crate::eventos::InputId;

use crate::perfil_cache::{AccionCache, AppCache, CondicionTrigger, ExtraCache, RemapeoCache};

use std::sync::{Mutex, OnceLock};

// ======================================================
// 📦 CACHE REMAPEOS
// ======================================================

static CACHE: OnceLock<Mutex<Vec<RemapeoCache>>> = OnceLock::new();

fn obtener_cache() -> &'static Mutex<Vec<RemapeoCache>> {
    CACHE.get_or_init(|| Mutex::new(Vec::new()))
}

// ======================================================
// 🖥️ ESTADO APLICACIONES
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct AppEstadoCache {
    pub app: AppCache,

    pub activa: bool,
}

static APPS: OnceLock<Mutex<Vec<AppEstadoCache>>> = OnceLock::new();

fn obtener_apps() -> &'static Mutex<Vec<AppEstadoCache>> {
    APPS.get_or_init(|| Mutex::new(Vec::new()))
}

// ======================================================
// 🟢 INSTANCIA ACTIVA
// ======================================================

struct InstanciaActiva {
    id: String,

    entrada: Vec<InputId>,
}

static ACTIVA: OnceLock<Mutex<Option<InstanciaActiva>>> = OnceLock::new();

fn obtener_activa() -> &'static Mutex<Option<InstanciaActiva>> {
    ACTIVA.get_or_init(|| Mutex::new(None))
}

// ======================================================
// 📤 ORDEN RUNTIME
// ======================================================

#[derive(Clone, Debug)]
pub enum OrdenRuntime {
    Iniciar {
        id: String,

        accion: AccionCache,

        extra: Option<ExtraCache>,
    },

    Detener {
        id: String,
    },
}

// ======================================================
// 📤 RESOLUCIÓN
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub enum ResolucionEntrada {
    Pasar,

    Consumir,

    AnalizarCondicion,
}

// ======================================================
// ✍️ ESCRIBIR CACHE
// ======================================================

pub fn escribir_cache(remapeos: Vec<RemapeoCache>) {
    *obtener_cache().lock().unwrap() = remapeos;
}

// ======================================================
// ✍️ ESCRIBIR FILA
// ======================================================

pub fn escribir_fila(remapeo: RemapeoCache) {
    obtener_cache().lock().unwrap().push(remapeo);
}

// ======================================================
// 🗑️ BORRAR CACHE
// ======================================================

pub fn borrar_cache() {
    obtener_cache().lock().unwrap().clear();
}

// ======================================================
// 🗑️ BORRAR FILA
// ======================================================

pub fn borrar_fila(id: &str) {
    obtener_cache()
        .lock()
        .unwrap()
        .retain(|remapeo| remapeo.id != id);
}

// ======================================================
// 🖥️ ACTUALIZAR ESTADO APP
// ======================================================

pub fn actualizar_estado_app(app: AppCache, activa: bool) {
    let mut apps = obtener_apps().lock().unwrap();

    if let Some(existente) = apps.iter_mut().find(|estado| estado.app == app) {
        existente.activa = activa;
    } else {
        apps.push(AppEstadoCache { app, activa });
    }
}

// ======================================================
// 🖥️ APP HABILITADA
// ======================================================

fn app_habilitada(app: &AppCache, apps: &[AppEstadoCache]) -> bool {
    match apps.iter().find(|estado| estado.app == *app) {
        Some(estado) => estado.activa,

        None => true,
    }
}

// ======================================================
// 🟢 REVISAR FINALIZACIÓN
// ======================================================

fn revisar_finalizacion(entrada: &[InputId]) {
    let mut activa = obtener_activa().lock().unwrap();

    let Some(instancia) = activa.as_ref() else {
        return;
    };

    let sigue_completa = instancia
        .entrada
        .iter()
        .all(|input| entrada.contains(input));

    if sigue_completa {
        return;
    }

    let id = instancia.id.clone();

    *activa = None;

    drop(activa);

    crate::runtime::ejecutar(OrdenRuntime::Detener { id });
}

// ======================================================
// 🟢 INICIAR INSTANCIA
// ======================================================

fn iniciar_instancia(remapeo: RemapeoCache, entrada: &[InputId]) {
    *obtener_activa().lock().unwrap() = Some(InstanciaActiva {
        id: remapeo.id.clone(),

        entrada: entrada.to_vec(),
    });

    crate::runtime::ejecutar(OrdenRuntime::Iniciar {
        id: remapeo.id,

        accion: remapeo.accion,

        extra: remapeo.extra,
    });
}

// ======================================================
// 🔎 RESOLVER ENTRADA
// ======================================================

pub fn resolver_entrada(
    entrada: &[InputId],

    condicion: Option<CondicionTrigger>,
) -> ResolucionEntrada {
    revisar_finalizacion(entrada);

    if entrada.is_empty() {
        return ResolucionEntrada::Pasar;
    }

    let cache = obtener_cache().lock().unwrap();

    let apps = obtener_apps().lock().unwrap();

    let mut posibles = 0;

    let mut exactas = 0;

    let mut candidato: Option<RemapeoCache> = None;

    for fila in cache.iter() {
        if !app_habilitada(&fila.trigger.app, &apps) {
            continue;
        }

        if fila.trigger.entrada.starts_with(entrada) {
            posibles += 1;
        }

        if fila.trigger.entrada.as_slice() != entrada {
            continue;
        }

        if let Some(condicion_pedida) = &condicion {
            if fila.trigger.condicion != *condicion_pedida {
                continue;
            }
        }

        exactas += 1;

        if exactas == 1 {
            candidato = Some(fila.clone());
        } else {
            candidato = None;
        }
    }

    drop(cache);

    drop(apps);

    if condicion.is_some() {
        return match (exactas, candidato) {
            (1, Some(remapeo)) => {
                iniciar_instancia(remapeo, entrada);

                ResolucionEntrada::Consumir
            }

            _ => ResolucionEntrada::Pasar,
        };
    }

    if posibles == 0 {
        return ResolucionEntrada::Pasar;
    }

    if posibles == exactas && exactas == 1 {
        if let Some(remapeo) = candidato {
            iniciar_instancia(remapeo, entrada);

            return ResolucionEntrada::Consumir;
        }
    }

    if posibles == exactas && exactas > 1 {
        return ResolucionEntrada::AnalizarCondicion;
    }

    ResolucionEntrada::AnalizarCondicion
}
