////////////////////Encabezado por archivo://////////////////////////////////////////////

1-Nombre archivo

2- Resumen general de qué hace

3- Qué informacion recibe/quien lo llama/qué va antes

4-Que información sale al final.

5- una lista de cada función y lo que hace en pocas palabras

En cada uno de ellos agrega un ejemplo del formato de info que maneja o como se transforma.

El objetivo: cada archivo será una etapa. Cada etapa tendrá su resumen y detalle en el encabezado. Si queda algo pendiente de analizar lo notaremos al instante por incongruencias. Si hay que modificar algo luego sabremos al instante donde ubicarlo.

////////resumen de 12 archivos en orden///////////////////////////////////////////////////
Orden definitivo
ETAPA 1 — Base del modelo

Estos archivos definen las estructuras que utilizarán todos los demás.

eventos.rs
evento_trigger.rs
perfilcache.rs
ETAPA 2 — Cache

Aquí definimos cómo responde la cache.

cache.rs

Aquí desaparecerá la lógica antigua (buscar, tiene_condiciones_posibles, etc.) y quedará únicamente la API definitiva que utilizará el Analizador.

ETAPA 3 — Analizador

Es el corazón del sistema.

analizador_trigger.rs

Este será probablemente el archivo más grande de la migración.

Aquí quedará toda la lógica de:

coincidencias
prefijos
candidatos
doble
mantenido
simple
liberación

Tanto Runtime como Captura usarán exactamente este motor.

ETAPA 4 — Captura

Ahora que el Analizador ya sabe interpretar secuencias completas.

capturador_trigger.rs
captura.rs

Estos archivos quedarán muy pequeños.

Captura solamente almacenará InputEvent.

Nada más.

ETAPA 5 — Entrada

Cuando todo lo anterior ya exista.

entrada.rs

Aquí simplemente elegirá:

Modo captura

↓

guardar InputEvent

↓

timeout

↓

analizador

o

Modo runtime

↓

analizador

No habrá lógica duplicada.

ETAPA 6 — Runtime

Cuando el Analizador ya entregue un EventoTrigger definitivo.

runtime.rs

Aquí prácticamente desaparecerá toda la lógica relacionada con triggers.

Runtime solamente hará:

EventoTrigger

↓

buscar acción

↓

emitir
ETAPA 7 — Salida
back_salida.rs

Aquí probablemente no cambie casi nada.

ETAPA 8 — Diccionarios
pulsadores.rs

Sólo si es necesario.

ETAPA 9 — Configuración
config.rs

Ajustar tiempos si hace falta.

////////////////////////////////////////////////////////////////
PLAN — Refactorización Capturador / Analizador V3
Objetivo general

Eliminar toda la lógica duplicada entre Capturador y Analizador.

Al finalizar:

existe un solo algoritmo que interpreta triggers.
el Runtime y el botón Capturar usan exactamente el mismo analizador.
el Capturador sólo graba InputEvents.
ETAPA 1 — Simplificar Capturador
Objetivo

Convertir Capturador en un grabador de eventos.

Archivos
capturador_trigger.rs
Cambios

Eliminar completamente:

construir()
inputs_down()
EventoTrigger::simple()
cualquier decisión sobre modificadores
cualquier decisión sobre gatillo

Agregar solamente:

eventos() -> &[InputEvent]

para entregar el buffer completo.

Debe conservar:

recibir()
comprobar_timeout()
limpiar()
Resultado esperado

Capturador deja de conocer:

Trigger
Modificadores
Gatilo
Simple
Doble
Mantenido

Sólo conoce:

Vec<InputEvent>
Punto crítico

Ninguno.

Sólo afecta un archivo.

ETAPA 2 — Enseñar al Analizador a analizar capturas
Objetivo

Mover TODA la interpretación al Analizador.

Archivos
analizador_trigger.rs
Cambios

Agregar:

analizar_captura(eventos)

Esta función:

recibe un Vec<InputEvent>
reconstruye presionados
aplica exactamente el mismo algoritmo
devuelve EventoTrigger

No consulta Runtime.

No ejecuta acciones.

No consulta captura.

Sólo interpreta.

Punto crítico

Aquí desaparece definitivamente la lógica duplicada.

ETAPA 3 — Conectar Captura
Objetivo

Cambiar Entrada.

Archivos
entrada.rs

Actualmente:

Capturador
↓

construir()
↓

Captura

Debe quedar:

Capturador

↓

timeout

↓

Analizador::analizar_captura()

↓

Captura
Punto crítico

Entrada deja de construir triggers.

ETAPA 4 — Unificar algoritmo interno
Objetivo

Que Runtime y Captura usen exactamente el mismo motor.

Archivo
analizador_trigger.rs

Hoy existen caminos distintos.

Quedará:

Runtime
↓
Motor

Captura
↓
Motor

Una sola implementación.

Punto crítico

No debe cambiar el comportamiento del Runtime.

ETAPA 5 — Limpiar responsabilidades
Archivos
capturador_trigger.rs
analizador_trigger.rs
entrada.rs

Eliminar código muerto.

Eliminar funciones que ya no existen.

Eliminar imports.

Eliminar comentarios antiguos.

ETAPA 6 — Compilación
Revisar

Todos los errores.

No agregar hacks.

No agregar variables temporales.

No modificar Runtime.

Resultado esperado

La arquitectura queda así:

Hook

↓

InputEvent

↓

Capturador (solo graba)
│
│ timeout
▼

AnalizadorTrigger
│
├──────────────► Runtime
│
└──────────────► Captura/UI
Ventajas obtenidas
✅ Una sola lógica de interpretación.
✅ Un solo lugar donde se decide qué es un trigger.
✅ El botón Capturar y el Runtime hablan exactamente el mismo lenguaje.
✅ Se elimina la mayor fuente de inconsistencias del proyecto.
✅ Facilita la futura optimización que comentaste: filtrar primero por caché y sólo analizar tiempos cuando realmente exista una combinación candidata con condición Doble o Mantenido.

Creo que esta es una base mucho más sólida para seguir evolucionando el sistema sin volver a caer en duplicación de lógica.

//////////////////////////////////////////////
📌Solución BUG hook teclado: Agregar esta linea en src-tauri\src\lib.rs
.device_event_filter(tauri::DeviceEventFilter::Always)
////////////////////////////////////////////

📌 Ideas pendientes (no hacer todavía)
Arquitectura
⬜ Evaluar si AnalizadorTrigger debe convertirse únicamente en el analizador lógico mientras BufferEventos decide cuándo una secuencia está completa.
⬜ Elaborar un diccionario oficial de términos del proyecto para evitar ambigüedades futuras.
Rendimiento
⬜ Revisar el tamaño óptimo del BufferEventos.
⬜ Optimizar la reutilización de memoria del buffer.
Extensiones futuras
⬜ Integrar joystick utilizando la misma arquitectura de captura.
⬜ Evaluar soporte para nuevos tipos de triggers si fueran necesarios.

------ Recordar limpiar comandos.rs
------ igual captura.rs (eliminar)

------Documentar oficialmente el flujo del motor, igual que hicimos con el flujo Perfil → Compilador → Cache → Runtime.

Algo como:
Windows / Interception
│
▼
CapturadorTrigger
│
▼
BufferEventos
│
▼
AnalizadorTrigger
│
▼
ProcesadorEventos
│
▼
Runtime
│
▼
Emisor

Ese diagrama será muy útil cuando dentro de unos meses tengamos que volver al código.

----- Cuando terminemos el motor, podríamos hacer que BufferEventos tenga un modo de depuración que imprima la línea temporal completa. Algo como:

00.000 Ctrl Down
00.120 A Down
00.145 A Up
00.310 Ctrl Up

Sería una herramienta excelente para depurar problemas de triggers sin tocar el resto del sistema.
