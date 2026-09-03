// ======================================================
// 🧩 core_Macro
// ------------------------------------------------------
// Modelo del archivo de Macro (carpeta de usuario /Macros,
// *.json). No es una FilaPerfil ni vive dentro del perfil —
// es un documento aparte que la fila tipo === "macro" solo
// referencia por nombre/ruta (ver core_perfil.ts,
// AccionCache::Macro en perfil_cache.rs).
//
// Mismo criterio que FilaPerfil: cada PasoMacro es un objeto
// plano con TODOS los campos de los 7 tipos siempre
// presentes, con efecto solo según `tipo` — no una unión
// discriminada. Evita que cambiar el Tipo de un paso ya
// armado borre los datos de los otros tipos, y espeja tal
// cual del lado Rust (misma razón que abrirAccion/
// portapapelesAccion/etc. en FilaPerfil).
//
// Espejo exacto de MacroJson / PasoMacroJson en
// macro_json.rs (camelCase vía #[serde(rename_all =
// "camelCase")]) — viaja tal cual hacia Rust al guardar, con
// UNA excepción: el campo teclaAccion de un paso tecla_mouse
// es un Trigger (modificadores/gatillo con forma Entrada —
// {tipo, codigo, nombre}, la misma que arma el capturador vía
// obtener_captura/EntradaCapturaUI), mientras que Rust espera
// ahí un TriggerJson con Input{fuente, control}. La tabla
// principal salva esta diferencia adentro de Rust (comando
// compilar_perfil recibe la forma UI cruda —EntradaUI— y la
// convierte con perfil_ui::convertir_trigger antes de guardar
// perfil_json). Los comandos de Macro (macro_guardar_paso,
// macro_guardar_como) no tienen ese paso intermedio del lado
// Rust — MacroArchivoJson deserializa TriggerJson estricto
// directamente — así que la conversión se hace acá antes de
// invocar, con macroArchivoParaBackend() más abajo. Sin esto,
// guardar cualquier paso Tecla/Mouse con un trigger capturado
// fallaba con "missing field `fuente`".
// ======================================================

import type { Trigger } from "./core_trigger";
import { crearTrigger } from "./core_trigger";
import type { Entrada, TipoEntrada } from "./core_entrada";
import { crearEntrada } from "./core_entrada";
import { traducirLote } from "./core_traductor";

// ======================================================
// 🗂️ ARCHIVO DE MACRO
// ======================================================

export interface MacroArchivo {
  nombre: string;

  pasos: PasoMacro[];
}

export function crearMacroArchivo(nombre: string): MacroArchivo {
  return {
    nombre,

    pasos: [],
  };
}

// ======================================================
// 📝 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// filaPerfil.accionReferencia guarda el nombre de la macro
// asignada a la fila (ver comp_popup_macro_accion.ts) — mismo
// campo genérico que ya usa "multimedia", mismo criterio de
// texto que textoAccionMultimedia/textoMenuAccion.
// ======================================================

export function textoMacroAccion(accionReferencia: string | null): string {
  return accionReferencia ? accionReferencia : "Seleccionar macro";
}

// ======================================================
// 🎚️ COMPORTAMIENTO (columna Extra de la tabla) — Etapa 8A
// ------------------------------------------------------
// Reemplaza el viejo textoMacroExtra (mostraba "cantidad de
// pasos" — ya no aplica: Extra deja de ser la puerta al
// editor, ver comp_popup_macro_accion.ts). Ahora Extra es
// simplemente el selector de Comportamiento, mismo espíritu
// que abrirExtra/portapapelesExtra: una opción persistente a
// elegir, no una acción a ejecutar.
//
// "unaEjecucion" y "toggle" comparten mecanismo en Runtime
// (Etapa 8B) — la diferencia es solo de etiqueta acá. Solo
// "teclaMantenida" es mecánicamente distinta (depende de
// Down/Up físico real).
// ======================================================

export type ComportamientoMacro =
  | "una_ejecucion"
  | "toggle"
  | "tecla_mantenida";

export interface MacroExtraPerfil {
  comportamiento: ComportamientoMacro;
  // Muestra el overlay Indicador_Macro (🟢 paso/total) mientras esta
  // macro se ejecuta — ver vent_indicador_macro_main.ts, modo play.
  // Por defecto apagado.
  indicadorEjecucion: boolean;
}

export function crearMacroExtra(): MacroExtraPerfil {
  return {
    comportamiento: "una_ejecucion",
    indicadorEjecucion: false,
  };
}

export function textoComportamientoMacro(
  comportamiento: ComportamientoMacro,
): string {
  switch (comportamiento) {
    case "toggle":
      return "Comportamiento: Toggle";

    case "tecla_mantenida":
      return "Comportamiento: Tecla mantenida";

    default:
      return "Comportamiento: Una ejecución";
  }
}

// ======================================================
// 🧱 TIPOS DE PASO
// ======================================================

export type TipoPasoMacro =
  | "tecla_mouse"
  | "espera"
  | "bucle"
  | "coordenada"
  | "pegar"
  | "abrir"
  | "multimedia";

// Mismo vocabulario que filaPerfil.extra para tecla_mouse tras
// el rediseño (ver core_trigger.ts / compilador.rs::
// convertir_extra): Simple/Doble/Triple/Mantenido dejaron de
// ser valores de Extra — se leen de teclaAccion.condicion (el
// gatillo capturado). Extra queda en solo tres valores:
// "" (Ninguno) | "normal" | "turbo". Sin "repeticion_rueda" —
// no hay Rueda dentro de una Macro.
export type ExtraTeclaMouseMacro = "" | "normal" | "turbo";

// Etapa F: arrastre diferido. "down" retiene mods+gatillo abajo
// hasta que un paso "up" posterior con la misma secuencia los
// libere. "" (Ninguno) es el comportamiento normal de siempre —
// ver Reglas 14-16.
export type RetencionTeclaMacro = "" | "down" | "up";

// Mismo vocabulario que CoordenadaPerfil.ubicacion
// (core_coordenada.ts), sin postAccion — ver nota de
// PasoCoordenadaMacro más abajo.
export type UbicacionPasoMacro =
  | "absoluta"
  | "relativa_cursor"
  | "relativa_ventana";

export type ModoVentanaPasoMacro = "porcentaje" | "pixeles";

export type PuntoReferenciaPasoMacro =
  | "sup_izq"
  | "sup_der"
  | "centro"
  | "inf_izq"
  | "inf_der";

// Mismo vocabulario que AbrirExtraPerfil (core_abrir.ts).
export type IniciarPasoMacro = "ventana" | "minimizado" | "maximizado";

export type InstanciasPasoMacro = "unica" | "multiple";

// Mismo vocabulario que filaPerfil.accionReferencia para
// tipo === "multimedia" (ver core_perfil.ts).
export type ComandoPasoMacro =
  | "volumen_subir"
  | "volumen_bajar"
  | "silenciar"
  | "play_pausa"
  | "detener"
  | "siguiente"
  | "anterior";

// "en_app" reusa el Filtro de App de la fila Macro que
// contiene esta macro (compilador.rs), no tiene programa
// propio acá — ver spec, sección Multimedia.
export type AlcancePasoMacro = "global" | "en_app";

// ======================================================
// 📄 PASO
// ------------------------------------------------------
// marcador: solo asignable a un paso anterior a un paso
//   Bucle existente, y solo mientras ningún otro paso ya
//   tenga esa misma letra (ver reglas en la spec, sección
//   Marcador). null si no está marcado.
// ======================================================

export interface PasoMacro {
  tipo: TipoPasoMacro;

  marcador: string | null;

  // Solo relevantes cuando tipo === "tecla_mouse".
  teclaAccion: Trigger;

  teclaExtra: ExtraTeclaMouseMacro;

  // Etapa F: marca "Solo Down"/"Solo Up" del subtítulo "Limitar"
  // (Regla 14). "" cuando no está limitado.
  teclaRetencion: RetencionTeclaMacro;

  // Un solo campo de Duración (ms), con dos usos según
  // contexto — no hay Up físico real dentro de una Macro, así
  // que ambos casos hay que simularlos con tiempo fijo:
  // • teclaAccion.condicion === "mantenido" (con Extra Ninguno)
  //   → cuánto se mantiene abajo el DOWN antes del UP
  //     (equivalente al tiempo que en el gatillo físico real
  //     dura la pulsación sostenida).
  // • teclaExtra !== "" (Normal/Turbo) → cuánto dura en total
  //     el bucle de repetición (equivalente a cuánto tiempo se
  //     mantendría apretado el gatillo físico real).
  // Se muestra en el editor cuando aplica alguno de los dos
  // casos; null mientras no se configuró.
  teclaDuracionMs: number | null;

  // Solo relevante cuando tipo === "espera".
  esperaMs: number;

  // Solo relevantes cuando tipo === "bucle". marcadorDestino
  // es la letra de Marcador a la que vuelve (null hasta
  // elegir un paso anterior). Un solo algoritmo (sin distinción
  // con_fin/sin_fin, ver Etapa 8B): resta 1 en cada visita: al
  // llegar a 0, resetea al valor programado y sigue de largo
  // — listo para una próxima visita si está anidado dentro de
  // otro bucle (permite bucles anidados, ver spec).
  bucleMarcadorDestino: string | null;

  bucleVeces: number;

  // Solo relevantes cuando tipo === "coordenada". Solo mueve
  // el mouse, no hace click (eso es un paso "tecla_mouse"
  // aparte). posicionInicial es una opción nueva, única y
  // excluyente: si está en true, el resto de estos campos se
  // ignora — guarda la posición del mouse al inicio de la
  // ejecución de la macro en vez de una ubicación fija.
  coordPosicionInicial: boolean;

  coordUbicacion: UbicacionPasoMacro;

  coordModoVentana: ModoVentanaPasoMacro;

  coordPuntoReferencia: PuntoReferenciaPasoMacro;

  coordX: number | null;

  coordY: number | null;

  // Nota/Grupo (antes "App") copiados de la CoordenadaBanco al
  // momento de seleccionar (mismo criterio que CoordenadaPerfil.
  // nota/aplicacion en core_coordenada.ts) — solo para mostrar en
  // el box informativo del popup, no en vivo contra el gestor.
  coordNota: string;

  coordAplicacion: string;

  // Solo relevante cuando tipo === "pegar". Misma ruta sirve
  // para un fijado del Portapapeles (con "copiar ruta") o
  // cualquier archivo del disco — back_portapapeles::pegar()
  // decide por extensión, no por origen. Formatos soportados:
  // .txt y .png únicamente (decisión: no se amplía).
  pegarRuta: string | null;

  // Solo relevantes cuando tipo === "abrir". Mismos 5 campos
  // que AbrirAccionPerfil/AbrirExtraPerfil (core_abrir.ts),
  // aplanados acá en vez de anidados — mismo criterio que el
  // resto de PasoMacro.
  abrirRuta: string | null;

  abrirIniciar: IniciarPasoMacro;

  abrirInstancias: InstanciasPasoMacro;

  abrirCon: string | null;

  abrirArgumento: string;

  // Solo relevantes cuando tipo === "multimedia".
  multimediaComando: ComandoPasoMacro | null;

  multimediaAlcance: AlcancePasoMacro;

  // Nota de texto plano, independiente del tipo — no se envía
  // al ejecutar la macro (columna Nota del editor, spec punto
  // 11). "" cuando no tiene nota.
  nota: string;
}

// ======================================================
// ➕ CREAR PASO
// ======================================================

export function crearPasoMacro(tipo: TipoPasoMacro): PasoMacro {
  return {
    tipo,

    marcador: null,

    teclaAccion: crearTrigger(),

    teclaExtra: "",

    teclaRetencion: "",

    teclaDuracionMs: null,

    esperaMs: 500,

    bucleMarcadorDestino: null,

    bucleVeces: 1,

    coordPosicionInicial: false,

    coordUbicacion: "absoluta",

    coordModoVentana: "pixeles",

    coordPuntoReferencia: "sup_izq",

    coordX: null,

    coordY: null,

    coordNota: "",

    coordAplicacion: "",

    pegarRuta: null,

    abrirRuta: null,

    abrirIniciar: "ventana",

    abrirInstancias: "multiple",

    abrirCon: null,

    abrirArgumento: "",

    multimediaComando: null,

    multimediaAlcance: "global",

    nota: "",
  };
}

// ======================================================
// 📋 CLONAR PASO
// ------------------------------------------------------
// Usada tanto por "Duplicar" (botón ⟫ del editor) como al
// clonar un paso internamente. El duplicado nunca hereda el
// marcador del original (ver spec) — a qué paso volvería el
// Bucle quedaría ambiguo con dos pasos marcados igual.
//
// Si el paso es un Bucle, tampoco hereda su letra propia
// (bucleMarcadorDestino) ni la cantidad de veces recorridas en
// memoria — el duplicado nace como un Bucle limpio, con una letra
// nueva (asignada por el llamador, ver botonDuplicar en
// comp_popup_macro_editor.ts) y sin destino todavía elegido.
// ======================================================

export function clonarPasoMacro(paso: PasoMacro): PasoMacro {
  return {
    ...paso,

    marcador: null,

    bucleMarcadorDestino:
      paso.tipo === "bucle" ? null : paso.bucleMarcadorDestino,

    teclaAccion: {
      ...paso.teclaAccion,

      modificadores: [...paso.teclaAccion.modificadores],
    },
  };
}

// ======================================================
// 📥 IMPORTAR PASOS DE OTRA MACRO
// ------------------------------------------------------
// A diferencia de clonarPasoMacro (Duplicar), acá SÍ hay que
// preservar la relación Bucle→Marcador de la macro de origen —
// solo se renombran las letras que ya estén en uso en
// pasosDestino, para no generar un choque de dos Bucles con la
// misma letra al pegar los pasos importados al final. Se arma
// un mapa letra-vieja→letra-nueva una sola vez (recorriendo los
// Bucles de origen en orden) y se aplica tanto a
// bucleMarcadorDestino (en el propio paso Bucle) como a
// marcador (en la fila que ese Bucle señala como destino) —
// ambos campos guardan la MISMA letra en la macro de origen.
// Letras que no colisionan con pasosDestino se mantienen igual.
// ======================================================

export function importarPasosMacro(
  pasosOrigen: PasoMacro[],
  pasosDestino: PasoMacro[],
): PasoMacro[] {
  const usadasEnDestino = new Set(
    pasosDestino
      .filter((p) => p.tipo === "bucle")
      .map((p) => p.bucleMarcadorDestino)
      .filter((m): m is string => m !== null),
  );

  const mapaLetras = new Map<string, string>();

  // pasosDestino + las letras ya reasignadas en esta misma
  // importación cuentan como "en uso" para letraBucleDisponible,
  // así dos Bucles importados con la misma letra de origen no
  // reciben la misma letra nueva.
  const letrasEnUso = [...pasosDestino];

  pasosOrigen
    .filter((p) => p.tipo === "bucle" && p.bucleMarcadorDestino !== null)
    .forEach((p) => {
      const letraVieja = p.bucleMarcadorDestino as string;

      if (mapaLetras.has(letraVieja)) {
        return;
      }

      if (!usadasEnDestino.has(letraVieja)) {
        mapaLetras.set(letraVieja, letraVieja);

        return;
      }

      const letraNueva = letraBucleDisponibleEntre(letrasEnUso);

      mapaLetras.set(letraVieja, letraNueva);

      // Reserva la letra nueva para el resto de esta importación
      // (no es un Bucle real, solo ocupa el marcador para que
      // letraBucleDisponibleEntre no la vuelva a ofrecer).
      letrasEnUso.push({ ...crearPasoMacro("bucle"), bucleMarcadorDestino: letraNueva });
    });

  return pasosOrigen.map((paso) => {
    const clonado = clonarPasoMacro(paso);

    if (paso.marcador !== null && mapaLetras.has(paso.marcador)) {
      clonado.marcador = mapaLetras.get(paso.marcador) as string;
    } else {
      clonado.marcador = paso.marcador;
    }

    if (paso.tipo === "bucle" && paso.bucleMarcadorDestino !== null) {
      clonado.bucleMarcadorDestino =
        mapaLetras.get(paso.bucleMarcadorDestino) ?? paso.bucleMarcadorDestino;
    }

    return clonado;
  });
}

// Misma progresión A..Z, luego A1/B1... que letraBucleDisponible
// (ui/comp_popup_macro_editor.ts) pero recibiendo la lista de
// pasos "en uso" por parámetro en vez de un array fijo — se
// duplica acá (en vez de importar desde el editor) porque
// core_macro.ts es el módulo de más bajo nivel y no depende de
// la capa de componentes.
function letraBucleDisponibleEntre(pasos: PasoMacro[]): string {
  const usadas = new Set(
    pasos
      .filter((p) => p.tipo === "bucle")
      .map((p) => p.bucleMarcadorDestino)
      .filter((m): m is string => m !== null),
  );

  const ALFABETO = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

  for (const letra of ALFABETO) {
    if (!usadas.has(letra)) {
      return letra;
    }
  }

  let indice = 0;

  while (true) {
    const letra = `${ALFABETO[indice % 26]}${Math.floor(indice / 26) + 1}`;

    if (!usadas.has(letra)) {
      return letra;
    }

    indice++;
  }
}

// ======================================================
// 🏷️ TEXTO DE TIPO (panel Funciones, resúmenes)
// ======================================================

export function textoTipoPasoMacro(tipo: TipoPasoMacro): string {
  switch (tipo) {
    case "tecla_mouse":
      return "🔠 Tecla/Mouse";

    case "espera":
      return "⏳ Espera";

    case "bucle":
      return "🔁 Bucle";

    case "coordenada":
      return "📌 Coordenada";

    case "pegar":
      return "📋 Pegar";

    case "abrir":
      return "📂 Abrir";

    case "multimedia":
      return "🎵 Multimedia";
  }
}

// ======================================================
// 🔣 ÍCONO DE TIPO (columna Tipo del editor — solo ícono)
// ------------------------------------------------------
// Mismo emoji que textoTipoPasoMacro, sin el texto — la celda
// Tipo de cada fila de paso (Etapa 7) muestra solo esto.
// ======================================================

export function iconoTipoPasoMacro(tipo: TipoPasoMacro): string {
  switch (tipo) {
    case "tecla_mouse":
      return "🔠";

    case "espera":
      return "⏳";

    case "bucle":
      return "🔁";

    case "coordenada":
      return "📌";

    case "pegar":
      return "📋";

    case "abrir":
      return "📂";

    case "multimedia":
      return "🎵";
  }
}

// ======================================================
// 🔤 LETRAS DE MARCADOR DISPONIBLES
// ------------------------------------------------------
// A, B, C... — sin límite práctico (26 debería alcanzar de
// sobra; si algún día no alcanza, se puede extender a AA/AB
// sin romper nada de lo ya guardado, el campo es un string
// libre). Usado tanto para ofrecer la próxima letra libre a un
// paso Bucle nuevo como para el selector de la columna
// Marcador (sección 3 de la spec).
// ======================================================

const ALFABETO_MARCADOR = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

export function letraMarcadorDisponible(pasos: PasoMacro[]): string {
  const usadas = new Set(
    pasos.map((paso) => paso.marcador).filter((m): m is string => m !== null),
  );

  for (const letra of ALFABETO_MARCADOR) {
    if (!usadas.has(letra)) {
      return letra;
    }
  }

  // Casos extremos (>26 marcadores en una sola macro): se
  // sigue con AA, AB... para no romper, aunque en la práctica
  // no debería llegar a pasar nunca.
  let indice = 0;

  while (true) {
    const letra = `${ALFABETO_MARCADOR[indice % 26]}${Math.floor(indice / 26) + 1}`;

    if (!usadas.has(letra)) {
      return letra;
    }

    indice++;
  }
}

// ======================================================
// 📤 PREPARAR PARA EL BACKEND (Trigger → forma Input de Rust)
// ------------------------------------------------------
// Convierte cada Entrada {tipo, codigo, nombre} de
// paso.teclaAccion a la forma {fuente, control} que Rust
// espera en TriggerJson/Input (perfil_json.rs) — ver comentario
// de cabecera del archivo. Mismo mapeo tipo→fuente que usa
// perfil_ui::convertir_fuente para la tabla principal
// (Teclado→keyboard, Mouse→mouse, Multimedia→multimedia,
// Joystick→joystick). Se descarta `nombre` (Rust no lo usa,
// solo viaja para mostrar el texto en la UI).
//
// Llamar SIEMPRE antes de invoke("macro_guardar_paso" | "macro_
// guardar_como", { macroArchivo }) — nunca mandar el
// macroArchivo del estado del editor tal cual.
// ======================================================

interface InputBackend {
  fuente: string;

  control: string;
}

interface TriggerBackend {
  modificadores: InputBackend[];

  gatillo: InputBackend | null;

  condicion: string;
}

function tipoEntradaAFuente(tipo: TipoEntrada): string {
  switch (tipo) {
    case "Teclado":
      return "keyboard";

    case "Mouse":
      return "mouse";

    case "Multimedia":
      return "multimedia";

    case "Joystick":
      return "joystick";
  }
}

function entradaAInputBackend(entrada: Entrada): InputBackend {
  return {
    fuente: tipoEntradaAFuente(entrada.tipo),

    control: entrada.codigo,
  };
}

function triggerABackend(trigger: Trigger): TriggerBackend {
  return {
    modificadores: trigger.modificadores.map(entradaAInputBackend),

    gatillo: trigger.gatillo ? entradaAInputBackend(trigger.gatillo) : null,

    condicion: trigger.condicion,
  };
}

export function macroArchivoParaBackend(
  macroArchivo: MacroArchivo,
): Record<string, unknown> {
  return {
    ...macroArchivo,

    pasos: macroArchivo.pasos.map((paso) => ({
      ...paso,

      teclaAccion: triggerABackend(paso.teclaAccion),

      teclaRetencion: paso.teclaRetencion === "" ? null : paso.teclaRetencion,
    })),
  };
}

// ======================================================
// 📥 RECIBIR DEL BACKEND (forma Input de Rust → Trigger)
// ------------------------------------------------------
// Camino inverso de macroArchivoParaBackend: invoke("macro_
// abrir" | "macro_guardar_como") devuelve teclaAccion con
// {fuente, control} (Rust nunca conoció la forma {tipo,
// codigo, nombre} que usa la UI — ver comentario de cabecera
// del archivo), así que sin esto triggerATexto/triggerAHTML
// leerían undefined en paso.teclaAccion.tipo/codigo/nombre
// para cualquier macro guardada con un trigger capturado.
//
// Mismo patrón que convertirEntrada en core_perfil_json.ts:
// un solo traducirLote() para todos los controles de todos
// los pasos (evita N round-trips a Tauri), `nombre` cae al
// control crudo si no matchea ningún pulsador conocido.
//
// Llamar SIEMPRE sobre lo que devuelve invoke("macro_abrir" |
// "macro_guardar_como") antes de usarlo como MacroArchivo del
// editor.
// ======================================================

function fuenteATipoEntrada(fuente: string): TipoEntrada {
  switch (fuente) {
    case "keyboard":
      return "Teclado";

    case "mouse":
      return "Mouse";

    case "multimedia":
      return "Multimedia";

    case "joystick":
      return "Joystick";

    default:
      return "Teclado";
  }
}

function inputBackendAEntrada(
  input: InputBackend,

  mapaNombres: Record<string, string>,
): Entrada {
  return crearEntrada(
    fuenteATipoEntrada(input.fuente),

    input.control,

    mapaNombres[input.control] ?? input.control,
  );
}

function triggerDesdeBackend(
  trigger: TriggerBackend,

  mapaNombres: Record<string, string>,
): Trigger {
  const resultado = crearTrigger();

  resultado.modificadores = trigger.modificadores.map((input) =>
    inputBackendAEntrada(input, mapaNombres),
  );

  resultado.gatillo = trigger.gatillo
    ? inputBackendAEntrada(trigger.gatillo, mapaNombres)
    : null;

  resultado.condicion = trigger.condicion as Trigger["condicion"];

  return resultado;
}

function recolectarControles(macroArchivo: {
  pasos: { teclaAccion: TriggerBackend }[];
}): string[] {
  const controles = new Set<string>();

  macroArchivo.pasos.forEach((paso) => {
    paso.teclaAccion.modificadores.forEach((input) =>
      controles.add(input.control),
    );

    if (paso.teclaAccion.gatillo) {
      controles.add(paso.teclaAccion.gatillo.control);
    }
  });

  return Array.from(controles);
}

export async function macroArchivoDesdeBackend(macroArchivoBackend: {
  nombre: string;

  pasos: (Omit<PasoMacro, "teclaAccion" | "teclaRetencion"> & {
    teclaAccion: TriggerBackend;
    teclaRetencion: string | null;
  })[];
}): Promise<MacroArchivo> {
  const controles = recolectarControles(macroArchivoBackend);

  const mapaNombres = controles.length
    ? await traducirLote(controles, "interno", "usuario")
    : {};

  return {
    ...macroArchivoBackend,

    pasos: macroArchivoBackend.pasos.map((paso) => ({
      ...paso,

      teclaAccion: triggerDesdeBackend(paso.teclaAccion, mapaNombres),

      teclaRetencion: (paso.teclaRetencion ??
        "") as PasoMacro["teclaRetencion"],
    })),
  };
}
