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
// NIVEL 1     — Repetición: Ninguno / Normal / Turbo (sobre
//               filaPerfil.extra, mismo vocabulario "" |
//               "normal" | "turbo" que usa compilador.rs vía
//               convertir_extra() — "Ninguno" ya no es
//               sinónimo fijo de "sin Extra": con condición
//               Mantenido se resuelve del lado de Rust a
//               ExtraCache::Mantener, para depender del Up
//               físico real en vez de un tiempo fijo)
// INTERRUPTOR — Coordenada (sobre filaPerfil.coordenada.activa).
//               No excluyente con el nivel de arriba: se puede
//               combinar cualquier repetición con Coordenada.
//               Al activarse, expande debajo:
//   COORDENADA — Botón 📌 Seleccionar/Cambiar (abre el gestor
//               de coordenadas — vent_coordenadas_main.ts —
//               con su catálogo filtrable + "Nueva coordenada",
//               que internamente reusa el flujo de captura
//               existente) + ▶ Probar al lado, SOLO si la
//               coordenada elegida es tipo Absoluta (Cursor/
//               Ventana no tienen nada útil que probar acá:
//               ver comentario junto al botón). Una vez
//               elegida, aparece debajo un box informativo
//               (popup-caja-interna) de solo lectura con el
//               resumen — Nota/App/Tipo/Medido en/Medido
//               desde/X/Y, copiados de la CoordenadaBanco al
//               momento de seleccionar (no en vivo: si se edita
//               esa coordenada desde el gestor después, hay que
//               volver a seleccionarla acá para traer el
//               cambio — ver CoordenadaPerfil en
//               core_coordenada.ts). "En relación a"/"Medido
//               en"/"Medido desde" YA NO se editan acá — son
//               responsabilidad exclusiva del gestor.
//   FINALIZAR EN — "Posición inicial"/"Posición final" (sobre
//               coordenada.postAccion) — igual que antes, solo
//               reubicado bajo Coordenada.
//
// El pin 📌 solo queda como emoji del texto del botón
// Seleccionar/Cambiar y del texto del botón Extra de la tabla
// (ver textoExtraTeclaMouse) — el toggle de Coordenada usa el
// interruptor deslizante genérico (crearInterruptor).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import type { PostAccionCoordenada } from "../core/core_coordenada";

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

import {
  type CoordenadaBancoJson,
  convertirCoordenadaBanco,
  coordenadaBancoAPerfil,
} from "../core/core_banco_coordenadas";

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
// 📝 TOOLTIP DEL BOTÓN EXTRA (columna de la tabla, ícono ∴)
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
// ▶️ PROBAR COORDENADA
// ------------------------------------------------------
// Mueve el cursor real al punto calculado (comandos.rs::
// probar_coordenada) — solo tiene sentido si la coordenada ya
// fue capturada (x/y no null).
// ======================================================

function probarCoordenada(coordenada: FilaPerfil["coordenada"]): void {
  if (coordenada.x === null || coordenada.y === null) {
    return;
  }

  invoke("probar_coordenada", {
    ubicacion: coordenada.ubicacion,
    modoVentana: coordenada.modoVentana,
    puntoReferencia: coordenada.puntoReferencia,
    x: coordenada.x,
    y: coordenada.y,
  }).catch(() => {});
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
// 📋 BOX INFORMATIVO — RESUMEN DE LA COORDENADA
// ------------------------------------------------------
// Box anidado (popup-caja-interna, mismo look que cualquier
// otro nivel subordinado del popup) que aparece bajo el botón
// Seleccionar/Cambiar una vez que hay una coordenada elegida.
// Solo informativo, no editable acá — para editar Nota/App/
// valores hay que ir al gestor de coordenadas.
//
// Orden fijo: Nota (sin título de línea) → App → Tipo →
// Medido en → Medido desde → X/Y. Cualquier línea que no
// aplique o esté vacía se omite entera (no queda un renglón
// en blanco) — Medido en/Medido desde solo tienen sentido
// para ubicación Ventana, y dentro de esa, Medido desde solo
// en modo Píxeles (mismas condiciones que ya usaba
// textoExtraTeclaMouse para el tooltip).
// ======================================================

function crearLineaResumen(titulo: string, valor: string): HTMLElement {
  const linea = document.createElement("div");

  linea.className = "popup-coordenada-resumen-linea";

  const spanTitulo = document.createElement("span");
  spanTitulo.className = "popup-coordenada-resumen-titulo";
  spanTitulo.textContent = `${titulo}:`;

  const spanValor = document.createElement("span");
  spanValor.className = "popup-coordenada-resumen-valor";
  spanValor.textContent = valor;

  linea.append(spanTitulo, spanValor);

  return linea;
}

function crearBoxResumenCoordenada(
  coordenada: FilaPerfil["coordenada"],
): HTMLElement {
  const box = document.createElement("div");

  box.className = "popup-caja-interna popup-coordenada-resumen";

  if (coordenada.nota) {
    const nota = document.createElement("div");
    nota.className = "popup-coordenada-resumen-nota";
    nota.textContent = coordenada.nota;
    box.append(nota);
  }

  if (coordenada.aplicacion) {
    box.append(crearLineaResumen("App", coordenada.aplicacion));
  }

  box.append(
    crearLineaResumen("Tipo", textoUbicacionCoordenada(coordenada.ubicacion)),
  );

  if (coordenada.ubicacion === "relativa_ventana") {
    box.append(
      crearLineaResumen(
        "Medido en",
        textoModoVentanaCoordenada(coordenada.modoVentana),
      ),
    );

    if (coordenada.modoVentana === "pixeles") {
      box.append(
        crearLineaResumen(
          "Medido desde",
          textoPuntoReferencia(coordenada.puntoReferencia),
        ),
      );
    }
  }

  const filaXY = document.createElement("div");
  filaXY.className = "popup-coordenada-resumen-linea";
  filaXY.append(
    crearLineaResumenXY("X", coordenada.x),
    crearLineaResumenXY("Y", coordenada.y),
  );
  box.append(filaXY);

  return box;
}

function crearLineaResumenXY(
  eje: "X" | "Y",
  valor: number | null,
): HTMLElement {
  const grupo = document.createElement("span");
  grupo.className = "popup-coordenada-resumen-xy";

  const spanTitulo = document.createElement("span");
  spanTitulo.className = "popup-coordenada-resumen-titulo";
  spanTitulo.textContent = `${eje}:`;

  const spanValor = document.createElement("span");
  spanValor.className = "popup-coordenada-resumen-valor";
  spanValor.textContent = valor === null ? "—" : String(valor);

  grupo.append(spanTitulo, spanValor);

  return grupo;
}

// ======================================================
// 📌 INICIAR SELECCIÓN
// ------------------------------------------------------
// Abre la ventana "Coordenadas guardadas" (comandos.rs,
// abrir_ventana_coordenadas — catálogo filtrable + "Nueva
// coordenada", que internamente reusa el flujo de captura
// existente) y sondea la coordenada elegida/creada ahí. Si
// el usuario cierra esa ventana sin elegir nada,
// obtener_seleccion_coordenada_banco nunca deja de devolver
// null — este polling queda huérfano hasta que se abra una
// fila/página nueva. Es aceptable: no hace nada, solo un
// invoke liviano cada 200ms.
// ======================================================

function iniciarSeleccion(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  evento: MouseEvent,
  alModificar: () => void,
): void {
  invoke("abrir_ventana_coordenadas").catch((error) => {
    // Antes esto se tragaba en silencio — si .build() falla del lado
    // de Rust (comandos.rs), era invisible. Ahora queda en la consola
    // de devtools de ESTA ventana (F12 en la ventana principal).
    console.error("abrir_ventana_coordenadas FALLÓ:", error);
  });

  const intervalo = setInterval(() => {
    invoke<CoordenadaBancoJson | null>("obtener_seleccion_coordenada_banco")
      .then((resultado) => {
        if (!resultado) {
          return;
        }

        clearInterval(intervalo);

        const elegida = coordenadaBancoAPerfil(
          convertirCoordenadaBanco(resultado),
        );

        Object.assign(filaPerfil.coordenada, elegida);

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

  popup.dataset.ayudaId = "popup-extra-tecla-mouse";

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
  // COORDENADA — subtítulo + botón Seleccionar/Cambiar (+ Probar
  // si Absoluta) + box informativo (si ya hay una elegida)
  // ------------------------------------------------------
  // "En relación a"/"Medido en"/"Medido desde" ya no viven acá —
  // son responsabilidad exclusiva del gestor de coordenadas (ver
  // vent_coordenadas_main.ts). Este popup solo elige CUÁL
  // coordenada del banco usar y muestra su resumen; para cambiar
  // esos valores hay que editarla desde el gestor.
  // ----------------------------------

  const tieneCoordenada = coordenada.x !== null && coordenada.y !== null;

  const labelCoordenada = document.createElement("span");
  labelCoordenada.className = "popup-fila-label";
  labelCoordenada.textContent = "Coordenada:";
  popup.append(labelCoordenada);

  const filaSeleccionar = document.createElement("div");
  filaSeleccionar.className = "popup-extra-fila-acciones";

  const botonSeleccionar = document.createElement("button");
  botonSeleccionar.className = "ui-btn popup-extra-capturar";
  botonSeleccionar.textContent = tieneCoordenada
    ? "📌 Cambiar"
    : "📌 Seleccionar";
  botonSeleccionar.addEventListener("click", () => {
    iniciarSeleccion(contexto, filaPerfil, evento, alModificar);
  });

  filaSeleccionar.append(botonSeleccionar);

  // Probar solo tiene sentido para Absoluta: Cursor y Ventana
  // siempre resuelven contra ESTA máquina/ventana en este momento
  // (el cursor está donde está, la ventana activa somos nosotros
  // mismos editando el perfil), así que "probar" ahí no ejercita
  // nada útil — a diferencia del botón ▶ del gestor, que si aplica
  // a cualquier tipo porque ahí se prueba la coordenada aislada,
  // no en el contexto de un remap real.
  if (tieneCoordenada && coordenada.ubicacion === "absoluta") {
    const botonProbar = document.createElement("button");
    botonProbar.className = "ui-btn popup-extra-probar";
    botonProbar.textContent = "▶ Probar";
    botonProbar.title = "Mover mouse a coordenada";
    botonProbar.addEventListener("click", () => {
      probarCoordenada(coordenada);
    });

    filaSeleccionar.append(botonProbar);
  }

  // Eliminar (✕ ghost, pegado al borde derecho vía margin-left:auto
  // en CSS) — solo limpia x/y de ESTA fila, no toca la coordenada
  // en el gestor (Usuario/Coordenadas.tsv sigue intacto). Con
  // coordenada.activa true pero x/y en null queda exactamente en el
  // mismo estado que "switch apagado" (ver convertir_coordenada en
  // compilador.rs) — no hace falta apagar el switch acá.
  if (tieneCoordenada) {
    const botonEliminar = document.createElement("button");
    botonEliminar.className = "ui-btn popup-extra-eliminar";
    botonEliminar.textContent = "✕";
    botonEliminar.title = "Eliminar coordenada";
    botonEliminar.addEventListener("click", () => {
      coordenada.x = null;
      coordenada.y = null;
      coordenada.nota = "";
      coordenada.aplicacion = "";

      cerrarVentanaCapturaCoordenada();
      reconstruirFila(contexto.id);
      alModificar();
      redibujar();
    });

    filaSeleccionar.append(botonEliminar);
  }

  popup.append(filaSeleccionar);

  if (tieneCoordenada) {
    popup.append(crearBoxResumenCoordenada(coordenada));
  }

  popup.append(crearSeparador());

  // ----------------------------------
  // FINALIZAR EN — Posición Inicial / Posición Final
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

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
