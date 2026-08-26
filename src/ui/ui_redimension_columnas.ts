// ======================================================
// ↔️ ui_Redimension_Columnas
// ======================================================

import type { Columna } from "./ui_columnas";

export const ANCHOS_DEFAULT: Record<string, number> = {
  //Ancho default al doble click en mover separador
  estado: 58,
  opciones: 42,

  app: 52,
  trigger: 150,

  tipo: 60,
  accion: 150,
  extra: 52,

  nota: 220,
};

export const ANCHO_MINIMO = 52;

// E1: generalizada para recibir columnas/anchosDefault por parámetro
// en vez de importar COLUMNAS/ANCHOS_DEFAULT fijos (permite reusar el
// mecanismo de arrastre en la tabla en árbol de Configuración, cuyas
// columnas son otras). selectorCelda: la tabla principal usa
// ".cabecera-celda"; la tabla en árbol usa ".configuracion-arbol-celda"
// (ver E4) — sin este parámetro, el querySelectorAll de abajo no
// encontraría celdas en la tabla en árbol y activarRedimensionColumnas
// quedaría sin efecto ahí.
export function activarRedimensionColumnas(
  cabecera: HTMLElement,
  columnas: Columna[],
  anchosDefault: Record<string, number>,
  selectorCelda: string = ".cabecera-celda",
): void {
  const celdas = cabecera.querySelectorAll<HTMLElement>(selectorCelda);

  function obtenerVariable(indice: number): string {
    return columnas[indice].ancho.replace("var(", "").replace(")", "");
  }

  function iniciarArrastre(inicioX: number, indice: number) {
    const variable = obtenerVariable(indice);

    const estilos = getComputedStyle(document.documentElement);

    const anchoInicial = parseFloat(estilos.getPropertyValue(variable));

    const mover = (evento: MouseEvent) => {
      const nuevo = Math.max(
        ANCHO_MINIMO,

        anchoInicial + evento.clientX - inicioX,
      );

      document.documentElement.style.setProperty(
        variable,

        `${nuevo}px`,
      );
    };

    const soltar = () => {
      window.removeEventListener("mousemove", mover);

      window.removeEventListener("mouseup", soltar);
    };

    window.addEventListener("mousemove", mover);

    window.addEventListener("mouseup", soltar);
  }

  function restaurarAncho(indice: number) {
    const columna = columnas[indice];

    const valor = anchosDefault[columna.id];

    if (!valor) {
      return;
    }

    document.documentElement.style.setProperty(
      columna.ancho.replace("var(", "").replace(")", ""),

      `${valor}px`,
    );
  }

  celdas.forEach((celda, indice) => {
    // Nota siempre ocupa espacio restante

    if (indice === celdas.length - 1) {
      return;
    }

    const divisor = document.createElement("div");

    divisor.className = "divisor-columna";

    celda.append(divisor);

    divisor.addEventListener("mousedown", (evento) => {
      evento.preventDefault();

      iniciarArrastre(evento.clientX, indice);
    });

    divisor.addEventListener("dblclick", () => {
      restaurarAncho(indice);
    });
  });
}
