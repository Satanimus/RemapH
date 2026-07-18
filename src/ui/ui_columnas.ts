// ======================================================
// 📋 ui_Columnas RemapH V3
// ------------------------------------------------------
// Fuente única de verdad para cabecera y filas.
// ======================================================

export type GrupoColumna=
    "general"|
    "input"|
    "respuesta"|
    "personalizacion";

export interface Columna{

    id:string;
    titulo:string;
    grupo:GrupoColumna;
    ancho:string;
}


export const COLUMNAS:Columna[]=[

    {
        id:"numero",
        titulo:"#",
        grupo:"general",
        ancho:"var(--col-num)"
    },

    {
        id:"estado",
        titulo:"Estado",
        grupo:"general",
        ancho:"var(--col-state)"
    },

    {
        id:"app",
        titulo:"App",
        grupo:"input",
        ancho:"var(--col-app)"
    },

    {
        id:"trigger",
        titulo:"Disparador",
        grupo:"input",
        ancho:"var(--col-trigger)"
    },

    {
        id:"tipo",
        titulo:"Tipo",
        grupo:"respuesta",
        ancho:"var(--col-type)"
    },

    {
        id:"accion",
        titulo:"Acción",
        grupo:"respuesta",
        ancho:"var(--col-action)"
    },

    {
        id:"ejecucion",
        titulo:"Ejecución",
        grupo:"respuesta",
        ancho:"var(--col-behavior)"
    },

    {
        id:"color",
        titulo:"Color",
        grupo:"personalizacion",
        ancho:"var(--col-color)"
    },

    {
        id:"nota",
        titulo:"Nota",
        grupo:"personalizacion",
        ancho:"auto"
    }

];