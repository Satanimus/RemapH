// ======================================================
// 📢 ui_Statusbar RemapH V3
// ======================================================

export function crearStatusbar():HTMLElement {

    const status=document.createElement("footer");

    status.className="statusbar";

    status.textContent=
        "Perfil activo.";

    return status;

}