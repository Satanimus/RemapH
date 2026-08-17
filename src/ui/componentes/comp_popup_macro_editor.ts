// ======================================================
// 🧩📝 comp_Popup_Macro_Editor
// ------------------------------------------------------
// Editor completo de una Macro (popup Tipo/Acción/Extra por
// paso), abierto desde el botón Extra de una fila tipo ===
// "macro" (ver crearExtra() en comp_controles.ts). Distinto
// de comp_popup_macro_accion.ts (que solo decide A CUÁL
// macro apunta la fila) — acá se editan los PASOS de la
// macro ya elegida (filaPerfil.accionReferencia).
//
// Mismo patrón persistente que el resto de popups Extra
// (menu_express/portapapeles/abrir): cada interacción
// actualiza el estado en memoria y redibuja el mismo popup en
// el lugar. A diferencia de esos, acá el guardado NO es
// instantáneo por campo — se guarda con un debounce corto vía
// macro_guardar (ver guardarConDebounce más abajo), porque
// escribir a disco en cada tecla de un input de texto sería
// excesivo. Al cerrar el popup (clic afuera) se fuerza un
// guardado final sin esperar el debounce.
//
// mostrarPopup() reemplaza TODA la capa global de popups, así
// que este editor no puede abrir sub-popups anidados sin
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

import { mostrarPopup } from "./comp_popup_contenedor";

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
  letraMarcadorDisponible,
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
// 💾 GUARDADO CON DEBOUNCE
// ------------------------------------------------------
// macro_guardar ya existe en el backend (ver comandos.rs,
// nota "el editor Etapa 5/6 reusa macro_guardar tal cual") —
// el estado del popup ES el modelo de datos (sin traducción
// intermedia, spec sección 1), así que guardar es enviar
// macroArchivo tal cual. Debounce corto para no escribir a
// disco en cada tecla de un input de texto (nombre, argumento,
// ruta de Pegar); guardarAhora() fuerza el guardado inmediato
// para el cierre del popup y para cambios "discretos" (elegir
// una opción de un grupo, no un input de texto en vivo).
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
    await invoke("macro_guardar", { macroArchivo });
  } catch (error) {
    console.error("❌ No se pudo guardar la macro:", error);
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

      montarEditor(evento, contexto, macroArchivo, filaPerfil.app.programa);
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
  macroArchivo: MacroArchivo,
  programaFiltroApp: string | null,
): void {
  // Índice del paso actualmente expandido (mostrando su Acción/Extra
  // en detalle) — null si ninguno. Puramente visual, no se guarda.
  // Se guarda el ID sintético (no el índice) para sobrevivir a un
  // reordenamiento por arrastre mientras sigue expandido.
  let idPasoExpandido: string | null = null;

  // Menú de opciones (Mover/Eliminar/Duplicar) del botón ⟫ de un
  // paso — igual criterio que idPasoExpandido, solo un menú abierto
  // a la vez.
  let idMenuAbierto: string | null = null;

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

  function dibujar(): void {
    if (controladorArrastre) {
      controladorArrastre.destruir();

      controladorArrastre = null;
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
    // 🏷️ TÍTULO
    // ----------------------------------

    const titulo = document.createElement("div");

    titulo.className = "popup-fila-label popup-macro-editor-titulo";

    titulo.textContent = `🧩 ${macroArchivo.nombre}`;

    popup.append(titulo);

    // ----------------------------------
    // 📋 LISTA DE PASOS
    // ----------------------------------

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
        ),
      );
    });

    popup.append(lista);

    // ----------------------------------
    // ➕ AGREGAR PASO
    // ----------------------------------

    popup.append(crearSeparador());

    popup.append(crearMenuAgregarPaso(macroArchivo, guardarYRedibujar));

    mostrarPopup(popup, evento.clientX, evento.clientY, () => {
      // Cierre del popup (clic afuera): fuerza el guardado final sin
      // esperar el debounce, y libera los listeners del componente
      // de arrastre (ver destruir() en util_arrastrable.ts) — si no
      // se llamara, quedarían escuchando document de una instancia
      // que ya no existe.
      guardarAhora(macroArchivo);

      if (controladorArrastre) {
        controladorArrastre.destruir();

        controladorArrastre = null;
      }

      reconstruirFila(contexto.id);
    });

    // El componente de arrastre necesita el contenedor YA en el DOM
    // (mostrarPopup ya lo insertó arriba) para poder registrar cada
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
  }

  dibujar();
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
): HTMLElement {
  const idPaso = idDePaso(paso);

  const contenedor = document.createElement("div");

  contenedor.className = "popup-macro-editor-paso";
  contenedor.dataset.pasoId = idPaso;

  // ----------------------------------
  // FILA PRINCIPAL (asa, #, marcador, tipo, acción)
  // ----------------------------------

  const filaPrincipal = document.createElement("div");

  filaPrincipal.className = "popup-macro-editor-paso-fila";

  // ⟫ Asa — clic corto abre el menú Mover/Eliminar/Duplicar (ver
  // crearMenuAsa), clic mantenido lo maneja util_arrastrable.ts
  // directamente sobre este mismo botón.
  const asa = document.createElement("button");

  asa.className = "ui-btn popup-macro-editor-asa";
  asa.textContent = "⟫";
  asa.title = "Mover / opciones";

  asa.addEventListener("click", () => {
    alternarMenu(idPaso);
  });

  filaPrincipal.append(asa);

  // # — puramente visual, número de fondo (spec sección 3), no es
  // un dato guardado ni se selecciona/edita.
  const numero = document.createElement("span");

  numero.className = "popup-macro-editor-numero";
  numero.textContent = `#${indice + 1}`;

  filaPrincipal.append(numero);

  // Columna Marcador — la columna existe (reserva espacio) en toda
  // fila apenas hay algún Bucle en la macro, pero el control real
  // (asignar/quitar letra) solo se ofrece en pasos que NO son Bucle
  // y que tienen al menos un Bucle en algún punto POSTERIOR (spec:
  // "solo los pasos anteriores a un Bucle pueden tomar su letra").
  // Un paso ya marcado se sigue mostrando aunque un reordenamiento
  // lo haya dejado sin ningún Bucle detrás — así el usuario puede
  // verlo y quitarlo, en vez de que el marcador quede "invisible"
  // pero todavía activo. Reglas 3-5 de la spec (letra estable al
  // reordenar, múltiples letras simultáneas — A, B, C..., una por
  // Bucle — soportado desde el modelo de datos vía
  // letraMarcadorDisponible) no imponen un único marcador global.
  if (
    hayBucle &&
    paso.tipo !== "bucle" &&
    (elegiblePorMarcador || paso.marcador)
  ) {
    filaPrincipal.append(
      crearControlMarcador(paso, macroArchivo, guardarYRedibujar),
    );
  } else if (hayBucle) {
    // Paso Bucle: espacio reservado sin control, para que el resto
    // de las columnas se sigan alineando con las filas que sí
    // muestran el selector de Marcador.
    const espacio = document.createElement("span");

    espacio.className = "popup-macro-editor-marcador-espacio";

    filaPrincipal.append(espacio);
  }

  // Tipo — botón que despliega el selector de los 7 tipos, en el
  // lugar (mismo patrón que abrirConExpandido).
  const botonTipo = document.createElement("button");

  botonTipo.className = "ui-btn popup-macro-editor-tipo";
  botonTipo.textContent = textoTipoPasoMacro(paso.tipo);

  const expandido = idPasoExpandido === idPaso;

  botonTipo.addEventListener("click", (eventoClick) => {
    eventoClick.stopPropagation();

    alternarExpandido(idPaso);
  });

  filaPrincipal.append(botonTipo);

  // Acción — resumen de una línea del paso (cerrado) o "editando"
  // mientras está expandido (el detalle real va debajo).
  const accion = document.createElement("button");

  accion.className = "ui-btn popup-macro-editor-accion";
  accion.textContent = expandido ? "Editando..." : textoAccionPaso(paso);

  accion.addEventListener("click", (eventoClick) => {
    eventoClick.stopPropagation();

    alternarExpandido(idPaso);
  });

  filaPrincipal.append(accion);

  contenedor.append(filaPrincipal);

  // ----------------------------------
  // MENÚ ⟫ (Mover ya lo activa util_arrastrable.ts con clic
  // mantenido — acá solo Eliminar / Duplicar, clic corto)
  // ----------------------------------

  if (idMenuAbierto === idPaso) {
    contenedor.append(
      crearMenuAsa(paso, indice, macroArchivo, guardarYRedibujar),
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
// 🔤 CONTROL DE MARCADOR (columna condicional)
// ------------------------------------------------------
// Reglas (spec sección 3):
// - Solo asignable a un paso ANTERIOR a un Bucle existente
//   (elegibilidad ya filtrada por el llamador — crearFilaPaso —
//   antes de instanciar este control).
// - Pueden coexistir varias letras a la vez en la misma macro
//   (una por cada Bucle que las referencia — ver ejemplo de
//   bucles anidados A/B en el plan, y letraMarcadorDisponible en
//   core_macro.ts, diseñada explícitamente para A, B, C... sin
//   límite). No hay "un solo marcador global": cada paso elegible
//   ofrece su propio botón para tomar la próxima letra libre.
// - Clic sobre "○" (sin marcar) asigna la próxima letra libre.
//   Clic sobre la letra ya asignada la quita (vuelve cualquier
//   Bucle que la referenciaba a "sin destino" — mismo criterio
//   que crearMenuAsa al eliminar un paso marcado, porque quitar
//   el marcador equivale a "borrar el paso marcado" desde el
//   punto de vista del Bucle).
// ======================================================

function crearControlMarcador(
  paso: PasoMacro,
  macroArchivo: MacroArchivo,
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
    boton.title = "Asignar marcador";
    boton.dataset.activo = "false";
  }

  boton.addEventListener("click", (evento) => {
    evento.stopPropagation();

    if (paso.marcador) {
      // Quitar: cualquier Bucle que apuntaba acá vuelve a "sin
      // destino" — mismo efecto que "se borró el paso marcado"
      // (spec), aplicado sin borrar el paso en sí.
      const letra = paso.marcador;

      paso.marcador = null;

      macroArchivo.pasos.forEach((p) => {
        if (p.tipo === "bucle" && p.bucleMarcadorDestino === letra) {
          p.bucleMarcadorDestino = null;
        }
      });
    } else {
      paso.marcador = letraMarcadorDisponible(macroArchivo.pasos);
    }

    guardarYRedibujar();
  });

  return boton;
}

// ======================================================
// ⟫ MENÚ DEL ASA (Eliminar / Duplicar)
// ------------------------------------------------------
// "Mover" no es una opción de este menú — se activa con clic
// MANTENIDO sobre el mismo botón ⟫, resuelto enteramente por
// util_arrastrable.ts (ver registrarFila en montarEditor). Acá
// solo se atiende el clic CORTO.
// ======================================================

function crearMenuAsa(
  paso: PasoMacro,
  indice: number,
  macroArchivo: MacroArchivo,
  guardarYRedibujar: () => void,
): HTMLElement {
  const menu = document.createElement("div");

  menu.className = "popup-lista popup-macro-editor-menu-asa";

  const botonDuplicar = document.createElement("button");

  botonDuplicar.className = "ui-btn";
  botonDuplicar.textContent = "📋 Duplicar";

  botonDuplicar.addEventListener("click", () => {
    macroArchivo.pasos.splice(indice + 1, 0, clonarPasoMacro(paso));

    guardarYRedibujar();
  });

  menu.append(botonDuplicar);

  const botonEliminar = document.createElement("button");

  botonEliminar.className = "ui-btn popup-macro-editor-eliminar";
  botonEliminar.textContent = "🗑️ Eliminar";

  botonEliminar.addEventListener("click", () => {
    // Si este paso estaba marcado, cualquier Bucle que apuntara acá
    // vuelve a "sin destino" — mismo criterio que crearControlMarcador
    // al quitar el marcador manualmente (spec: "si se borra el paso
    // marcado, el Bucle vuelve al estado recién creado sin destino").
    if (paso.marcador) {
      const letra = paso.marcador;

      macroArchivo.pasos.forEach((p) => {
        if (p.tipo === "bucle" && p.bucleMarcadorDestino === letra) {
          p.bucleMarcadorDestino = null;
        }
      });
    }

    macroArchivo.pasos.splice(indice, 1);

    guardarYRedibujar();
  });

  menu.append(botonEliminar);

  return menu;
}

// ======================================================
// ➕ MENÚ "AGREGAR PASO" (7 tipos)
// ======================================================

function crearMenuAgregarPaso(
  macroArchivo: MacroArchivo,
  guardarYRedibujar: () => void,
): HTMLElement {
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
    boton.textContent = `+ ${textoTipoPasoMacro(tipo)}`;

    boton.addEventListener("click", () => {
      macroArchivo.pasos.push(crearPasoMacro(tipo));

      guardarYRedibujar();
    });

    grupo.append(boton);
  });

  return grupo;
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

  // ----------------------------------
  // Selector de Tipo (siempre presente en el detalle expandido)
  // ----------------------------------

  const tipos: { texto: string; valor: TipoPasoMacro }[] = [
    { texto: "⌨️ Tecla/Mouse", valor: "tecla_mouse" },
    { texto: "⏳ Espera", valor: "espera" },
    { texto: "🔁 Bucle", valor: "bucle" },
    { texto: "🖱️ Coordenada", valor: "coordenada" },
    { texto: "📋 Pegar", valor: "pegar" },
    { texto: "📂 Abrir", valor: "abrir" },
    { texto: "🎚️ Multimedia", valor: "multimedia" },
  ];

  detalle.append(
    crearFilaPopup(
      "Tipo",
      crearGrupoOpciones(tipos, paso.tipo, (valor) => {
        if (valor === paso.tipo) {
          return;
        }

        // Al cambiar de Tipo, si este paso estaba marcado y el nuevo
        // tipo también puede tener marcador, se conserva (marcar no
        // depende del tipo). Si el nuevo tipo es "bucle" y este paso
        // tenía marcador propio, no tiene sentido (un Bucle no puede
        // ser destino de sí mismo) — se limpia. El resto de los
        // campos NO se resetea (mismo criterio que core_macro.ts:
        // "objeto plano con todos los campos siempre presentes",
        // cambiar el Tipo no borra los datos de los otros tipos).
        if (valor === "bucle" && paso.marcador) {
          const letra = paso.marcador;

          paso.marcador = null;

          macroArchivo.pasos.forEach((p) => {
            if (p.tipo === "bucle" && p.bucleMarcadorDestino === letra) {
              p.bucleMarcadorDestino = null;
            }
          });
        }

        paso.tipo = valor;

        guardarYRedibujar();
      }),
    ),
  );

  detalle.append(crearSeparador());

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
  input.placeholder = "Ruta de archivo (.txt / .png)";
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
