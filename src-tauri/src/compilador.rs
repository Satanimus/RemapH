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
//
// convertir_menu_express()
//     Resuelve tipo == "menu_express" → Option<AccionCache::
//     MenuExpress>. Empaqueta menu_accion + menu_extra en un solo
//     AccionCache (no hay ExtraCache aparte para este tipo). Filtra
//     en silencio los botones cuyo fila_id ya no exista en el
//     perfil, y los reordena por posición de la fila referenciada
//     en la tabla. None si el menú queda sin botones.
// ------------------------------------------------------

use crate::cache;

use crate::eventos::InputId;

use crate::perfil_cache::{
    AccionCache, AlcanceMultimedia, AppCache, ColorBotonMenu, ComandoMultimedia,
    ComportamientoMenu, CoordenadaCache, ExtraCache, FormaMenu, MenuBotonCache, PostAccionCache,
    PuntoReferenciaCache, RemapeoCache, TamanoMenu, TriggerCache, UbicacionCache, UbicacionMenu,
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

    // Cualquier ventana MenuExpress abierta puede estar mostrando
    // botones que ya no existen (fila borrada/editada) — se cierran
    // todas al recompilar en vez de intentar sincronizarlas en
    // caliente (decisión del usuario, ver back_menu_express.rs).
    crate::back_menu_express::cerrar_todas();
}

// ======================================================
// 📦 COMPILAR PERFIL
// ======================================================

pub fn compilar_perfil(perfil: &perfil_json) -> Vec<RemapeoCache> {
    perfil
        .remapeos
        .iter()
        .filter_map(|remapeo| compilar_remapeo(remapeo, perfil))
        .collect()
}

// ======================================================
// 🧩 COMPILAR REMAPEO
// ======================================================

fn compilar_remapeo(remapeo: &RemapeoJson, perfil: &perfil_json) -> Option<RemapeoCache> {
    if remapeo.estado != "ON" {
        return None;
    }

    let accion = convertir_accion(remapeo, perfil)?;

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

fn convertir_accion(remapeo: &RemapeoJson, perfil: &perfil_json) -> Option<AccionCache> {
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

        "menu_express" => convertir_menu_express(remapeo, perfil),

        // Tipos todavía decorativos en la UI (Portapapeles, etc.): no
        // producen ninguna acción real todavía. La fila entera se
        // descarta al compilar (ver compilar_remapeo), igual que una
        // fila en OFF.
        _ => None,
    }
}

// ======================================================
// ⚡ CONVERTIR MENU EXPRESS
// ------------------------------------------------------
// botones: se descarta en silencio cualquier fila_id que ya no
// exista en el perfil actual (fila borrada mientras tanto — mismo
// criterio de descarte silencioso que el resto del compilador, ver
// nota de convertir_accion más arriba). El orden final es por
// posición de la fila referenciada dentro de perfil.remapeos (el
// "número de fila" que ve el usuario en la columna Número), no el
// orden en que se fueron agregando los botones (ver spec / nota en
// perfil_json.rs::MenuAccionJson).
//
// Si no queda ningún botón (todos referenciaban filas ya borradas,
// o el menú nunca tuvo ninguno agregado), la fila entera se
// descarta — un MenuExpress vacío no se puede abrir (ver spec).
// ======================================================

fn convertir_menu_express(remapeo: &RemapeoJson, perfil: &perfil_json) -> Option<AccionCache> {
    let mut botones: Vec<(usize, MenuBotonCache)> = remapeo
        .menu_accion
        .botones
        .iter()
        .filter_map(|boton| {
            let posicion = perfil
                .remapeos
                .iter()
                .position(|fila| fila.id == boton.fila_id)?;

            // Color de la fila REFERENCIADA (pulido, punto "Color
            // botón") — no el color de la fila MenuExpress. Se
            // resuelve acá porque perfil.remapeos ya está a mano
            // (mismo find que resuelve `posicion` arriba); evita que
            // back_menu_express.rs tenga que volver a buscar la fila
            // del lado de la ventana.
            let color = perfil.remapeos[posicion].color.clone();

            Some((
                posicion,
                MenuBotonCache {
                    fila_id: boton.fila_id.clone(),
                    renombrar: boton.renombrar.clone(),
                    color,
                },
            ))
        })
        .collect();

    if botones.is_empty() {
        return None;
    }

    botones.sort_by_key(|(posicion, _)| *posicion);

    let botones = botones.into_iter().map(|(_, boton)| boton).collect();

    Some(AccionCache::MenuExpress {
        nombre: remapeo.menu_accion.nombre.clone(),
        botones,
        forma: convertir_forma_menu(&remapeo.menu_extra.forma),
        columnas: remapeo.menu_extra.columnas,
        filas: remapeo.menu_extra.filas,
        comportamiento: convertir_comportamiento_menu(&remapeo.menu_extra.comportamiento),
        ubicacion: convertir_ubicacion_menu(&remapeo.menu_extra.ubicacion),
        tamano_boton: convertir_tamano_menu(&remapeo.menu_extra.tamano_boton),
        tamano_texto: convertir_tamano_menu(&remapeo.menu_extra.tamano_texto),
        color: remapeo.color.clone(),
        color_boton: convertir_color_boton_menu(&remapeo.menu_extra.color_boton),
    })
}

fn convertir_forma_menu(valor: &str) -> FormaMenu {
    match valor {
        "cuadricula" => FormaMenu::Cuadricula,
        _ => FormaMenu::Radial,
    }
}

fn convertir_comportamiento_menu(valor: &str) -> ComportamientoMenu {
    match valor {
        "efimero" => ComportamientoMenu::Efimero,
        _ => ComportamientoMenu::Toggle,
    }
}

fn convertir_ubicacion_menu(valor: &str) -> UbicacionMenu {
    match valor {
        "cursor" => UbicacionMenu::Cursor,
        _ => UbicacionMenu::Persistente,
    }
}

fn convertir_color_boton_menu(valor: &str) -> ColorBotonMenu {
    match valor {
        "color" => ColorBotonMenu::Color,
        _ => ColorBotonMenu::Monocromo,
    }
}

fn convertir_tamano_menu(valor: &str) -> TamanoMenu {
    match valor {
        "pequeno" => TamanoMenu::Pequeno,
        "grande" => TamanoMenu::Grande,
        _ => TamanoMenu::Mediano,
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

        "normal" => Some(ExtraCache::Normal),

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
