// ======================================================
// 📋🪟 vent_portapapeles_Main
// ------------------------------------------------------
// Punto de entrada de la ventana flotante de Portapapeles
// (portapapeles.html — página independiente, ver
// vite.config.ts). El id del Portapapeles a mostrar viaja en
// la URL (?id=...) — back_portapapeles.rs arma esa URL al
// crear la ventana (ver crear_ventana()).
//
// Los datos (fijados/rotativos/modo/etc.) se leen UNA vez al
// cargar, vía obtener_datos_portapapeles — mismo patrón que
// menu_express_main.ts con obtener_datos_menu_express: el
// backend ya dejó los datos listos ANTES de crear la ventana
// (back_portapapeles::abrir_o_alternar registra antes de
// llamar a crear_ventana), sin carrera posible. Cada operación
// de mutación (fijar/renombrar/editar/eliminar/limpiar/toggle
// Registro) recibe de vuelta el PortapapelesDatosUI ya
// actualizado (back_portapapeles::refrescar_datos, Etapa H) —
// no hace falta un segundo viaje para refrescar la lista.
//
// Siempre lista vertical (Portapapeles nunca es Radial, a
// diferencia de MenuExpress) — layout único, sin variante.
// ======================================================

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { aplicarOverridesApariencia } from "../core/core_apariencia";

import "../styles/styl_variables.css";
import "../styles/styl_portapapeles.css";

void aplicarOverridesApariencia();

// ======================================================
// 🧭 TIPOS
// ------------------------------------------------------
// Espejo de PortapapelesDatosUI / ElementoPortapapelesUI en
// back_portapapeles.rs (mismo vocabulario camelCase, ver
// #[serde(rename_all = "camelCase")] en Rust).
// ======================================================

interface ElementoDatos {
  ruta: string;

  nombre: string;

  extension: string;

  fijado: boolean;

  modificadoMs: number;
}

interface PortapapelesDatos {
  nombre: string;

  comportamiento: "toggle" | "efimero";

  ubicacion: "persistente" | "cursor";

  tamanoBoton: "pequeno" | "mediano" | "grande";

  tamanoTexto: "pequeno" | "mediano" | "grande";

  limite: number;

  color: string;

  registroActivo: boolean;

  fijados: ElementoDatos[];

  rotativos: ElementoDatos[];
}

// Espejo de OtroPortapapelesUI en back_portapapeles.rs (Cambio 2,
// Etapa 2.A) — nombre null significa que la ID no se encontró en
// ningún perfil guardado y el frontend debe mostrar la ID cruda.
interface OtroPortapapeles {
  id: string;

  nombre: string | null;
}

// ======================================================
// 🆔 ID DEL PORTAPAPELES (query string)
// ======================================================

const id = new URLSearchParams(window.location.search).get("id");

// ======================================================
// 🏗️ RAÍZ + ESTADO
// ======================================================

const raiz = document.getElementById("portapapeles")!;

let card: HTMLDivElement;
let cuerpo: HTMLDivElement;
let ultimosDatos: PortapapelesDatos | null = null;

// ID que se está MOSTRANDO ahora mismo (fijados + nombre en la
// barra). Igual a `id` (el de la URL/ventana) salvo mientras se
// está viendo el pool de otro Portapapeles (Cambio 2) — en ese
// caso se usa para: listar fijados alternativos, decidir bajo
// qué ID se fija un rotativo nuevo, y qué nombre mostrar. Se
// resetea solo al recargar la página (cerrar+reabrir la
// ventana), nunca se persiste — comportamiento pedido: "al
// cerrar y volver a llamar el portapapeles se regresa al
// portapapel original".
let idMostrado: string | null = id;
let nombreMostrado: string | null = null; // null = usar datos.nombre normal
// Últimos fijados alternativos obtenidos (verFijadosDe /
// aplicarResultadoMutacionFijado) — usados por actualizarEnVivo
// para no pisar la vista alternativa con los fijados reales de la
// ventana cuando llega un evento en vivo desde otra ventana/fila.
let fijadosAltActuales: ElementoDatos[] = [];

// Referencias a nodos que una actualización EN VIVO (evento
// "portapapeles-actualizado", ver ETAPA J.2 más abajo) actualiza in-
// place, sin pasar por construirEstructura() — así un popup abierto
// (Renombrar/Editar/Opciones, todos hijos directos de `card`, nunca
// de `cuerpo`) no se cierra solo porque llegó contenido nuevo desde
// afuera (otra fila en modo Registro, u otra ventana).
let tituloActual: HTMLSpanElement | null = null;
let botonRegistroActual: HTMLButtonElement | null = null;
// Nodo de texto del switch Registro (pista+bolita+texto, ver
// crearBarraInferior) — separado de botonRegistroActual porque el
// botón también carga la pista/bolita del switch.
let textoRegistroActual: HTMLSpanElement | null = null;

// Bandera anti-duplicado del LADO CLIENTE (paralela al bloqueo
// de back_portapapeles.rs::marcar_ignorar_proximo_cambio): mientras
// una operación de pegar/mutación está en vuelo, un doble-click
// accidental no debe disparar una segunda invocación en simultáneo.
let operacionEnVuelo = false;

// Desuscripción del listener de eventos (ETAPA J.2) — se guarda acá
// para poder limpiarla al cerrar la ventana (ver cerrar()).
let detenerEscucha: UnlistenFn | null = null;

// ======================================================
// 🚪 CERRAR
// ======================================================

function cerrar(): void {
  detenerEscucha?.();
  detenerEscucha = null;

  if (!id) return;

  invoke("cerrar_portapapeles", { id }).catch(() => {});
}

// ======================================================
// 🎨 FONDO TEÑIDO CON EL COLOR DE LA FILA
// ------------------------------------------------------
// Mismo criterio que menu_express_main.ts::aplicarColorFondo —
// mismo vocabulario --tag-<color> de styl_variables.css.
// ======================================================

function aplicarColorFondo(color: string): void {
  const variable = color ? `var(--tag-${color})` : "var(--panel2)";

  card.style.setProperty(
    "--portapapeles-color",
    `color-mix(in srgb, ${variable} 45%, rgba(20, 20, 30, 0.6))`,
  );

  card.style.setProperty(
    "--portapapeles-gradient",
    `linear-gradient(180deg, color-mix(in srgb, ${variable} 35%, rgba(30, 30, 45, 0.6)), color-mix(in srgb, ${variable} 15%, rgba(10, 10, 18, 0.9)))`,
  );
}

// ======================================================
// 📐 TAMAÑOS EN PX (config.rs)
// ------------------------------------------------------
// Se leen una sola vez al iniciar, vía obtener_tamanos_portapapeles
// — mismo patrón que menu_express_main.ts::leerTamanosMenuExpress.
// El tamaño de TEXTO reusa los mismos 3 valores que MenuExpress
// (obtener_tamanos_menu_express — ver config.rs, portapapeles_
// boton_* tiene funciones propias pero portapapeles_texto_* no).
// ======================================================

interface TamanosBoton {
  botonPequeno: { ancho: number; alto: number };
  botonMediano: { ancho: number; alto: number };
  botonGrande: { ancho: number; alto: number };
}

interface TamanosTexto {
  textoPequeno: number;
  textoMediano: number;
  textoGrande: number;
}

// Respaldo — solo se usa si los comandos llegaran a fallar.
let TAMANOS_BOTON: TamanosBoton = {
  botonPequeno: { ancho: 140, alto: 26 },
  botonMediano: { ancho: 180, alto: 32 },
  botonGrande: { ancho: 220, alto: 40 },
};

let TAMANOS_TEXTO: TamanosTexto = {
  textoPequeno: 10,
  textoMediano: 13,
  textoGrande: 16,
};

function tamanoTextoPx(tamano: PortapapelesDatos["tamanoTexto"]): number {
  if (tamano === "pequeno") return TAMANOS_TEXTO.textoPequeno;
  if (tamano === "grande") return TAMANOS_TEXTO.textoGrande;
  return TAMANOS_TEXTO.textoMediano;
}

function alturaBotonPx(tamano: PortapapelesDatos["tamanoBoton"]): number {
  if (tamano === "pequeno") return TAMANOS_BOTON.botonPequeno.alto;
  if (tamano === "grande") return TAMANOS_BOTON.botonGrande.alto;
  return TAMANOS_BOTON.botonMediano.alto;
}

async function leerTamanos(): Promise<void> {
  try {
    const boton = await invoke<{
      botonPequeno: [number, number];
      botonMediano: [number, number];
      botonGrande: [number, number];
    }>("obtener_tamanos_portapapeles");

    TAMANOS_BOTON = {
      botonPequeno: {
        ancho: boton.botonPequeno[0],
        alto: boton.botonPequeno[1],
      },
      botonMediano: {
        ancho: boton.botonMediano[0],
        alto: boton.botonMediano[1],
      },
      botonGrande: { ancho: boton.botonGrande[0], alto: boton.botonGrande[1] },
    };
  } catch {
    // Se queda con el respaldo de arriba.
  }

  try {
    const texto = await invoke<{
      textoPequeno: number;
      textoMediano: number;
      textoGrande: number;
    }>("obtener_tamanos_menu_express");

    TAMANOS_TEXTO = {
      textoPequeno: texto.textoPequeno,
      textoMediano: texto.textoMediano,
      textoGrande: texto.textoGrande,
    };
  } catch {
    // Se queda con el respaldo de arriba.
  }
}

// ======================================================
// 🏗️ ESTRUCTURA BASE (header + cuerpo + barra inferior)
// ======================================================

function construirEstructura(): {
  titulo: HTMLSpanElement;
  cuerpoNuevo: HTMLDivElement;
} {
  raiz.innerHTML = "";

  const nuevaCard = document.createElement("div");
  nuevaCard.className = "portapapeles-card";
  nuevaCard.setAttribute("data-tauri-drag-region", "");

  const header = document.createElement("div");
  header.className = "portapapeles-header";
  header.setAttribute("data-tauri-drag-region", "");

  const titulo = document.createElement("span");
  titulo.className = "portapapeles-titulo";
  titulo.setAttribute("data-tauri-drag-region", "");

  const botonCerrar = document.createElement("button");
  botonCerrar.className = "portapapeles-cerrar";
  botonCerrar.textContent = "×";
  botonCerrar.title = "Cerrar";
  botonCerrar.addEventListener("click", cerrar);

  const botonOtros = document.createElement("button");
  botonOtros.className = "portapapeles-boton-otros";
  botonOtros.textContent = "▼";
  botonOtros.title = "Ver Fijados de Otros Portapapeles";
  botonOtros.addEventListener("click", () => void abrirPopupOtros());

  header.append(titulo, botonOtros, botonCerrar);

  const cuerpoNuevo = document.createElement("div");
  cuerpoNuevo.className = "portapapeles-cuerpo";
  // Sin data-tauri-drag-region: es el contenedor con scroll, y
  // tenerlo acá hacía que arrastrar la barra de desplazamiento
  // arrastrara la ventana en vez de scrollear. La ventana ya se
  // puede arrastrar desde header/card.

  nuevaCard.append(header, cuerpoNuevo);
  raiz.append(nuevaCard);

  card = nuevaCard;
  cuerpo = cuerpoNuevo;

  return { titulo, cuerpoNuevo };
}

// ======================================================
// 👁️ TOOLTIP DE PREVISUALIZACIÓN (hover sobre el nombre)
// ------------------------------------------------------
// spec: "Al pasar el mouse sobre el botón con el nombre del
// elemento debe mostrarse una versión más extendida de su
// contenido en un popup. En el caso de las imágenes una
// miniatura o previsualización."
//
// El texto completo NO viaja en ElementoDatos (solo el nombre
// recortado a 20 caracteres, ya resuelto por back_portapapeles.rs)
// — para texto se re-lee el archivo bajo demanda con el asset
// protocol de Tauri (convertFileSrc + fetch); para imagen se
// muestra directo con <img src="asset://..."> sin necesidad de
// leer el contenido en JS.
// ======================================================

import { convertFileSrc } from "@tauri-apps/api/core";

let tooltipActual: HTMLDivElement | null = null;

// Se incrementa en cada mouseenter/ocultarTooltip para que una
// llamada de mostrarTooltip() que sigue esperando el fetch del
// texto (mouse ya afuera) pueda notar que quedó obsoleta y no
// inserte nada — sin esto, el tooltip quedaba pegado en pantalla
// porque el mouseleave ya había disparado ocultarTooltip() antes
// de que el fetch terminara y el tooltip recién se agregara al DOM.
let tooltipVigencia = 0;

function ocultarTooltip(): void {
  tooltipVigencia++;
  tooltipActual?.remove();
  tooltipActual = null;
}

async function mostrarTooltip(
  elemento: HTMLElement,
  datos: ElementoDatos,
): Promise<void> {
  ocultarTooltip();
  const vigenciaPropia = tooltipVigencia;

  const tooltip = document.createElement("div");
  tooltip.className = "portapapeles-tooltip";

  if (datos.extension === "png") {
    const img = document.createElement("img");
    img.src = convertFileSrc(datos.ruta);
    tooltip.append(img);
  } else {
    tooltip.textContent = "Cargando…";

    try {
      const texto = await fetch(convertFileSrc(datos.ruta)).then((respuesta) =>
        respuesta.text(),
      );

      if (vigenciaPropia !== tooltipVigencia) return;

      // Recorte defensivo: un .txt del pool puede en teoría crecer
      // más allá de lo esperable si se editó a mano fuera de la
      // app — el tooltip no debe volverse gigante por eso.
      tooltip.textContent =
        texto.length > 600 ? `${texto.slice(0, 600)}…` : texto;
    } catch {
      if (vigenciaPropia !== tooltipVigencia) return;
      tooltip.textContent = datos.nombre;
    }
  }

  if (vigenciaPropia !== tooltipVigencia) return;

  document.body.append(tooltip);

  const rect = elemento.getBoundingClientRect();
  const tooltipRect = tooltip.getBoundingClientRect();

  let x = rect.left;
  let y = rect.bottom + 4;

  if (x + tooltipRect.width > window.innerWidth) {
    x = Math.max(0, window.innerWidth - tooltipRect.width - 4);
  }

  if (y + tooltipRect.height > window.innerHeight) {
    y = Math.max(0, rect.top - tooltipRect.height - 4);
  }

  tooltip.style.left = `${x}px`;
  tooltip.style.top = `${y}px`;

  tooltipActual = tooltip;
}

// ======================================================
// 📌 FIJAR / DESFIJAR
// ======================================================

async function alternarFijado(datos: ElementoDatos): Promise<void> {
  if (!id || operacionEnVuelo) return;

  operacionEnVuelo = true;

  try {
    if (datos.fijado) {
      // Desfijar: no depende de bajo qué ID estaba fijado, opera
      // directo sobre la ruta.
      const actualizados = await invoke<PortapapelesDatos | null>(
        "portapapeles_desfijar",
        { id, ruta: datos.ruta },
      );
      await aplicarResultadoMutacionFijado(actualizados);
    } else if (idMostrado && idMostrado !== id) {
      // Vista alternativa: fijar bajo la ID que se está mostrando.
      const actualizados = await invoke<PortapapelesDatos | null>(
        "portapapeles_fijar_como",
        { idVentana: id, idDestino: idMostrado, ruta: datos.ruta },
      );
      await aplicarResultadoMutacionFijado(actualizados);
    } else {
      const actualizados = await invoke<PortapapelesDatos | null>(
        "portapapeles_fijar",
        { id, ruta: datos.ruta },
      );
      if (actualizados) renderizar(actualizados);
    }
  } catch {
    // Sin datos nuevos que mostrar — la lista se queda como estaba.
  } finally {
    operacionEnVuelo = false;
  }
}

// Tras fijar/desfijar mientras se está en vista alternativa, la
// respuesta trae los fijados de la ventana REAL (no los de
// idMostrado) — hay que volver a pedir los de idMostrado aparte y
// pisar solo ese campo antes de renderizar.
async function aplicarResultadoMutacionFijado(
  actualizados: PortapapelesDatos | null,
): Promise<void> {
  if (!actualizados) return;

  if (!idMostrado || idMostrado === id) {
    renderizar(actualizados);
    return;
  }

  let fijadosAlt: ElementoDatos[];
  try {
    fijadosAlt = await invoke<ElementoDatos[]>(
      "portapapeles_listar_fijados_de",
      { idObjetivo: idMostrado },
    );
  } catch {
    fijadosAlt = [];
  }

  fijadosAltActuales = fijadosAlt;

  renderizar({
    ...actualizados,
    fijados: fijadosAlt,
    nombre: nombreMostrado ?? idMostrado,
  });
}

// ======================================================
// 📌➡️📋 PEGAR (click en el nombre)
// ------------------------------------------------------
// spec: "Al clickear en él se pega el contenido del archivo al
// portapapeles y a la ventana activa." El bloqueo anti-duplicado
// real vive del lado Rust (back_portapapeles::pegar ya marca
// ignorar_proximo_cambio antes de escribir) — acá solo se evita
// una segunda invocación en simultáneo mientras la primera sigue
// en vuelo.
// ======================================================

async function pegar(datos: ElementoDatos): Promise<void> {
  if (operacionEnVuelo) return;

  operacionEnVuelo = true;

  try {
    await invoke("portapapeles_pegar", { ruta: datos.ruta });
  } catch (error) {
    // DIAGNÓSTICO TEMPORAL: antes esto se tragaba en silencio.
    // Logueado para ver la causa real en la consola de
    // `npm run tauri dev` mientras se investiga el problema de
    // pegado en Paint — sacar una vez resuelto.
    console.error("portapapeles_pegar falló:", error);
  } finally {
    operacionEnVuelo = false;
  }
}

// ======================================================
// ⋯ POPUP DE OPCIONES (Renombrar / Editar / Eliminar)
// ======================================================

function cerrarPopup(): void {
  document.querySelector(".portapapeles-popup-overlay")?.remove();

  // Espejo de enfocarVentanaParaEdicion() — la ventana vuelve a
  // WS_EX_NOACTIVATE (no roba foco a la app activa del usuario) al
  // cerrar cualquier popup, sea o no uno de los que lo había pedido.
  // Sin costo llamarlo de más: restaurar_no_activacion es un no-op si
  // la ventana ya estaba así.
  if (id) {
    invoke("portapapeles_desenfocar_ventana", { id }).catch(() => {});
  }
}

// spec: "Popup editar y renombrar deben tomar el foco al ser creados
// para poder escribir en ellos." La ventana se crea con
// WS_EX_NOACTIVATE (ver crear_ventana en back_portapapeles.rs) para
// no robarle el foco a la app activa — eso también bloquea el tecleo
// real en cualquier input/textarea de acá, así que antes de enfocar
// el campo del popup hay que pedirle al backend que habilite la
// activación real de la ventana (y le dé el foco).
async function enfocarVentanaParaEdicion(): Promise<void> {
  if (!id) return;

  try {
    await invoke("portapapeles_enfocar_ventana", { id });
  } catch {
    // Sin foco real del SO — el input.focus() de todos modos se
    // intenta más abajo, por si acaso.
  }
}

function abrirPopupOpciones(datos: ElementoDatos): void {
  cerrarPopup();

  const overlay = document.createElement("div");
  overlay.className = "portapapeles-popup-overlay";
  overlay.addEventListener("click", (evento) => {
    if (evento.target === overlay) cerrarPopup();
  });

  const popup = document.createElement("div");
  popup.className = "portapapeles-popup";

  const titulo = document.createElement("div");
  titulo.className = "portapapeles-popup-titulo";
  titulo.textContent = datos.nombre;
  popup.append(titulo);

  const opcionRenombrar = document.createElement("button");
  opcionRenombrar.className = "portapapeles-popup-opcion";
  opcionRenombrar.textContent = "Renombrar";
  opcionRenombrar.addEventListener("click", () => abrirPopupRenombrar(datos));
  popup.append(opcionRenombrar);

  if (datos.extension === "txt") {
    const opcionEditar = document.createElement("button");
    opcionEditar.className = "portapapeles-popup-opcion";
    opcionEditar.textContent = "Editar";
    opcionEditar.addEventListener("click", () => abrirPopupEditar(datos));
    popup.append(opcionEditar);
  }

  const opcionEliminar = document.createElement("button");
  opcionEliminar.className = "portapapeles-popup-opcion peligro";
  opcionEliminar.textContent = "Eliminar";
  opcionEliminar.addEventListener("click", () => eliminarElemento(datos));
  popup.append(opcionEliminar);

  overlay.append(popup);
  card.append(overlay);
}

async function abrirPopupRenombrar(datos: ElementoDatos): Promise<void> {
  cerrarPopup();

  const overlay = document.createElement("div");
  overlay.className = "portapapeles-popup-overlay";

  const popup = document.createElement("div");
  popup.className = "portapapeles-popup";

  const titulo = document.createElement("div");
  titulo.className = "portapapeles-popup-titulo";
  titulo.textContent = "Renombrar";
  popup.append(titulo);

  const input = document.createElement("input");
  input.className = "portapapeles-popup-input";
  input.type = "text";
  input.maxLength = 50;
  input.value = datos.nombre;
  popup.append(input);

  const acciones = document.createElement("div");
  acciones.className = "portapapeles-popup-acciones";

  const botonCancelar = document.createElement("button");
  botonCancelar.className = "portapapeles-popup-boton";
  botonCancelar.textContent = "Cancelar";
  botonCancelar.addEventListener("click", cerrarPopup);

  const botonGuardar = document.createElement("button");
  botonGuardar.className = "portapapeles-popup-boton primario";
  botonGuardar.textContent = "Guardar";
  botonGuardar.addEventListener("click", async () => {
    const nuevoNombre = input.value.trim();
    if (!nuevoNombre || !id) return;

    try {
      const actualizados = await invoke<PortapapelesDatos | null>(
        "portapapeles_renombrar",
        { id, ruta: datos.ruta, nuevoNombre },
      );

      await aplicarResultadoMutacionFijado(actualizados);
    } catch {
      // El popup se cierra igual — no hay nada más coherente que
      // hacer con un renombre fallido acá.
    }

    cerrarPopup();
  });

  acciones.append(botonCancelar, botonGuardar);
  popup.append(acciones);

  overlay.append(popup);
  card.append(overlay);

  await enfocarVentanaParaEdicion();
  input.focus();
  input.select();
}

async function abrirPopupEditar(datos: ElementoDatos): Promise<void> {
  cerrarPopup();

  // spec: "antes que se abra se actualiza su fecha para quedar de
  // los primeros, pero que no se dé la orden de actualizar la ui" —
  // congela este elemento en el tope del orden mientras el popup está
  // abierto, para que aplicar_limite() (si en paralelo hay Registro
  // activo en otra fila) no lo elimine por ser el rotativo más
  // antiguo. portapapeles_marcar_reciente es silencioso a propósito:
  // no dispara refrescar_datos ni notificar_ventanas_abiertas, así
  // que no reordena nada visualmente hasta la próxima actualización
  // real (al Guardar, o cuando llegue algo nuevo).
  if (id) {
    invoke("portapapeles_marcar_reciente", { ruta: datos.ruta }).catch(
      () => {},
    );
  }

  const overlay = document.createElement("div");
  overlay.className = "portapapeles-popup-overlay";

  // spec: "Debe generarse expandido dentro de toda la ventana y
  // cambiar de tamaño con ella" — a diferencia del resto de popups
  // (Opciones/Renombrar), que quedan chicos y centrados, Editar usa
  // el ancho/alto completo del overlay (--popup--editar en
  // portapapeles.css), así que sigue ocupando toda la ventana si el
  // usuario la redimensiona mientras el popup está abierto.
  const popup = document.createElement("div");
  popup.className = "portapapeles-popup portapapeles-popup--editar";

  const titulo = document.createElement("div");
  titulo.className = "portapapeles-popup-titulo";
  // spec: "Al lado del texto Editar debe decir el nombre del
  // elemento que está editando."
  titulo.textContent = `Editar — ${datos.nombre || "(sin nombre)"}`;
  popup.append(titulo);

  const textarea = document.createElement("textarea");
  textarea.className = "portapapeles-popup-textarea";
  textarea.value = "Cargando…";
  popup.append(textarea);

  const acciones = document.createElement("div");
  acciones.className = "portapapeles-popup-acciones";

  const botonCancelar = document.createElement("button");
  botonCancelar.className = "portapapeles-popup-boton";
  botonCancelar.textContent = "Cancelar";
  botonCancelar.addEventListener("click", cerrarPopup);

  const botonGuardar = document.createElement("button");
  botonGuardar.className = "portapapeles-popup-boton primario";
  botonGuardar.textContent = "Guardar";
  botonGuardar.addEventListener("click", async () => {
    if (!id) return;

    try {
      const actualizados = await invoke<PortapapelesDatos | null>(
        "portapapeles_editar",
        { id, ruta: datos.ruta, contenido: textarea.value },
      );

      await aplicarResultadoMutacionFijado(actualizados);
    } catch {
      // El popup se cierra igual — sin datos nuevos que aplicar.
    }

    cerrarPopup();
  });

  acciones.append(botonCancelar, botonGuardar);
  popup.append(acciones);

  overlay.append(popup);
  card.append(overlay);

  try {
    textarea.value = await fetch(convertFileSrc(datos.ruta)).then((respuesta) =>
      respuesta.text(),
    );
  } catch {
    textarea.value = "";
  }

  await enfocarVentanaParaEdicion();
  textarea.focus();
}

async function eliminarElemento(datos: ElementoDatos): Promise<void> {
  cerrarPopup();

  if (!id) return;

  try {
    const actualizados = await invoke<PortapapelesDatos | null>(
      "portapapeles_eliminar",
      { id, ruta: datos.ruta },
    );

    await aplicarResultadoMutacionFijado(actualizados);
  } catch {
    // Sin datos nuevos que mostrar.
  }
}

// ======================================================
// 🆔📌 POPUP "OTROS PORTAPAPELES" (Cambio 2)
// ------------------------------------------------------
// Reutiliza las clases CSS del popup de opciones ya existente
// (portapapeles-popup-overlay/-popup/-popup-titulo/-popup-opcion)
// para heredar el mismo look sin CSS nuevo.
// ======================================================

async function abrirPopupOtros(): Promise<void> {
  if (!id) return;
  cerrarPopup();

  // Se consulta con idMostrado (no `id`): si ya estamos viendo un
  // segundo Portapapeles, ese no debe listarse a sí mismo en el
  // popup (bug: al reabrir el popup desde la vista alternativa,
  // seguía apareciendo el propio Portapapeles que ya se estaba
  // mostrando, en vez de quedar oculto/actualizado).
  const idConsulta = idMostrado ?? id;

  let otros: OtroPortapapeles[];
  try {
    otros = await invoke<OtroPortapapeles[]>("portapapeles_listar_otros", {
      id: idConsulta,
    });
  } catch {
    otros = [];
  }

  const overlay = document.createElement("div");
  overlay.className = "portapapeles-popup-overlay";
  overlay.addEventListener("click", (evento) => {
    if (evento.target === overlay) cerrarPopup();
  });

  const popup = document.createElement("div");
  popup.className = "portapapeles-popup";

  const titulo = document.createElement("div");
  titulo.className = "portapapeles-popup-titulo";
  titulo.textContent = "Fijados de otros Portapapeles";
  popup.append(titulo);

  if (otros.length === 0) {
    const vacio = document.createElement("div");
    vacio.className = "portapapeles-popup-opcion";
    vacio.textContent = "No hay otros Portapapeles con fijados.";
    popup.append(vacio);
  }

  otros.forEach((otro) => {
    const boton = document.createElement("button");
    boton.className = "portapapeles-popup-opcion";
    boton.textContent = otro.nombre ?? otro.id;
    boton.addEventListener("click", () => void verFijadosDe(otro));
    popup.append(boton);
  });

  overlay.append(popup);
  card.append(overlay);
}

// ======================================================
// 🔀 CAMBIAR DE VISTA (switch de ID mostrada) — Cambio 2
// ======================================================

async function verFijadosDe(otro: OtroPortapapeles): Promise<void> {
  cerrarPopup();
  if (!ultimosDatos) return;

  idMostrado = otro.id;
  nombreMostrado = otro.nombre ?? otro.id;

  let fijadosAlt: ElementoDatos[];
  try {
    fijadosAlt = await invoke<ElementoDatos[]>(
      "portapapeles_listar_fijados_de",
      { idObjetivo: otro.id },
    );
  } catch {
    fijadosAlt = [];
  }

  fijadosAltActuales = fijadosAlt;

  // Reusa rotativos/registro/color de ultimosDatos — spec: "diseño
  // y color se mantiene, solo cambian los fijos y el nombre".
  renderizar({
    ...ultimosDatos,
    fijados: fijadosAlt,
    nombre: nombreMostrado ?? otro.id,
  });
}

// ======================================================
// 🔘 CREAR UNA FILA DE ELEMENTO
// ======================================================

function crearFila(
  datos: ElementoDatos,
  alturaBoton: number,
  tamanoTexto: number,
): HTMLDivElement {
  const fila = document.createElement("div");
  fila.className = "portapapeles-fila";
  fila.style.height = `${alturaBoton}px`;

  const botonFijar = document.createElement("button");
  botonFijar.className = "portapapeles-boton-fijar";
  if (datos.fijado) botonFijar.classList.add("fijado");
  botonFijar.textContent = datos.fijado ? "📌" : "○";
  botonFijar.title = datos.fijado ? "Desfijar" : "Fijar";
  botonFijar.addEventListener("click", () => alternarFijado(datos));

  const botonNombre = document.createElement("button");
  botonNombre.className = "portapapeles-boton-nombre";
  botonNombre.textContent = datos.nombre || "(sin nombre)";
  botonNombre.style.fontSize = `${tamanoTexto}px`;
  botonNombre.addEventListener("click", () => pegar(datos));

  // Debounce de 500ms: si el mouse sale antes de que se cumpla, se
  // cancela sin llegar a mostrar el tooltip.
  let temporizadorTooltip: ReturnType<typeof setTimeout> | null = null;

  botonNombre.addEventListener("mouseenter", () => {
    temporizadorTooltip = setTimeout(() => {
      temporizadorTooltip = null;
      void mostrarTooltip(botonNombre, datos);
    }, 500);
  });
  botonNombre.addEventListener("mouseleave", () => {
    if (temporizadorTooltip !== null) {
      clearTimeout(temporizadorTooltip);
      temporizadorTooltip = null;
    }
    ocultarTooltip();
  });

  const botonOpciones = document.createElement("button");
  botonOpciones.className = "portapapeles-boton-opciones";
  botonOpciones.textContent = "⋯";
  botonOpciones.title = "Opciones";
  botonOpciones.addEventListener("click", () => abrirPopupOpciones(datos));

  fila.append(botonFijar, botonNombre, botonOpciones);

  return fila;
}

// ======================================================
// 🔀 TOGGLE MODO REGISTRO / LIMPIAR TODO
// ======================================================

async function alternarRegistro(): Promise<void> {
  if (!id || !ultimosDatos || operacionEnVuelo) return;

  operacionEnVuelo = true;

  try {
    const actualizados = await invoke<PortapapelesDatos | null>(
      "portapapeles_toggle_registro",
      {
        id,
        activar: !ultimosDatos.registroActivo,
        limite: ultimosDatos.limite,
      },
    );

    if (actualizados) renderizar(actualizados);
  } catch {
    // Sin datos nuevos que mostrar.
  } finally {
    operacionEnVuelo = false;
  }
}

async function limpiarTodo(): Promise<void> {
  if (!id || operacionEnVuelo) return;

  operacionEnVuelo = true;

  try {
    const actualizados = await invoke<PortapapelesDatos | null>(
      "portapapeles_limpiar_todo",
      { id },
    );

    if (actualizados) renderizar(actualizados);
  } catch {
    // Sin datos nuevos que mostrar.
  } finally {
    operacionEnVuelo = false;
  }
}

// ======================================================
// 🖼️ PINTAR LISTADO (fijados + separador + rotativos)
// ------------------------------------------------------
// Solo toca `cuerpo` (fijados/rotativos) — nunca `card` completo,
// para que una actualización EN VIVO (actualizarEnVivo, más abajo)
// pueda llamar esto sin arrancar de encima un popup abierto, que
// vive como hijo directo de `card`, no de `cuerpo`.
// ======================================================

function pintarListado(datos: PortapapelesDatos): void {
  ocultarTooltip();
  cuerpo.innerHTML = "";

  const alturaBoton = alturaBotonPx(datos.tamanoBoton);
  const tamanoTexto = tamanoTextoPx(datos.tamanoTexto);

  if (datos.fijados.length === 0 && datos.rotativos.length === 0) {
    const vacio = document.createElement("div");
    vacio.className = "portapapeles-vacio";
    vacio.textContent = "Portapapel vacío";
    cuerpo.append(vacio);
    return;
  }

  datos.fijados.forEach((elemento) => {
    cuerpo.append(crearFila(elemento, alturaBoton, tamanoTexto));
  });

  if (datos.fijados.length > 0 && datos.rotativos.length > 0) {
    const separador = document.createElement("div");
    separador.className = "portapapeles-separador";
    cuerpo.append(separador);
  }

  datos.rotativos.forEach((elemento) => {
    cuerpo.append(crearFila(elemento, alturaBoton, tamanoTexto));
  });
}

// ======================================================
// 🔀 BARRA INFERIOR (Modo Registro / Limpiar todo)
// ------------------------------------------------------
// Se crea una sola vez por render completo (crearBarraInferior);
// actualizarBotonRegistro solo cambia texto/clase del botón ya
// existente — es lo que usa actualizarEnVivo para no duplicar la
// barra en cada evento.
// ======================================================

// Texto del switch Registro — nodo de texto propio (no
// botonRegistro.textContent a secas) porque el botón también carga
// la pista+bolita del switch (ver más abajo); pisar todo el
// textContent se las llevaría puestas.
function actualizarBotonRegistro(datos: PortapapelesDatos): void {
  if (!botonRegistroActual || !textoRegistroActual) return;

  // Mismo vocabulario data-activo que .popup-switch (ver
  // comp_popup_grupo.ts::crearInterruptor / styl_layout.css) — acá se
  // reimplementa en portapapeles.css porque esta ventana no importa
  // styl_general.css (rompería la transparencia, ver nota al inicio
  // del archivo de estilos).
  botonRegistroActual.dataset.activo = datos.registroActivo ? "true" : "false";
  textoRegistroActual.textContent = datos.registroActivo
    ? "Registro ON"
    : "Registro Off";
}

function crearBarraInferior(datos: PortapapelesDatos): void {
  const barraInferior = document.createElement("div");
  barraInferior.className = "portapapeles-barra-inferior";

  // spec: "Debe tener el estilo del switch 'Coordenada' del popup
  // extra (Ese boton deslizante gris que pasa a Cyan)" — misma
  // estructura pista+bolita que crearInterruptor() en
  // comp_popup_grupo.ts (ver estilos espejados en portapapeles.css).
  const botonRegistro = document.createElement("button");
  botonRegistro.className =
    "portapapeles-boton-barra portapapeles-switch-registro";

  const pista = document.createElement("span");
  pista.className = "portapapeles-switch-pista";

  const bolita = document.createElement("span");
  bolita.className = "portapapeles-switch-bolita";
  pista.append(bolita);

  const textoRegistro = document.createElement("span");

  botonRegistro.append(pista, textoRegistro);
  botonRegistro.addEventListener("click", alternarRegistro);

  // spec: "Boton Limpiar todo debe tener doble confirmación: Al pasar
  // el mouse sobre él cambia letras y borde rojo (CSS puro, ver
  // .portapapeles-boton-limpiar:hover), al hacer click dice
  // '⚠️ Confirmar limpieza' con letras rojas, siguiente click limpia
  // rotatorios."
  const botonLimpiar = document.createElement("button");
  botonLimpiar.className =
    "portapapeles-boton-barra portapapeles-boton-limpiar";
  botonLimpiar.textContent = "Limpiar todo";

  let confirmando = false;

  const salirDeConfirmacion = () => {
    if (!confirmando) return;
    confirmando = false;
    botonLimpiar.classList.remove("confirmando");
    botonLimpiar.textContent = "Limpiar todo";
  };

  botonLimpiar.addEventListener("click", () => {
    if (!confirmando) {
      confirmando = true;
      botonLimpiar.classList.add("confirmando");
      botonLimpiar.textContent = "⚠️ Confirmar limpieza";
      return;
    }

    salirDeConfirmacion();
    limpiarTodo();
  });

  // Si el usuario se va sin confirmar, el segundo click no debe
  // quedar "armado" para la próxima vez que pase el mouse por acá.
  botonLimpiar.addEventListener("mouseleave", salirDeConfirmacion);

  barraInferior.append(botonRegistro, botonLimpiar);
  card.append(barraInferior);

  botonRegistroActual = botonRegistro;
  textoRegistroActual = textoRegistro;
  actualizarBotonRegistro(datos);
}

// ======================================================
// 🖼️ RENDERIZAR (completo)
// ------------------------------------------------------
// Reconstruye la ventana entera (header + listado + barra inferior),
// cerrando cualquier popup abierto de paso — se usa al iniciar y como
// resultado directo de una acción propia del usuario en esta misma
// ventana (fijar/renombrar/editar/eliminar/limpiar/toggle Registro),
// donde ese cierre es esperable porque la acción es la que lo generó.
// ======================================================

function renderizar(datos: PortapapelesDatos): void {
  ultimosDatos = datos;

  const { titulo } = construirEstructura();
  tituloActual = titulo;

  titulo.textContent = datos.nombre || "Portapapeles";
  aplicarColorFondo(datos.color);

  pintarListado(datos);
  crearBarraInferior(datos);
}

// ======================================================
// 🔴 ACTUALIZAR EN VIVO (evento "portapapeles-actualizado") — J.2
// ------------------------------------------------------
// spec: "Cuando se llama a actualizar la ventana solo debe
// actualizarse los listados de elementos Fijos y rotativos. Si llega
// a haber abierto un popup de editar o renombrar no debe cerrarse."
// A diferencia de renderizar(), NUNCA llama a construirEstructura()
// — no toca el popup (hijo de `card`, no de `cuerpo`).
// ======================================================

function actualizarEnVivo(datos: PortapapelesDatos): void {
  ultimosDatos = datos;

  if (!idMostrado || idMostrado === id) {
    if (tituloActual) tituloActual.textContent = datos.nombre || "Portapapeles";
    pintarListado(datos);
  } else {
    // Vista alternativa activa: los rotativos sí vienen de `datos`
    // (pool global, sigue siendo válido), pero los fijados/nombre
    // mostrados NO deben pisarse con los de la ventana real.
    // Limitación conocida: si llega un evento en vivo estando en
    // vista alternativa, los FIJADOS mostrados no se refrescan en
    // tiempo real (p. ej. si alguien fijó algo bajo esa misma ID
    // desde otra ventana) — se muestran los últimos obtenidos por
    // verFijadosDe/aplicarResultadoMutacionFijado. Queda pendiente
    // para una vuelta futura, no es parte de lo pedido.
    if (tituloActual) {
      tituloActual.textContent =
        nombreMostrado || datos.nombre || "Portapapeles";
    }
    pintarListado({
      ...datos,
      fijados: fijadosAltActuales,
      nombre: nombreMostrado ?? datos.nombre,
    });
  }

  aplicarColorFondo(datos.color);
  actualizarBotonRegistro(datos);
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  await leerTamanos();

  if (!id) {
    const { titulo, cuerpoNuevo } = construirEstructura();
    titulo.textContent = "Portapapeles";
    const error = document.createElement("div");
    error.className = "portapapeles-vacio";
    error.textContent = "Falta el id del Portapapeles en la URL.";
    cuerpoNuevo.append(error);
    return;
  }

  let datos: PortapapelesDatos | null;

  try {
    datos = await invoke<PortapapelesDatos | null>(
      "obtener_datos_portapapeles",
      {
        id,
      },
    );
  } catch {
    datos = null;
  }

  // No debería pasar (back_portapapeles registra los datos ANTES de
  // crear esta ventana) — pero si el Portapapeles ya se cerró/
  // recompiló justo en el medio, no hay nada coherente que mostrar.
  if (!datos) {
    const { titulo, cuerpoNuevo } = construirEstructura();
    titulo.textContent = "Portapapeles";
    const error = document.createElement("div");
    error.className = "portapapeles-vacio";
    error.textContent = "Este Portapapeles ya no está disponible.";
    cuerpoNuevo.append(error);
    return;
  }

  renderizar(datos);

  // El evento ya viene filtrado por ventana (back_portapapeles.rs usa
  // emit_to() con el label de esta ventana, ver notificar_ventanas_
  // abiertas()) — no hace falta comparar ids del lado del cliente.
  try {
    detenerEscucha = await listen<PortapapelesDatos>(
      "portapapeles-actualizado",
      (evento) => actualizarEnVivo(evento.payload),
    );
  } catch {
    // Sin escucha en vivo — la ventana sigue funcionando igual con
    // las actualizaciones que ya llegan como retorno directo de cada
    // comando (fijar/renombrar/editar/eliminar/limpiar/toggle).
  }
}

iniciar();
