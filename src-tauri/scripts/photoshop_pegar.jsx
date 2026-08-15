// ======================================================
// 🎨 photoshop_pegar.jsx
// ------------------------------------------------------
// Se embebe en el binario de RemapH (ver back_pegado_
// personalizado.rs, include_str!) y se ejecuta con Photoshop
// ya abierto como instancia activa. No recibe ningún argumento:
// pegar() ya escribió el contenido al portapapeles del sistema
// ANTES de que este script corra, así que solo hace falta
// pegarlo en el documento activo.
// ======================================================

if (app.documents.length > 0) {
    app.activeDocument.paste();
}