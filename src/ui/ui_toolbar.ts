// ======================================================
// 🧰 ui_Toolbar RemapH V3
// ------------------------------------------------------
// Barra superior principal.
//
// Estados del perfil:
//
// 🟢 PERFIL ACTIVO
// 🟡 PERFIL EDITADO
// 🔴 PERFIL PAUSADO
// ======================================================


// ======================================================
// 🚀 CREAR TOOLBAR
// ======================================================

export function crearToolbar(

    alCrearFila:
        () => void,

    alGuardar:
        () => Promise<void>

):

    HTMLElement

{

    const toolbar =
        document.createElement(

            "header"

        );


    toolbar.className =
        "toolbar";


    toolbar.innerHTML = `

        <div class="toolbar-left">

            <div class="titulo">
                RemapH V3
            </div>

        </div>


        <div class="toolbar-center">

            <button class="perfil-selector">
                Default ▾
            </button>


            <button class="perfil-estado">
                PERFIL ACTIVO
            </button>

        </div>


        <div class="toolbar-right">

            <button class="btn-nueva-fila">
                + Fila
            </button>


            <button class="configuracion">
                ⚙
            </button>

        </div>

    `;


    const botonNuevaFila =
        toolbar.querySelector<HTMLButtonElement>(

            ".btn-nueva-fila"

        );


    botonNuevaFila?.addEventListener(

        "click",

        () => {

            alCrearFila();

        }

    );


    const botonEstado =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-estado"

        );


    botonEstado?.addEventListener(

        "click",

        async () => {

            if (

                botonEstado.dataset.estado !==
                "editado"

            ) {

                return;

            }


            botonEstado.disabled =
                true;


            try {

                await alGuardar();


                marcarPerfilActivo(

                    toolbar

                );

            }

            catch(error) {

                console.error(

                    "❌ No se pudo guardar el perfil:",

                    error

                );

            }

            finally {

                botonEstado.disabled =
                    false;

            }

        }

    );


    return toolbar;

}


// ======================================================
// 🟡 MARCAR PERFIL EDITADO
// ======================================================

export function marcarPerfilEditado(

    toolbar:
        HTMLElement

):

    void

{

    const botonEstado =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-estado"

        );


    if (!botonEstado) {

        return;

    }


    botonEstado.textContent =
        "PERFIL EDITADO";


    botonEstado.dataset.estado =
        "editado";

}


// ======================================================
// 🟢 MARCAR PERFIL ACTIVO
// ======================================================

export function marcarPerfilActivo(

    toolbar:
        HTMLElement

):

    void

{

    const botonEstado =
        toolbar.querySelector<HTMLButtonElement>(

            ".perfil-estado"

        );


    if (!botonEstado) {

        return;

    }


    botonEstado.textContent =
        "PERFIL ACTIVO";


    botonEstado.dataset.estado =
        "activo";

}