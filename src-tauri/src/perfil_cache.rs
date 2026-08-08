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

    Triple,

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
    //
    // El segundo campo es la condición capturada para esta Acción
    // (Simple/Doble/Triple/Mantenido) — antes se descartaba al
    // compilar y toda Acción se ejecutaba como si fuera Simple.
    // Runtime decide con esto cómo ejecutar el combo (ver
    // ejecutar_emitir() en runtime.rs):
    // • Simple    → un solo down+up.
    // • Doble     → dos down+up, separados por
    //               config::delay_entre_salida_doble().
    // • Triple    → tres down+up, separados por el mismo
    //               config::delay_entre_salida_doble() (se reusa,
    //               sin campo propio).
    // • Mantenido → down, espera config::tiempo_salida_mantenido(),
    //               up.
    Emitir(Vec<InputId>, CondicionTrigger),

    Macro(String),

    AbrirArchivo(String),

    Ui(String),

    // Acción tipo Multimedia. No es un Emitir (no pasa por
    // Interception/back_interception): se ejecuta con
    // SendInput/keybd_event de Windows directo (alcance Global) o
    // leyendo/escribiendo la sesión de audio de un proceso vía
    // winmix (alcance En App) — ver back_multimedia.rs.
    Multimedia(ComandoMultimedia, AlcanceMultimedia),

    // Acción tipo MenuExpress. nombre/botones vienen de la columna
    // Acción de la fila (menu_accion); forma/columnas/filas/
    // comportamiento/ubicacion/tamaños vienen de la columna Extra
    // (menu_extra) — compilador.rs los empaqueta juntos acá porque
    // Runtime recibe un solo AccionCache (no hay ExtraCache aparte
    // para este tipo, ver perfil_json.rs). botones ya viene filtrado
    // (fila_id que ya no existe en el perfil se descarta en
    // silencio) y ordenado por posición de la fila referenciada en
    // la tabla — nunca vacío: convertir_menu_express (compilador.rs)
    // descarta la fila entera (None) si queda en 0 botones.
    MenuExpress {
        nombre: String,
        botones: Vec<MenuBotonCache>,
        forma: FormaMenu,
        columnas: u32,
        filas: u32,
        comportamiento: ComportamientoMenu,
        ubicacion: UbicacionMenu,
        tamano_boton: TamanoMenu,
        tamano_texto: TamanoMenu,
        // Color de la FILA MenuExpress (mismo vocabulario que la
        // paleta de color de fila — "cyan"/"green"/etc., ver
        // styl_variables.css --tag-<color>). Único campo "decorativo"
        // que sí viaja hasta acá pese a la regla general de
        // perfil_cache (ver header del archivo) — acá tiene un uso
        // funcional real: back_menu_express.rs lo usa como color base
        // del fondo semitransparente de la ventana (ver spec).
        color: String,
        // Nueva variable global de popup Extra (pulido): decide si
        // cada botón/gajo se tiñe con el color de SU PROPIA fila
        // referenciada (Color) o hereda el color de fondo de la
        // ventana de arriba (Monocromo, default — mismo criterio
        // "decorativo pero funcional" que `color`, ver comentario
        // arriba). El color efectivo POR BOTÓN ya viene resuelto en
        // cada MenuBotonCache.color (ver más abajo) — acá solo se
        // guarda el modo, para que back_menu_express.rs decida si
        // reenviárselo a la ventana o no.
        color_boton: ColorBotonMenu,
    },
}

// ======================================================
// ⚡ MENU EXPRESS CACHE
// ------------------------------------------------------
// Piezas compiladas del tipo "menu_express" — ver AccionCache::
// MenuExpress más arriba. Espejo de FormaMenu/ComportamientoMenu/
// UbicacionMenu/TamanoMenu (TS, core_menu_express.ts) y de forma/
// comportamiento/ubicacion/tamano_boton/tamano_texto (Rust,
// perfil_json.rs::MenuExpressExtraJson), ya resueltos a enum en vez
// de viajar como String hasta Runtime.
// ======================================================

#[derive(Clone, Debug, PartialEq)]
pub struct MenuBotonCache {
    // Id interno de la fila referenciada (no su número de orden en
    // la tabla) — con esto Runtime/back_menu_express.rs buscan la
    // fila en la caché ya compilada al ejecutar el botón (etapa 7).
    pub fila_id: String,

    pub renombrar: String,

    // Color de la FILA REFERENCIADA (fila_id) — no el color de la
    // fila MenuExpress (ese es AccionCache::MenuExpress::color, el
    // del fondo). Mismo vocabulario que la paleta de color de fila
    // ("cyan"/"green"/etc., "" si esa fila no tiene color asignado).
    // Solo tiene efecto visual cuando color_boton == Color (ver
    // AccionCache::MenuExpress::color_boton) — resuelto acá en vez
    // de en back_menu_express.rs para no tener que buscar la fila de
    // nuevo del lado de la ventana (compilador.rs ya tiene
    // perfil.remapeos a mano en convertir_menu_express).
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorBotonMenu {
    Monocromo,
    Color,
}

impl ColorBotonMenu {
    pub fn como_str(&self) -> &'static str {
        match self {
            ColorBotonMenu::Monocromo => "monocromo",
            ColorBotonMenu::Color => "color",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormaMenu {
    Radial,
    Cuadricula,
}

impl FormaMenu {
    /// Inverso de compilador.rs::convertir_forma_menu — usado por
    /// back_menu_express.rs para mandarle el dato a la ventana (TS
    /// trabaja con el mismo vocabulario string que core_menu_express.ts).
    pub fn como_str(&self) -> &'static str {
        match self {
            FormaMenu::Radial => "radial",
            FormaMenu::Cuadricula => "cuadricula",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComportamientoMenu {
    Toggle,
    Efimero,
}

impl ComportamientoMenu {
    pub fn como_str(&self) -> &'static str {
        match self {
            ComportamientoMenu::Toggle => "toggle",
            ComportamientoMenu::Efimero => "efimero",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UbicacionMenu {
    Persistente,
    Cursor,
}

impl UbicacionMenu {
    pub fn como_str(&self) -> &'static str {
        match self {
            UbicacionMenu::Persistente => "persistente",
            UbicacionMenu::Cursor => "cursor",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TamanoMenu {
    Pequeno,
    Mediano,
    Grande,
}

impl TamanoMenu {
    pub fn como_str(&self) -> &'static str {
        match self {
            TamanoMenu::Pequeno => "pequeno",
            TamanoMenu::Mediano => "mediano",
            TamanoMenu::Grande => "grande",
        }
    }
}

// ======================================================
// 🎚️ COMANDO MULTIMEDIA
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComandoMultimedia {
    VolumenSubir,
    VolumenBajar,
    Silenciar,
    PlayPausa,
    Detener,
    Siguiente,
    Anterior,
}

impl ComandoMultimedia {
    /// true para los 3 comandos de Volumen — son los únicos que
    /// admiten alcance En App (ver AlcanceMultimedia). Los de
    /// Reproducción solo existen como Global.
    pub fn es_de_volumen(&self) -> bool {
        matches!(
            self,
            ComandoMultimedia::VolumenSubir
                | ComandoMultimedia::VolumenBajar
                | ComandoMultimedia::Silenciar
        )
    }
}

// ======================================================
// 🌐 ALCANCE MULTIMEDIA
// ------------------------------------------------------
// EnApp ya trae el nombre del programa resuelto en tiempo de
// compilación (compilador.rs lo saca de remapeo.app.programa) — así
// runtime.rs/back_multimedia.rs no necesitan volver a mirar
// TriggerCache/AppCache para ejecutar.
// ======================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlcanceMultimedia {
    Global,

    EnApp { programa: String },
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
    Normal,

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
    /// true si la receta de runt_extra para este Extra necesita
    /// esperar el Up físico real (vía Iniciar sin Finalizar) en vez
    /// de mandar Iniciar + Detener juntos apenas se confirma el
    /// trigger — porque la receta termina en "ESPERAR DETENER"
    /// (Mantener/ClickSostenido) o repite en bucle hasta que la
    /// orden de detener lo corte en su REPETIR (Turbo/Normal).
    /// Independiente de la Condición que lo disparó (Simple/Doble/
    /// Mantenido): lo que decide si el final es diferido es el
    /// Extra, no el trigger.
    pub fn requiere_up_real(&self) -> bool {
        matches!(
            self,
            ExtraCache::Normal
                | ExtraCache::Turbo
                | ExtraCache::Mantener
                | ExtraCache::ClickSostenido
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
