// ======================================================
// 🧩📝 comp_Popup_Macro_Editor
// ------------------------------------------------------
// Editor completo de una Macro (popup Tipo/Acción/Extra por
// paso), abierto desde el popup Acción de una fila tipo ===
// "macro" (ver comp_popup_macro_accion.ts). Distinto de ese
// archivo (que solo decide A CUÁL macro apunta la fila) — acá
// se editan los PASOS de la macro ya elegida
// (filaPerfil.accionReferencia).
//
// Ciclo de vida vía CACHE (ver macro_cache.rs): al abrir se
// trabaja siempre sobre una copia en cache, nunca directo
// sobre el archivo de usuario. Los cambios "en vivo" del
// editor (agregar/mover/editar pasos) se escriben con
// debounce corto en la cache (macro_guardar_paso, ver más
// abajo) — el archivo de usuario solo se reescribe al hacer
// click en "Guardar" (macro_guardar, promueve cache → disco).
// "Cancelar" descarta la cache sin tocar el disco
// (macro_cancelar). "Guardar como" clona la cache actual a un
// archivo de usuario nuevo (macro_guardar_como).
//
// El popup se monta con mostrarPopupFijo() (no mostrarPopup):
// no se cierra con click afuera — solo con Cancelar/Guardar,
// que son las únicas acciones que definen qué pasa con la
// cache. Es arrastrable mediante una barra de título propia
// (ver crearBarraTitulo).
//
// mostrarPopupFijo() reemplaza TODA la capa global de popups,
// así que este editor no puede abrir sub-popups anidados sin
// destruirse a sí mismo (perdería el estado de arrastre, los
// pasos expandidos, etc.). Por eso todo despliegue de opciones
// por paso (elegir Tipo, capturar tecla, elegir comando
// multimedia, elegir ubicación de coordenada...) se resuelve
// EXPANDIENDO la fila del paso hacia abajo, dentro del mismo
// árbol — mismo criterio que abrirConExpandido en
// comp_popup_abrir_extra.ts, aplicado a cada paso.
//
// El array PasoMacro no tiene id propio (ver core_macro.ts) —
// para el componente de arrastre (que necesita ids string
// estables) y para reconciliar la captura de teclas asincrónica
// por paso, este editor les asigna un id sintético en memoria
// (idsPasos, un WeakMap-like por índice reconstruido en cada
// dibujado) que NUNCA se persiste — ver asignarIdsPasos().
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopupFijo, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import type {
  MacroArchivo,
  PasoMacro,
  TipoPasoMacro,
  ExtraTeclaMouseMacro,
  UbicacionPasoMacro,
  ModoVentanaPasoMacro,
  PuntoReferenciaPasoMacro,
  IniciarPasoMacro,
  InstanciasPasoMacro,
  ComandoPasoMacro,
  AlcancePasoMacro,
} from "../../core/core_macro";

import {
  crearPasoMacro,
  clonarPasoMacro,
  textoTipoPasoMacro,
  iconoTipoPasoMacro,
} from "../../core/core_macro";

import type { Trigger } from "../../core/core_trigger";
import { triggerATexto, triggerAHTML } from "../../core/core_trigger";

import {
  COMANDOS_VOLUMEN,
  COMANDO_SILENCIAR,
  COMANDOS_REPRODUCCION_PRINCIPAL,
  COMANDOS_REPRODUCCION_PISTA,
  esComandoDeVolumen,
} from "../../core/core_multimedia";
import type { OpcionMultimedia } from "../../core/core_multimedia";

import {
  esRutaExe,
  nombreDeRuta,
  extensionDeRuta,
} from "../../core/core_abrir";

import {
  crearGrupoOpciones,
  crearFilaPopup,
  crearInterruptor,
} from "./comp_popup_grupo";

import { crearControladorArrastre } from "../util/util_arrastrable";
import type { ControladorArrastre } from "../util/util_arrastrable";

// ======================================================
// 📦 MODELOS BACKEND (ícono + "abrir con", mismo shape que
// usan comp_popup_abrir_accion.ts / comp_popup_abrir_con.ts)
// ======================================================

interface IconoJson {
  ancho: number;
  alto: number;
  pixeles: string;
}

interface ProgramaJson {
  nombre: string;
  ruta: string;
}

// ======================================================
// 🆔 IDS SINTÉTICOS POR PASO (solo en memoria, no persisten)
// ------------------------------------------------------
// Un WeakMap indexado por la REFERENCIA del objeto PasoMacro
// (no por posición, que cambia al reordenar) — mientras el
// popup siga abierto, el mismo objeto in-memory conserva su
// id entre redibujados (reordenar no crea objetos nuevos,
// solo reordena el array). Se resetea (Map nuevo) cada vez
// que se abre el editor desde cero.
// ======================================================

let idsPasosActual: WeakMap<PasoMacro, string> | null = null;
let contadorIdPaso = 0;

function idDePaso(paso: PasoMacro): string {
  if (!idsPasosActual) {
    idsPasosActual = new WeakMap();
  }

  let id = idsPasosActual.get(paso);

  if (!id) {
    id = `paso_${contadorIdPaso++}`;

    idsPasosActual.set(paso, id);
  }

  return id;
}

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 💾 GUARDADO EN CACHE (con debounce)
// ------------------------------------------------------
// macro_guardar_paso escribe SOLO en la copia de cache
// (ver macro_cache.rs / macros::guardar_desde_cache) — el
// archivo de usuario en disco no se toca hasta que el botón
// "Guardar" del popup invoque macro_guardar. Debounce corto
// para no escribir en cada tecla de un input de texto
// (nombre, argumento, ruta de Pegar); guardarAhora() fuerza
// el guardado inmediato para cambios "discretos" (elegir una
// opción de un grupo, no un input de texto en vivo).
// ======================================================

const DEBOUNCE_GUARDADO_MS = 400;

let timerGuardado: ReturnType<typeof setTimeout> | null = null;

function cancelarDebounceGuardado(): void {
  if (timerGuardado) {
    clearTimeout(timerGuardado);

    timerGuardado = null;
  }
}

async function guardarAhora(macroArchivo: MacroArchivo): Promise<void> {
  cancelarDebounceGuardado();

  try {
    await invoke("macro_guardar_paso", { macroArchivo });
  } catch (error) {
    console.error("❌ No se pudo guardar la macro en cache:", error);
  }
}

function guardarConDebounce(macroArchivo: MacroArchivo): void {
  cancelarDebounceGuardado();

  timerGuardado = setTimeout(() => {
    timerGuardado = null;

    guardarAhora(macroArchivo);
  }, DEBOUNCE_GUARDADO_MS);
}

// ======================================================
// ⌨️ CAPTURA DE TECLA POR PASO
// ------------------------------------------------------
// Mismo mecanismo de polling que comp_capturador.ts
// (iniciar_captura / obtener_captura), con un filaId
// SINTÉTICO ("macro:<idPaso>") en vez del id de una fila real
// del perfil — perfil_ui.rs::iniciar_captura solo usa fila_id
// como texto de reconciliación del polling, no valida que
// exista una fila con ese id (ver perfil_ui.rs). columna
// siempre "Accion" (no hay Trigger dentro de un paso de Macro,
// solo la tecla a simular).
// ======================================================

function capturarTeclaPaso(
  idPaso: string,
  alCapturar: (trigger: Trigger) => void,
  alCancelar: () => void,
): void {
  invoke("iniciar_captura", {
    filaId: `macro:${idPaso}`,
    columna: "Accion",
  });

  let capturando = true;

  const esperar = async (): Promise<void> => {
    while (capturando) {
      const capturado = await invoke<[string, string, unknown | null] | null>(
        "obtener_captura",
      );

      if (capturado) {
        const [filaId, columna, trigger] = capturado;

        if (filaId !== `macro:${idPaso}` || columna !== "Accion") {
          await new Promise((resolver) => setTimeout(resolver, 50));

          continue;
        }

        capturando = false;

        if (trigger === null) {
          alCancelar();

          return;
        }

        alCapturar(trigger as Trigger);

        return;
      }

      await new Promise((resolver) => setTimeout(resolver, 50));
    }
  };

  esperar();
}

// ======================================================
// 🎨 ÍCONO — FALLBACK Y REAL (mismo patrón que
// comp_popup_abrir_accion.ts / comp_popup_abrir_con.ts)
// ======================================================

function crearIconoFallback(emoji: string): HTMLElement {
  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = emoji;

  return icono;
}

function crearIconoDesdeJson(datos: IconoJson): HTMLElement {
  const canvas = document.createElement("canvas");

  canvas.width = datos.ancho;

  canvas.height = datos.alto;

  const contexto = canvas.getContext("2d");

  if (!contexto) {
    return crearIconoFallback("📂");
  }

  const pixeles = Uint8ClampedArray.from(atob(datos.pixeles), (caracter) =>
    caracter.charCodeAt(0),
  );

  contexto.putImageData(new ImageData(pixeles, datos.ancho, datos.alto), 0, 0);

  canvas.className = "app-icono";

  return canvas;
}

// ======================================================
// 🔢 CAMPO NUMÉRICO (commit al blur/Enter — mismo patrón que
// crearCampoLimite en comp_popup_portapapeles_extra.ts)
// ======================================================

function crearCampoNumero(
  valorActual: number,
  minimo: number,
  onCambiar: (nuevoValor: number) => void,
): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "text";
  input.className = "popup-input popup-input-numero";
  input.value = String(valorActual);

  const confirmar = () => {
    const valor = parseInt(input.value, 10);

    onCambiar(Number.isFinite(valor) && valor >= minimo ? valor : minimo);
  };

  input.addEventListener("blur", confirmar);

  input.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      input.blur();
    }
  });

  return input;
}

// ======================================================
// 📝 TEXTO RESUMEN DE ACCIÓN (columna Acción de cada fila,
// cerrada — antes de expandir el paso)
// ======================================================

function textoAccionPaso(paso: PasoMacro): string {
  switch (paso.tipo) {
    case "tecla_mouse":
      return paso.teclaAccion.gatillo
        ? triggerATexto(paso.teclaAccion)
        : "Sin capturar";

    case "espera":
      return `${paso.esperaMs} ms`;

    case "bucle":
      return paso.bucleMarcadorDestino
        ? `Volver a ${paso.bucleMarcadorDestino} ×${paso.bucleVeces}`
        : "Sin destino";

    case "coordenada":
      if (paso.coordPosicionInicial) {
        return "Posición inicial";
      }

      return paso.coordX !== null && paso.coordY !== null
        ? `X: ${paso.coordX}, Y: ${paso.coordY}`
        : "Sin capturar";

    case "pegar":
      return paso.pegarRuta ? nombreDeRuta(paso.pegarRuta) : "Sin ruta";

    case "abrir":
      return paso.abrirRuta ? nombreDeRuta(paso.abrirRuta) : "Seleccionar...";

    case "multimedia": {
      const TODOS: OpcionMultimedia[] = [
        ...COMANDOS_VOLUMEN,
        COMANDO_SILENCIAR,
        ...COMANDOS_REPRODUCCION_PRINCIPAL,
        ...COMANDOS_REPRODUCCION_PISTA,
      ];

      const opcion = TODOS.find(
        (item) => item.valor === paso.multimediaComando,
      );

      return opcion ? `${opcion.icono} ${opcion.texto}` : "Sin comando";
    }
  }
}

// ======================================================
// 🧩📝 ABRIR EDITOR DE MACRO
// ======================================================

export function abrirEditorMacro(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const nombreMacro = filaPerfil.accionReferencia;

  if (!nombreMacro) {
    return;
  }

  invoke<MacroArchivo>("macro_abrir", { nombre: nombreMacro })
    .then((macroArchivo) => {
      idsPasosActual = new WeakMap();

      montarEditor(evento, contexto, macroArchivo, filaPerfil);
    })
    .catch((error) => {
      console.error("❌ No se pudo abrir la macro:", error);
    });
}

// ======================================================
// 🏗️ MONTAJE DEL EDITOR (estado vivo + primer dibujado)
// ======================================================

function montarEditor(
  evento: MouseEvent,
  contexto: ContextoFila,
  macroArchivoInicial: MacroArchivo,
  filaPerfil: FilaPerfil,
): void {
  const programaFiltroApp = filaPerfil.app.programa;

  // El nombre viaja en el propio macroArchivo, pero además se guarda
  // acá aparte: es la CLAVE de cache (macro_cache::CACHE_MACROS), la
  // que hay que usar en macro_guardar_paso/macro_guardar/macro_cancelar
  // incluso si el usuario renombra en el medio (ver alConfirmarRenombrar).
  let nombreCache = macroArchivoInicial.nombre;

  let macroArchivo = macroArchivoInicial;

  // Posición del popup — se fija en la apertura y la mueve el
  // arrastre de la barra de título (crearBarraTitulo). Un redibujado
  // (dibujar()) NO debe volver a centrarlo en el mouse.
  let posicionX = evento.clientX;
  let posicionY = evento.clientY;

  // Índice del paso actualmente expandido (mostrando su Acción/Extra
  // en detalle) — null si ninguno. Puramente visual, no se guarda.
  // Se guarda el ID sintético (no el índice) para sobrevivir a un
  // reordenamiento por arrastre mientras sigue expandido.
  let idPasoExpandido: string | null = null;

  // Menú de opciones (Mover/Eliminar/Duplicar) del botón ⋮ de un
  // paso — igual criterio que idPasoExpandido, solo un menú abierto
  // a la vez.
  let idMenuAbierto: string | null = null;

  // Formulario inline de renombrar (botón [...] de la barra) — solo
  // uno puede estar abierto a la vez, y reemplaza el nombre de la
  // barra mientras dure.
  let renombrando = false;

  let controladorArrastre: ControladorArrastre | null = null;

  const redibujar = (): void => {
    dibujar();
  };

  const guardarYRedibujar = (): void => {
    guardarConDebounce(macroArchivo);
    redibujar();
  };

  // Cambios "en vivo" (inputs de texto) guardan con debounce SIN
  // redibujar todo el popup (perdería el foco) — mismo criterio que
  // inputNombre en comp_popup_menu_express_editor.ts.
  const guardarSinRedibujar = (): void => {
    guardarConDebounce(macroArchivo);
  };

  // ----------------------------------
  // 🖱️ ARRASTRE DE LA BARRA DE TITULO
  // ------------------------------------------------------
  // mousedown en la barra → mousemove reposiciona el popup dentro de
  // la ventana → mouseup libera. No usa crearControladorArrastre (ese
  // es para reordenar filas, no para mover ventanas) — patrón simple
  // propio, análogo al de comp_popup_col_resizer más adelante.
  // ----------------------------------

  function iniciarArrastrePopup(
    eventoInicial: MouseEvent,
    contenedorPopup: HTMLElement,
  ): void {
    eventoInicial.preventDefault();

    // La posición real en pantalla puede no coincidir con
    // posicionX/posicionY: si el popup no entraba en la ventana al
    // abrirse, mostrarPopupFijo (vía ajustarPosicionDentroDeVentana)
    // lo reubicó usando right/bottom en vez de left/top, sin
    // actualizar estas variables. Usar el valor viejo acá producía el
    // salto en el primer arrastre — se recalcula desde la posición
    // real de pantalla en cada inicio de arrastre.
    const rect = contenedorPopup.getBoundingClientRect();

    posicionX = rect.left;
    posicionY = rect.top;

    const offsetX = eventoInicial.clientX - posicionX;
    const offsetY = eventoInicial.clientY - posicionY;

    const alMover = (eventoMover: MouseEvent): void => {
      posicionX = eventoMover.clientX - offsetX;
      posicionY = eventoMover.clientY - offsetY;

      contenedorPopup.style.left = `${posicionX}px`;
      contenedorPopup.style.top = `${posicionY}px`;
      contenedorPopup.style.right = "";
      contenedorPopup.style.bottom = "";
    };

    const alSoltar = (): void => {
      document.removeEventListener("mousemove", alMover);
      document.removeEventListener("mouseup", alSoltar);
    };

    document.addEventListener("mousemove", alMover);
    document.addEventListener("mouseup", alSoltar);
  }

  // ----------------------------------
  // 🏷️ BARRA DE TÍTULO (arrastrable, con [...] Renombrar)
  // ----------------------------------

  function crearBarraTitulo(contenedorPopup: HTMLElement): HTMLElement {
    const barra = document.createElement("div");

    barra.className = "popup-macro-barra";

    if (renombrando) {
      const input = document.createElement("input");

      input.className = "popup-input popup-macro-barra-input";
      input.type = "text";
      input.value = nombreCache;

      const confirmarRenombre = async (): Promise<void> => {
        const nuevoNombre = input.value.trim();

        if (!nuevoNombre || nuevoNombre === nombreCache) {
          renombrando = false;

          redibujar();

          return;
        }

        try {
          const nombreFinal = await invoke<string>("macro_renombrar", {
            nombreActual: nombreCache,
            nombreNuevo: nuevoNombre,
          });

          nombreCache = nombreFinal;
          macroArchivo.nombre = nombreFinal;

          filaPerfil.accionReferencia = nombreFinal;

          reconstruirFila(contexto.id);
        } catch (error) {
          console.error("❌ No se pudo renombrar la macro:", error);
        }

        renombrando = false;

        redibujar();
      };

      input.addEventListener("keydown", (eventoTecla) => {
        if (eventoTecla.key === "Enter") {
          confirmarRenombre();
        }

        if (eventoTecla.key === "Escape") {
          renombrando = false;

          redibujar();
        }
      });

      const botonConfirmar = crearBoton({ texto: "✓", titulo: "Confirmar" });

      botonConfirmar.addEventListener("click", confirmarRenombre);

      const botonCancelarRenombre = crearBoton({
        texto: "✕",
        titulo: "Cancelar",
      });

      botonCancelarRenombre.addEventListener("click", () => {
        renombrando = false;

        redibujar();
      });

      barra.append(input, botonConfirmar, botonCancelarRenombre);

      requestAnimationFrame(() => {
        input.focus();
        input.select();
      });

      return barra;
    }

    const nombre = document.createElement("span");

    nombre.className = "popup-macro-barra-nombre";
    nombre.textContent = `🧩 ${macroArchivo.nombre}`;

    const botonRenombrar = crearBoton({
      texto: "...",
      titulo: "Renombrar",
    });

    botonRenombrar.classList.add("popup-macro-barra-boton");

    botonRenombrar.addEventListener("click", (eventoClick) => {
      eventoClick.stopPropagation();

      renombrando = true;

      redibujar();
    });

    barra.append(nombre, botonRenombrar);

    barra.addEventListener("mousedown", (eventoDown) => {
      // Ignora el mousedown que empieza en el botón [...] — ese
      // clic es para renombrar, no para arrastrar.
      if ((eventoDown.target as HTMLElement).closest("button")) {
        return;
      }

      iniciarArrastrePopup(eventoDown, contenedorPopup);
    });

    return barra;
  }

  // ----------------------------------
  // 🔚 ZONA INFERIOR (Cancelar / Guardar como / Guardar)
  // ----------------------------------

  function crearPieBotones(): HTMLElement {
    const pie = document.createElement("div");

    pie.className = "popup-macro-editor-pie";

    const botonCancelar = crearBoton({ texto: "Cancelar" });

    botonCancelar.addEventListener("click", () => {
      invoke("macro_cancelar", { nombre: nombreCache }).catch((error) => {
        console.error("❌ No se pudo cancelar la edición de la macro:", error);
      });

      cerrarEditor();
    });

    const botonGuardarComo = crearBoton({ texto: "Guardar como" });

    botonGuardarComo.addEventListener("click", () => {
      abrirFormularioGuardarComo();
    });

    const botonGuardar = crearBoton({ texto: "Guardar" });

    botonGuardar.addEventListener("click", async () => {
      cancelarDebounceGuardado();

      try {
        await invoke("macro_guardar_paso", { macroArchivo });

        await invoke("macro_guardar", { nombre: nombreCache });
      } catch (error) {
        console.error("❌ No se pudo guardar la macro:", error);

        return;
      }

      cerrarEditor();
    });

    pie.append(botonCancelar, botonGuardarComo, botonGuardar);

    return pie;
  }

  // ----------------------------------
  // ✏️ FORMULARIO "GUARDAR COMO"
  // ------------------------------------------------------
  // Mismo patrón visual que abrirFormularioNombre() en
  // comp_popup_macro_accion.ts, pero inline dentro del mismo popup
  // fijo (no puede abrir otro popup — ver nota de cabecera del
  // archivo). Al confirmar, la fila pasa a apuntar a la macro nueva
  // y el editor sigue abierto sobre ELLA (no sobre el origen).
  // ----------------------------------

  function abrirFormularioGuardarComo(): void {
    const overlay = document.createElement("div");

    overlay.className = "popup-macro-guardarcomo-overlay";

    const caja = document.createElement("div");

    caja.className = "popup-macro-guardarcomo-caja";

    const input = document.createElement("input");

    input.className = "popup-input";
    input.type = "text";
    input.value = `${macroArchivo.nombre} (copia)`;
    input.placeholder = "Nombre de la macro";

    const botones = document.createElement("div");

    botones.className = "popup-confirmar-botones";

    const botonCancelar = crearBoton({ texto: "Cancelar" });

    botonCancelar.addEventListener("click", () => {
      overlay.remove();
    });

    const botonConfirmar = crearBoton({ texto: "Guardar como" });

    const confirmar = async (): Promise<void> => {
      cancelarDebounceGuardado();

      try {
        await invoke("macro_guardar_paso", { macroArchivo });

        const resultado = await invoke<MacroArchivo>("macro_guardar_como", {
          nombreOrigen: nombreCache,
          nombreNuevo: input.value.trim() || null,
        });

        nombreCache = resultado.nombre;
        macroArchivo = resultado;

        filaPerfil.accionReferencia = resultado.nombre;

        reconstruirFila(contexto.id);
      } catch (error) {
        console.error("❌ No se pudo guardar como nueva macro:", error);

        overlay.remove();

        return;
      }

      overlay.remove();

      redibujar();
    };

    botonConfirmar.addEventListener("click", confirmar);

    input.addEventListener("keydown", (eventoTecla) => {
      if (eventoTecla.key === "Enter") {
        confirmar();
      }

      if (eventoTecla.key === "Escape") {
        overlay.remove();
      }
    });

    botones.append(botonCancelar, botonConfirmar);

    caja.append(input, botones);

    overlay.append(caja);

    document.querySelector(".popup-macro-editor")?.appendChild(overlay);

    requestAnimationFrame(() => {
      input.focus();
      input.select();
    });
  }

  // ----------------------------------
  // 🚪 CIERRE DEL EDITOR
  // ------------------------------------------------------
  // Ya NO se dispara por click afuera (el popup es fijo, ver
  // mostrarPopupFijo) — solo por Cancelar o Guardar, cada uno
  // decide antes qué pasa con la cache. Acá solo se libera el
  // componente de arrastre de filas y se oculta el popup.
  // ----------------------------------

  function cerrarEditor(): void {
    if (controladorArrastre) {
      controladorArrastre.destruir();

      controladorArrastre = null;
    }

    ocultarPopup();

    reconstruirFila(contexto.id);
  }

  function dibujar(): void {
    if (controladorArrastre) {
      controladorArrastre.destruir();

      controladorArrastre = null;
    }

    // Preservar tamaño y ancho de columna Extra ajustados por el
    // usuario antes de que mostrarPopupFijo destruya el popup actual.
    const popupPrevio = document.querySelector<HTMLElement>(
      ".popup-macro-editor",
    );
    let anchoGuardado: string | null = null;
    let altoGuardado: string | null = null;
    let colExtraGuardado: string | null = null;

    if (popupPrevio) {
      const rect = popupPrevio.getBoundingClientRect();

      anchoGuardado = `${rect.width}px`;
      altoGuardado = `${rect.height}px`;
    }

    const colDerechaPrevio = document.querySelector<HTMLElement>(
      ".popup-macro-editor-columna-derecha",
    );

    if (colDerechaPrevio) {
      colExtraGuardado =
        colDerechaPrevio.style.getPropertyValue("--col-extra-width");
    }

    const popup = document.createElement("div");

    popup.className = "popup-extra popup-macro-editor";

    // hayBucle: si existe al menos un Bucle en la macro, determina
    // si la columna Marcador existe/se reserva en TODAS las filas
    // (spec: "la columna solo existe cuando hay al menos un paso
    // Bucle"). hayBucleDespuesDe(indice): elegibilidad puntual de
    // CADA paso para tomar una letra nueva — solo los anteriores a
    // algún Bucle pueden hacerlo (spec sección 3) — pasada por fila
    // a crearFilaPaso/crearControlMarcador más abajo.
    const hayBucle = macroArchivo.pasos.some((paso) => paso.tipo === "bucle");

    const hayBucleDespuesDe = (indice: number): boolean =>
      macroArchivo.pasos
        .slice(indice + 1)
        .some((paso) => paso.tipo === "bucle");

    // ----------------------------------
    // 🏷️ BARRA DE TÍTULO ARRASTRABLE
    // ----------------------------------

    popup.append(crearBarraTitulo(popup));

    // ----------------------------------
    // 🧱 CUERPO EN DOS COLUMNAS
    // ------------------------------------------------------
    // Izquierda: panel fijo "Funciones" (los 7 tipos de paso, ver
    // crearPanelFunciones). Derecha: lista de pasos. La barra de
    // título y el pie de botones quedan FUERA de este bloque, como
    // franjas horizontales completas arriba/abajo de las columnas.
    // ----------------------------------

    const cuerpo = document.createElement("div");

    cuerpo.className = "popup-macro-editor-cuerpo";

    cuerpo.append(crearPanelFunciones(macroArchivo, guardarYRedibujar));

    // ----------------------------------
    // 📋 LISTA DE PASOS
    // ----------------------------------

    const columnaDerecha = document.createElement("div");

    columnaDerecha.className = "popup-macro-editor-columna-derecha";

    // Anchos iniciales de columna como variables CSS en el contenedor
    // del editor (spec F3) — los resizers del encabezado (F2) y las
    // celdas de cada fila de paso (Etapa 7) leen las mismas variables,
    // así quedan sincronizados sin duplicar el ancho en dos lugares.
    columnaDerecha.style.setProperty("--col-asa-width", "28px");
    columnaDerecha.style.setProperty("--col-numero-width", "32px");
    columnaDerecha.style.setProperty("--col-tipo-width", "40px");
    columnaDerecha.style.setProperty(
      "--col-extra-width",
      colExtraGuardado ?? "260px",
    );

    columnaDerecha.append(crearEncabezadoColumnas(columnaDerecha));

    const lista = document.createElement("div");

    lista.className = "popup-macro-editor-lista";

    if (macroArchivo.pasos.length === 0) {
      const vacio = document.createElement("span");

      vacio.className = "app-popup-lista-titulo";

      vacio.textContent = "Todavía no agregaste ningún paso";

      lista.append(vacio);
    }

    macroArchivo.pasos.forEach((paso, indice) => {
      lista.append(
        crearFilaPaso(
          paso,
          indice,
          macroArchivo,
          hayBucle,
          hayBucleDespuesDe(indice),
          programaFiltroApp,
          idPasoExpandido,
          idMenuAbierto,
          (nuevoId) => {
            idPasoExpandido = idPasoExpandido === nuevoId ? null : nuevoId;
            idMenuAbierto = null;

            redibujar();
          },
          (nuevoId) => {
            idMenuAbierto = idMenuAbierto === nuevoId ? null : nuevoId;

            redibujar();
          },
          guardarYRedibujar,
          guardarSinRedibujar,
          redibujar,
          (idPasoAMover) => {
            idMenuAbierto = null;

            // El controlador se reasigna en cada dibujar() — se lee
            // en el momento del click (no se captura antes), porque
            // para entonces ya está montado y registrado más abajo.
            controladorArrastre?.activarModoMoverPara(idPasoAMover);
          },
        ),
      );
    });

    columnaDerecha.append(lista);

    cuerpo.append(columnaDerecha);

    popup.append(cuerpo);

    // ----------------------------------
    // 🔚 CANCELAR / GUARDAR COMO / GUARDAR
    // ----------------------------------

    popup.append(crearSeparador());

    popup.append(crearPieBotones());

    mostrarPopupFijo(popup, posicionX, posicionY);

    // Restaurar tamaño ajustado por el usuario (resize: both).
    // Se aplica después de mostrarPopupFijo porque antes el elemento
    // no está en el DOM y las asignaciones de style se perderían.
    if (anchoGuardado) {
      popup.style.width = anchoGuardado;
    }

    if (altoGuardado) {
      popup.style.height = altoGuardado;
    }

    // El componente de arrastre necesita el contenedor YA en el DOM
    // (mostrarPopupFijo ya lo insertó arriba) para poder registrar cada
    // fila-paso y medir sus posiciones.
    controladorArrastre = crearControladorArrastre({
      contenedor: lista,
      obtenerOrdenIds: () => macroArchivo.pasos.map((paso) => idDePaso(paso)),
      onReordenar: (nuevoOrden) => {
        const porId = new Map(
          macroArchivo.pasos.map((paso) => [idDePaso(paso), paso]),
        );

        macroArchivo.pasos = nuevoOrden
          .map((id) => porId.get(id))
          .filter((paso): paso is PasoMacro => !!paso);

        guardarConDebounce(macroArchivo);

        // aplicarNuevoOrden (util_arrastrable.ts) ya reordenó
        // físicamente los nodos en `lista` antes de llamar a este
        // callback — solo falta refrescar el texto de cada número
        // para que la numeración quede secuencial tras soltar (spec
        // punto 11: el # es puramente visual/derivado, no debe poder
        // quedar desordenado). Un redibujar() completo acá cortaría
        // de raíz la animación de acomodo que arranca justo después
        // de este callback, así que se ajusta solo el texto.
        nuevoOrden.forEach((id, indice) => {
          const fila = lista.querySelector<HTMLElement>(
            `[data-paso-id="${id}"]`,
          );

          const numero = fila?.querySelector<HTMLElement>(
            ".popup-macro-editor-numero",
          );

          if (numero) {
            numero.textContent = `#${indice + 1}`;
          }
        });
      },
      onSalirModoMover: () => {
        // No hace falta redibujar acá — salirModoMover ya limpió las
        // clases de selección directamente sobre el DOM existente.
      },
    });

    lista.querySelectorAll<HTMLElement>("[data-paso-id]").forEach((fila) => {
      const idPaso = fila.dataset.pasoId!;

      const asa = fila.querySelector<HTMLElement>(".popup-macro-editor-asa");

      if (asa) {
        controladorArrastre!.registrarFila(idPaso, fila, asa);
      }
    });

    // Cerrar menú/lista-tipo abierto al hacer click fuera de él.
    // Se registra con setTimeout para no dispararse en el mismo click
    // que abrió el menú. Se usa capture para interceptar antes que
    // los botones internos (que tienen stopPropagation).
    if (idMenuAbierto) {
      const cerrarAlClickFuera = (evento: MouseEvent): void => {
        const menuAbierto = popup.querySelector<HTMLElement>(
          ".popup-lista, .popup-macro-editor-menu-asa",
        );

        if (menuAbierto && !menuAbierto.contains(evento.target as Node)) {
          document.removeEventListener("click", cerrarAlClickFuera, true);

          idMenuAbierto = null;

          redibujar();
        }
      };

      setTimeout(() => {
        document.addEventListener("click", cerrarAlClickFuera, true);
      }, 0);
    }
  }

  dibujar();
}

// ======================================================
// 📐 ENCABEZADO DE COLUMNAS (# · ⁝ · Tipo · Extra · Nota)
// ------------------------------------------------------
// Fila fija arriba de la lista de pasos (spec punto 10). Único
// separador arrastrable: entre Extra y Nota (spec: "las demás
// columnas tendrán ancho fijo", "la flexible será la columna
// Nota"). Arrastrarlo ajusta --col-extra-width — Nota (flex:1
// en CSS) se acomoda sola al espacio que sobra, sin variable
// propia.
// ======================================================

const COLUMNAS_ENCABEZADO: { nombre: string; etiqueta: string }[] = [
  { nombre: "numero", etiqueta: "#" },
  { nombre: "asa", etiqueta: "⁝" },
  { nombre: "tipo", etiqueta: "Tipo" },
  { nombre: "extra", etiqueta: "Extra" },
  { nombre: "nota", etiqueta: "Nota" },
];

function crearEncabezadoColumnas(columnaDerecha: HTMLElement): HTMLElement {
  const encabezado = document.createElement("div");

  encabezado.className = "popup-macro-editor-header";

  COLUMNAS_ENCABEZADO.forEach((columna) => {
    const celda = document.createElement("div");

    celda.className = "popup-macro-col-header";
    celda.dataset.columna = columna.nombre;
    celda.textContent = columna.etiqueta;

    encabezado.append(celda);

    // Único separador visible: entre Extra y Nota (spec punto 7).
    // El resto de las columnas tiene ancho fijo, sin resizer.
    if (columna.nombre === "extra") {
      encabezado.append(crearResizerColumna(columnaDerecha));
    }
  });

  return encabezado;
}

function crearResizerColumna(columnaDerecha: HTMLElement): HTMLElement {
  const resizer = document.createElement("div");

  resizer.className = "popup-macro-col-resizer";

  const variable = "--col-extra-width";

  resizer.addEventListener("mousedown", (eventoInicial) => {
    eventoInicial.preventDefault();

    const anchoInicial = parseFloat(
      getComputedStyle(columnaDerecha).getPropertyValue(variable),
    );

    const xInicial = eventoInicial.clientX;

    const alMover = (eventoMover: MouseEvent): void => {
      // El resizer queda a la derecha de Extra: arrastrar hacia la
      // derecha amplía Extra, hacia la izquierda la encoge.
      const delta = eventoMover.clientX - xInicial;

      const nuevoAncho = Math.max(20, anchoInicial + delta);

      columnaDerecha.style.setProperty(variable, `${nuevoAncho}px`);
    };

    const alSoltar = (): void => {
      document.removeEventListener("mousemove", alMover);
      document.removeEventListener("mouseup", alSoltar);
    };

    document.addEventListener("mousemove", alMover);
    document.addEventListener("mouseup", alSoltar);
  });

  return resizer;
}

// ======================================================
// 📄 FILA DE UN PASO (cerrada o expandida)
// ======================================================

function crearFilaPaso(
  paso: PasoMacro,
  indice: number,
  macroArchivo: MacroArchivo,
  hayBucle: boolean,
  elegiblePorMarcador: boolean,
  programaFiltroApp: string | null,
  idPasoExpandido: string | null,
  idMenuAbierto: string | null,
  alternarExpandido: (idPaso: string) => void,
  alternarMenu: (idPaso: string) => void,
  guardarYRedibujar: () => void,
  guardarSinRedibujar: () => void,
  redibujar: () => void,
  activarModoMover: (idPaso: string) => void,
): HTMLElement {
  const idPaso = idDePaso(paso);

  const contenedor = document.createElement("div");

  contenedor.className = "popup-macro-editor-paso";
  contenedor.dataset.pasoId = idPaso;

  // ----------------------------------
  // FILA PRINCIPAL (#, ⁝, Tipo, Extra [marcador + acción], Nota)
  // ----------------------------------

  const filaPrincipal = document.createElement("div");

  filaPrincipal.className = "popup-macro-editor-paso-fila";

  // # — fuera del elemento arrastrable (mismo patrón que el carril
  // de números de la tabla principal en ui_tabla.ts, spec punto
  // G5/11): puramente visual, no se guarda ni se selecciona/edita,
  // y no se mueve junto con el resto de la fila al arrastrarla. Va
  // primero, a la izquierda de ⁝ (spec bug 6).
  const numero = document.createElement("span");

  numero.className = "popup-macro-editor-numero";
  numero.textContent = `#${indice + 1}`;

  filaPrincipal.append(numero);

  // ⁝ Asa — reusa el mismo botón de opciones de fila de ventana
  // principal (mismo comportamiento de arrastre, con su marcador de
  // 6 puntos): clic corto abre el popup Mover/Duplicar/Eliminar
  // (ver crearMenuAsa), clic mantenido lo maneja util_arrastrable.ts
  // directamente sobre este mismo botón.
  const asa = document.createElement("button");

  asa.className = "ui-btn popup-macro-editor-asa";
  asa.textContent = "⁝";
  asa.title = "Opciones";

  asa.addEventListener("click", () => {
    alternarMenu(idPaso);
  });

  filaPrincipal.append(asa);

  // Tipo — SOLO ÍCONO (spec punto 11/G3). Clic despliega la lista
  // vertical de los 7 tipos in-place (mismo look que abrirPopupTipo
  // de ventana principal — comp_popup_abrir.ts — pero expandida
  // dentro de la propia fila, porque el editor de Macro no puede
  // abrir un popup anidado sin destruirse — ver nota de cabecera).
  const tipoAbierto = idMenuAbierto === `tipo:${idPaso}`;

  const botonTipo = document.createElement("button");

  botonTipo.className = "ui-btn popup-macro-editor-tipo";
  botonTipo.textContent = iconoTipoPasoMacro(paso.tipo);
  botonTipo.title = textoTipoPasoMacro(paso.tipo);

  const expandido = idPasoExpandido === idPaso;

  botonTipo.addEventListener("click", (eventoClick) => {
    eventoClick.stopPropagation();

    alternarMenu(`tipo:${idPaso}`);
  });

  filaPrincipal.append(botonTipo);

  // Extra — sin el listado de Tipos (spec punto 11): el resumen de
  // una línea del paso (cerrado) o "editando" mientras está
  // expandido (el detalle real va debajo). La columna Marcador
  // (asignar/quitar letra de Bucle) vive al principio de esta
  // celda cuando corresponde — no tiene columna propia en el
  // encabezado ⁝/#/Tipo/Extra/Nota.
  const extra = document.createElement("div");

  extra.className = "popup-macro-editor-extra";

  // Columna Marcador — tres casos:
  // 1. Paso Bucle: muestra su propia letra (bucleMarcadorDestino) con
  //    estilo invertido (borde azul, fondo transparente, letra azul).
  // 2. Paso no-Bucle anterior a algún Bucle: muestra círculo ciclable
  //    si hay letras de Bucle sin fila asignada, u oculta cuando todas
  //    las letras ya tienen su fila. Si ya tiene marcador, lo muestra.
  // 3. Resto con hayBucle: espacio reservado para alinear columnas.
  if (hayBucle && paso.tipo === "bucle") {
    extra.append(crearIconoBucle(paso));
  } else if (hayBucle && paso.tipo !== "bucle" && elegiblePorMarcador) {
    const letrasNecesitadas = calcularLetrasNecesitadas(macroArchivo.pasos);
    const todasCubiertas = letrasNecesitadas.length === 0;

    if (paso.marcador || !todasCubiertas) {
      extra.append(
        crearControlMarcador(
          paso,
          macroArchivo,
          letrasNecesitadas,
          guardarYRedibujar,
        ),
      );
    } else {
      const espacio = document.createElement("span");

      espacio.className = "popup-macro-editor-marcador-espacio";

      extra.append(espacio);
    }
  } else if (hayBucle) {
    const espacio = document.createElement("span");

    espacio.className = "popup-macro-editor-marcador-espacio";

    extra.append(espacio);
  }

  const accion = document.createElement("button");

  accion.className = "ui-btn popup-macro-editor-accion";
  accion.textContent = expandido ? "Editando..." : textoAccionPaso(paso);

  accion.addEventListener("click", (eventoClick) => {
    eventoClick.stopPropagation();

    alternarExpandido(idPaso);
  });

  extra.append(accion);

  filaPrincipal.append(extra);

  // Nota — input de texto plano, letra gris, no se envía al
  // ejecutar (spec punto 11/G4). Guarda con debounce, sin
  // redibujar todo el popup (perdería el foco), mismo criterio que
  // el resto de inputs de texto en vivo del editor.
  const inputNota = document.createElement("input");

  inputNota.className = "popup-macro-nota";
  inputNota.type = "text";
  inputNota.placeholder = "...";
  inputNota.value = paso.nota;

  inputNota.addEventListener("click", (eventoClick) => {
    eventoClick.stopPropagation();
  });

  inputNota.addEventListener("input", () => {
    paso.nota = inputNota.value;

    guardarSinRedibujar();
  });

  filaPrincipal.append(inputNota);

  contenedor.append(filaPrincipal);

  // ----------------------------------
  // ⁝ MENÚ DEL BOTÓN DE OPCIONES (Mover / Duplicar / Eliminar)
  // Y LISTA VERTICAL DE TIPOS — comparten idMenuAbierto (uno solo
  // desplegado a la vez), distinguidos por prefijo de id.
  // ----------------------------------

  if (idMenuAbierto === idPaso) {
    // elementoMenu se referencia dentro de cerrarMenuLigero antes de
    // quedar asignado — seguro porque ese callback solo se invoca
    // desde un click posterior (ver comentario en crearMenuAsa).
    let elementoMenu: HTMLElement | undefined;

    elementoMenu = crearMenuAsa(
      paso,
      indice,
      idPaso,
      macroArchivo,
      () => alternarMenu(idPaso),
      () => elementoMenu?.remove(),
      activarModoMover,
    );

    contenedor.append(elementoMenu);
  }

  if (tipoAbierto) {
    contenedor.append(
      crearListaTipoPaso(paso, macroArchivo, () =>
        alternarMenu(`tipo:${idPaso}`),
      ),
    );
  }

  // ----------------------------------
  // DETALLE EXPANDIDO (Tipo + Acción/Extra completos)
  // ----------------------------------

  if (expandido) {
    contenedor.append(
      crearDetalleExpandido(
        paso,
        idPaso,
        macroArchivo,
        programaFiltroApp,
        guardarYRedibujar,
        guardarSinRedibujar,
        redibujar,
      ),
    );
  }

  return contenedor;
}

// ======================================================
// 🔤 HELPERS DE LETRAS DE BUCLE
// ======================================================

// Letras ya usadas como bucleMarcadorDestino entre todos los Bucles.
function letraBucleDisponible(pasos: PasoMacro[]): string {
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

// Letras de Bucles que no tienen ninguna fila con marcador igual a ellas.
function calcularLetrasNecesitadas(pasos: PasoMacro[]): string[] {
  const marcadoresFila = new Set(
    pasos.map((p) => p.marcador).filter((m): m is string => m !== null),
  );

  return pasos
    .filter((p) => p.tipo === "bucle" && p.bucleMarcadorDestino !== null)
    .map((p) => p.bucleMarcadorDestino as string)
    .filter((letra) => !marcadoresFila.has(letra));
}

// ======================================================
// 🔵 ICONO DE BUCLE (su propia letra, estilo invertido)
// ------------------------------------------------------
// Muestra bucleMarcadorDestino con borde azul, fondo
// transparente, letra azul — invertido al marcador de fila
// (que tiene fondo cyan y letra blanca).
// ======================================================

function crearIconoBucle(paso: PasoMacro): HTMLElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn popup-macro-editor-marcador";
  boton.dataset.activo = "false";
  boton.dataset.esBucle = "true";
  boton.textContent = paso.bucleMarcadorDestino ?? "?";
  boton.title = `Bucle ${paso.bucleMarcadorDestino ?? ""}`;
  boton.disabled = true;

  return boton;
}

// ======================================================
// 🔤 CONTROL DE MARCADOR (columna condicional — filas no-Bucle)
// ------------------------------------------------------
// Click cicla entre letras de Bucles que necesitan fila asignada.
// Primer click: primera letra necesitada. Siguiente: siguiente.
// Cuando se agotan o el paso ya tenía la última: vuelve a ○.
// ======================================================

function crearControlMarcador(
  paso: PasoMacro,
  macroArchivo: MacroArchivo,
  letrasNecesitadas: string[],
  guardarYRedibujar: () => void,
): HTMLElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn popup-macro-editor-marcador";

  if (paso.marcador) {
    boton.textContent = paso.marcador;
    boton.title = "Quitar marcador";
    boton.dataset.activo = "true";
  } else {
    boton.textContent = "○";
    boton.title = "Asignar marcador para Bucle";
    boton.dataset.activo = "false";
  }

  boton.addEventListener("click", (evento) => {
    evento.stopPropagation();

    if (paso.marcador) {
      // Ciclar a la siguiente letra necesitada, o volver a ○.
      const indiceActual = letrasNecesitadas.indexOf(paso.marcador);
      const siguiente = letrasNecesitadas[indiceActual + 1] ?? null;

      if (siguiente === null) {
        // Quitar: cualquier Bucle que apuntaba acá vuelve a sin
        // destino visible (la letra en el Bucle se mantiene, pero
        // la fila ya no la cubre).
        paso.marcador = null;
      } else {
        paso.marcador = siguiente;
      }
    } else {
      // Asignar primera letra necesitada disponible.
      paso.marcador = letrasNecesitadas[0] ?? null;
    }

    guardarYRedibujar();
  });

  return boton;
}

// ======================================================
// ⁝ MENÚ DEL BOTÓN DE OPCIONES (Mover / Duplicar / Eliminar)
// ------------------------------------------------------
// Reusa el mismo patrón que el popup de Opciones de fila de
// ventana principal (comp_popup_abrir.ts): "Mover" activa el modo
// mover del controlador de arrastre para este paso puntual (sin
// pasar por el clic mantenido sobre el asa) — a partir de ahí el
// usuario arrastra con el mouse o mueve con las flechas, igual
// que con el mantenido. Sin íconos en los botones. "Eliminar" con
// letra roja y borde rojo al pasar el mouse (clase
// popup-btn-peligro, Etapa 8).
// ======================================================

function crearMenuAsa(
  paso: PasoMacro,
  indice: number,
  idPaso: string,
  macroArchivo: MacroArchivo,
  cerrarMenu: () => void,
  cerrarMenuLigero: () => void,
  activarModoMover: (idPaso: string) => void,
): HTMLElement {
  const menu = document.createElement("div");

  menu.className = "popup-lista popup-macro-editor-menu-asa";

  // "Mover" NO puede cerrar el menú con un redibujado completo: eso
  // destruiría y recrearía el controladorArrastre recién activado
  // (dibujar() lo destruye/reconstruye siempre, ver montarEditor),
  // perdiendo la selección que activarModoMoverPara acaba de aplicar.
  // Por eso usa cerrarMenuLigero (solo saca este menú del DOM) en vez
  // de cerrarMenu (que sí redibuja completo).
  const botonMover = document.createElement("button");

  botonMover.className = "ui-btn";
  botonMover.textContent = "Mover";

  botonMover.addEventListener("click", () => {
    activarModoMover(idPaso);
    cerrarMenuLigero();
  });

  menu.append(botonMover);

  const botonDuplicar = document.createElement("button");

  botonDuplicar.className = "ui-btn";
  botonDuplicar.textContent = "Duplicar";

  botonDuplicar.addEventListener("click", () => {
    macroArchivo.pasos.splice(indice + 1, 0, clonarPasoMacro(paso));

    guardarConDebounce(macroArchivo);
    cerrarMenu();
  });

  menu.append(botonDuplicar);

  const botonEliminar = document.createElement("button");

  botonEliminar.className = "ui-btn popup-btn-peligro";
  botonEliminar.textContent = "Eliminar";

  botonEliminar.addEventListener("click", () => {
    // Si se elimina un Bucle, cualquier fila que tenía su letra
    // de marcador queda huérfana — se limpia para que los círculos
    // no muestren una letra que ya no existe.
    if (paso.tipo === "bucle" && paso.bucleMarcadorDestino) {
      const letra = paso.bucleMarcadorDestino;

      macroArchivo.pasos.forEach((p) => {
        if (p.marcador === letra) {
          p.marcador = null;
        }
      });
    }

    macroArchivo.pasos.splice(indice, 1);

    guardarConDebounce(macroArchivo);
    cerrarMenu();
  });

  menu.append(botonEliminar);

  return menu;
}

// ======================================================
// 🔽 LISTA VERTICAL DE TIPOS (celda Tipo, expandida in-place)
// ------------------------------------------------------
// Mismo look que abrirPopupTipo/crearListaTipo de ventana
// principal (comp_popup_abrir.ts: clases popup-lista/
// popup-tipo-item/popup-tipo-icono con ícono + texto), pero
// montada dentro de la propia fila en vez de vía mostrarPopup —
// el editor de Macro no puede abrir un popup anidado sin
// destruirse (ver nota de cabecera del archivo).
//
// Al seleccionar un tipo DIFERENTE del actual, resetea los datos
// de Extra del paso (spec punto 11): se reconstruye el paso desde
// crearPasoMacro(nuevoTipo), preservando únicamente lo que es
// independiente del tipo — marcador y nota.
// ======================================================

const TIPOS_PASO_MACRO: TipoPasoMacro[] = [
  "tecla_mouse",
  "espera",
  "bucle",
  "coordenada",
  "pegar",
  "abrir",
  "multimedia",
];

function crearListaTipoPaso(
  paso: PasoMacro,
  macroArchivo: MacroArchivo,
  cerrarMenu: () => void,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "popup-lista popup-macro-editor-menu-asa";

  TIPOS_PASO_MACRO.forEach((tipo) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn popup-tipo-item";

    const icono = document.createElement("span");

    icono.className = "popup-tipo-icono";
    icono.textContent = iconoTipoPasoMacro(tipo);

    const texto = document.createElement("span");

    texto.textContent = textoTipoPasoMacro(tipo).replace(/^\S+\s/, "");

    boton.append(icono, texto);

    boton.addEventListener("click", () => {
      if (tipo !== paso.tipo) {
        // Si el paso actual ES Bucle y cambia a otro tipo, limpiar
        // filas que tenían su letra (la letra desaparece con él).
        if (paso.tipo === "bucle" && paso.bucleMarcadorDestino) {
          const letraVieja = paso.bucleMarcadorDestino;

          macroArchivo.pasos.forEach((p) => {
            if (p.marcador === letraVieja) {
              p.marcador = null;
            }
          });
        }

        const nuevoPaso = crearPasoMacro(tipo);

        nuevoPaso.marcador = paso.marcador;
        nuevoPaso.nota = paso.nota;

        if (tipo === "bucle") {
          nuevoPaso.bucleMarcadorDestino = letraBucleDisponible(
            macroArchivo.pasos,
          );
        }

        Object.assign(paso, nuevoPaso);
      }

      guardarConDebounce(macroArchivo);
      cerrarMenu();
    });

    lista.append(boton);
  });

  return lista;
}

// ======================================================
// 📌 PANEL "FUNCIONES" (columna izquierda fija, 7 tipos)
// ------------------------------------------------------
// Antes vivía al pie del popup (crearMenuAgregarPaso); desde la
// Etapa 5 pasa a ser la columna izquierda fija del editor, con
// subtítulo "Funciones" (spec punto 8). Sin el prefijo "+ " que
// tenía cada botón.
// ======================================================

function crearPanelFunciones(
  macroArchivo: MacroArchivo,
  guardarYRedibujar: () => void,
): HTMLElement {
  const panel = document.createElement("div");

  panel.className = "popup-macro-funciones";

  const subtitulo = document.createElement("span");

  subtitulo.className = "popup-macro-funciones-titulo";
  subtitulo.textContent = "Funciones";

  panel.append(subtitulo);

  const tipos: TipoPasoMacro[] = [
    "tecla_mouse",
    "espera",
    "bucle",
    "coordenada",
    "pegar",
    "abrir",
    "multimedia",
  ];

  const grupo = document.createElement("div");

  grupo.className = "popup-grupo popup-macro-editor-agregar";

  tipos.forEach((tipo) => {
    const boton = document.createElement("button");

    boton.className = "ui-btn popup-opcion";
    boton.textContent = textoTipoPasoMacro(tipo);

    boton.addEventListener("click", () => {
      const nuevoPaso = crearPasoMacro(tipo);

      if (tipo === "bucle") {
        nuevoPaso.bucleMarcadorDestino = letraBucleDisponible(
          macroArchivo.pasos,
        );
      }

      macroArchivo.pasos.push(nuevoPaso);

      guardarYRedibujar();
    });

    grupo.append(boton);
  });

  panel.append(grupo);

  return panel;
}

// ======================================================
// 🔽 DETALLE EXPANDIDO — despacha según tipo
// ======================================================

function crearDetalleExpandido(
  paso: PasoMacro,
  idPaso: string,
  macroArchivo: MacroArchivo,
  programaFiltroApp: string | null,
  guardarYRedibujar: () => void,
  guardarSinRedibujar: () => void,
  redibujar: () => void,
): HTMLElement {
  const detalle = document.createElement("div");

  detalle.className = "popup-caja-interna popup-macro-editor-detalle";

  switch (paso.tipo) {
    case "tecla_mouse":
      detalle.append(
        crearDetalleTeclaMouse(paso, idPaso, guardarYRedibujar, redibujar),
      );
      break;

    case "espera":
      detalle.append(crearDetalleEspera(paso, guardarYRedibujar));
      break;

    case "bucle":
      detalle.append(crearDetalleBucle(paso, macroArchivo, guardarYRedibujar));
      break;

    case "coordenada":
      detalle.append(crearDetalleCoordenada(paso, guardarYRedibujar));
      break;

    case "pegar":
      detalle.append(
        crearDetallePegar(paso, guardarYRedibujar, guardarSinRedibujar),
      );
      break;

    case "abrir":
      detalle.append(
        crearDetalleAbrir(
          paso,
          guardarYRedibujar,
          guardarSinRedibujar,
          redibujar,
        ),
      );
      break;

    case "multimedia":
      detalle.append(
        crearDetalleMultimedia(paso, programaFiltroApp, guardarYRedibujar),
      );
      break;
  }

  return detalle;
}

// ======================================================
// ⌨️ DETALLE — Tecla/Mouse
// ------------------------------------------------------
// Reusa el capturador de combos (capturarTeclaPaso, mismo
// mecanismo de invoke que comp_capturador.ts) — la Condición
// (Simple/Doble/Triple/Mantenido) viaja adentro del propio
// Trigger capturado, ya no es parte de Extra (mismo rediseño
// que la fila principal, ver core_trigger.ts/compilador.rs).
// Extra queda en Ninguno/Normal/Turbo. El campo Duración
// aparece cuando el DOWN necesita un tiempo simulado que en
// una macro no llega de un Up físico real: condición
// Mantenido con Extra Ninguno (dura el sostenido), o
// cualquier condición con Extra Normal/Turbo (dura el bucle
// de repetición) — ver comentario de teclaDuracionMs en
// core_macro.ts.
// ======================================================

function crearDetalleTeclaMouse(
  paso: PasoMacro,
  idPaso: string,
  guardarYRedibujar: () => void,
  redibujar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  const botonCapturar = document.createElement("button");

  botonCapturar.className = "ui-btn capturador popup-macro-editor-capturador";

  const refrescarTexto = () => {
    if (paso.teclaAccion.gatillo) {
      botonCapturar.innerHTML = `
        <div class="trigger-contenido">
          ${triggerAHTML(paso.teclaAccion)}
        </div>
      `;
    } else {
      botonCapturar.textContent = "🚩 Capturar";
    }
  };

  refrescarTexto();

  botonCapturar.addEventListener("click", () => {
    botonCapturar.textContent = "Esperando...";

    capturarTeclaPaso(
      idPaso,
      (trigger) => {
        paso.teclaAccion = trigger;

        guardarYRedibujar();
      },
      () => {
        // Captura inválida/cancelada (mismo criterio que
        // comp_capturador.ts): se redibuja para volver a
        // "Capturar" en vez de quedar en "Esperando...".
        redibujar();
      },
    );
  });

  contenedor.append(crearFilaPopup("Combo", botonCapturar));

  // Extra (Repetición): mismo vocabulario que Tecla/Mouse de la
  // tabla principal tras el rediseño — Simple/Mantenido ya no son
  // opciones acá, se leen de paso.teclaAccion.condicion (el
  // gatillo capturado arriba). Sin "repeticion_rueda" (no hay
  // gatillo Rueda dentro de una Macro, ver core_macro.ts).
  const extraOpciones: { texto: string; valor: ExtraTeclaMouseMacro }[] = [
    { texto: "Ninguno", valor: "" },
    { texto: "Normal", valor: "normal" },
    { texto: "Turbo", valor: "turbo" },
  ];

  contenedor.append(
    crearFilaPopup(
      "Extra",
      crearGrupoOpciones(extraOpciones, paso.teclaExtra, (valor) => {
        // No hace falta limpiar teclaDuracionMs al cambiar de Extra
        // — se conserva por si se vuelve a necesitar después,
        // simplemente deja de mostrarse/usarse mientras tanto.
        paso.teclaExtra = valor;

        guardarYRedibujar();
      }),
    ),
  );

  // Duración (ms) — aparece cuando hace falta un tiempo simulado
  // que en una macro no llega de un Up físico real: condición
  // Mantenido con Extra Ninguno (dura el DOWN sostenido), o
  // cualquier condición con Extra Normal/Turbo (dura el bucle de
  // repetición). Con Extra Ninguno + Simple/Doble/Triple no hace
  // falta — el combo se envía una sola vez, sin tiempo que
  // configurar.
  const necesitaDuracion =
    paso.teclaExtra !== "" || paso.teclaAccion.condicion === "mantenido";

  if (necesitaDuracion) {
    contenedor.append(
      crearFilaPopup(
        "Duración (ms)",
        crearCampoNumero(paso.teclaDuracionMs ?? 100, 1, (nuevoValor) => {
          paso.teclaDuracionMs = nuevoValor;

          guardarYRedibujar();
        }),
      ),
    );
  }

  return contenedor;
}

// ======================================================
// ⏳ DETALLE — Espera
// ======================================================

function crearDetalleEspera(
  paso: PasoMacro,
  guardarYRedibujar: () => void,
): HTMLElement {
  return crearFilaPopup(
    "Tiempo (ms)",
    crearCampoNumero(paso.esperaMs, 0, (nuevoValor) => {
      paso.esperaMs = nuevoValor;

      guardarYRedibujar();
    }),
  );
}

// ======================================================
// 🔁 DETALLE — Bucle
// ------------------------------------------------------
// bucleMarcadorDestino solo puede ser una letra ya asignada a
// un paso ANTERIOR a este Bucle en el array (spec sección 3).
// Si no hay ninguna letra asignada todavía, se muestra el
// aviso de que primero hay que marcar un paso anterior.
// ======================================================

function crearDetalleBucle(
  paso: PasoMacro,
  macroArchivo: MacroArchivo,
  guardarYRedibujar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  const indiceBucle = macroArchivo.pasos.indexOf(paso);

  const marcadoresDisponibles = macroArchivo.pasos
    .slice(0, indiceBucle)
    .map((p) => p.marcador)
    .filter((m): m is string => m !== null);

  if (marcadoresDisponibles.length === 0) {
    const aviso = document.createElement("span");

    aviso.className = "app-popup-lista-titulo";

    aviso.textContent =
      "Marcá un paso anterior (columna Marcador) para elegir el destino";

    contenedor.append(aviso);
  } else {
    const opciones = marcadoresDisponibles.map((letra) => ({
      texto: letra,
      valor: letra,
    }));

    contenedor.append(
      crearFilaPopup(
        "Volver a",
        crearGrupoOpciones(
          opciones,
          paso.bucleMarcadorDestino ?? "",
          (valor) => {
            paso.bucleMarcadorDestino = valor;

            guardarYRedibujar();
          },
        ),
      ),
    );
  }

  contenedor.append(
    crearFilaPopup(
      "Veces",
      crearCampoNumero(paso.bucleVeces, 1, (nuevoValor) => {
        paso.bucleVeces = nuevoValor;

        guardarYRedibujar();
      }),
    ),
  );

  // "Modo" (Con fin/Sin fin) se sacó en la Etapa 8A: el Bucle pasa a
  // un solo algoritmo (resta 1 en cada visita, resetea al llegar a
  // 0 y sigue de largo — listo para una próxima visita si está
  // anidado dentro de otro bucle, ver core_macro.ts / Etapa 8B).

  return contenedor;
}

// ======================================================
// 🖱️ DETALLE — Coordenada
// ------------------------------------------------------
// "Posición inicial" es única y excluyente (spec tipo de paso
// 4): al activarse, oculta el resto de las variantes de
// ubicación. Reusa el mismo comando de captura
// (abrir_ventana_captura_coordenada / obtener_resultado_coordenada)
// que comp_popup_coordenada.ts, pasándole directamente los
// campos planos del paso en vez de filaPerfil.coordenada.
// ======================================================

function crearDetalleCoordenada(
  paso: PasoMacro,
  guardarYRedibujar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  contenedor.append(
    crearInterruptor("Posición inicial", paso.coordPosicionInicial, () => {
      paso.coordPosicionInicial = !paso.coordPosicionInicial;

      guardarYRedibujar();
    }),
  );

  if (paso.coordPosicionInicial) {
    return contenedor;
  }

  contenedor.append(crearSeparador());

  const ubicacionOpciones: { texto: string; valor: UbicacionPasoMacro }[] = [
    { texto: "Absoluta", valor: "absoluta" },
    { texto: "Cursor", valor: "relativa_cursor" },
    { texto: "Ventana", valor: "relativa_ventana" },
  ];

  contenedor.append(
    crearFilaPopup(
      "Ubicación relativa a:",
      crearGrupoOpciones(ubicacionOpciones, paso.coordUbicacion, (valor) => {
        paso.coordUbicacion = valor;

        paso.coordX = null;
        paso.coordY = null;

        guardarYRedibujar();
      }),
    ),
  );

  if (paso.coordUbicacion === "relativa_ventana") {
    const caja = document.createElement("div");

    caja.className = "popup-caja-interna";

    const modoOpciones: { texto: string; valor: ModoVentanaPasoMacro }[] = [
      { texto: "Píxeles", valor: "pixeles" },
      { texto: "Porcentaje", valor: "porcentaje" },
    ];

    caja.append(
      crearFilaPopup(
        "Método de Medición",
        crearGrupoOpciones(modoOpciones, paso.coordModoVentana, (valor) => {
          paso.coordModoVentana = valor;

          paso.coordPuntoReferencia = "sup_izq";
          paso.coordX = null;
          paso.coordY = null;

          guardarYRedibujar();
        }),
      ),
    );

    if (paso.coordModoVentana === "pixeles") {
      const puntoOpciones: {
        texto: string;
        valor: PuntoReferenciaPasoMacro;
      }[] = [
        { texto: "Sup-Izq", valor: "sup_izq" },
        { texto: "Sup-Der", valor: "sup_der" },
        { texto: "Centro", valor: "centro" },
        { texto: "Inf-Izq", valor: "inf_izq" },
        { texto: "Inf-Der", valor: "inf_der" },
      ];

      caja.append(
        crearFilaPopup(
          "Punto de Referencia",
          crearGrupoOpciones(
            puntoOpciones,
            paso.coordPuntoReferencia,
            (valor) => {
              paso.coordPuntoReferencia = valor;

              paso.coordX = null;
              paso.coordY = null;

              guardarYRedibujar();
            },
            "popup-grupo-grid3",
          ),
        ),
      );
    }

    contenedor.append(caja);
  }

  contenedor.append(crearSeparador());

  const botonCapturar = document.createElement("button");

  botonCapturar.className = "ui-btn popup-extra-capturar";

  const textoCapturarActual = (): string => {
    if (paso.coordX === null || paso.coordY === null) {
      return "📌 Capturar Coordenada";
    }

    if (
      paso.coordUbicacion === "relativa_ventana" &&
      paso.coordModoVentana === "porcentaje"
    ) {
      return `📌 H: ${paso.coordX}%, V: ${paso.coordY}%`;
    }

    return `📌 X: ${paso.coordX}, Y: ${paso.coordY}`;
  };

  botonCapturar.textContent = textoCapturarActual();

  botonCapturar.addEventListener("click", () => {
    invoke("abrir_ventana_captura_coordenada", {
      ubicacion: paso.coordUbicacion,
      modoVentana: paso.coordModoVentana,
      puntoReferencia: paso.coordPuntoReferencia,
    }).catch((error) => {
      console.error("abrir_ventana_captura_coordenada FALLÓ:", error);
    });

    const intervalo = setInterval(() => {
      invoke<[number, number] | null>("obtener_resultado_coordenada")
        .then((resultado) => {
          if (!resultado) {
            return;
          }

          clearInterval(intervalo);

          paso.coordX = resultado[0];
          paso.coordY = resultado[1];

          guardarYRedibujar();
        })
        .catch(() => {
          clearInterval(intervalo);
        });
    }, 200);
  });

  contenedor.append(botonCapturar);

  return contenedor;
}

// ======================================================
// 📋 DETALLE — Pegar Ruta/Texto
// ------------------------------------------------------
// Un solo campo (pegarRuta) — llama directo a
// back_portapapeles::pegar(ruta) en tiempo de ejecución (Etapa
// 8), acá solo se elige la ruta. Mismo selector Archivo/Carpeta
// que comp_popup_abrir_accion.ts, sin filtro de extensión en
// el diálogo (la validación .txt/.png es responsabilidad del
// ejecutor, no de este selector — igual criterio que "abrir").
// ======================================================

function crearDetallePegar(
  paso: PasoMacro,
  guardarYRedibujar: () => void,
  guardarSinRedibujar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  const input = document.createElement("input");

  input.type = "text";
  input.className = "popup-input";
  input.placeholder = "Solo ruta a archivo .txt/.png o escriba texto a pegar.";
  input.value = paso.pegarRuta ?? "";

  const confirmarTexto = () => {
    paso.pegarRuta = input.value.trim() || null;

    guardarSinRedibujar();
  };

  input.addEventListener("blur", confirmarTexto);

  input.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      input.blur();
    }
  });

  contenedor.append(crearFilaPopup("Ruta", input));

  const botonExaminar = document.createElement("button");

  botonExaminar.className = "ui-btn";
  botonExaminar.textContent = "📄 Examinar...";

  botonExaminar.addEventListener("click", async () => {
    const ruta = await invoke<string | null>("seleccionar_archivo", {
      extensiones: ["txt", "png"],
    });

    if (!ruta) {
      return;
    }

    paso.pegarRuta = ruta;

    input.value = ruta;

    guardarYRedibujar();
  });

  contenedor.append(botonExaminar);

  return contenedor;
}

// ======================================================
// 📂 DETALLE — Abrir Archivo/Programa
// ------------------------------------------------------
// Mismos 5 campos que AccionCache::AbrirArchivo, aplanados en
// el paso (spec tipo de paso 6). El selector de ruta y el
// listado "Abrir con" reusan los mismos comandos Tauri que
// comp_popup_abrir_accion.ts / comp_popup_abrir_con.ts
// (seleccionar_archivo, seleccionar_carpeta,
// obtener_icono_ruta, obtener_programas_abrir_con), pero
// operando sobre los campos planos del paso en vez de
// filaPerfil.abrirAccion/abrirExtra — esos componentes están
// atados a FilaPerfil/ContextoFila (reconstruirFila) y no se
// pueden reusar tal cual para un paso que no es una fila real
// del perfil.
// ======================================================

function crearDetalleAbrir(
  paso: PasoMacro,
  guardarYRedibujar: () => void,
  guardarSinRedibujar: () => void,
  redibujar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  // ----------------------------------
  // Selección de ruta (Archivo / Carpeta)
  // ----------------------------------

  const botonRuta = document.createElement("button");

  botonRuta.className = "ui-btn capturador";

  const iconoRuta = crearIconoFallback("📂");

  botonRuta.append(iconoRuta);

  const nombreRuta = document.createElement("span");

  nombreRuta.textContent = paso.abrirRuta
    ? nombreDeRuta(paso.abrirRuta)
    : "Seleccionar...";

  botonRuta.append(nombreRuta);

  botonRuta.title = paso.abrirRuta ?? "Seleccionar...";

  if (paso.abrirRuta) {
    invoke<IconoJson | null>("obtener_icono_ruta", { ruta: paso.abrirRuta })
      .then((iconoJson) => {
        if (!iconoJson) {
          return;
        }

        botonRuta.replaceChild(crearIconoDesdeJson(iconoJson), iconoRuta);
      })
      .catch(() => {});
  }

  botonRuta.addEventListener("click", (evento) => {
    evento.stopPropagation();

    const lista = document.createElement("div");

    lista.className = "popup-lista";

    const botonArchivo = document.createElement("button");

    botonArchivo.className = "ui-btn";
    botonArchivo.textContent = "📄 Archivo...";

    botonArchivo.addEventListener("click", async () => {
      const ruta = await invoke<string | null>("seleccionar_archivo", {
        extensiones: null,
      });

      if (!ruta) {
        return;
      }

      paso.abrirRuta = ruta;

      // abrirCon/argumento fueron elegidos para el archivo anterior
      // — mismo criterio de limpieza que aplicarRuta() en
      // comp_popup_abrir_accion.ts.
      paso.abrirCon = null;
      paso.abrirArgumento = "";

      guardarYRedibujar();
    });

    const botonCarpeta = document.createElement("button");

    botonCarpeta.className = "ui-btn";
    botonCarpeta.textContent = "📁 Carpeta...";

    botonCarpeta.addEventListener("click", async () => {
      const ruta = await invoke<string | null>("seleccionar_carpeta");

      if (!ruta) {
        return;
      }

      paso.abrirRuta = ruta;
      paso.abrirCon = null;
      paso.abrirArgumento = "";

      guardarYRedibujar();
    });

    lista.append(botonArchivo, botonCarpeta);

    // Se inserta como caja expandida debajo del botón, en vez de un
    // popup aparte (mostrarPopup destruiría todo el editor) — se
    // reemplaza cualquier lista de selección de ruta anterior que
    // hubiera quedado abierta en este mismo detalle.
    const cajaAnterior = contenedor.querySelector(
      ".popup-macro-editor-ruta-lista",
    );

    if (cajaAnterior) {
      cajaAnterior.remove();

      return;
    }

    lista.classList.add("popup-caja-interna", "popup-macro-editor-ruta-lista");

    botonRuta.insertAdjacentElement("afterend", lista);
  });

  contenedor.append(crearFilaPopup("Ruta", botonRuta));

  // ----------------------------------
  // Iniciar
  // ----------------------------------

  const iniciarOpciones: { texto: string; valor: IniciarPasoMacro }[] = [
    { texto: "Ventana", valor: "ventana" },
    { texto: "Minimizado", valor: "minimizado" },
    { texto: "Maximizado", valor: "maximizado" },
  ];

  contenedor.append(
    crearFilaPopup(
      "Iniciar",
      crearGrupoOpciones(iniciarOpciones, paso.abrirIniciar, (valor) => {
        paso.abrirIniciar = valor;

        guardarYRedibujar();
      }),
    ),
  );

  // ----------------------------------
  // Instancias
  // ----------------------------------

  const instanciasOpciones: { texto: string; valor: InstanciasPasoMacro }[] = [
    { texto: "Única", valor: "unica" },
    { texto: "Múltiple", valor: "multiple" },
  ];

  contenedor.append(
    crearFilaPopup(
      "Instancias",
      crearGrupoOpciones(instanciasOpciones, paso.abrirInstancias, (valor) => {
        paso.abrirInstancias = valor;

        guardarYRedibujar();
      }),
    ),
  );

  contenedor.append(crearSeparador());

  // ----------------------------------
  // Abrir con (documento/carpeta) — Argumento (.exe)
  // ----------------------------------

  if (esRutaExe(paso.abrirRuta)) {
    const input = document.createElement("input");

    input.type = "text";
    input.className = "popup-input";
    input.placeholder = "--argumento";
    input.value = paso.abrirArgumento;

    const confirmar = () => {
      paso.abrirArgumento = input.value;

      guardarSinRedibujar();
    };

    input.addEventListener("blur", confirmar);

    input.addEventListener("keydown", (eventoTecla) => {
      if (eventoTecla.key === "Enter") {
        input.blur();
      }
    });

    contenedor.append(crearFilaPopup("Argumento", input));
  } else {
    const botonAbrirCon = document.createElement("button");

    botonAbrirCon.className = "ui-btn";
    botonAbrirCon.textContent = paso.abrirCon
      ? nombreDeRuta(paso.abrirCon)
      : "Predeterminado";

    if (paso.abrirCon) {
      botonAbrirCon.title = paso.abrirCon;
    }

    botonAbrirCon.addEventListener("click", async () => {
      const cajaAnterior = contenedor.querySelector(
        ".popup-macro-editor-abrircon-lista",
      );

      if (cajaAnterior) {
        cajaAnterior.remove();

        return;
      }

      const caja = await crearListaAbrirConPaso(paso, () => {
        redibujar();
      });

      caja.classList.add("popup-macro-editor-abrircon-lista");

      botonAbrirCon.insertAdjacentElement("afterend", caja);
    });

    contenedor.append(crearFilaPopup("Abrir con", botonAbrirCon));
  }

  return contenedor;
}

// ======================================================
// 📂🗂️ LISTADO "ABRIR CON" — versión para un PasoMacro
// ------------------------------------------------------
// Mismo contenido/fuente que crearListaAbrirCon() en
// comp_popup_abrir_con.ts (registro de Windows vía
// obtener_programas_abrir_con), reimplementado acá porque el
// original está atado a FilaPerfil/ContextoFila
// (reconstruirFila). alSeleccionar la llama cualquier ítem
// elegido — quien la pasa (crearDetalleAbrir) redibuja el
// detalle completo y colapsa el listado.
// ======================================================

async function crearListaAbrirConPaso(
  paso: PasoMacro,
  alSeleccionar: () => void,
): Promise<HTMLElement> {
  const extension = extensionDeRuta(paso.abrirRuta);

  const programas = await invoke<ProgramaJson[]>(
    "obtener_programas_abrir_con",
    { extension },
  );

  const contenedor = document.createElement("div");

  contenedor.className = "popup-caja-interna app-popup-lista-caja";

  const lista = document.createElement("div");

  lista.className = "app-popup-lista";

  const crearBotonPrograma = (
    nombre: string,
    ruta: string | null,
    activo: boolean,
    iconoEmoji: string,
  ): HTMLButtonElement => {
    const boton = document.createElement("button");

    boton.className = "ui-btn app-popup-programa";
    boton.dataset.activo = activo ? "true" : "false";

    const icono = document.createElement("span");

    icono.className = "app-popup-global-icono";
    icono.textContent = iconoEmoji;

    boton.append(icono);

    const spanNombre = document.createElement("span");

    spanNombre.className = "app-popup-nombre";
    spanNombre.textContent = nombre;

    boton.append(spanNombre);

    if (ruta) {
      boton.title = ruta;

      invoke<IconoJson | null>("obtener_icono_ruta", { ruta })
        .then((iconoJson) => {
          if (!iconoJson) {
            return;
          }

          boton.replaceChild(crearIconoDesdeJson(iconoJson), icono);
        })
        .catch(() => {});
    }

    boton.addEventListener("click", () => {
      paso.abrirCon = ruta;

      alSeleccionar();
    });

    return boton;
  };

  lista.append(
    crearBotonPrograma("Predeterminado", null, paso.abrirCon === null, "⭯"),
  );

  programas.forEach((programa) => {
    lista.append(
      crearBotonPrograma(
        programa.nombre,
        programa.ruta,
        programa.ruta === paso.abrirCon,
        "▣",
      ),
    );
  });

  contenedor.append(lista);
  contenedor.append(crearSeparador());

  const botonExaminar = document.createElement("button");

  botonExaminar.className = "ui-btn app-popup-programa";

  const iconoExaminar = document.createElement("span");

  iconoExaminar.className = "app-popup-global-icono";
  iconoExaminar.textContent = "📂";

  const nombreExaminar = document.createElement("span");

  nombreExaminar.className = "app-popup-nombre";
  nombreExaminar.textContent = "Examinar...";

  botonExaminar.append(iconoExaminar, nombreExaminar);

  botonExaminar.addEventListener("click", async () => {
    const ruta = await invoke<string | null>("seleccionar_archivo", {
      extensiones: ["exe"],
    });

    if (!ruta) {
      return;
    }

    paso.abrirCon = ruta;

    alSeleccionar();
  });

  contenedor.append(botonExaminar);

  return contenedor;
}

// ======================================================
// 🎚️ DETALLE — Multimedia
// ------------------------------------------------------
// Mismas categorías/comandos que comp_popup_multimedia.ts,
// operando sobre paso.multimediaComando. "En App" solo se
// ofrece si el comando elegido es de Volumen (spec tipo de
// paso 7) — el Filtro de App de la FILA MACRO contenedora
// decide el programa en tiempo de compilación/ejecución
// (compilador.rs), acá solo se elige el alcance.
// ======================================================

function crearDetalleMultimedia(
  paso: PasoMacro,
  programaFiltroApp: string | null,
  guardarYRedibujar: () => void,
): HTMLElement {
  const contenedor = document.createElement("div");

  const conIcono = (
    opciones: OpcionMultimedia[],
  ): { texto: string; valor: ComandoPasoMacro }[] =>
    opciones.map((opcion) => ({
      texto: `${opcion.icono} ${opcion.texto}`,
      valor: opcion.valor,
    }));

  const elegirComando = (comando: ComandoPasoMacro) => {
    paso.multimediaComando = comando;

    if (!esComandoDeVolumen(comando) && paso.multimediaAlcance === "en_app") {
      paso.multimediaAlcance = "global";
    }

    guardarYRedibujar();
  };

  const actual = paso.multimediaComando as ComandoPasoMacro;

  contenedor.append(
    crearFilaPopup(
      "Volumen",
      crearGrupoOpciones(conIcono(COMANDOS_VOLUMEN), actual, elegirComando),
    ),
  );

  contenedor.append(
    crearGrupoOpciones(conIcono([COMANDO_SILENCIAR]), actual, elegirComando),
  );

  contenedor.append(crearSeparador());

  contenedor.append(
    crearFilaPopup(
      "Reproducción",
      crearGrupoOpciones(
        conIcono(COMANDOS_REPRODUCCION_PRINCIPAL),
        actual,
        elegirComando,
      ),
    ),
  );

  contenedor.append(
    crearGrupoOpciones(
      conIcono(COMANDOS_REPRODUCCION_PISTA),
      actual,
      elegirComando,
    ),
  );

  contenedor.append(crearSeparador());

  // ----------------------------------
  // Alcance (Global / En App)
  // ------------------------------------------------------
  // "En App" reusa el Filtro de App de la FILA MACRO
  // contenedora (spec tipo de paso 7) — si esa fila es global
  // (sin programa filtrado), no hay de dónde sacar el
  // programa y la opción no debería ofrecerse (mismo criterio
  // que motivoDeshabilitado() en comp_popup_multimedia_extra.ts,
  // adaptado: acá la fuente es programaFiltroApp en vez de
  // filaPerfil.app.programa directo, porque este popup edita
  // un PASO, no la fila).
  // ----------------------------------

  const motivo = !programaFiltroApp
    ? "La fila Macro no tiene Filtro de App — asigná un programa en la columna App"
    : !esComandoDeVolumen(paso.multimediaComando)
      ? "En App solo está disponible para los comandos de Volumen"
      : undefined;

  const alcanceOpciones: {
    texto: string;
    valor: AlcancePasoMacro;
    deshabilitado?: boolean;
    titulo?: string;
  }[] = [
    { texto: "Global", valor: "global" },
    {
      texto: "En App",
      valor: "en_app",
      deshabilitado: !!motivo,
      titulo: motivo,
    },
  ];

  contenedor.append(
    crearFilaPopup(
      "Alcance",
      crearGrupoOpciones(alcanceOpciones, paso.multimediaAlcance, (valor) => {
        paso.multimediaAlcance = valor;

        guardarYRedibujar();
      }),
    ),
  );

  if (motivo) {
    const ayuda = document.createElement("span");

    ayuda.className = "app-popup-lista-titulo";
    ayuda.textContent = motivo;

    contenedor.append(ayuda);
  }

  return contenedor;
}
