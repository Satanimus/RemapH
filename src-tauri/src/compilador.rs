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
//
// convertir_portapapeles()
//     Resuelve tipo == "portapapeles" → AccionCache::Portapapeles
//     (nunca None). Empaqueta portapapeles_accion + portapapeles_extra
//     en un solo AccionCache, mismo criterio que MenuExpress — sin
//     referencias a otras filas, así que nunca hay datos faltantes.
//
// convertir_abrir()
//     Resuelve tipo == "abrir" → Option<AccionCache::AbrirArchivo>.
//     None si todavía no se eligió ruta (dato faltante, descarte
//     silencioso de siempre) O si la ruta ya no existe en disco —
//     este segundo caso además registra una AdvertenciaCompilacion
//     con el número de fila, para que la UI pueda avisar "(Fila N)
//     Archivo o programa no encontrado." y mostrar la fila OFF ⚠️.
//
// convertir_macro()
//     Resuelve tipo == "macro" → Option<AccionCache::Macro>. Mismo
//     criterio que convertir_abrir(): None sin advertencia si
//     todavía no se eligió ninguna macro (dato faltante), None CON
//     advertencia si se eligió una pero el archivo /Macros/<nombre>
//     .json ya no existe (se borró/renombró desde afuera). A
//     diferencia de AbrirArchivo, AccionCache::Macro solo guarda el
//     NOMBRE (no el contenido/pasos ya resueltos) — el JSON de la
//     macro se lee y se interpreta recién en Runtime, al ejecutarse
//     (Etapa 8), no acá. Motivo: la macro es un archivo propio,
//     editable en cualquier momento desde su propio popup (guardado
//     directo, ver comp_popup_macro_editor.ts) sin pasar por
//     "recompilar el perfil" — si el compilador incrustara el
//     contenido en la cache, cualquier edición de la macro quedaría
//     vieja hasta la próxima recompilación del perfil que la usa.
//     Compilar solo confirma que la referencia sigue siendo válida,
//     igual que Abrir confirma que la ruta sigue existiendo. Desde
//     la Etapa 8A también resuelve el programa del Filtro de App de
//     la fila (para el paso Multimedia "En App" dentro de la macro)
//     y el Comportamiento (remapeo.macro_extra.comportamiento, ya
//     convertido a enum vía convertir_comportamiento_macro()).
// ------------------------------------------------------

use crate::cache;

use crate::eventos::InputId;

use crate::macro_usuario;

use crate::perfil_cache::{
    AccionCache, AlcanceMultimedia, AppCache, ColorBotonMenu, ComandoMultimedia,
    ComportamientoMacro, ComportamientoMenu, CondicionTrigger, CoordenadaCache, ExtraCache,
    FormaMenu, IniciarVentana, InstanciasAbrir, MenuBotonCache, PostAccionCache,
    PuntoReferenciaCache, RemapeoCache, TamanoBotonPortapapeles, TamanoMenu, TriggerCache,
    UbicacionCache, UbicacionMenu,
};

use crate::perfil_json::{perfil_json, AppJson, CoordenadaJson, ItemFilaJson, RemapeoJson};

use serde::Serialize;

use std::path::Path;

// ======================================================
// ⚠️ ADVERTENCIA DE COMPILACIÓN
// ------------------------------------------------------
// Aviso no fatal generado al compilar: la fila se descarta de la
// cache (no se ejecuta) pero el motivo se le muestra al usuario en
// vez de fallar en silencio — hoy solo lo genera convertir_abrir()
// (ruta que ya no existe), pero cualquier chequeo futuro con el
// mismo criterio ("dato guardado pero inválido, no solo faltante")
// puede sumar advertencias acá.
// ======================================================

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AdvertenciaCompilacion {
    pub fila: usize,

    pub mensaje: String,
}

// ======================================================
// 📦 RESULTADO COMPILACIÓN
// ------------------------------------------------------
// Lo que compilar() devuelve hacia perfil.rs/comandos.rs: activo
// espeja cache::esta_vacia() (mismo criterio de siempre, ver
// guardar_perfil() en perfil.rs), advertencias es la lista de
// AdvertenciaCompilacion generada durante esta compilación puntual
// (no se acumula entre llamadas).
// ======================================================

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResultadoCompilacion {
    pub activo: bool,

    pub advertencias: Vec<AdvertenciaCompilacion>,
}

// ======================================================
// ⚙️ COMPILAR
// ======================================================

pub fn compilar(perfil: &perfil_json) -> ResultadoCompilacion {
    let (remapeos, advertencias) = compilar_perfil(perfil);

    cache::borrar_cache();
    cache::escribir_cache(remapeos);

    crate::back_app::revisar_apps();

    // Cualquier ventana MenuExpress abierta puede estar mostrando
    // botones que ya no existen (fila borrada/editada) — se cierran
    // todas al recompilar en vez de intentar sincronizarlas en
    // caliente (decisión del usuario, ver back_menu_express.rs).
    crate::back_menu_express::cerrar_todas();

    // Mismo criterio para Portapapeles: cierra todas las ventanas
    // abiertas Y vacía ACTIVOS (deteniendo cualquier Registro que
    // hubiera quedado corriendo), tratando la recompilación como un
    // reinicio — ver back_portapapeles.rs::cerrar_todas().
    crate::back_portapapeles::cerrar_todas();

    ResultadoCompilacion {
        activo: !cache::esta_vacia(),
        advertencias,
    }
}

// ======================================================
// 📦 COMPILAR PERFIL
// ------------------------------------------------------
// numero_fila viaja en base 1 (el mismo "número de fila" que ve el
// usuario en la columna Número) — se calcula acá con enumerate(),
// no se guarda en ningún lado, así que siempre refleja el orden
// actual del perfil.
// ======================================================

pub fn compilar_perfil(perfil: &perfil_json) -> (Vec<RemapeoCache>, Vec<AdvertenciaCompilacion>) {
    let mut advertencias = Vec::new();

    let remapeos = perfil
        .filas
        .iter()
        .filter_map(|item| match item {
            ItemFilaJson::Fila(remapeo) => Some(remapeo),
            ItemFilaJson::Separador(_) => None,
        })
        .enumerate()
        .filter_map(|(indice, remapeo)| {
            compilar_remapeo(indice + 1, remapeo, perfil, &mut advertencias)
        })
        .collect();

    (remapeos, advertencias)
}

// ======================================================
// 🧩 COMPILAR REMAPEO
// ======================================================

fn compilar_remapeo(
    numero_fila: usize,
    remapeo: &RemapeoJson,
    perfil: &perfil_json,
    advertencias: &mut Vec<AdvertenciaCompilacion>,
) -> Option<RemapeoCache> {
    if remapeo.estado != "ON" {
        return None;
    }

    let accion = convertir_accion(numero_fila, remapeo, perfil, advertencias)?;

    // El Extra (Turbo/Mantener/Normal-con-repetición) es un molde de
    // Idioma Runtime pensado para una Acción tipo Emitir (down/up de
    // una tecla física) — ver runt_extra.rs::obtener() y
    // runtime.rs::sustituir_accion(), que deja las líneas del molde
    // intactas (sin sustituir nada) para cualquier Acción que no sea
    // Emitir. Para menu_express, portapapeles, multimedia, abrir y
    // macro esto es un problema real, no solo un desperdicio:
    // ejecutar_accion() en runtime.rs desvía CUALQUIER fila con
    // `extra: Some(_)` hacia ese molde ANTES de llegar al match que
    // ejecuta la acción real (abrir_o_alternar / back_multimedia::
    // ejecutar / abrir_archivo / runt_macro::ejecutar), así que una
    // fila de cualquiera de estos cinco tipos cuyo `filaPerfil.extra`
    // haya quedado en "normal" (default de fila nueva, ver
    // crearFila() en core_perfil.ts — ninguno de los cinco ofrece
    // configurar Extra genérico, así que ese default nunca se
    // sobreescribe) terminaba ejecutando un molde de no-ops en vez
    // del comando real: la tecla se consumía (el match sí ocurre)
    // pero no salía nada — mismo bug reportado para Multimedia
    // ("consume la tecla pero no genera ninguna acción"), reproducido
    // acá para "abrir" (no abría carpeta/archivo/exe pese a estar
    // bien configurado) y, sin este agregado, hubiera pasado igual
    // con "macro" en cuanto tuviera ejecutor propio. Se fuerza a None
    // sin importar qué haya guardado remapeo.extra.
    let extra = if remapeo.tipo == "menu_express"
        || remapeo.tipo == "portapapeles"
        || remapeo.tipo == "multimedia"
        || remapeo.tipo == "abrir"
        || remapeo.tipo == "macro"
    {
        None
    } else {
        // [FIX] La promoción "Ninguno"+Mantenido→ExtraCache::Mantener
        // (ver convertir_extra) tiene que mirar la condición del
        // GATILLO DE SALIDA (accion_trigger, el que decide cómo se
        // emite la Acción — ej. el Click de "tecla_mouse"), no la
        // condición del trigger de ENTRADA (cómo se detecta Q). Antes
        // se pasaba remapeo.trigger.condicion acá: una fila "Q simple
        // → Click Mantenido, Extra Ninguno" nunca promovía a Mantener
        // porque el trigger de entrada era Simple, no Mantenido — el
        // Click se emitía con el sleep fijo de ejecutar_emitir en vez
        // de sostenerse hasta el Up real de Q. Ahora se usa la
        // condición que de verdad viaja en la Acción (AccionCache::
        // Emitir(_, condicion), la única que tiene ese concepto);
        // para el resto de las Acciones (Ui, etc., sin condición
        // propia) se neutraliza con Simple, mismo resultado que antes
        // (no promueve).
        let condicion_salida = match &accion {
            AccionCache::Emitir(_, condicion) => condicion,
            _ => &CondicionTrigger::Simple,
        };

        convertir_extra(&remapeo.extra, condicion_salida)
    };

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

fn convertir_accion(
    numero_fila: usize,
    remapeo: &RemapeoJson,
    perfil: &perfil_json,
    advertencias: &mut Vec<AdvertenciaCompilacion>,
) -> Option<AccionCache> {
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

        "macro" => convertir_macro(numero_fila, remapeo, advertencias),

        "ui" => Some(AccionCache::Ui(referencia(remapeo)?)),

        "multimedia" => {
            let comando = convertir_comando_multimedia(remapeo.accion_referencia.as_deref()?)?;

            let alcance = convertir_alcance_multimedia(remapeo, &comando);

            Some(AccionCache::Multimedia(comando, alcance))
        }

        "menu_express" => convertir_menu_express(remapeo, perfil),

        "portapapeles" => Some(convertir_portapapeles(remapeo)),

        "abrir" => convertir_abrir(numero_fila, remapeo, advertencias),

        // Tipos todavía decorativos en la UI (sin implementar del
        // lado Rust): no producen ninguna acción real todavía. La
        // fila entera se descarta al compilar (ver compilar_remapeo),
        // igual que una fila en OFF.
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
// posición de la fila referenciada dentro de las filas normales de
// perfil.filas (el
// "número de fila" que ve el usuario en la columna Número), no el
// orden en que se fueron agregando los botones (ver spec / nota en
// perfil_json.rs::MenuAccionJson).
//
// Si no queda ningún botón (todos referenciaban filas ya borradas,
// o el menú nunca tuvo ninguno agregado), la fila entera se
// descarta — un MenuExpress vacío no se puede abrir (ver spec).
// ======================================================

fn convertir_menu_express(remapeo: &RemapeoJson, perfil: &perfil_json) -> Option<AccionCache> {
    // Solo filas normales son referenciables por fila_id (mismo
    // criterio de numero_fila en compilar_perfil, Regla 18: los
    // separadores no cuentan ni participan de esta posición).
    let filas_referenciables: Vec<&RemapeoJson> = perfil
        .filas
        .iter()
        .filter_map(|item| match item {
            ItemFilaJson::Fila(remapeo) => Some(remapeo),
            ItemFilaJson::Separador(_) => None,
        })
        .collect();

    let mut botones: Vec<(usize, MenuBotonCache)> = remapeo
        .menu_accion
        .botones
        .iter()
        .filter_map(|boton| {
            let posicion = filas_referenciables
                .iter()
                .position(|fila| fila.id == boton.fila_id)?;

            // Color de la fila REFERENCIADA (pulido, punto "Color
            // botón") — no el color de la fila MenuExpress. Se
            // resuelve acá porque filas_referenciables ya está a
            // mano (mismo find que resuelve `posicion` arriba);
            // evita que back_menu_express.rs tenga que volver a
            // buscar la fila del lado de la ventana.
            let color = filas_referenciables[posicion].color.clone();

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
// 📋 CONVERTIR PORTAPAPELES
// ------------------------------------------------------
// A diferencia de convertir_menu_express, nunca descarta la fila
// (siempre Some): no depende de ninguna referencia a otra fila del
// perfil (no hay "botones" que puedan quedar huérfanos), así que no
// hay ningún dato requerido que pueda faltar. nombre vacío es un
// estado válido — significa que la ventana usa su título por
// defecto (decisión de back_portapapeles.rs, etapa G), no que la
// fila esté incompleta.
// ======================================================

fn convertir_portapapeles(remapeo: &RemapeoJson) -> AccionCache {
    AccionCache::Portapapeles {
        nombre: remapeo.portapapeles_accion.nombre.clone(),
        comportamiento: convertir_comportamiento_menu(&remapeo.portapapeles_extra.comportamiento),
        ubicacion: convertir_ubicacion_menu(&remapeo.portapapeles_extra.ubicacion),
        tamano_boton: convertir_tamano_boton_portapapeles(&remapeo.portapapeles_extra.tamano_boton),
        tamano_texto: convertir_tamano_menu(&remapeo.portapapeles_extra.tamano_texto),
        limite: remapeo.portapapeles_extra.limite,
        color: remapeo.color.clone(),
    }
}

fn convertir_tamano_boton_portapapeles(valor: &str) -> TamanoBotonPortapapeles {
    match valor {
        "pequeno" => TamanoBotonPortapapeles::Pequeno,
        "grande" => TamanoBotonPortapapeles::Grande,
        _ => TamanoBotonPortapapeles::Mediano,
    }
}

// ======================================================
// 📂 CONVERTIR ABRIR
// ------------------------------------------------------
// None si todavía no se eligió ruta (dato faltante, descarte
// silencioso de siempre, sin advertencia). Si ya hay ruta pero
// Path::exists() da false, la fila también se descarta pero además
// se registra una AdvertenciaCompilacion — el dato existe pero es
// inválido (el archivo se movió/borró desde que se guardó), a
// diferencia de "todavía no se capturó". La ruta ya validada acá
// nunca se vuelve a comprobar en runtime.rs.
// ======================================================

fn convertir_abrir(
    numero_fila: usize,
    remapeo: &RemapeoJson,
    advertencias: &mut Vec<AdvertenciaCompilacion>,
) -> Option<AccionCache> {
    let ruta = remapeo.abrir_accion.ruta.clone()?;

    if !Path::new(&ruta).exists() {
        advertencias.push(AdvertenciaCompilacion {
            fila: numero_fila,
            mensaje: "Archivo o programa no encontrado.".to_string(),
        });

        return None;
    }

    Some(AccionCache::AbrirArchivo {
        ruta,
        iniciar: convertir_iniciar_ventana(&remapeo.abrir_extra.iniciar),
        instancias: convertir_instancias_abrir(&remapeo.abrir_extra.instancias),
        abrir_con: remapeo.abrir_extra.abrir_con.clone(),
        argumento: remapeo.abrir_extra.argumento.clone(),
    })
}

// ======================================================
// 🧩 CONVERTIR MACRO
// ------------------------------------------------------
// Ver entrada en el índice de funciones más arriba. La ruta se
// resuelve con macro_usuario::ruta_macro() — mismo módulo que ya
// usan macros.rs/comandos.rs para leer/guardar macros — solo para
// comprobar existencia, nunca se lee/parsea el contenido acá.
// ======================================================

fn convertir_macro(
    numero_fila: usize,
    remapeo: &RemapeoJson,
    advertencias: &mut Vec<AdvertenciaCompilacion>,
) -> Option<AccionCache> {
    let nombre = referencia(remapeo)?;

    let existe = macro_usuario::ruta_macro(&nombre)
        .map(|ruta| ruta.exists())
        .unwrap_or(false);

    if !existe {
        advertencias.push(AdvertenciaCompilacion {
            fila: numero_fila,
            mensaje: "La macro seleccionada ya no existe.".to_string(),
        });

        return None;
    }

    Some(AccionCache::Macro {
        nombre,
        programa: remapeo.app.programa.clone(),
        comportamiento: convertir_comportamiento_macro(&remapeo.macro_extra.comportamiento),
    })
}

fn convertir_comportamiento_macro(valor: &str) -> ComportamientoMacro {
    match valor {
        "toggle" => ComportamientoMacro::Toggle,
        "tecla_mantenida" => ComportamientoMacro::TeclaMantenida,
        _ => ComportamientoMacro::UnaEjecucion,
    }
}

fn convertir_iniciar_ventana(valor: &str) -> IniciarVentana {
    match valor {
        "minimizado" => IniciarVentana::Minimizado,

        "maximizado" => IniciarVentana::Maximizado,

        _ => IniciarVentana::Ventana,
    }
}

fn convertir_instancias_abrir(valor: &str) -> InstanciasAbrir {
    match valor {
        "unica" => InstanciasAbrir::Unica,

        _ => InstanciasAbrir::Multiple,
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

// ======================================================
// 🔁 CONVERTIR EXTRA (Repetición: Ninguno/Normal/Turbo)
// ------------------------------------------------------
// "Ninguno" (extra == "") ya no es sinónimo fijo de "sin
// Extra" (None): con condición Mantenido en el GATILLO DE
// SALIDA (la condición que viaja en AccionCache::Emitir, no
// la del trigger de entrada — ver el llamado en
// compilar_remapeo), ejecutar la Acción una sola vez SIN
// Extra significa quedar apretado hasta un sleep fijo, no
// hasta el Up físico real — por eso acá se deriva a
// Some(ExtraCache::Mantener), el mismo molde de
// runt_extra.rs ([ACCION_DOWN] ESPERAR DETENER [ACCION_UP])
// que ya usa Mantener/ClickSostenido para esperar el Up real.
// Con cualquier otra condición (Simple/Doble/Triple), "Ninguno"
// sigue siendo None (un solo toque, sin repetición).
//
// "mantener" se mantiene reconocido acá por compatibilidad con
// perfiles guardados antes de este cambio — la UI ya no ofrece
// esa opción (ver comp_popup_coordenada.ts), el nuevo camino
// para llegar a ExtraCache::Mantener es "Ninguno"+Mantenido.
// ======================================================

fn convertir_extra(extra: &str, condicion: &CondicionTrigger) -> Option<ExtraCache> {
    match extra {
        "" => match condicion {
            CondicionTrigger::Mantenido => Some(ExtraCache::Mantener),

            _ => None,
        },

        "normal" => Some(ExtraCache::Normal),

        "turbo" => Some(ExtraCache::Turbo),

        "mantener" => Some(ExtraCache::Mantener),

        "toggle" => Some(ExtraCache::Toggle),

        // Exclusivo de gatillo Rueda — ver PLAN_RUEDA_REPETICION.md.
        // El frontend solo ofrece este valor cuando el gatillo
        // capturado es la Rueda (comp_popup_coordenada.ts /
        // extrasPermitidosTeclaMouse en core_trigger.ts).
        "repeticion_rueda" => Some(ExtraCache::RepeticionRueda),

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
