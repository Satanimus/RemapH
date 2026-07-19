// ======================================================
// 🔵 comp_Indicador RemapH V3
// ------------------------------------------------------
// Lucecita de estado reutilizable (verde/rojo).
// No sabe qué representa: solo pinta según el booleano
// que le pasen. Ej: caché activa, captura en curso, etc.
// ======================================================

export function crearIndicador(clase:string):HTMLElement{

    const punto=document.createElement("span");

    punto.className=`indicador ${clase}`;
    punto.dataset.estado="inactivo";

    return punto;

}

export function actualizarIndicador(
    punto:HTMLElement,
    activo:boolean
):void{

    punto.dataset.estado=activo?"activo":"inactivo";

}