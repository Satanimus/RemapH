// ======================================================
// 🧮 core_Analisis_Grabacion
// ------------------------------------------------------
// Etapa E del Grabador de Macro: traduce los eventos crudos
// devueltos por tomar_eventos_grabacion_macro() (Etapa D, vía
// EventoGrabadoCapturaUI en comandos.rs) a PasoMacro[] listos
// para insertarse en la tabla del editor (Etapa G).
//
// No conoce Tauri ni invoke() — recibe el arreglo de eventos
// ya traído por el llamador (mismo criterio de separación que
// el resto de core_*.ts: análisis puro, sin IO).
// ======================================================

import type { Entrada } from "./core_entrada";
import { crearTrigger } from "./core_trigger";
import type { Trigger } from "./core_trigger";
import type { PasoMacro, PuntoReferenciaPasoMacro } from "./core_macro";
import { crearPasoMacro } from "./core_macro";
import type { ConfigInicioGrabacion } from "./core_grabacion_macro";

// ======================================================
// 📦 EVENTO GRABADO (forma UI)
// ------------------------------------------------------
// Espejo de EventoGrabadoCapturaUI (comandos.rs, camelCase).
// entrada ya viene traducida a {tipo, codigo, nombre} — mismo
// shape que Entrada (core_entrada.ts).
// ======================================================

export interface EventoGrabadoUI {
  entrada: Entrada;

  state: "Down" | "Up" | "Pulse";

  magnitud: number | null;

  momentoMs: number;

  posicion: [number, number] | null;

  ventana: [number, number, number, number] | null;
}

// ======================================================
// 📐 VENTANA ACTIVA (forma cruda [x, y, ancho, alto])
// ======================================================

interface VentanaTupla {
  x: number;
  y: number;
  ancho: number;
  alto: number;
}

function ventanaDesdeTupla(
  ventana: [number, number, number, number],
): VentanaTupla {
  return { x: ventana[0], y: ventana[1], ancho: ventana[2], alto: ventana[3] };
}

// ======================================================
// 📐 PUNTO DE REFERENCIA → COORDENADA ABSOLUTA
// ------------------------------------------------------
// Mismo espejo que puntoReferenciaAbsoluto() en
// vent_captura_main.ts / punto_referencia_absoluto() en
// back_coordenada.rs.
// ======================================================

function puntoReferenciaAbsoluto(
  referencia: PuntoReferenciaPasoMacro,
  ventana: VentanaTupla,
): { x: number; y: number } {
  switch (referencia) {
    case "sup_izq":
      return { x: ventana.x, y: ventana.y };

    case "sup_der":
      return { x: ventana.x + ventana.ancho, y: ventana.y };

    case "centro":
      return {
        x: ventana.x + ventana.ancho / 2,
        y: ventana.y + ventana.alto / 2,
      };

    case "inf_izq":
      return { x: ventana.x, y: ventana.y + ventana.alto };

    case "inf_der":
      return { x: ventana.x + ventana.ancho, y: ventana.y + ventana.alto };
  }
}

// ======================================================
// 📍 POSICIÓN → COORDENADA DE PASO
// ------------------------------------------------------
// Traduce una posición absoluta de pantalla a los campos
// coordUbicacion/coordModoVentana/coordX/coordY de un
// PasoMacro tipo "coordenada", según el Modo de Coordenadas
// elegido en el popup de inicio (Regla 2). Punto de referencia
// fijo "sup_izq" en modo ventana_pixeles (mismo default que
// crearPasoMacro() — el popup de inicio no pide punto de
// referencia, ver Etapa C).
// ======================================================

interface CoordenadaCalculada {
  coordUbicacion: PasoMacro["coordUbicacion"];
  coordModoVentana: PasoMacro["coordModoVentana"];
  coordPuntoReferencia: PuntoReferenciaPasoMacro;
  coordX: number;
  coordY: number;
}

function posicionACoordenada(
  posicion: [number, number],
  ventana: [number, number, number, number] | null,
  config: ConfigInicioGrabacion,
): CoordenadaCalculada {
  const [cursorX, cursorY] = posicion;

  if (config.tipoCoordenada === "absoluta" || !ventana) {
    return {
      coordUbicacion: "absoluta",
      coordModoVentana: "pixeles",
      coordPuntoReferencia: "sup_izq",
      coordX: cursorX,
      coordY: cursorY,
    };
  }

  const ventanaTupla = ventanaDesdeTupla(ventana);

  if (config.medidoEn === "porcentaje") {
    const h = ((cursorX - ventanaTupla.x) / ventanaTupla.ancho) * 100;
    const v = ((cursorY - ventanaTupla.y) / ventanaTupla.alto) * 100;

    return {
      coordUbicacion: "relativa_ventana",
      coordModoVentana: "porcentaje",
      coordPuntoReferencia: "sup_izq",
      coordX: h,
      coordY: v,
    };
  }

  // medidoEn === "pixeles"
  const base = puntoReferenciaAbsoluto(config.medidoDesde, ventanaTupla);

  return {
    coordUbicacion: "relativa_ventana",
    coordModoVentana: "pixeles",
    coordPuntoReferencia: config.medidoDesde,
    coordX: cursorX - base.x,
    coordY: cursorY - base.y,
  };
}

// ======================================================
// ⏱️ TRATAMIENTO DE ESPERAS
// ------------------------------------------------------
// Espejo de ModoEsperaGrabacion (core_grabacion_macro.ts,
// Regla 3).
// ======================================================

function calcularEsperaMs(
  deltaReal: number,
  config: ConfigInicioGrabacion,
): number {
  switch (config.modoEspera) {
    case "limitar_maximo":
      return Math.min(deltaReal, config.msEspera);

    case "fijo":
      return config.msEspera;

    default:
      return deltaReal;
  }
}

// ======================================================
// 🧩 GRUPO ABIERTO (combo en construcción)
// ------------------------------------------------------
// Regla 1: el grupo sigue abierto mientras algo siga físicamente
// presionado (teclasVivas no vacío), sin límite de tiempo — ya no
// se compara contra una ventana fija entre eventos.
// ======================================================

interface GrupoAbierto {
  // Orden de aparición de las entradas de este grupo. La última es
  // el "gatillo actual" (gatilloActual()); todas las anteriores son
  // los modificadores. Puede contener códigos repetidos cuando un
  // multi-tap se aplanó (Regla 3: "1+2+2+2").
  secuencia: Entrada[];

  // Condición resultante del multi-tap del gatillo actual (Regla
  // 3): "doble"/"triple" mientras los toques quedan colapsados sin
  // agregarse a secuencia; vuelve a "simple" en cuanto se aplana o
  // se pasa a una tecla nueva.
  condicion: "simple" | "doble" | "triple";

  // Toques ya contados del gatillo actual sin agregarse a secuencia
  // (0 = todavía un solo toque, sin intento de doble/triple).
  tapsGatilloActual: number;

  // true una vez que el gatillo actual "se aplanó" (ventana de
  // doble/triple superada, o ya se llegó a triple y sigue
  // repitiéndose) — de ahí en más, cualquier repique de esa misma
  // tecla se agrega directo a secuencia, sin reintentar doble/triple.
  gatilloLiteral: boolean;

  // momentoMs del último Up de cada código — insumo de
  // condicionMultiTap() para medir la ventana desde el toque
  // anterior (mismo criterio que cache.rs).
  ultimoUpDe: Map<string, number>;

  // Entradas Down que todavía no recibieron su Up — mientras no
  // esté vacío, el grupo sigue abierto (Regla 1).
  teclasVivas: string[];

  posicion: [number, number] | null;

  ventana: [number, number, number, number] | null;

  // Posición/ventana en el momento del Up del gatillo (botón de
  // mouse), si trajo una — null hasta que ese Up llega. Solo se
  // usa para detectar arrastre (Regla nueva: Down en un punto, Up
  // en otro distinto ⇒ el grupo emite Down/Up diferido en vez de
  // colapsar en un único paso Tecla/Mouse).
  posicionAlSoltar: [number, number] | null;

  ventanaAlSoltar: [number, number, number, number] | null;

  momentoApertura: number;

  momentoUltimo: number;

  // Etapa C — Patrón Diferido (Reglas 4-6). true en cuanto el grupo
  // deja de poder representarse como una sola fila (Down de tecla
  // nueva tras un ciclo ya completo, o Up en orden no anidado).
  diferido: boolean;

  // Registro cronológico de cada Down/Up del grupo — insumo de
  // emitirDiferido() para reconstruir las líneas diferidas.
  eventosCrudos: {
    entrada: Entrada;
    state: "Down" | "Up";
    momentoMs: number;
  }[];

  // Snapshot de grupo.secuencia.slice(0, -1) tomado en el instante
  // en que Regla 4 dispara diferido=true (modificador(es) sostenidos
  // que enmarcan el grupo — uno solo, o varios si venían pegados al
  // principio sin tecla intermedia, Regla 5). null mientras no se
  // haya tomado el snapshot, o si Regla 6 (orden no anidado) invalida
  // cualquier agrupación: en ese caso el grupo entero se reconstruye
  // línea por línea desde eventosCrudos, sin envoltorio.
  wrapperDiferido: Entrada[] | null;
}

// El "gatillo actual" siempre es la última entrada agregada a
// secuencia — la tecla que se está disambiguando (mismo criterio
// que Sesion.objetivo() en cache.rs).
function gatilloActual(grupo: GrupoAbierto): Entrada {
  return grupo.secuencia[grupo.secuencia.length - 1];
}

// ======================================================
// 🔁 CONDICIÓN DE MULTI-TAP (Regla 3)
// ------------------------------------------------------
// Decide si un nuevo toque del MISMO código que el gatillo actual
// colapsa como doble/triple (misma ventana que cache.rs:
// tiempo_doble, medida desde el Up del toque anterior — se
// reutiliza igual entre toque 1→2 y 2→3) o si hay que aplanar:
// ventana superada, o ya se alcanzó triple (tapsPrevios >= 3, tope
// de CondicionTrigger).
// ======================================================

function condicionMultiTap(
  ultimoUp: number | undefined,
  momentoMs: number,
  tiempoDoble: number,
  tapsPrevios: number,
): "doble" | "triple" | "aplanar" {
  if (tapsPrevios >= 2) {
    return "aplanar";
  }

  const dentroDeVentana =
    ultimoUp !== undefined && momentoMs - ultimoUp <= tiempoDoble;

  if (!dentroDeVentana) {
    return "aplanar";
  }

  return tapsPrevios === 0 ? "doble" : "triple";
}

// ======================================================
// 🧵 EMITIR DIFERIDO (Patrón Diferido, Reglas 4-6)
// ------------------------------------------------------
// Reconstruye los pasos de un grupo diferido a partir de
// eventosCrudos (registro cronológico de cada Down/Up).
//
// Modo con envoltorio (grupo.wrapperDiferido no nulo — Reglas
// 4-5): el/los modificador(es) sostenidos que enmarcan el grupo
// salen como una línea "(down)" al principio y una línea "(up)"
// al final; cada tecla intermedia que completa su propio ciclo
// down/up mientras el envoltorio sigue sostenido sale como una
// única fila simple (Regla 4), en el orden real en que ocurrieron.
//
// Modo sin envoltorio (grupo.wrapperDiferido null — Regla 6,
// orden de liberación no anidado): no hay envoltorio confiable
// que fijar — cada evento crudo del grupo entero sale como su
// propia línea Down o Up independiente, en el orden real en que
// ocurrió.
// ======================================================

function triggerDeTecla(
  gatillo: Entrada,
  modificadores: Entrada[] = [],
): Trigger {
  const trigger = crearTrigger();
  trigger.modificadores = modificadores;
  trigger.gatillo = gatillo;
  trigger.condicion = "simple";
  return trigger;
}

function emitirDiferido(
  grupo: GrupoAbierto,
  _config: ConfigInicioGrabacion,
): PasoMacro[] {
  const pasos: PasoMacro[] = [];

  // Regla 6: sin envoltorio confiable — línea por línea, literal.
  if (!grupo.wrapperDiferido) {
    for (const evento of grupo.eventosCrudos) {
      const paso = crearPasoMacro("tecla_mouse");
      paso.teclaAccion = triggerDeTecla(evento.entrada);
      paso.teclaRetencion = evento.state === "Down" ? "down" : "up";
      pasos.push(paso);
    }
    return pasos;
  }

  // Reglas 4-5: envoltorio sostenido al principio y al final.
  const wrapper = grupo.wrapperDiferido;
  const codigosWrapper = new Set(wrapper.map((entrada) => entrada.codigo));

  const pasoDown = crearPasoMacro("tecla_mouse");
  pasoDown.teclaAccion = triggerDeTecla(
    wrapper[wrapper.length - 1],
    wrapper.slice(0, -1),
  );
  pasoDown.teclaRetencion = "down";
  pasos.push(pasoDown);

  // Teclas intermedias (Regla 4): cada una completa su propio ciclo
  // down/up mientras el envoltorio sigue sostenido — una sola fila
  // simple por ciclo completo, en el orden real en que ocurrieron.
  const abiertas = new Map<string, Entrada>();

  for (const evento of grupo.eventosCrudos) {
    if (codigosWrapper.has(evento.entrada.codigo)) {
      continue;
    }

    if (evento.state === "Down") {
      abiertas.set(evento.entrada.codigo, evento.entrada);
      continue;
    }

    if (!abiertas.has(evento.entrada.codigo)) {
      continue;
    }

    abiertas.delete(evento.entrada.codigo);

    const pasoIntermedio = crearPasoMacro("tecla_mouse");
    pasoIntermedio.teclaAccion = triggerDeTecla(evento.entrada);
    pasos.push(pasoIntermedio);
  }

  const pasoUp = crearPasoMacro("tecla_mouse");
  pasoUp.teclaAccion = triggerDeTecla(
    wrapper[wrapper.length - 1],
    wrapper.slice(0, -1),
  );
  pasoUp.teclaRetencion = "up";
  pasos.push(pasoUp);

  return pasos;
}

// ======================================================
// 🚪 CERRAR GRUPO → PASOS
// ------------------------------------------------------
// Devuelve, en orden: Espera (si > 0 y no es el primer grupo de
// la sesión) + Coordenada (si hubo posición capturada Y difiere
// de la última posición ya emitida) + Tecla/Mouse (Regla 7).
//
// Arrastre (Down en un punto, Up en otro): si el gatillo es de
// tipo Mouse y posicionAlSoltar difiere de la posición de
// apertura, un único paso "Tecla/Mouse" simple perdería el
// traslado — en vez de eso se emiten DOS pasos encadenados con
// teclaRetencion "down"/"up" (mismo mecanismo que el arrastre
// diferido editable a mano, ver core_macro.ts), cada uno con su
// propia Coordenada, igual que si el usuario lo hubiera armado
// manualmente en el editor.
// ======================================================

function cerrarGrupo(
  grupo: GrupoAbierto,
  config: ConfigInicioGrabacion,
  momentoCierreAnterior: number | null,
  omitirCoordenada: boolean,
): PasoMacro[] {
  const pasos: PasoMacro[] = [];

  // momentoCierreAnterior null == primer grupo de la sesión: se
  // omite el tiempo entre F9 (arranque real de la grabación,
  // momento_ms = 0 en grabacion_macro.rs) y la primera tecla —
  // ese tiempo de "reacción" del usuario no es parte de la macro.
  if (momentoCierreAnterior !== null) {
    const deltaReal = grupo.momentoApertura - momentoCierreAnterior;
    const esperaMs = calcularEsperaMs(Math.max(deltaReal, 0), config);

    if (esperaMs > 0) {
      const pasoEspera = crearPasoMacro("espera");
      pasoEspera.esperaMs = esperaMs;
      pasos.push(pasoEspera);
    }
  }

  const esArrastre =
    gatilloActual(grupo).tipo === "Mouse" &&
    !!grupo.posicion &&
    !!grupo.posicionAlSoltar &&
    (grupo.posicion[0] !== grupo.posicionAlSoltar[0] ||
      grupo.posicion[1] !== grupo.posicionAlSoltar[1]);

  // omitirCoordenada: la posición del mouse es la misma que la del
  // último paso Coordenada ya agregado — no hace falta repetirla.
  if (grupo.posicion && !omitirCoordenada) {
    const coordenada = posicionACoordenada(
      grupo.posicion,
      grupo.ventana,
      config,
    );

    const pasoCoordenada = crearPasoMacro("coordenada");
    pasoCoordenada.coordUbicacion = coordenada.coordUbicacion;
    pasoCoordenada.coordModoVentana = coordenada.coordModoVentana;
    pasoCoordenada.coordPuntoReferencia = coordenada.coordPuntoReferencia;
    pasoCoordenada.coordX = coordenada.coordX;
    pasoCoordenada.coordY = coordenada.coordY;
    pasos.push(pasoCoordenada);
  }

  // Regla 4-6 (Etapa C): grupo diferido — ya no cabe en una sola fila
  // Tecla/Mouse ni en el par down/up de arrastre de abajo (que asume
  // un único gatillo). Se delega la construcción de los pasos de
  // Tecla/Mouse completa a emitirDiferido().
  if (grupo.diferido) {
    pasos.push(...emitirDiferido(grupo, config));
    return pasos;
  }

  const trigger = crearTrigger();
  trigger.modificadores = grupo.secuencia.slice(0, -1);
  trigger.gatillo = gatilloActual(grupo);
  trigger.condicion = grupo.condicion;

  if (!esArrastre) {
    const pasoTecla = crearPasoMacro("tecla_mouse");
    pasoTecla.teclaAccion = trigger;
    pasos.push(pasoTecla);

    return pasos;
  }

  // Arrastre: paso Down (reusa el trigger + Coordenada ya
  // agregados arriba, que corresponden al punto de apertura).
  const pasoDown = crearPasoMacro("tecla_mouse");
  pasoDown.teclaAccion = trigger;
  pasoDown.teclaRetencion = "down";
  pasos.push(pasoDown);

  // Coordenada del punto de soltado, siempre presente (es la
  // razón de ser de la rama de arrastre).
  const coordenadaSoltar = posicionACoordenada(
    grupo.posicionAlSoltar as [number, number],
    grupo.ventanaAlSoltar,
    config,
  );

  const pasoCoordenadaSoltar = crearPasoMacro("coordenada");
  pasoCoordenadaSoltar.coordUbicacion = coordenadaSoltar.coordUbicacion;
  pasoCoordenadaSoltar.coordModoVentana = coordenadaSoltar.coordModoVentana;
  pasoCoordenadaSoltar.coordPuntoReferencia =
    coordenadaSoltar.coordPuntoReferencia;
  pasoCoordenadaSoltar.coordX = coordenadaSoltar.coordX;
  pasoCoordenadaSoltar.coordY = coordenadaSoltar.coordY;
  pasos.push(pasoCoordenadaSoltar);

  const pasoUp = crearPasoMacro("tecla_mouse");
  // Mismo trigger (secuencia idéntica) — es lo que
  // validarRetencionMacro exige para emparejar down/up.
  const triggerUp = crearTrigger();
  triggerUp.modificadores = grupo.secuencia.slice(0, -1);
  triggerUp.gatillo = gatilloActual(grupo);
  triggerUp.condicion = grupo.condicion;
  pasoUp.teclaAccion = triggerUp;
  pasoUp.teclaRetencion = "up";
  pasos.push(pasoUp);

  return pasos;
}

// ======================================================
// 🎬 ANÁLISIS PRINCIPAL
// ------------------------------------------------------
// Recorre los eventos en orden manteniendo un único grupo
// abierto a la vez (Reglas 1-7). Patrón Diferido (Reglas 4-6:
// tecla nueva tras un ciclo ya completo, u orden de liberación
// no anidado) ya queda resuelto en esta etapa — marca
// grupo.diferido y, si corresponde, fija grupo.wrapperDiferido;
// cerrarGrupo() delega la construcción final de esos pasos a
// emitirDiferido().
// ======================================================

export function analizarGrabacion(
  eventos: EventoGrabadoUI[],
  config: ConfigInicioGrabacion,
  tiempoDoble: number,
): PasoMacro[] {
  const pasos: PasoMacro[] = [];

  let grupo: GrupoAbierto | null = null;

  // null == todavía no se cerró ningún grupo (arranque de sesión,
  // Regla revisada: se omite el tiempo antes del primer paso).
  let momentoCierreAnterior: number | null = null;

  // Última posición de mouse efectivamente puesta en un paso
  // Coordenada — null hasta el primer grupo con posición. Se
  // actualiza siempre que un grupo trae posición, la haya emitido
  // o no (si es igual a la anterior, "seguir en la misma posición"
  // no cambia).
  let ultimaPosicionEmitida: [number, number] | null = null;

  const emitirCierre = (
    grupoACerrar: GrupoAbierto,
    momentoCierre: number,
  ): void => {
    const posicionIgual =
      !!grupoACerrar.posicion &&
      !!ultimaPosicionEmitida &&
      grupoACerrar.posicion[0] === ultimaPosicionEmitida[0] &&
      grupoACerrar.posicion[1] === ultimaPosicionEmitida[1];

    pasos.push(
      ...cerrarGrupo(
        grupoACerrar,
        config,
        momentoCierreAnterior,
        posicionIgual,
      ),
    );

    if (grupoACerrar.posicion) {
      ultimaPosicionEmitida =
        grupoACerrar.posicionAlSoltar ?? grupoACerrar.posicion;
    }

    momentoCierreAnterior = momentoCierre;
  };

  const cerrar = (): void => {
    if (!grupo) {
      return;
    }

    emitirCierre(grupo, grupo.momentoUltimo);
    grupo = null;
  };

  const crearGrupo = (evento: EventoGrabadoUI): GrupoAbierto => ({
    secuencia: [evento.entrada],
    condicion: "simple",
    tapsGatilloActual: 0,
    gatilloLiteral: false,
    ultimoUpDe: new Map(),
    teclasVivas: [evento.entrada.codigo],
    posicion: evento.posicion,
    ventana: evento.ventana,
    posicionAlSoltar: null,
    ventanaAlSoltar: null,
    momentoApertura: evento.momentoMs,
    momentoUltimo: evento.momentoMs,
    diferido: false,
    eventosCrudos:
      evento.state === "Down"
        ? [
            {
              entrada: evento.entrada,
              state: "Down",
              momentoMs: evento.momentoMs,
            },
          ]
        : [],
    wrapperDiferido: null,
  });

  for (const evento of eventos) {
    if (evento.state === "Down") {
      // Auto-repetición del SO: mientras se mantiene una tecla
      // presionada, Windows reenvía Down periódicamente sin que
      // haya habido Up de por medio. Si el código ya está vivo en
      // el grupo actual, es esa repetición — se descarta entera.
      if (grupo && grupo.teclasVivas.includes(evento.entrada.codigo)) {
        grupo.momentoUltimo = evento.momentoMs;
        continue;
      }

      if (!grupo) {
        grupo = crearGrupo(evento);
        continue;
      }

      const actual = gatilloActual(grupo);

      // Repique del MISMO gatillo ya aplanado (cola de la Regla 3):
      // se agrega directo a secuencia, literal, sin reintentar
      // doble/triple y SIN pasar por la comprobación de Patrón
      // Diferido de abajo — un repique del propio gatillo nunca la
      // dispara, sin importar cuántas veces se repita ("1+2+2+2+2",
      // no importa cuántos "2" haya).
      if (grupo.gatilloLiteral && evento.entrada.codigo === actual.codigo) {
        grupo.secuencia.push(evento.entrada);
        grupo.teclasVivas.push(evento.entrada.codigo);
        grupo.momentoUltimo = evento.momentoMs;
        grupo.eventosCrudos.push({
          entrada: evento.entrada,
          state: "Down",
          momentoMs: evento.momentoMs,
        });
        continue;
      }

      // Regla 3: repique del MISMO gatillo — intenta colapsar en
      // doble/triple; si la ventana se superó o ya se llegó a
      // triple, aplana (agrega literal a secuencia).
      if (evento.entrada.codigo === actual.codigo) {
        const resultado = condicionMultiTap(
          grupo.ultimoUpDe.get(actual.codigo),
          evento.momentoMs,
          tiempoDoble,
          grupo.tapsGatilloActual,
        );

        if (resultado === "aplanar") {
          for (let i = 0; i < grupo.tapsGatilloActual; i++) {
            grupo.secuencia.push(actual);
          }
          grupo.secuencia.push(evento.entrada);
          grupo.condicion = "simple";
          grupo.gatilloLiteral = true;
          grupo.tapsGatilloActual = 0;
        } else {
          grupo.tapsGatilloActual += 1;
          grupo.condicion = resultado;
        }

        grupo.teclasVivas.push(evento.entrada.codigo);
        grupo.momentoUltimo = evento.momentoMs;
        grupo.eventosCrudos.push({
          entrada: evento.entrada,
          state: "Down",
          momentoMs: evento.momentoMs,
        });
        continue;
      }

      // Regla 1 / Regla 4: tecla nueva. Si el grupo ya es diferido,
      // o si alguna tecla anterior de la secuencia ya completó su
      // ciclo down/up mientras el grupo seguía abierto (secuencia
      // más larga que teclasVivas), esta tecla ya no se suma a
      // secuencia — el grupo pasa a Patrón Diferido (Regla 4) y el
      // envoltorio sostenido queda fijado en grupo.secuencia.slice(0,
      // -1) tal como estaba justo antes de este evento (Regla 5 si
      // ese envoltorio tiene 2+ teclas pegadas al principio).
      if (grupo.diferido || grupo.secuencia.length > grupo.teclasVivas.length) {
        if (!grupo.diferido) {
          grupo.wrapperDiferido = grupo.secuencia.slice(0, -1);
        }
        grupo.diferido = true;
      } else {
        grupo.secuencia.push(evento.entrada);
        grupo.condicion = "simple";
        grupo.gatilloLiteral = false;
        grupo.tapsGatilloActual = 0;
      }

      grupo.teclasVivas.push(evento.entrada.codigo);
      grupo.momentoUltimo = evento.momentoMs;
      grupo.eventosCrudos.push({
        entrada: evento.entrada,
        state: "Down",
        momentoMs: evento.momentoMs,
      });
      continue;
    }

    if (evento.state === "Up") {
      if (!grupo) {
        continue;
      }

      grupo.ultimoUpDe.set(evento.entrada.codigo, evento.momentoMs);

      // Posición al soltar el GATILLO (el botón de mouse que
      // manda la acción, no un modificador que se suelta antes) —
      // es la que se compara contra la de apertura para detectar
      // arrastre. Se toma del propio evento Up, que trae su
      // posición/ventana actual del back-end.
      if (evento.entrada.codigo === gatilloActual(grupo).codigo) {
        grupo.posicionAlSoltar = evento.posicion;
        grupo.ventanaAlSoltar = evento.ventana;
      }

      // Regla 6: orden de liberación no anidado — se suelta una
      // tecla que no es el gatillo actual MIENTRAS el gatillo actual
      // sigue vivo (si el gatillo ya se hubiese soltado antes, esto
      // sería un cierre normal en orden anidado, Regla 2). Invalida
      // cualquier envoltorio ya fijado: el grupo entero se reconstruye
      // línea por línea desde eventosCrudos (emitirDiferido, modo sin
      // envoltorio).
      if (
        evento.entrada.codigo !== gatilloActual(grupo).codigo &&
        grupo.teclasVivas.includes(gatilloActual(grupo).codigo)
      ) {
        grupo.diferido = true;
        grupo.wrapperDiferido = null;
      }

      grupo.teclasVivas = grupo.teclasVivas.filter(
        (codigo) => codigo !== evento.entrada.codigo,
      );
      grupo.momentoUltimo = evento.momentoMs;
      grupo.eventosCrudos.push({
        entrada: evento.entrada,
        state: "Up",
        momentoMs: evento.momentoMs,
      });

      if (grupo.teclasVivas.length === 0) {
        cerrar();
      }
      continue;
    }

    // Pulse (Regla 12: incluye Rueda) — Regla 1: se une al grupo
    // abierto si algo sigue sostenido, sin comparar tiempo.
    if (grupo && grupo.teclasVivas.length > 0) {
      emitirCierre(
        {
          secuencia: [...grupo.secuencia, evento.entrada],
          condicion: "simple",
          tapsGatilloActual: 0,
          gatilloLiteral: false,
          ultimoUpDe: grupo.ultimoUpDe,
          teclasVivas: [],
          posicion: grupo.posicion ?? evento.posicion,
          ventana: grupo.ventana ?? evento.ventana,
          posicionAlSoltar: null,
          ventanaAlSoltar: null,
          momentoApertura: grupo.momentoApertura,
          momentoUltimo: evento.momentoMs,
          diferido: false,
          eventosCrudos: [],
          wrapperDiferido: null,
        },
        evento.momentoMs,
      );

      grupo = null;
      continue;
    }

    cerrar();

    emitirCierre(crearGrupo(evento), evento.momentoMs);
  }

  cerrar();

  return pasos;
}
