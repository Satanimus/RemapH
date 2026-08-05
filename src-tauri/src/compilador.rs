// ======================================================
// 🔨 COMPILADOR
// ======================================================
// ETAPA X DEL FLUJO
// ------------------------------------------------------
// 1. ¿Qué hace este archivo?
//
// Convierte un perfil editable (perfil_json)
// en un perfil optimizado para Runtime (perfil_cache).
//
// No ejecuta.
// No conoce dispositivos físicos.
// No conoce Runtime.
//
// Su responsabilidad:
// • Convertir estructuras.
// • Preparar triggers para búsqueda rápida.
// • Convertir respuestas de usuario en AccionCache.
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
// Recibe:
//
// perfil_json
//
// Contiene:
// • Remapeos.
// • Trigger.
// • Respuesta.
//
// Solo compila filas activas.
// ------------------------------------------------------
// 3. ¿Quién llama este archivo?
//
// Sistema de activación de perfil.
//
// Utilizado por:
// • Cache.
// • Sistema de perfiles.
//
// ------------------------------------------------------
// 4. ¿Qué información entrega?
//
// Entrega:
//
// Vec<RemapeoCache>
//
// Ejemplo:
//
// Trigger usuario:
//
// CTRL + A
// Doble
// Firefox
//
// ↓
//
// TriggerCache:
//
// app
// entrada
// condicion
//
// Acción usuario:
//
// Tecla B
//
// ↓
//
// AccionCache:
//
// Emitir keyboard:B
//
// ------------------------------------------------------
// 5. Funciones del archivo
//
// compilar()
//     Compila perfil completo y reemplaza Cache.
//     Ademas ejecuta orden revisar_app() para conocer si Apps del filtro están activas
// compilar_perfil()
//     Convierte todas las filas activas.
//
// compilar_remapeo()
//     Convierte una fila completa.
//
// convertir_app()
//     Convierte AppJson → AppCache.
//
// convertir_input()
//     Convierte Input → InputId.
//
// convertir_entrada()
//     Aplana modificadores + gatillo de un TriggerJson
//     en un solo Vec<InputId>.
//
// convertir_accion()
//     Resuelve accion_trigger/accion_referencia según
//     tipo → Option<AccionCache>. None si el tipo es
//     decorativo (todavía no soportado) O si la fila está
//     en "ON" pero le faltan datos requeridos para su tipo
//     (ej: tecla_mouse sin gatillo capturado todavía) — en
//     los dos casos la fila se descarta de la cache, nunca
//     se panickea por datos incompletos.
//
// convertir_coordenada()
//     Resuelve CoordenadaJson (strings de UI) → Option<CoordenadaCache>
//     (ubicación ya resuelta a números), cuando coordenada.activa es
//     true. None si todavía no se capturó — mismo criterio de
//     descarte silencioso.
// ------------------------------------------------------

use crate::cache;

use crate::eventos::InputId;

use crate::perfil_cache::{
    AccionCache, AlcanceMultimedia, AppCache, ComandoMultimedia, CoordenadaCache, ExtraCache,
    PostAccionCache, PuntoReferenciaCache, RemapeoCache, TriggerCache, UbicacionCache,
};

use crate::perfil_json::{perfil_json, AppJson, CoordenadaJson, RemapeoJson};

// ======================================================
// ⚙️ COMPILAR
// ======================================================

pub fn compilar(perfil: &perfil_json) {
    let remapeos = compilar_perfil(perfil);

    cache::borrar_cache();
    cache::escribir_cache(remapeos);

    crate::back_app::revisar_apps();
}

// ======================================================
// 📦 COMPILAR PERFIL
// ======================================================

pub fn compilar_perfil(perfil: &perfil_json) -> Vec<RemapeoCache> {
    perfil
        .remapeos
        .iter()
        .filter_map(compilar_remapeo)
        .collect()
}

// ======================================================
// 🧩 COMPILAR REMAPEO
// ======================================================

fn compilar_remapeo(remapeo: &RemapeoJson) -> Option<RemapeoCache> {
    if remapeo.estado != "ON" {
        return None;
    }

    let accion = convertir_accion(remapeo)?;

    let extra = convertir_extra(&remapeo.extra);

    // Coordenada ya no depende de un tipo aparte: es un extra
    // independiente de tecla_mouse. Si está activa pero todavía no se
    // capturó (x/y en None), la fila se descarta en silencio (mismo
    // criterio que una Acción sin capturar).
    let coordenada = if remapeo.coordenada.activa {
        Some(convertir_coordenada(&remapeo.coordenada)?)
    } else {
        None
    };

    Some(RemapeoCache {
        id: remapeo.id.clone(),

        trigger: TriggerCache {
            app: convertir_app(&remapeo.app),

            entrada: convertir_entrada(&remapeo.trigger),

            condicion: remapeo.trigger.condicion.clone(),
        },

        accion,

        extra,

        coordenada,
    })
}

// ======================================================
// 🖥️ CONVERTIR APP
// ======================================================

fn convertir_app(app: &AppJson) -> AppCache {
    match &app.programa {
        None => AppCache::Global,

        Some(nombre) => AppCache::Programa {
            nombre: nombre.clone(),

            segundo_plano: app.segundo_plano,
        },
    }
}

// ======================================================
// 🆔 CONVERTIR INPUT
// ======================================================

fn convertir_input(input: &crate::perfil_json::Input) -> InputId {
    InputId::new(&input.fuente, &input.control)
}

// ======================================================
// ⌨️ CONVERTIR ENTRADA (aplanar trigger)
// ------------------------------------------------------
// modificadores + gatillo, en ese orden, a un solo
// Vec<InputId> — es lo único que Cache necesita para
// comparar (no distingue modificador de gatillo).
// ======================================================

fn convertir_entrada(trigger: &crate::perfil_json::TriggerJson) -> Vec<InputId> {
    trigger
        .modificadores
        .iter()
        .map(convertir_input)
        .chain(trigger.gatillo.iter().map(convertir_input))
        .collect()
}

// ======================================================
// ⚡ CONVERTIR ACCIÓN
// ------------------------------------------------------
// accion_trigger / accion_referencia es una caja cuyo
// contenido depende de tipo: para tecla_mouse se usan
// modificadores + gatillo del accion_trigger, aplanados
// en un solo Vec<InputId> con convertir_input (misma
// convención que convertir_entrada del lado trigger:
// modificadores primero, gatillo al final). La condición
// del accion_trigger (Simple/Doble/Mantenido) SÍ se usa —
// viaja junto en Emitir(inputs, condicion) para que Runtime
// sepa si tiene que repetir el combo (Doble) o dejarlo
// abajo un rato (Mantenido). Para macro/archivo/ui se usa
// accion_referencia tal cual.
// Cualquier otro tipo (decorativo, sin implementar todavía
// en Rust) hace que la fila se descarte al compilar.
//
// Una fila puede estar en "ON" y aun así no tener todavía
// el dato que su tipo necesita (ej: se capturó el Trigger
// pero no la Acción). Eso NO es un error del programa —
// pasa mientras se arma la fila — así que se descarta de
// la cache en silencio, igual que un tipo decorativo. Antes
// esto panicaba y tiraba abajo toda la app al arrancar con
// cualquier perfil que tuviera una fila así guardada.
// ======================================================

fn convertir_accion(remapeo: &RemapeoJson) -> Option<AccionCache> {
    match remapeo.tipo.as_str() {
        "tecla_mouse" => {
            let trigger = remapeo.accion_trigger.as_ref()?;

            let gatillo = trigger.gatillo.as_ref()?;

            let inputs = trigger
                .modificadores
                .iter()
                .map(convertir_input)
                .chain(std::iter::once(convertir_input(gatillo)))
                .collect();

            Some(AccionCache::Emitir(inputs, trigger.condicion.clone()))
        }

        "macro" => Some(AccionCache::Macro(referencia(remapeo)?)),

        "archivo" => Some(AccionCache::AbrirArchivo(referencia(remapeo)?)),

        "ui" => Some(AccionCache::Ui(referencia(remapeo)?)),

        "multimedia" => {
            let comando = convertir_comando_multimedia(remapeo.accion_referencia.as_deref()?)?;

            let alcance = convertir_alcance_multimedia(remapeo, &comando);

            Some(AccionCache::Multimedia(comando, alcance))
        }

        // Tipos todavía decorativos en la UI (Portapapeles, etc.): no
        // producen ninguna acción real todavía. La fila entera se
        // descarta al compilar (ver compilar_remapeo), igual que una
        // fila en OFF.
        _ => None,
    }
}

// ======================================================
// 🎚️ CONVERTIR COMANDO MULTIMEDIA
// ------------------------------------------------------
// None si accion_referencia todavía no se eligió, o trae un valor
// desconocido — mismo criterio de descarte silencioso que el resto
// de convertir_accion (nunca panic por un dato incompleto).
// ======================================================

fn convertir_comando_multimedia(valor: &str) -> Option<ComandoMultimedia> {
    match valor {
        "volumen_subir" => Some(ComandoMultimedia::VolumenSubir),
        "volumen_bajar" => Some(ComandoMultimedia::VolumenBajar),
        "silenciar" => Some(ComandoMultimedia::Silenciar),
        "play_pausa" => Some(ComandoMultimedia::PlayPausa),
        "detener" => Some(ComandoMultimedia::Detener),
        "siguiente" => Some(ComandoMultimedia::Siguiente),
        "anterior" => Some(ComandoMultimedia::Anterior),
        _ => None,
    }
}

// ======================================================
// 🌐 CONVERTIR ALCANCE MULTIMEDIA
// ------------------------------------------------------
// "en_app" solo es válido para comandos de Volumen y solo si hay un
// programa elegido en la columna App — cualquier otro caso (mal
// dato guardado, condición cambió después) cae a Global en vez de
// descartar la fila entera; la UI ya impone estas reglas al elegir,
// esto es solo la red de seguridad del lado Rust.
// ======================================================

fn convertir_alcance_multimedia(
    remapeo: &RemapeoJson,
    comando: &ComandoMultimedia,
) -> AlcanceMultimedia {
    if remapeo.extra_multimedia != "en_app" || !comando.es_de_volumen() {
        return AlcanceMultimedia::Global;
    }

    match &remapeo.app.programa {
        Some(programa) => AlcanceMultimedia::EnApp {
            programa: programa.clone(),
        },

        None => AlcanceMultimedia::Global,
    }
}

// ======================================================
// 📎 REFERENCIA (accion_referencia requerida)
// ------------------------------------------------------
// None si falta — la fila se descarta (ver nota arriba),
// nunca se panickea por un dato incompleto.
// ======================================================

fn referencia(remapeo: &RemapeoJson) -> Option<String> {
    remapeo.accion_referencia.clone()
}

// ======================================================
// 🧩 CONVERTIR EXTRA
// ======================================================

fn convertir_extra(extra: &str) -> Option<ExtraCache> {
    match extra {
        "" => None,

        "turbo" => Some(ExtraCache::Turbo),

        "mantener" => Some(ExtraCache::Mantener),

        "toggle" => Some(ExtraCache::Toggle),

        _ => None,
    }
}

// ======================================================
// 🖱️ CONVERTIR COORDENADA
// ------------------------------------------------------
// None si todavía no se capturó (x/y en None) — la fila se
// descarta en compilar_remapeo, mismo criterio que una
// Acción sin capturar.
// ======================================================

fn convertir_coordenada(coordenada: &CoordenadaJson) -> Option<CoordenadaCache> {
    let x = coordenada.x?;
    let y = coordenada.y?;

    let ubicacion = match coordenada.ubicacion.as_str() {
        "relativa_cursor" => UbicacionCache::RelativaCursor {
            offset_x: x,
            offset_y: y,
        },

        "relativa_ventana" => match coordenada.modo_ventana.as_str() {
            "porcentaje" => UbicacionCache::RelativaVentanaPorcentaje { h: x, v: y },

            _ => UbicacionCache::RelativaVentanaPixeles {
                offset_x: x,
                offset_y: y,
                referencia: convertir_punto_referencia(&coordenada.punto_referencia),
            },
        },

        _ => UbicacionCache::Absoluta { x, y },
    };

    let post_accion = match coordenada.post_accion.as_str() {
        "inicial" => PostAccionCache::Inicial,

        _ => PostAccionCache::Final,
    };

    Some(CoordenadaCache {
        ubicacion,
        post_accion,
    })
}

fn convertir_punto_referencia(valor: &str) -> PuntoReferenciaCache {
    match valor {
        "sup_der" => PuntoReferenciaCache::SupDer,

        "centro" => PuntoReferenciaCache::Centro,

        "inf_izq" => PuntoReferenciaCache::InfIzq,

        "inf_der" => PuntoReferenciaCache::InfDer,

        _ => PuntoReferenciaCache::SupIzq,
    }
}
