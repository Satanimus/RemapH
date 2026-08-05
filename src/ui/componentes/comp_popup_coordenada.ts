// ======================================================
// 🖱️📌 comp_Popup_Coordenada
// ------------------------------------------------------
// Popup Extra cuando filaPerfil.tipo es "click_coordenada".
// Distinto del popup Extra simple (comp_popup_abrir.ts) que
// usan los demás tipos — acá el popup queda abierto entre
// selecciones (cada fila es un grupo de opciones, no un
// selector de una sola vez): al elegir una opción se
// actualiza el estado y se vuelve a dibujar el mismo popup
// en el lugar (mostrarPopup reemplaza el contenido sin
// cerrar la capa), en vez de cerrarlo.
//
// FILA 1 — Tipo de repetición (Normal/Mantener/Turbo)
// FILA 2 — Ubicación (Absoluta/Relativa a cursor/Relativa a
//          ventana) + sub-popup lateral si es Relativa a
//          ventana (modo Porcentaje/Píxeles + punto de
//          referencia, este último solo en Píxeles)
// FILA 3 — 📌 Capturar (abre la ventana de captura — ver
//          comandos.rs; la ventana en sí es la Etapa 3, acá
//          ya queda conectado el polling del resultado)
// FILA 4 — Post-acción (Posición inicial/Posición final)
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import type {
  TipoRepeticionCoordenada,
  UbicacionCoordenada,
  ModoVentanaCoordenada,
  PuntoReferenciaCoordenada,
  PostAccionCoordenada,
} from "../../core/core_coordenada";

import { textoCoordenada } from "../../core/core_coordenada";

// ======================================================
// 🚪 CERRAR VENTANA DE CAPTURA (auto-cancelación)
// ------------------------------------------------------
// Si la ventana overlay está abierta y el usuario cambia
// cualquier opción que la deja incoherente (ubicación, modo
// de ventana, punto de referencia, o el tipo de la fila deja
// de ser click_coordenada), se cierra sola — no puede quedar
// una ventana de captura en el aire sin saber para qué
// ubicación está calculando. Si no había ninguna abierta,
// cerrar_ventana_captura_coordenada no hace nada (comandos.rs
// ya maneja el caso "no existe la ventana").
// ======================================================

export function cerrarVentanaCapturaCoordenada(): void {
  invoke("cerrar_ventana_captura_coordenada").catch(() => {});
}

// ======================================================
// 🔘 GRUPO DE OPCIONES (fila de botones tipo radio)
// ======================================================

function crearGrupoOpciones<T extends string>(
  opciones: { texto: string; valor: T }[],
  valorActual: T,
  onSeleccionar: (valor: T) => void,
): HTMLElement {
  const grupo = document.createElement("div");

  grupo.className = "coordenada-popup-grupo";

  opciones.forEach((opcion) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn coordenada-popup-opcion";

    boton.textContent = opcion.texto;

    boton.dataset.activo = opcion.valor === valorActual ? "true" : "false";

    boton.addEventListener("click", () => {
      onSeleccionar(opcion.valor);
    });

    grupo.append(boton);
  });

  return grupo;
}

// ======================================================
// 🏷️ FILA CON ETIQUETA
// ======================================================

function crearFilaPopup(etiqueta: string, contenido: HTMLElement): HTMLElement {
  const fila = document.createElement("div");

  fila.className = "coordenada-popup-fila";

  const label = document.createElement("span");

  label.className = "coordenada-popup-label";

  label.textContent = etiqueta;

  fila.append(label, contenido);

  return fila;
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
// 🪟 SUB-POPUP LATERAL: RELATIVA A VENTANA
// ======================================================

function abrirSubPopupVentana(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const coordenada = filaPerfil.coordenada;

  const popup = document.createElement("div");

  popup.className = "coordenada-popup";

  const redibujar = () => abrirSubPopupVentana(evento, contexto, filaPerfil);

  const modoOpciones: { texto: string; valor: ModoVentanaCoordenada }[] = [
    { texto: "Porcentaje", valor: "porcentaje" },
    { texto: "Píxeles", valor: "pixeles" },
  ];

  popup.append(
    crearFilaPopup(
      "Modo de medición",
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

    popup.append(
      crearFilaPopup(
        "Punto de referencia",
        crearGrupoOpciones(
          puntoOpciones,
          coordenada.puntoReferencia,
          (valor) => {
            coordenada.puntoReferencia = valor;
            coordenada.x = null;
            coordenada.y = null;

            cerrarVentanaCapturaCoordenada();
            reconstruirFila(contexto.id);
            redibujar();
          },
        ),
      ),
    );
  }

  mostrarPopup(popup, evento.clientX + 240, evento.clientY);
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
        abrirPopupCoordenada(evento, contexto, filaPerfil);
      })
      .catch(() => {
        clearInterval(intervalo);
      });
  }, 200);
}

// ======================================================
// 🖱️📌 ABRIR POPUP COORDENADA
// ======================================================

export function abrirPopupCoordenada(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const coordenada = filaPerfil.coordenada;

  const popup = document.createElement("div");

  popup.className = "coordenada-popup";

  const redibujar = () => abrirPopupCoordenada(evento, contexto, filaPerfil);

  // ----------------------------------
  // FILA 1 — Tipo de repetición
  // ----------------------------------

  const repeticionOpciones: {
    texto: string;
    valor: TipoRepeticionCoordenada;
  }[] = [
    { texto: "Normal", valor: "" },
    { texto: "Mantener", valor: "mantener" },
    { texto: "Turbo", valor: "turbo" },
  ];

  popup.append(
    crearFilaPopup(
      "Tipo de repetición",
      crearGrupoOpciones(
        repeticionOpciones,
        coordenada.tipoRepeticion,
        (valor) => {
          coordenada.tipoRepeticion = valor;

          reconstruirFila(contexto.id);
          redibujar();
        },
      ),
    ),
  );

  // ----------------------------------
  // FILA 2 — Ubicación
  // ----------------------------------

  const ubicacionOpciones: { texto: string; valor: UbicacionCoordenada }[] = [
    { texto: "Absoluta", valor: "absoluta" },
    { texto: "Relativa a cursor", valor: "relativa_cursor" },
    { texto: "Relativa a ventana", valor: "relativa_ventana" },
  ];

  popup.append(
    crearFilaPopup(
      "Ubicación",
      crearGrupoOpciones(ubicacionOpciones, coordenada.ubicacion, (valor) => {
        coordenada.ubicacion = valor;

        // Los ejes x/y significan otra cosa en cada ubicación — hay
        // que volver a capturar.
        coordenada.x = null;
        coordenada.y = null;

        cerrarVentanaCapturaCoordenada();
        reconstruirFila(contexto.id);
        redibujar();
      }),
    ),
  );

  if (coordenada.ubicacion === "relativa_ventana") {
    const modoTexto =
      coordenada.modoVentana === "porcentaje" ? "Porcentaje" : "Píxeles";

    const botonVentana = document.createElement("button");

    botonVentana.className = "ui-btn coordenada-popup-sublista";

    botonVentana.textContent = `Configurar ventana (${modoTexto})  ▸`;

    botonVentana.addEventListener("click", () => {
      abrirSubPopupVentana(evento, contexto, filaPerfil);
    });

    popup.append(botonVentana);
  }

  popup.append(crearSeparador());

  // ----------------------------------
  // FILA 3 — Capturar
  // ----------------------------------

  const botonCapturar = document.createElement("button");

  botonCapturar.className = "ui-btn coordenada-popup-capturar";

  botonCapturar.textContent = textoCoordenada(coordenada);

  botonCapturar.addEventListener("click", () => {
    iniciarCaptura(contexto, filaPerfil, evento);
  });

  popup.append(botonCapturar);

  popup.append(crearSeparador());

  // ----------------------------------
  // FILA 4 — Post-acción
  // ----------------------------------

  const postAccionOpciones: { texto: string; valor: PostAccionCoordenada }[] = [
    { texto: "Posición inicial", valor: "inicial" },
    { texto: "Posición final", valor: "final" },
  ];

  popup.append(
    crearFilaPopup(
      "Al finalizar ir a",
      crearGrupoOpciones(postAccionOpciones, coordenada.postAccion, (valor) => {
        coordenada.postAccion = valor;

        reconstruirFila(contexto.id);
        redibujar();
      }),
    ),
  );

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
