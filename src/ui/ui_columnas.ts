// ======================================================
// 📋 ui_Columnas
// ------------------------------------------------------
// Fuente única de verdad para cabecera y filas.
// ======================================================

export type GrupoColumna =
  | "general"
  | "input"
  | "respuesta"
  | "personalizacion";

export interface Columna {
  id: string;
  titulo: string;
  grupo: GrupoColumna;
  ancho: string;
}

export const COLUMNAS: Columna[] = [
  {
    id: "estado",
    titulo: "Estado",
    grupo: "general",
    ancho: "var(--col-state)",
  },

  {
    id: "opciones",
    titulo: "⁝",
    grupo: "general",
    ancho: "var(--col-options)",
  },

  {
    id: "app",
    titulo: "App",
    grupo: "input",
    ancho: "var(--col-app)",
  },

  {
    id: "trigger",
    titulo: "Disparador",
    grupo: "input",
    ancho: "var(--col-trigger)",
  },

  {
    id: "tipo",
    titulo: "Tipo",
    grupo: "respuesta",
    ancho: "var(--col-type)",
  },

  {
    id: "accion",
    titulo: "Acción",
    grupo: "respuesta",
    ancho: "var(--col-action)",
  },

  {
    id: "extra",
    titulo: "Extra",
    grupo: "respuesta",
    ancho: "var(--col-behavior)",
  },

  {
    id: "nota",
    titulo: "Nota",
    grupo: "personalizacion",
    ancho: "auto",
  },
];
