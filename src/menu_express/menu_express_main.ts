// ======================================================
// ⚡🪟 menu_express_Main
// ------------------------------------------------------
// Punto de entrada de la ventana flotante de MenuExpress
// (menu_express.html — página independiente, ver
// vite.config.ts). El id del menú a mostrar viaja en la URL
// (?id=...) — back_menu_express.rs arma esa URL al crear la
// ventana (ver crear_ventana()).
//
// Los datos (nombre/botones/forma/etc.) se leen UNA sola vez
// al cargar, vía obtener_datos_menu_express — mismo patrón que
// captura_main.ts con obtener_config_captura_activa: el
// backend ya dejó los datos listos ANTES de crear la ventana
// (back_menu_express::abrir_o_alternar registra antes de
// llamar a crear_ventana), sin carrera posible.
//
// ETAPA 5: layout lista vertical simple, sin radial/cuadrícula
// (llega en la etapa 6) y sin ejecución real de los botones al
// hacer clic (llega en la etapa 7, junto con Mantenido/Turbo
// emulados y el cierre real Toggle/Efímero tras el up).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import "../styles/styl_variables.css";
import "./menu_express.css";

// ======================================================
// 🧭 TIPOS
// ------------------------------------------------------
// Espejo de MenuExpressDatosUI / MenuBotonUI en
// back_menu_express.rs (mismo vocabulario string que
// core_menu_express.ts).
// ======================================================

interface MenuBotonDatos {
  filaId: string;

  renombrar: string;
}

interface MenuExpressDatos {
  nombre: string;

  botones: MenuBotonDatos[];

  forma: "radial" | "cuadricula";

  columnas: number;

  filas: number;

  comportamiento: "toggle" | "efimero";

  ubicacion: "persistente" | "cursor";

  tamanoBoton: "pequeno" | "mediano" | "grande";

  tamanoTexto: "pequeno" | "mediano" | "grande";

  color: string;
}

// ======================================================
// 🆔 ID DEL MENÚ (query string)
// ======================================================

const id = new URLSearchParams(window.location.search).get("id");

// ======================================================
// 🏗️ ARMAR DOM BASE
// ======================================================

const raiz = document.getElementById("menu-express")!;

const card = document.createElement("div");
card.className = "menu-express-card";
card.setAttribute("data-tauri-drag-region", "");

const header = document.createElement("div");
header.className = "menu-express-header";
header.setAttribute("data-tauri-drag-region", "");

const titulo = document.createElement("span");
titulo.className = "menu-express-titulo";
titulo.setAttribute("data-tauri-drag-region", "");

const botonCerrar = document.createElement("button");
botonCerrar.className = "menu-express-cerrar";
botonCerrar.textContent = "×";
botonCerrar.title = "Cerrar";

header.append(titulo, botonCerrar);

const cuerpo = document.createElement("div");
cuerpo.className = "menu-express-cuerpo";

card.append(header, cuerpo);
raiz.append(card);

// ======================================================
// 🚪 CERRAR
// ======================================================

function cerrar(): void {
  if (!id) return;

  invoke("cerrar_menu_express", { id }).catch(() => {});
}

botonCerrar.addEventListener("click", cerrar);

// ======================================================
// 🎨 FONDO TEÑIDO CON EL COLOR DE LA FILA
// ------------------------------------------------------
// Mismo vocabulario que la paleta de color de fila
// (--tag-<color> en styl_variables.css). color-mix() para la
// transparencia — el fallback sólido en menu_express.css cubre
// navegadores sin soporte.
// ======================================================

function aplicarColorFondo(color: string): void {
  const variable = color ? `var(--tag-${color})` : "var(--panel2)";

  card.style.setProperty(
    "--menu-color",
    `color-mix(in srgb, ${variable} 55%, rgba(20, 20, 30, 0.85))`,
  );
}

// ======================================================
// 🔘 RENDERIZAR BOTONES (lista simple — etapa 5)
// ------------------------------------------------------
// Radial/Cuadrícula (según datos.forma) llegan en la etapa 6.
// Acá siempre se dibuja como lista vertical, sin importar la
// forma elegida — es una simplificación deliberada de esta
// etapa, no el comportamiento final.
// ======================================================

function renderizarBotones(datos: MenuExpressDatos): void {
  cuerpo.innerHTML = "";

  if (datos.botones.length === 0) {
    const vacio = document.createElement("div");
    vacio.className = "menu-express-vacio";
    vacio.textContent = "Este menú no tiene botones.";
    cuerpo.append(vacio);
    return;
  }

  datos.botones.forEach((boton) => {
    const elemento = document.createElement("button");

    elemento.className = `menu-express-boton tam-${datos.tamanoBoton} txt-${datos.tamanoTexto}`;
    elemento.textContent = boton.renombrar || "(sin nombre)";
    elemento.title = boton.renombrar;

    // Placeholder de esta etapa: la ejecución real (buscar la fila
    // en la caché compilada y llamar a runtime::ejecutar vía
    // menu_express_boton_down/up) llega en la etapa 7. Por ahora
    // solo confirma que el dato de cada botón (filaId) llegó bien
    // hasta acá.
    elemento.addEventListener("click", () => {
      console.log(
        `[MenuExpress] clic en botón (placeholder, sin ejecutar) filaId=${boton.filaId}`,
      );
    });

    cuerpo.append(elemento);
  });
}

// ======================================================
// 🏁 INICIAR
// ======================================================

async function iniciar(): Promise<void> {
  if (!id) {
    titulo.textContent = "Menú";
    cuerpo.innerHTML = "";
    const error = document.createElement("div");
    error.className = "menu-express-vacio";
    error.textContent = "Falta el id del menú en la URL.";
    cuerpo.append(error);
    return;
  }

  let datos: MenuExpressDatos | null;

  try {
    datos = await invoke<MenuExpressDatos | null>(
      "obtener_datos_menu_express",
      { id },
    );
  } catch {
    datos = null;
  }

  // No debería pasar (back_menu_express registra los datos ANTES de
  // crear esta ventana) — pero si el menú ya se cerró/recompiló
  // justo en el medio, no hay nada coherente que mostrar.
  if (!datos) {
    titulo.textContent = "Menú";
    cuerpo.innerHTML = "";
    const error = document.createElement("div");
    error.className = "menu-express-vacio";
    error.textContent = "Este menú ya no está disponible.";
    cuerpo.append(error);
    return;
  }

  titulo.textContent = datos.nombre || "Menú";
  aplicarColorFondo(datos.color);
  renderizarBotones(datos);
}

iniciar();
