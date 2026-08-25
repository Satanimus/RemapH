// ======================================================
// ∴💬 comp_Tooltip_Extra
// ------------------------------------------------------
// Tooltip flotante custom para el botón Extra (ícono ∴) de la
// tabla — reemplaza el title nativo del navegador porque este
// no permite colorear el "Subtítulo:" de cada línea distinto
// del valor. Mismo patrón de posicionamiento (fixed, clamp al
// viewport) que .portapapeles-tooltip (ver
// vent_portapapeles_main.ts), pero síncrono y sin imagen: el
// texto ya viene armado como líneas "Subtítulo: Valor"
// separadas por "\n" (ver texto*Extra() en cada core_*.ts) —
// acá solo se parte cada línea en el primer ": " para pintar
// la parte de la izquierda.
// ======================================================

let tooltipActivo: HTMLElement | null = null;

function ocultarTooltipExtra(): void {
  tooltipActivo?.remove();
  tooltipActivo = null;
}

function crearLineaTooltip(linea: string): HTMLElement {
  const contenedor = document.createElement("div");
  contenedor.className = "extra-tooltip-linea";

  const separador = linea.indexOf(": ");
  if (separador === -1) {
    contenedor.textContent = linea;
    return contenedor;
  }

  const subtitulo = document.createElement("span");
  subtitulo.className = "extra-tooltip-subtitulo";
  subtitulo.textContent = linea.slice(0, separador + 2);

  contenedor.append(
    subtitulo,
    document.createTextNode(linea.slice(separador + 2)),
  );
  return contenedor;
}

function mostrarTooltipExtra(elemento: HTMLElement, texto: string): void {
  ocultarTooltipExtra();

  const tooltip = document.createElement("div");
  tooltip.className = "extra-tooltip";
  texto.split("\n").forEach((linea) => {
    tooltip.append(crearLineaTooltip(linea));
  });

  document.body.append(tooltip);
  tooltipActivo = tooltip;

  const rect = elemento.getBoundingClientRect();
  const tooltipRect = tooltip.getBoundingClientRect();

  let x = rect.left;
  let y = rect.bottom + 4;

  if (x + tooltipRect.width > window.innerWidth) {
    x = Math.max(0, window.innerWidth - tooltipRect.width - 4);
  }
  if (y + tooltipRect.height > window.innerHeight) {
    y = rect.top - tooltipRect.height - 4;
  }

  tooltip.style.left = `${x}px`;
  tooltip.style.top = `${y}px`;
}

// ======================================================
// 🔌 ENGANCHE (hover sobre el botón Extra)
// ------------------------------------------------------
// El botón se recrea entero cada vez que cambia una opción del
// popup (reconstruirFila reconstruye toda la celda, ver
// ui_tabla_control.ts) — no hace falta releer el texto en cada
// hover, el string queda fijo para la vida de ESTE botón.
// ======================================================
export function activarTooltipExtra(boton: HTMLElement, texto: string): void {
  boton.addEventListener("mouseenter", () => {
    mostrarTooltipExtra(boton, texto);
  });
  boton.addEventListener("mouseleave", ocultarTooltipExtra);
  boton.addEventListener("click", ocultarTooltipExtra);
}
