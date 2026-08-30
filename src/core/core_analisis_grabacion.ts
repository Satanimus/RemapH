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
  coordX: number;
  coordY: number;
}

function posicionACoordenada(
  posicion: [number, number],
  ventana: [number, number, number, number] | null,
  config: ConfigInicioGrabacion,
): CoordenadaCalculada {
  const [cursorX, cursorY] = posicion;

  if (config.claveModoCoordenadas === "absoluta" || !ventana) {
    return {
      coordUbicacion: "absoluta",
      coordModoVentana: "pixeles",
      coordX: cursorX,
      coordY: cursorY,
    };
  }

  const ventanaTupla = ventanaDesdeTupla(ventana);

  if (config.claveModoCoordenadas === "ventana_porcentaje") {
    const h = ((cursorX - ventanaTupla.x) / ventanaTupla.ancho) * 100;
    const v = ((cursorY - ventanaTupla.y) / ventanaTupla.alto) * 100;

    return {
      coordUbicacion: "relativa_ventana",
      coordModoVentana: "porcentaje",
      coordX: h,
      coordY: v,
    };
  }

  // ventana_pixeles
  const base = puntoReferenciaAbsoluto("sup_izq", ventanaTupla);

  return {
    coordUbicacion: "relativa_ventana",
    coordModoVentana: "pixeles",
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
// Regla 9: mismo análisis que el capturador de teclas normal
// — mientras las pulsaciones caen dentro de ventanaComboMs, se
// arman como modificadores + gatillo de un mismo Trigger.
// ======================================================

interface GrupoAbierto {
  modificadores: Entrada[];

  gatillo: Entrada;

  // Entradas Down que todavía no recibieron su Up — mientras
  // no esté vacío, el grupo sigue abierto aunque se haya vuelto
  // a superar ventanaComboMs (Regla 9 solo gobierna cuándo un
  // Down nuevo se suma como modificador vs. abre grupo nuevo;
  // el cierre por Up es independiente, Regla 6/Etapa E8).
  teclasVivas: string[];

  posicion: [number, number] | null;

  ventana: [number, number, number, number] | null;

  momentoApertura: number;

  momentoUltimo: number;
}

// ======================================================
// 🚪 CERRAR GRUPO → PASOS
// ------------------------------------------------------
// Devuelve, en orden: Espera (si > 0) + Coordenada (si hubo
// posición capturada) + Tecla/Mouse (Regla 7).
// ======================================================

function cerrarGrupo(
  grupo: GrupoAbierto,
  config: ConfigInicioGrabacion,
  momentoCierreAnterior: number,
): PasoMacro[] {
  const pasos: PasoMacro[] = [];

  const deltaReal = grupo.momentoApertura - momentoCierreAnterior;
  const esperaMs = calcularEsperaMs(Math.max(deltaReal, 0), config);

  if (esperaMs > 0) {
    const pasoEspera = crearPasoMacro("espera");
    pasoEspera.esperaMs = esperaMs;
    pasos.push(pasoEspera);
  }

  if (grupo.posicion) {
    const coordenada = posicionACoordenada(
      grupo.posicion,
      grupo.ventana,
      config,
    );

    const pasoCoordenada = crearPasoMacro("coordenada");
    pasoCoordenada.coordUbicacion = coordenada.coordUbicacion;
    pasoCoordenada.coordModoVentana = coordenada.coordModoVentana;
    pasoCoordenada.coordX = coordenada.coordX;
    pasoCoordenada.coordY = coordenada.coordY;
    pasos.push(pasoCoordenada);
  }

  const pasoTecla = crearPasoMacro("tecla_mouse");
  const trigger = crearTrigger();
  trigger.modificadores = grupo.modificadores;
  trigger.gatillo = grupo.gatillo;
  trigger.condicion = "simple";
  pasoTecla.teclaAccion = trigger;
  pasos.push(pasoTecla);

  return pasos;
}

// ======================================================
// 🎬 ANÁLISIS PRINCIPAL
// ------------------------------------------------------
// Recorre los eventos en orden manteniendo un único grupo
// abierto a la vez (Reglas 6, 7, 9, 10, 12). Down/Up de
// arrastre diferido "(down)"/"(up)" explícito NO se resuelve
// acá — Etapa E solo agrupa combos y separa Coordenada/Tecla;
// la sintaxis "(down)" es editable después por el usuario en
// el editor (Regla 13, Etapa F para el soporte en runtime).
// ======================================================

export function analizarGrabacion(
  eventos: EventoGrabadoUI[],
  config: ConfigInicioGrabacion,
  ventanaComboMs: number,
): PasoMacro[] {
  const pasos: PasoMacro[] = [];

  let grupo: GrupoAbierto | null = null;
  let momentoCierreAnterior = 0;

  const cerrar = (): void => {
    if (!grupo) {
      return;
    }

    pasos.push(...cerrarGrupo(grupo, config, momentoCierreAnterior));
    momentoCierreAnterior = grupo.momentoUltimo;
    grupo = null;
  };

  for (const evento of eventos) {
    if (evento.state === "Down") {
      if (!grupo) {
        grupo = {
          modificadores: [],
          gatillo: evento.entrada,
          teclasVivas: [evento.entrada.codigo],
          posicion: evento.posicion,
          ventana: evento.ventana,
          momentoApertura: evento.momentoMs,
          momentoUltimo: evento.momentoMs,
        };
        continue;
      }

      if (evento.momentoMs - grupo.momentoUltimo <= ventanaComboMs) {
        grupo.modificadores.push(grupo.gatillo);
        grupo.gatillo = evento.entrada;
        grupo.teclasVivas.push(evento.entrada.codigo);
        grupo.momentoUltimo = evento.momentoMs;
        continue;
      }

      cerrar();
      grupo = {
        modificadores: [],
        gatillo: evento.entrada,
        teclasVivas: [evento.entrada.codigo],
        posicion: evento.posicion,
        ventana: evento.ventana,
        momentoApertura: evento.momentoMs,
        momentoUltimo: evento.momentoMs,
      };
      continue;
    }

    if (evento.state === "Up") {
      if (!grupo) {
        continue;
      }

      grupo.teclasVivas = grupo.teclasVivas.filter(
        (codigo) => codigo !== evento.entrada.codigo,
      );
      grupo.momentoUltimo = evento.momentoMs;

      if (grupo.teclasVivas.length === 0) {
        cerrar();
      }
      continue;
    }

    // Pulse (Regla 12: incluye Rueda)
    if (grupo && evento.momentoMs - grupo.momentoUltimo <= ventanaComboMs) {
      const modificadores = [...grupo.modificadores, grupo.gatillo];

      pasos.push(
        ...cerrarGrupo(
          {
            modificadores,
            gatillo: evento.entrada,
            teclasVivas: [],
            posicion: grupo.posicion ?? evento.posicion,
            ventana: grupo.ventana ?? evento.ventana,
            momentoApertura: grupo.momentoApertura,
            momentoUltimo: evento.momentoMs,
          },
          config,
          momentoCierreAnterior,
        ),
      );

      momentoCierreAnterior = evento.momentoMs;
      grupo = null;
      continue;
    }

    cerrar();

    pasos.push(
      ...cerrarGrupo(
        {
          modificadores: [],
          gatillo: evento.entrada,
          teclasVivas: [],
          posicion: evento.posicion,
          ventana: evento.ventana,
          momentoApertura: evento.momentoMs,
          momentoUltimo: evento.momentoMs,
        },
        config,
        momentoCierreAnterior,
      ),
    );

    momentoCierreAnterior = evento.momentoMs;
  }

  cerrar();

  return pasos;
}
