// ======================================================
// 🖱️📌 comp_Popup_Coordenada
// ------------------------------------------------------
// Popup Extra completo de Tecla/Mouse (filaPerfil.tipo ===
// "tecla_mouse"). Popup persistente: al elegir una opción se
// actualiza el estado y se vuelve a dibujar el mismo popup en
// el lugar (mostrarPopup reemplaza el contenido sin cerrar la
// capa), en vez de cerrarlo. Todo vive en UN solo popup que
// se extiende hacia abajo — no hay sub-popups laterales.
//
// NIVEL 1 — Repetición: Ninguno / Normal / Turbo (sobre
//          filaPerfil.extra, mismo vocabulario "" | "normal" |
//          "turbo" que usa compilador.rs vía convertir_extra()
//          — "Ninguno" ya no es sinónimo fijo de "sin Extra":
//          con condición Mantenido se resuelve del lado de
//          Rust a ExtraCache::Mantener, para depender del Up
//          físico real en vez de un tiempo fijo)
// INTERRUPTOR — Coordenada (sobre filaPerfil.coordenada.activa).
//          No excluyente con el nivel de arriba: se puede
//          combinar cualquier repetición con Coordenada. Al
//          activarse, expande debajo:
//   NIVEL 2 — "En relación a:" (Absoluta/Relativa a cursor/
//            Relativa a ventana). Si es Relativa a ventana, se
//            agrega una caja interna (otro nivel visual, mismo
//            popup) con "Medido en:" (Porcentaje/Píxeles) y,
//            solo en Píxeles, "Medido desde:".
//   BOTÓN   — 📌 Capturar Coordenada (abre la ventana de
//            captura — ver comandos.rs; conecta el polling
//            del resultado). Botón principal, cyan oscuro. Ya
//            no agrega "(desde Sup-Izq)" al valor — queda solo
//            el valor (ver textoCoordenada en core_coordenada.ts).
//   NIVEL 3 — "Finalizar en:" (Posición inicial/Posición final)
//
// El pin 📌 solo queda como emoji del texto del botón de
// captura y del texto del botón Extra de la tabla (ver
// textoExtraTeclaMouse) — el toggle de Coordenada usa el
// interruptor deslizante genérico (crearInterruptor).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import type {
  UbicacionCoordenada,
  ModoVentanaCoordenada,
  PuntoReferenciaCoordenada,
  PostAccionCoordenada,
} from "../core/core_coordenada";

import { textoCoordenada } from "../core/core_coordenada";

import {
  textoPuntoReferencia,
  textoUbicacionCoordenada,
  textoModoVentanaCoordenada,
  textoPostAccionCoordenada,
} from "../core/core_coordenada";

import {
  crearGrupoOpciones,
  crearFilaPopup,
  crearInterruptor,
} from "./comp_popup_grupo";

import {
  extrasPermitidosTeclaMouse,
  esGatilloRueda,
} from "../core/core_trigger";

// ======================================================
// 🧭 VOCABULARIO Ninguno / Normal / Turbo / Repetición
// ------------------------------------------------------
// Mismos valores que EXTRA_OPCIONES de comp_popup_abrir.ts
// (comparten filaPerfil.extra y convertir_extra() del lado
// Rust) — solo cambian los textos mostrados acá, específicos
// del popup de Tecla/Mouse. "Repetición" (repeticion_rueda) es
// exclusiva de gatillo Rueda — ver filtrado en
// extrasPermitidosTeclaMouse (core_trigger.ts) y
// PLAN_RUEDA_REPETICION.md.
//
// Nota Rueda: con gatillo Rueda, el valor "normal" se comporta
// igual que Extra Simple (match dispara Iniciar+Finalizar de una,
// sin depender de un Up real). Para que quede claro en pantalla,
// esa opción se MUESTRA como "Simple" cuando el trigger es Rueda
// — ver textoOpcionNivel1 más abajo. El valor interno sigue
// siendo "normal" (mismo que compilador.rs / ExtraCache::Normal);
// esto es solo un cambio de texto en la UI, no de código/valor.
// ======================================================

const EXTRA_TECLA_MOUSE_OPCIONES: { texto: string; valor: string }[] = [
  { texto: "Ninguno", valor: "" },
  { texto: "Normal", valor: "normal" },
  { texto: "Turbo", valor: "turbo" },
  { texto: "Repetición", valor: "repeticion_rueda" },
];

function textoOpcionNivel1(
  opcion: { texto: string; valor: string },
  esRueda: boolean,
): string {
  if (esRueda && opcion.valor === "normal") {
    return "Simple";
  }

  return opcion.texto;
}

// ======================================================
// 📝 TOOLTIP DEL BOTÓN EXTRA (columna de la tabla, ícono ⁘)
// ------------------------------------------------------
// Lista de líneas "Subtítulo: Elección". "Repetición" siempre
// presente; el resto de las líneas (Coordenada en adelante)
// solo se agregan si corresponde: el bloque completo solo si
// el toggle Coordenada está activo, "Medido en"/"Medido desde"
// solo si la ubicación/modo los vuelve relevantes — mismas
// condiciones que muestran esas filas dentro del popup.
// ======================================================

export function textoExtraTeclaMouse(filaPerfil: FilaPerfil): string {
  const opcion = EXTRA_TECLA_MOUSE_OPCIONES.find(
    (opcion) => opcion.valor === filaPerfil.extra,
  );

  const base = opcion
    ? textoOpcionNivel1(opcion, esGatilloRueda(filaPerfil.trigger))
    : filaPerfil.extra;

  const lineas = [`Repetición: ${base}`];

  const coordenada = filaPerfil.coordenada;

  if (coordenada.activa) {
    lineas.push(
      `En relación a: ${textoUbicacionCoordenada(coordenada.ubicacion)}`,
    );

    if (coordenada.ubicacion === "relativa_ventana") {
      lineas.push(
        `Medido en: ${textoModoVentanaCoordenada(coordenada.modoVentana)}`,
      );

      if (coordenada.modoVentana === "pixeles") {
        lineas.push(
          `Medido desde: ${textoPuntoReferencia(coordenada.puntoReferencia)}`,
        );
      }
    }

    lineas.push(
      `Finalizar en: ${textoPostAccionCoordenada(coordenada.postAccion)}`,
    );

    lineas.push(`Coordenada: ${textoCoordenada(coordenada)}`);
  }

  return lineas.join("\n");
}

// ======================================================
// 🚪 CERRAR VENTANA DE CAPTURA (auto-cancelación)
// ------------------------------------------------------
// Si la ventana overlay está abierta y el usuario cambia
// cualquier opción que la deja incoherente (ubicación, modo
// de ventana, punto de referencia, se apaga el toggle
// Coordenada, o el tipo de la fila deja de ser tecla_mouse),
// se cierra sola — no puede quedar una ventana de captura en
// el aire sin saber para qué ubicación está calculando. Si no
// había ninguna abierta, cerrar_ventana_captura_coordenada no
// hace nada (comandos.rs ya maneja el caso "no existe la
// ventana").
// ======================================================

export function cerrarVentanaCapturaCoordenada(): void {
  invoke("cerrar_ventana_captura_coordenada").catch(() => {});
}

// ======================================================
// ➖ SEPARADOR (reusa el mismo estilo del popup de App)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 🧱 CAJA INTERNA: RELATIVA A VENTANA
// ------------------------------------------------------
// Ya no es un sub-popup lateral aparte: es un bloque más,
// dentro del mismo popup, visualmente marcado como "otro
// nivel" (borde + fondo más oscuro que el panel — clase
// popup-caja-interna). Contiene Método de medición y, solo
// en Píxeles, Punto de referencia.
// ======================================================

function crearCajaVentana(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
  redibujar: () => void,
): HTMLElement {
  const coordenada = filaPerfil.coordenada;

  const caja = document.createElement("div");

  caja.className = "popup-caja-interna";

  const modoOpciones: { texto: string; valor: ModoVentanaCoordenada }[] = [
    { texto: "Píxeles", valor: "pixeles" },
    { texto: "Porcentaje", valor: "porcentaje" },
  ];

  caja.append(
    crearFilaPopup(
      "Medido en:",
      crearGrupoOpciones(modoOpciones, coordenada.modoVentana, (valor) => {
        coordenada.modoVentana = valor;

        // El punto de referencia y la coordenada guardada dejan de
        // tener sentido al cambiar de modo — hay que volver a
        // capturar (ver misma lógica al cambiar de Ubicación abajo).
        coordenada.puntoReferencia = "sup_izq";
        coordenada.x = null;
        coordenada.y = null;

        cerrarVentanaCapturaCoordenada();
        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  // Para Porcentaje el punto de referencia siempre es Sup-Izq — no
  // se muestran las opciones (ver spec).
  if (coordenada.modoVentana === "pixeles") {
    const puntoOpciones: { texto: string; valor: PuntoReferenciaCoordenada }[] =
      [
        { texto: "Sup-Izq", valor: "sup_izq" },
        { texto: "Sup-Der", valor: "sup_der" },
        { texto: "Centro", valor: "centro" },
        { texto: "Inf-Izq", valor: "inf_izq" },
        { texto: "Inf-Der", valor: "inf_der" },
      ];

    caja.append(
      crearFilaPopup(
        "Medido desde:",
        crearGrupoOpciones(
          puntoOpciones,
          coordenada.puntoReferencia,
          (valor) => {
            coordenada.puntoReferencia = valor;
            coordenada.x = null;
            coordenada.y = null;

            cerrarVentanaCapturaCoordenada();
            reconstruirFila(contexto.id);
            alModificar();
            redibujar();
          },
          "popup-grupo-grid3",
        ),
      ),
    );
  }

  return caja;
}

// ======================================================
// 📌 INICIAR CAPTURA
// ------------------------------------------------------
// Abre la ventana overlay pasándole la ubicación/modo/punto
// de referencia activos de la fila (comandos.rs los fija en
// captura_coordenada.rs ANTES de crear la ventana, así que
// captura.html los lee sin condición de carrera) y sondea el
// resultado. Si el usuario la cierra con Cancelar sin
// guardar, obtener_resultado_coordenada nunca deja de
// devolver null — este polling queda huérfano hasta que se
// abra una fila/página nueva. Es aceptable: no hace nada,
// solo un invoke liviano cada 200ms.
// ======================================================

function iniciarCaptura(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  evento: MouseEvent,
  alModificar: () => void,
): void {
  const coordenada = filaPerfil.coordenada;

  invoke("abrir_ventana_captura_coordenada", {
    ubicacion: coordenada.ubicacion,
    modoVentana: coordenada.modoVentana,
    puntoReferencia: coordenada.puntoReferencia,
  }).catch((error) => {
    // Antes esto se tragaba en silencio — si .build() falla del lado
    // de Rust (comandos.rs), era invisible. Ahora queda en la consola
    // de devtools de ESTA ventana (F12 en la ventana principal).
    console.error("abrir_ventana_captura_coordenada FALLÓ:", error);
  });

  const intervalo = setInterval(() => {
    invoke<[number, number] | null>("obtener_resultado_coordenada")
      .then((resultado) => {
        if (!resultado) {
          return;
        }

        clearInterval(intervalo);

        filaPerfil.coordenada.x = resultado[0];
        filaPerfil.coordenada.y = resultado[1];

        reconstruirFila(contexto.id);
        alModificar();
        abrirPopupExtraTeclaMouse(evento, contexto, filaPerfil, alModificar);
      })
      .catch(() => {
        clearInterval(intervalo);
      });
  }, 200);
}

// ======================================================
// 🖱️📌 ABRIR POPUP EXTRA (Tecla/Mouse)
// ======================================================

export function abrirPopupExtraTeclaMouse(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const coordenada = filaPerfil.coordenada;

  const popup = document.createElement("div");

  popup.className = "popup-extra";

  const redibujar = () =>
    abrirPopupExtraTeclaMouse(evento, contexto, filaPerfil, alModificar);

  // ----------------------------------
  // NIVEL 1 — Repetición: Ninguno / Normal / Turbo / Repetición
  //          (rueda)
  // ------------------------------------------------------
  // La lista de opciones se filtra según gatillo/condición del
  // Trigger (ver extrasPermitidosTeclaMouse en core_trigger.ts).
  // Si el valor guardado quedó fuera del set permitido (p. ej. la
  // fila tenía Turbo y se recapturó el gatillo como Rueda, o tenía
  // Repetición y la Condición pasó a Mantenido), se corrige acá
  // mismo a "normal" antes de dibujar — es el único valor presente
  // en las 3 variantes de permitidos (no-rueda, rueda-simple,
  // rueda-otro); "" no siempre lo está (rueda no la ofrece).
  // ----------------------------------

  const permitidos = extrasPermitidosTeclaMouse(filaPerfil.trigger);

  if (!permitidos.includes(filaPerfil.extra)) {
    filaPerfil.extra = "normal";

    reconstruirFila(contexto.id);
    alModificar();
  }

  const esRueda = esGatilloRueda(filaPerfil.trigger);

  const opcionesExtra = EXTRA_TECLA_MOUSE_OPCIONES.filter((opcion) =>
    permitidos.includes(opcion.valor),
  ).map((opcion) => ({
    ...opcion,
    texto: textoOpcionNivel1(opcion, esRueda),
  }));

  popup.append(
    crearFilaPopup(
      "Repetición",
      crearGrupoOpciones(opcionesExtra, filaPerfil.extra, (valor) => {
        filaPerfil.extra = valor;

        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // INTERRUPTOR — Coordenada
  // ----------------------------------

  popup.append(
    crearInterruptor("Coordenada", coordenada.activa, () => {
      coordenada.activa = !coordenada.activa;

      if (!coordenada.activa) {
        cerrarVentanaCapturaCoordenada();
      }

      reconstruirFila(contexto.id);
      alModificar();
      redibujar();
    }),
  );

  if (!coordenada.activa) {
    mostrarPopup(popup, evento.clientX, evento.clientY);

    return;
  }

  popup.append(crearSeparador());

  // ----------------------------------
  // NIVEL 2 — Ubicación (solo si Coordenada está activa)
  // ----------------------------------

  const ubicacionOpciones: { texto: string; valor: UbicacionCoordenada }[] = [
    { texto: "Absoluta", valor: "absoluta" },
    { texto: "Cursor", valor: "relativa_cursor" },
    { texto: "Ventana", valor: "relativa_ventana" },
  ];

  popup.append(
    crearFilaPopup(
      "En relación a:",
      crearGrupoOpciones(ubicacionOpciones, coordenada.ubicacion, (valor) => {
        coordenada.ubicacion = valor;

        // Los ejes x/y significan otra cosa en cada ubicación — hay
        // que volver a capturar.
        coordenada.x = null;
        coordenada.y = null;

        cerrarVentanaCapturaCoordenada();
        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  // Caja interna (mismo popup, otro nivel visual) — solo si la
  // ubicación es Relativa a Ventana.
  if (coordenada.ubicacion === "relativa_ventana") {
    popup.append(
      crearCajaVentana(contexto, filaPerfil, alModificar, redibujar),
    );
  }

  // ----------------------------------
  // NIVEL 3 — Post-acción
  // ----------------------------------

  const postAccionOpciones: { texto: string; valor: PostAccionCoordenada }[] = [
    { texto: "Posición Inicial", valor: "inicial" },
    { texto: "Posición Final", valor: "final" },
  ];

  popup.append(
    crearFilaPopup(
      "Finalizar en:",
      crearGrupoOpciones(postAccionOpciones, coordenada.postAccion, (valor) => {
        coordenada.postAccion = valor;

        reconstruirFila(contexto.id);
        alModificar();
        redibujar();
      }),
    ),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // BOTÓN — Capturar Coordenada
  // ----------------------------------

  const botonCapturar = document.createElement("button");

  botonCapturar.className = "ui-btn popup-extra-capturar";

  botonCapturar.textContent = `📌 ${textoCoordenada(coordenada)}`;

  botonCapturar.addEventListener("click", () => {
    iniciarCaptura(contexto, filaPerfil, evento, alModificar);
  });

  popup.append(botonCapturar);

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
